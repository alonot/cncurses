use std::{
    mem::take, sync::{Arc, Mutex}
};

use ncurses::{endwin, BUTTON1_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, KEY_BTAB};

use crate::{interfaces::EVENT, LOGLn};

#[derive(Default)]
pub struct CSSStyle<'a> {
    pub padding: &'a str,
    pub margin: &'a str,
    pub background_color: i16,
    pub color: i16,
    pub flex: u32,
    pub flex_direction: &'a str,
    pub taborder: i32,
    pub border_color: i16,
    pub boxsizing: &'a str,
    pub border: i32,
    pub top: &'a str,
    pub left: &'a str,
    pub height: &'a str,
    pub width: &'a str,
    pub scroll: &'a str,
    pub z_index: i32,
}

fn parse_dimension<'a>(mut d: &'a str) -> DIMEN {
    d = d.trim();
    let percen: Option<f32> = {
        if let Some(d) = d.strip_suffix('%') {
            d.parse().ok()
        } else {
            None
        }
    };
    if let Some(p) = percen {
        return DIMEN::PERCENT(p / 100.);
    } else {
        return  DIMEN::INT(d.parse::<i32>().expect("Invalid Dimension")) ;
    }
}

fn parse_multi_dimens<'a>(mut d: &'a str) -> [DIMEN; 4] {
    d = d.trim();
    d
        .split(' ')
        .into_iter()
        .map(|c| parse_dimension(c))
        .collect::<Vec<DIMEN>>()
        .try_into()
        .unwrap_or_else(|v: Vec<DIMEN>| panic!("Expected 4 dimens: {}", v.len()))
}

fn parse_flex_direction<'a>(d: &'a str) -> FLEXDIRECTION {
    match d.trim() {
        "vertical" => {
            FLEXDIRECTION::VERTICAL
        },
        "horizontal" => {
            LOGLn!("h");
            FLEXDIRECTION::HORIZONTAL
        },
        _ => {
            panic!("Invalid Flex Direction")
        }
    }
}

fn parse_overflow<'a>(d: &'a str) -> OVERFLOWBEHAVIOUR {
    match d.trim() {
        "scroll" => {
            OVERFLOWBEHAVIOUR::SCROLL
        },
        "visible" => {
            OVERFLOWBEHAVIOUR::VISIBLE
        },
        "hidden" => {
            OVERFLOWBEHAVIOUR::HIDDEN
        },
        _ => {
            panic!("Invalid Overflow")
        }
    }
}

fn parse_box_sizing<'a>(d: &'a str) -> BOXSIZING {
    match d.trim() {
        "border-box" => {
            BOXSIZING::BORDERBOX
        },
        "content-box" => {
            BOXSIZING::CONTENTBOX
        },
        _ => {
            panic!("Invalid Box Sizing")
        }
    }
}

impl<'a> CSSStyle<'a> {
    pub(crate) fn create_style(&self) -> Style {
        let mut style = Style::default();
        if !self.height.is_empty() {
            style.height = parse_dimension(self.height);
        }
        if !self.width.is_empty() {
            style.width = parse_dimension(self.width);
        }
        if !self.padding.is_empty() {
            let mut dimens = parse_multi_dimens(self.padding);
            style.paddingtop = take(&mut dimens[0]);
            style.paddingbottom = take(&mut dimens[1]);
            style.paddingleft = take(&mut dimens[2]);
            style.paddingright = take(&mut dimens[3]);
        }
        if !self.margin.is_empty() {
            let mut dimens = parse_multi_dimens(self.margin);
            style.margintop = take(&mut dimens[0]);
            style.marginbottom = take(&mut dimens[1]);
            style.marginleft = take(&mut dimens[2]);
            style.marginright = take(&mut dimens[3]);
        }
        style.z_index = self.z_index;
        style.background_color = self.background_color;
        style.border = self.border;
        style.border_color = self.border_color;
        style.flex = self.flex;
        style.taborder = self.taborder;
        if !self.flex_direction.is_empty() {
            style.flex_direction = parse_flex_direction(self.flex_direction);
        }
        if !self.boxsizing.is_empty() {
            style.boxsizing = parse_box_sizing(self.boxsizing);
        }
        if !self.scroll.is_empty() {
            style.scroll = parse_overflow(self.scroll);
        }
        if !self.top.is_empty() {
            style.top = parse_dimension(self.top);
        }
        if !self.left.is_empty() {
            style.left = parse_dimension(self.left);
        }
        style
    }
}

#[derive(Debug)]
pub enum DIMEN {
    INT(i32),
    PERCENT(f32),
}

impl Default for DIMEN {
    fn default() -> Self {
        DIMEN::INT(0)
    }
}

impl DIMEN {
    fn verify(mut self) -> Self {
        match self {
            DIMEN::INT(i) => {
                if i < FIT_CONTENT {
                    endwin();
                    panic!("Invalid Dimens: Dimens:INT() >= -1");
                }
            }
            DIMEN::PERCENT(mut p) => {
                if p > 100.0 || p < 0.0 {
                    endwin();
                    panic!("Invalid Dimens: 0 <= Dimen:PERCEN() <= 100");
                }
                p = p / 100.0;
                self = DIMEN::PERCENT(p);
            }
        }
        self
    }
}

#[derive(Default)]
pub(crate) struct Style {
    pub(crate) height: DIMEN,
    pub(crate) width: DIMEN,
    pub(crate) top: DIMEN,
    pub(crate) left: DIMEN,
    pub(crate) paddingleft: DIMEN,
    pub(crate) paddingtop: DIMEN,
    pub(crate) paddingright: DIMEN,
    pub(crate) paddingbottom: DIMEN,
    pub(crate) marginleft: DIMEN,
    pub(crate) margintop: DIMEN,
    pub(crate) marginright: DIMEN,
    pub(crate) marginbottom: DIMEN,

    pub(crate) border: i32,
    pub(crate) border_color: i16,
    pub(crate) color: i16,
    pub(crate) background_color: i16,
    pub(crate) taborder: i32,
    pub(crate) boxsizing: BOXSIZING,
    pub(crate) flex: u32,
    pub(crate) flex_direction: FLEXDIRECTION,
    pub(crate) z_index: i32,
    pub(crate) onclick_bubble: Option<Arc<Mutex<dyn FnMut(&mut EVENT)>>>, // should be a clousure
    pub(crate) onscroll_bubble: Option<Arc<Mutex<dyn FnMut(&mut EVENT)>>>, // should be a clousure
    pub(crate) onclick_capture: Option<Arc<Mutex<dyn FnMut(&mut EVENT)>>>, // should be a clousure
    pub(crate) onscroll_capture: Option<Arc<Mutex<dyn FnMut(&mut EVENT)>>>, // should be a clousure
    pub(crate) onfocus: Option<Arc<Mutex<dyn FnMut()>>>, // should be a clousure
    pub(crate) onunfocus: Option<Arc<Mutex<dyn FnMut()>>>, // should be a clousure
    pub(crate) render: bool,
    pub(crate) scroll: OVERFLOWBEHAVIOUR,
}

unsafe impl Send for Style{}

pub const FIT_CONTENT: i32 = -1;
pub const MAX_CONTENT: i32 = -2;

impl Style {
    pub(crate) fn default() -> Style {
        Style {
            height: DIMEN::INT(FIT_CONTENT),
            width: DIMEN::INT(FIT_CONTENT),
            top: DIMEN::default(),
            left: DIMEN::default(),
            paddingleft: DIMEN::default(),
            paddingtop: DIMEN::default(),
            paddingright: DIMEN::default(),
            paddingbottom: DIMEN::default(),
            marginleft: DIMEN::default(),
            margintop: DIMEN::default(),
            marginright: DIMEN::default(),
            marginbottom: DIMEN::default(),
            border: 0,
            border_color: -1,
            color: -1,
            background_color: -2,
            flex_direction: FLEXDIRECTION::default(),
            boxsizing: BOXSIZING::default(),
            flex: 0,
            taborder: -1,
            z_index: 0,
            onclick_bubble: None,
            onscroll_bubble: None,
            onclick_capture: None,
            onscroll_capture: None,
            onfocus: None,
            onunfocus: None,
            render: true,
            scroll: OVERFLOWBEHAVIOUR::HIDDEN,
        }
    }
    pub(crate) fn set_style(&mut self, v: STYLE) {
        match v {
            STYLE::TABORDER(t) => self.taborder = t,
            STYLE::HIEGHT(h) => self.height = h.verify(),
            STYLE::WIDTH(w) => self.width = w.verify(),
            STYLE::TOP(t) => self.top = t.verify(),
            STYLE::LEFT(t) => self.left = t.verify(),
            STYLE::PADDINGLEFT(p) => self.paddingleft = p.verify(),
            STYLE::PADDINGTOP(p) => self.paddingtop = p.verify(),
            STYLE::PADDINGRIGHT(p) => self.paddingright = p.verify(),
            STYLE::PADDINGBOTTOM(p) => self.paddingbottom = p.verify(),
            STYLE::MARGINLEFT(p) => self.paddingleft = p.verify(),
            STYLE::MARGINTOP(p) => self.paddingtop = p.verify(),
            STYLE::MARGINRIGHT(p) => self.paddingright = p.verify(),
            STYLE::MARGINBOTTOM(p) => self.paddingbottom = p.verify(),
            STYLE::BORDER(b) => self.border = b as i32,
            STYLE::FLEX(f) => self.flex = f,
            STYLE::FLEXDIRECTION(f) => self.flex_direction = f,
            STYLE::BOXSIZING(f) => self.boxsizing = f,
            STYLE::BACKGROUNDCOLOR(bg) => self.background_color = bg,
            STYLE::TEXTCOLOR(bg) => self.color = bg,
            STYLE::BORDERCOLOR(bg) => self.border_color = bg,
            STYLE::ZINDEX(z) => self.z_index = z,
            STYLE::OVERFLOW(overflow_behaviour) => self.scroll = overflow_behaviour,
        }
    }

    pub(crate) fn from_style(styles: Vec<STYLE>) -> Style {
        let mut style_obj = Style::default();

        styles.into_iter().for_each(|v| {
            style_obj.set_style(v);
        });

        style_obj
    }

    /**
     * Handles the incoming event with correct event handler.
     * returns if the event should propogate further or not.
     * true: stop_propogation
     */
    pub(crate) fn handle_event(&self, event: &mut EVENT, capture: bool) {
        let mut fnc_opt: &Option<Arc<Mutex<dyn FnMut(&mut EVENT)>>> = &None;

        if let Some(mevent) = event.mevent {
            if mevent.bstate == BUTTON1_PRESSED as u32 || mevent.bstate == BUTTON3_PRESSED as u32 {
                // left mouse clicked                           // right click
                if capture {
                    fnc_opt = &self.onclick_capture;
                } else {
                    fnc_opt = &self.onclick_bubble;
                }
            } else if mevent.bstate == BUTTON4_PRESSED as u32
                || mevent.bstate == BUTTON5_PRESSED as u32
            {
                // scroll up                                            // scroll down
                if capture {
                    fnc_opt = &self.onscroll_capture;
                } else {
                    fnc_opt = &self.onscroll_bubble;
                }
            }
        } else if !capture { 
            match event.key {
                _ => {}
            }
        }

        if let Some(fnc) = fnc_opt {
            fnc.lock().unwrap()(event);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OVERFLOWBEHAVIOUR {
    VISIBLE,
    HIDDEN,
    SCROLL,
}

impl Default for OVERFLOWBEHAVIOUR {
    fn default() -> Self {
        OVERFLOWBEHAVIOUR::HIDDEN
    }
}

#[derive(Debug)]
pub enum FLEXDIRECTION {
    VERTICAL,
    HORIZONTAL,
}

impl Default for FLEXDIRECTION {
    fn default() -> Self {
        FLEXDIRECTION::VERTICAL
    }
}

pub enum BOXSIZING {
    /** The padding is taken within the content dimensions. If height is set to FITCONTENT then boxsizing will be forced to border box for height. Similarly for width too. */
    BORDERBOX,
    /** The padding is outside the content dimensions */
    CONTENTBOX,
}

impl Default for BOXSIZING {
    fn default() -> Self {
        BOXSIZING::CONTENTBOX
    }
}

pub enum STYLE {
    HIEGHT(DIMEN),
    WIDTH(DIMEN),
    /** relative to current position */
    TOP(DIMEN),
    LEFT(DIMEN),
    PADDINGLEFT(DIMEN),
    PADDINGTOP(DIMEN),
    PADDINGRIGHT(DIMEN),
    PADDINGBOTTOM(DIMEN),
    MARGINLEFT(DIMEN),
    MARGINTOP(DIMEN),
    MARGINRIGHT(DIMEN),
    MARGINBOTTOM(DIMEN),
    TABORDER(i32),
    BORDER(bool),
    BACKGROUNDCOLOR(i16),
    TEXTCOLOR(i16),
    BORDERCOLOR(i16),
    BOXSIZING(BOXSIZING),
    /** 0 means unset. Actual Height and width dimensions with INT gets priority over flex. if they are set with PERCEN then flex gets priority. */
    FLEX(u32),
    /**Default Vertical */
    FLEXDIRECTION(FLEXDIRECTION),
    ZINDEX(i32),
    OVERFLOW(OVERFLOWBEHAVIOUR),
}
