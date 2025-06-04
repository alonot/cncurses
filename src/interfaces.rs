use std::{
    any::Any,
    cell::RefCell,
    clone,
    mem::take,
    rc::Rc,
    sync::{Arc, Mutex},
};

use dyn_clone::DynClone;
use ncurses::{
    endwin, newwin, BUTTON1_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, MENU, MEVENT, PANEL, WINDOW
};

use crate::nmodels::IView::IView;

pub trait Stateful: DynClone + Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn Stateful) -> bool;
}
impl<T: Clone + Any + Send + PartialEq> Stateful for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn Stateful) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |other| self == other)
    }
}

dyn_clone::clone_trait_object!(Stateful);

pub(crate) enum IViewContent {
    CHIDREN(Vec<Arc<Mutex<IView>>>),
    TEXT(String),
}

impl Default for IViewContent {
    fn default() -> Self {
        IViewContent::TEXT("".to_string())
    }
}

pub(crate) struct MENUSTRUCT {
    menu: MENU,
    win: WINDOW,
}

/**
 * Never use this multi-threaded
 */
pub(crate) enum BASICSTRUCT {
    WIN(WINDOW),
    PANEL(PANEL),
    MENU(MENUSTRUCT),
}

unsafe impl Send for BASICSTRUCT {}
unsafe impl Sync for BASICSTRUCT {}

impl Default for BASICSTRUCT {
    fn default() -> Self {
        BASICSTRUCT::WIN(newwin(0, 0, 0, 0))
    }
}

pub trait Component: Any + Send {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>;
    fn __base__(&self) -> Option<Arc<Mutex<IView>>> {
        None
    }
    fn __key__(&self) -> Option<&String> {
        None
    }
}

impl<T: Component> ComponentBuilder<T> for T {
    fn build(self) -> Arc<Mutex<dyn Component>> {
        Arc::new(Mutex::new(self))
    }
}

pub trait ComponentBuilder<T> {
    fn build(self) -> Arc<Mutex<dyn Component>>;
}

pub enum DIMEN {
    INT(i32),
    PERCENT(f32),
}

impl Default for DIMEN {
    fn default() -> Self {
        DIMEN::INT(0)
    }
}


#[derive(Debug)]
pub struct EVENT {
    pub(crate) mevent: Option<MEVENT>,
    pub(crate) key: i32,
    pub(crate) clientx: i32,
    pub(crate) clienty: i32,
    pub(crate) propogate: bool,
    pub(crate) default: bool,
}

impl EVENT {
    pub(crate) fn new(ch: i32) -> EVENT {
        EVENT { mevent: None, key: 0, clientx: 0, clienty: 0, propogate: true, default: true }
    }

    pub fn get_mevent(&mut self) -> &Option<MEVENT> {
        &self.mevent
    }
    
    pub fn get_key(&mut self) -> i32 {
        self.key
    }
    pub fn get_clientx(&mut self) -> i32 {
        self.clientx
    }
    pub fn get_clienty(&mut self) -> i32 {
        self.clienty
    }

    pub fn stop_propogation(&mut self) {
        self.propogate = false;
    }
    
    pub fn prevent_default(&mut self) {
        self.default = false;
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

    pub(crate) taborder: i32,
    pub(crate) boxsizing: BOXSIZING,
    pub(crate) flex: u32,
    pub(crate) flex_direction: FLEXDIRECTION,
    pub(crate) background_color: i32,
    pub(crate) z_index: i32,
    pub(crate) onclick_bubble: Option<Arc<Mutex<dyn FnMut(&mut EVENT) + Send>>>, // should be a clousure
    pub(crate) onscroll_bubble: Option<Arc<Mutex<dyn FnMut(&mut EVENT) + Send>>>, // should be a clousure
    pub(crate) onclick_capture: Option<Arc<Mutex<dyn FnMut(&mut EVENT) + Send>>>, // should be a clousure
    pub(crate) onscroll_capture: Option<Arc<Mutex<dyn FnMut(&mut EVENT) + Send>>>, // should be a clousure
    pub(crate) render: bool,
    pub(crate) scroll: OVERFLOWBEHAVIOUR,
}

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
            flex_direction: FLEXDIRECTION::default(),
            boxsizing: BOXSIZING::default(),
            background_color: 0,
            flex: 0,
            taborder: -1,
            z_index: 0,
            onclick_bubble: None,
            onscroll_bubble: None,
            onclick_capture: None,
            onscroll_capture: None,
            render: true,
            scroll: OVERFLOWBEHAVIOUR::HIDDEN,
        }
    }
    pub(crate) fn set_style(&mut self, v: STYLE) {
        match v {
            STYLE::TABORDER(t) => self.taborder = t,
            STYLE::HIEGHT(h) => self.height = h,
            STYLE::WIDTH(w) => self.width = w,
            STYLE::TOP(t) => self.top = t,
            STYLE::LEFT(t) => self.left = t,
            STYLE::PADDINGLEFT(p) => self.paddingleft = p,
            STYLE::PADDINGTOP(p) => self.paddingtop = p,
            STYLE::PADDINGRIGHT(p) => self.paddingright = p,
            STYLE::PADDINGBOTTOM(p) => self.paddingbottom = p,
            STYLE::FLEX(f) => self.flex = f,
            STYLE::FLEXDIRECTION(f) => self.flex_direction = f,
            STYLE::BOXSIZING(f) => self.boxsizing = f,
            STYLE::BACKGROUNDCOLOR(bg) => self.background_color = bg,
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
        let mut fnc_opt: &Option<Arc<Mutex<dyn FnMut(&mut EVENT) + Send>>> = &None;

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
        } else if !capture { // TODO
            
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
    TABORDER(i32),
    BOXSIZING(BOXSIZING),
    /** 0 means unset. Actual Height and width dimensions with INT gets priority over flex. if they are set with PERCEN then flex gets priority. */
    FLEX(u32),
    /**Default Vertical */
    FLEXDIRECTION(FLEXDIRECTION),
    BACKGROUNDCOLOR(i32),
    ZINDEX(i32),
    OVERFLOW(OVERFLOWBEHAVIOUR),
}

/**
 * Hooks struct. Each Component will have its own object of this struct
 */
pub(crate) struct Fiber {
    pub(crate) key: String,
    pub(crate) head: usize,
    pub(crate) state: Vec<Box<dyn Stateful>>,
    pub(crate) changed: bool,
    pub(crate) component: Arc<Mutex<dyn Component>>, // for rendering and re-rendering
    pub(crate) iview: Option<Arc<Mutex<IView>>>,     // Corresponding IView this Component yields
    pub(crate) children: Vec<Arc<Mutex<Fiber>>>,
}

impl Fiber {
    /**
     * Adds the fiber to the global fiber list and returns the index
     */
    pub(crate) fn new(
        key: String,
        component: Arc<Mutex<dyn Component>>,
        changed: bool,
    ) -> Arc<Mutex<Fiber>> {
        Arc::new(Mutex::new(Fiber {
            key: key,
            head: 0,
            state: vec![],
            changed: changed,
            component: component,
            iview: None,
            children: vec![],
        }))
    }

    pub(crate) fn add_iview(&mut self, iview: Arc<Mutex<IView>>) {
        self.iview = Some(iview);
    }
}

pub(crate) struct Document {
    pub(crate) curr_fiber: Option<Arc<Mutex<Fiber>>>,
    pub(crate) taborder: Vec<Arc<Mutex<IView>>>,
    pub(crate) tabindex: usize,
}

impl Document {
    pub(crate) fn clear_fiber(&mut self) {
        self.curr_fiber = None;
    }

    /**
     * Assigns given fiber to global fiber
     * resets the head to 0
     * returns the previous fiber
     */
    pub(crate) fn re_assign_fiber(&mut self, fiber_lk_opt: Option<Arc<Mutex<Fiber>>>) -> Option<Arc<Mutex<Fiber>>> {
        if let Some(fiber_lk) = &fiber_lk_opt {
            fiber_lk.lock().unwrap().head = 0;
        }

        let prev_fiber = {
            self.curr_fiber.clone()
        };

        self.curr_fiber = fiber_lk_opt;

        prev_fiber
        // None
    }

    /**
     * Assigns given fiber to global fiber
     * does not reset the head of input fiber
     * returns the previous fiber
     */
    pub(crate) fn assign_fiber(
        &mut self,
        fiber_lk_opt: Option<Arc<Mutex<Fiber>>>,
    ) -> Option<Arc<Mutex<Fiber>>> {
        let prev_fiber = { self.curr_fiber.clone() };
        self.curr_fiber = fiber_lk_opt;

        prev_fiber
        // None
    }

    pub(crate) fn insert_tab_element(&mut self, element: Arc<Mutex<IView>>) {
        self.taborder.push(element);
        println!("INSERT");
        self.taborder.sort_by_key(|c| c.lock().unwrap().taborder);
    }

    pub(crate) fn clear_tab_order(&mut self) {
        self.tabindex = 0;
        self.taborder.clear();
    }

    pub(crate) fn advance_tab(&mut self) {
        self.tabindex += 1;
        if self.tabindex >= self.taborder.len() {
            self.tabindex = 0;
        }
    }

    /**
     * May Overflow
     */
    pub(crate) fn active_element(&self) -> Option<Arc<Mutex<IView>>> {
        if self.taborder.is_empty() {
            return None;
        }
        Some(self.taborder[self.tabindex].clone())
    }
}
