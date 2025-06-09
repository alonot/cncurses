/***
 * The internal View model
 */

use std::{
    cmp::max,
    i32::{MAX, MIN},
    sync::{Arc, Mutex},
};

use ncurses::{
    BUTTON_SHIFT, BUTTON1_PRESSED, BUTTON2_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, COLOR_PAIR,
    KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_UP, WINDOW, box_, copywin, delwin, mvwprintw, newpad,
    newwin, ungetch, wattroff, wattron, wbkgd,
};

use crate::{
    DOCUMENT, LOG, LOGLn, REMOVEINDEX,
    interfaces::{BASICSTRUCT, Component, EVENT, IViewContent},
    styles::{
        BOXSIZING, CSSStyle, DIMEN, FIT_CONTENT, FLEXDIRECTION, OVERFLOWBEHAVIOUR, POSITION, STYLE,
        Style,
    },
};

#[derive(Debug)]
pub(crate) struct RenderBox {
    pub(crate) toplefty: i32,
    pub(crate) topleftx: i32,
    pub(crate) bottomrighty: i32,
    pub(crate) bottomrightx: i32,
}

impl RenderBox {
    fn area(&self) -> i32 {
        (self.bottomrighty - self.toplefty) * (self.bottomrightx - self.topleftx)
    }
    pub(crate) fn update(&mut self, other: &Self) {
        if self.area() == 0 {
            self.topleftx = MAX;
            self.toplefty = MAX;
        }
        self.topleftx = self.topleftx.min(other.topleftx);
        self.toplefty = self.toplefty.min(other.toplefty);
        self.bottomrightx = self.bottomrightx.max(other.bottomrightx);
        self.bottomrighty = self.bottomrighty.max(other.bottomrighty);
    }
    pub(crate) fn add_to_all(&mut self, num: i32) {
        self.topleftx += num;
        self.toplefty += num;
        self.bottomrightx += num;
        self.bottomrighty += num;
    }

    /***
     * Checks if given points falls under this render box(boundary included)
     */
    pub(crate) fn is_inside(&self, point: (i32, i32)) -> bool {
        (point.0 >= self.toplefty && point.0 <= self.bottomrighty)
            && (point.1 >= self.topleftx && point.1 <= self.bottomrightx)
    }
}

#[derive(Default)]
pub(crate) struct IView {
    pub(crate) content: IViewContent,
    pub(crate) style: Style,
    pub(crate) children: Vec<Arc<Mutex<dyn Component>>>, // will be neglected if TViewContent::TEXT
    pub(crate) parent: Option<Arc<Mutex<IView>>>,
    pub(crate) basic_struct: Option<BASICSTRUCT>,
    pub(crate) id: i32,
    pub(crate) focused: bool,

    height: i32,
    width: i32,
    /** Only the dimen of content(without padding, border) */
    content_height: i32,
    /** Only the dimen of content(without padding, border) */
    content_width: i32,

    paddingleft: i32,
    paddingtop: i32,
    paddingright: i32,
    paddingbottom: i32,
    marginleft: i32,
    margintop: i32,
    marginright: i32,
    marginbottom: i32,
    top: i32,
    left: i32,
    /**Extra above the content width */
    extrax: i32,
    /**Extra above the content height */
    extray: i32,
    flex_wrap_on: bool,

    scrollx: i32,
    scrolly: i32,
    /**  Used to check scroll limit. Has Extra Padding values added during init */
    children_height: i32,
    /**  Used to check scroll limit */
    children_width: i32,
}

impl IView {
    /**Uses DOCUMENT lock() */
    pub(crate) fn new() -> IView {
        let id = {
            let mut document = DOCUMENT.lock().unwrap();
            let id = document.unique_id;
            document.unique_id += 1;
            id
        };

        IView {
            content: IViewContent::TEXT("".to_string()),
            children: vec![],
            style: Style::default(),
            parent: None,
            basic_struct: None,
            height: FIT_CONTENT,
            width: FIT_CONTENT,
            focused: false,
            flex_wrap_on: false,
            content_height: 0,
            content_width: 0,
            scrollx: 0,
            scrolly: 0,
            paddingleft: 0,
            paddingtop: 0,
            paddingright: 0,
            paddingbottom: 0,
            marginleft: 0,
            margintop: 0,
            marginright: 0,
            marginbottom: 0,
            top: 0,
            left: 0,
            extrax: 0,
            extray: 0,
            children_height: 0,
            children_width: 0,
            id: id,
        }
    }
    /**Uses DOCUMENT lock() */
    pub(crate) fn _new_with_styles(styles: Vec<STYLE>) -> IView {
        let mut iview = IView::new();
        iview.style = Style::from_style(styles);
        iview
    }
    /**Uses DOCUMENT lock() */
    pub(crate) fn from_text(text: String, styles: Vec<STYLE>) -> IView {
        let mut iview = IView::new();
        iview.style = Style::from_style(styles);
        iview.content = IViewContent::TEXT(text);
        iview
    }
    /**Uses DOCUMENT lock() */
    pub(crate) fn with_style_vec(
        styles: Vec<STYLE>,
        content: IViewContent,
        children: Vec<Arc<Mutex<dyn Component>>>,
    ) -> IView {
        let mut iview = IView::new();
        iview.style = Style::from_style(styles);
        iview.content = content;
        iview.children = children;
        iview
    }

    /**Uses DOCUMENT lock() */
    pub(crate) fn with_style(
        styles: CSSStyle,
        content: IViewContent,
        children: Vec<Arc<Mutex<dyn Component>>>,
    ) -> IView {
        let mut iview = IView::new();
        iview.style = styles.create_style();
        iview.content = content;
        iview.children = children;
        iview
    }

    pub(crate) fn set_style(mut self, style: STYLE) -> Self {
        self.style.set_style(style);
        self
    }

    pub(crate) fn build(self) -> Arc<Mutex<IView>> {
        Arc::new(Mutex::new(self))
    }

    /**
     * Copies only those values from the other which is not affected by __init__ , __render__
     */
    pub(crate) fn fill_box_infos_from_other(&mut self, other: &Self) {
        self.scrollx = other.scrollx;
        self.scrolly = other.scrolly;
    }

    /**
     * For this the parent should not have dimension depending on child
     */
    fn evaluate_flex(
        child: &mut std::sync::MutexGuard<'_, IView>,
        total_flex: u32,
        direction: &FLEXDIRECTION,
    ) {
        // checks if child has dimension corresponding to given direction set
        // if not then marks it as percentage
        if total_flex == 0 {
            return;
        }
        if child.style.flex == 0 {
            return;
        }
        match direction {
            FLEXDIRECTION::VERTICAL => {
                // check for height
                let percent = (child.style.flex as f32) / (total_flex as f32);
                child.style.height = DIMEN::PERCENT(percent);
            }
            FLEXDIRECTION::HORIZONTAL => {
                // check for width
                let percent = (child.style.flex as f32) / (total_flex as f32);
                child.style.width = DIMEN::PERCENT(percent);
                //     if !matches!(child.style.width, DIMEN::INT(_)) {
                // }
            }
        }
    }

    fn calculate_child_dimensions(&mut self, mut changed: bool) -> (i32, i32, bool) {
        let mut cheight = 0;
        let mut cwidth = 0;
        let depend_on_child = (self.content_height < 0) || (self.content_width < 0);

        let mut parent_height = self.content_height;
        let mut parent_width = self.content_width;
        // if parent_height >= 0 {
        //     parent_height += self.style.border * 2;
        // }
        // if parent_width >= 0 {
        //     parent_width += self.style.border * 2;
        // }

        let direction = &self.style.flex_direction;

        self.flex_wrap_on = (self.style.flex_wrap)
            & ((matches!(direction, FLEXDIRECTION::VERTICAL) & (parent_height != FIT_CONTENT))
                | (matches!(direction, FLEXDIRECTION::HORIZONTAL) & (parent_width != FIT_CONTENT)));

        // init the chidlren and calculate the new dimension if dependent on children
        match &self.content {
            IViewContent::CHIDREN(items) => {
                // get the children flex sum
                let total_flex = items
                    .iter()
                    .fold(0, |prev, child| prev + child.lock().unwrap().style.flex);

                let mut cheight_wrap = 0;
                let mut cwidth_wrap = 0;

                (cheight, cwidth, changed) =
                    items.iter().fold((0, 0, changed), |prev, child_lk| {
                        let taborder = {
                            let child = child_lk.lock().unwrap();
                            child.style.taborder
                        };

                        // add this to the tab order
                        if taborder >= 0 {
                            let mut document = DOCUMENT.lock().unwrap();
                            document.insert_tab_element(child_lk.clone());
                        }

                        let mut child = child_lk.lock().unwrap();
                        // If Child has flex , but no dimension then set the respective dimension as percentage
                        IView::evaluate_flex(&mut child, total_flex, direction);

                        let (childh, childw, changed) = child.__init__(parent_height, parent_width);

                        if self.flex_wrap_on {
                            let mut nexth = prev.0;
                            let mut nextw = prev.1;
                            let child_full_height = childh + child.marginbottom + child.margintop;
                            let child_full_width = childw + child.marginleft + child.marginright;
                            match direction {
                                FLEXDIRECTION::VERTICAL => {
                                    let _nexth =
                                        prev.0 + child_full_height;
                                    if _nexth < parent_height {
                                        nexth = _nexth;
                                        cheight_wrap = max(cheight_wrap,nexth);
                                    } else {
                                        cheight_wrap = max(cheight_wrap,max(prev.0, child_full_height));
                                        nexth = 0;
                                        cwidth_wrap += nextw;
                                    }
                                    nextw = max(nextw, childw + child.marginleft + child.marginright);
                                }
                                FLEXDIRECTION::HORIZONTAL => {
                                    let _nextw =
                                        prev.1 + childw + child.marginleft + child.marginright;
                                    if _nextw < parent_width {
                                        nextw = _nextw;
                                        cwidth_wrap = max(cwidth_wrap , nextw);
                                    } else {
                                        cwidth_wrap = max(cwidth_wrap , max(prev.1, child_full_width));
                                        nextw = 0;
                                        cheight_wrap += nexth;
                                    }
                                    nexth = max(nexth, childh + child.marginbottom + child.margintop);
                                }
                            }
                            // LOGLn!("FLEX : {} {} {} {} {}",cheight_wrap, nexth, nextw, childh, childw);
                            (nexth, nextw, prev.2 | changed)
                        } else {
                            match direction {
                                FLEXDIRECTION::VERTICAL => (
                                    prev.0 + childh + child.marginbottom + child.margintop,
                                    max(prev.1, childw + child.marginleft + child.marginright),
                                    prev.2 | changed,
                                ),
                                FLEXDIRECTION::HORIZONTAL => (
                                    max(prev.0, childh + child.marginbottom + child.margintop),
                                    prev.1 + childw + child.marginleft + child.marginright,
                                    prev.2 | changed,
                                ),
                            }
                        }
                    });

                if self.flex_wrap_on {
                    match direction {
                        FLEXDIRECTION::VERTICAL => {
                            cwidth = cwidth_wrap + cwidth;
                            cheight = cheight_wrap;
                        }
                        FLEXDIRECTION::HORIZONTAL => {
                            cwidth = cwidth_wrap;
                            cheight = cheight_wrap + cheight; // extra increase to count for last row
                        }
                    }
                }

                if (changed && depend_on_child) || self.basic_struct.is_none() {
                    // then only re-create/ create the window.
                    changed = true;

                    if self.content_height == FIT_CONTENT {
                        self.content_height = cheight;
                    }
                    if self.content_width == FIT_CONTENT {
                        self.content_width = cwidth;
                    }
                }
                // if self.content_height == 3 && self.content_width == 43 {
                // }
                // LOGLn!("L: {:p} {} {} {} {} {}",self, self.content_height, self.content_width, cheight, cwidth, self.flex_wrap_on);
            }
            IViewContent::TEXT(txt) => {
                if changed {
                    // update chieght and cwidth
                    if self.content_width <= 0 {
                        self.content_width = txt.len() as i32;
                    }

                    if self.content_width > 0 {
                        cheight = ((txt.len() as f32 / self.content_width as f32).ceil() as i32)
                            .max(self.content_height);
                        cwidth = self.content_width;
                    }

                    if self.content_height == FIT_CONTENT {
                        self.content_height = cheight;
                    }
                    // LOGLn!("L: {:p} {} {} {} {} {} ",self, txt, self.content_height, self.content_width, cheight, cwidth);
                }
            }
        };

        (cheight, cwidth, changed)
    }

    fn fill_box_infos(&mut self) {
        match self.style.paddingleft {
            DIMEN::PERCENT(percent) => {
                if self.content_width == FIT_CONTENT {
                    self.paddingleft = 0; // to be calculated later
                }
                self.paddingleft = (self.content_width as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.paddingleft = w;
            }
        }
        match self.style.paddingtop {
            DIMEN::PERCENT(percent) => {
                if self.content_height == FIT_CONTENT {
                    self.paddingtop = 0; // to be calculated later
                }
                self.paddingtop = (self.content_height as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.paddingtop = w;
            }
        }
        match self.style.paddingright {
            DIMEN::PERCENT(percent) => {
                if self.content_width == FIT_CONTENT {
                    self.paddingright = 0; // to be calculated later
                }
                self.paddingright = (self.content_width as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.paddingright = w;
            }
        }
        match self.style.paddingbottom {
            DIMEN::PERCENT(percent) => {
                if self.content_height == FIT_CONTENT {
                    self.paddingtop = 0; // to be calculated later
                }
                self.paddingbottom = (self.content_height as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.paddingbottom = w;
            }
        }
        match self.style.marginleft {
            DIMEN::PERCENT(percent) => {
                if self.content_width == FIT_CONTENT {
                    self.marginleft = 0; // to be calculated later
                }
                self.marginleft = (self.content_width as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.marginleft = w;
            }
        }
        match self.style.margintop {
            DIMEN::PERCENT(percent) => {
                if self.content_height == FIT_CONTENT {
                    self.margintop = 0; // to be calculated later
                }
                self.margintop = (self.content_height as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.margintop = w;
            }
        }
        match self.style.marginright {
            DIMEN::PERCENT(percent) => {
                if self.content_width == FIT_CONTENT {
                    self.marginright = 0; // to be calculated later
                }
                self.marginright = (self.content_width as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.marginright = w;
            }
        }
        match self.style.marginbottom {
            DIMEN::PERCENT(percent) => {
                if self.content_height == FIT_CONTENT {
                    self.margintop = 0; // to be calculated later
                }
                self.marginbottom = (self.content_height as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.marginbottom = w;
            }
        }
        match self.style.top {
            DIMEN::PERCENT(percent) => {
                if self.content_width == FIT_CONTENT {
                    self.top = 0; // to be calculated later
                }
                self.top = (self.content_width as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.top = w;
            }
        }
        match self.style.left {
            DIMEN::PERCENT(percent) => {
                if self.content_height == FIT_CONTENT {
                    self.margintop = 0; // to be calculated later
                }
                self.left = (self.content_height as f32 * percent).floor() as i32;
            }
            DIMEN::INT(w) => {
                self.left = w;
            }
        }
    }

    fn destroy_basic_struct(&mut self) {
        if let Some(prev_win) = &self.basic_struct {
            match prev_win {
                BASICSTRUCT::WIN(win) => {
                    delwin(*win);
                }
                BASICSTRUCT::_PANEL(_) => {
                    todo!()
                }
                BASICSTRUCT::_MENU(_) => {
                    todo!()
                }
            }
        };
    }

    fn init_basic_struct(&mut self) {
        match &self.content {
            IViewContent::CHIDREN(_) => {
                // LOGLn!(
                //     "WIN: {} {} {} {}",
                //     self.content_height + self.extray,
                //     self.content_width + self.extrax,
                //     self.height,
                //     self.width
                // );
                self.basic_struct = Some(BASICSTRUCT::WIN(newwin(
                    self.content_height + self.extray,
                    self.content_width + self.extrax,
                    0,
                    0,
                )));
            }
            IViewContent::TEXT(_) => {
                // create a pad
                // LOGLn!("{} {} {} {}", self.content_height + self.extray, self.content_width + self.extrax, self.height, self.width);
                let win = newwin(
                    self.content_height + self.extray,
                    self.content_width + self.extrax,
                    0,
                    0,
                );
                self.basic_struct = Some(BASICSTRUCT::WIN(win));
            }
        }
    }

    /**
     * Allocates actual ncurses window/panel/menu/form
     * input:
     *      parent_height,
     *      parent_width
     * returns self height, width and whether changed occured
     * Whenever this function is called, the tab order resets.
     */
    pub(crate) fn __init__(&mut self, parent_height: i32, parent_width: i32) -> (i32, i32, bool) {
        // we need to know height and width

        // height and width from children
        let changed = self.style.render;
        if changed {
            // if self.dimensions depends on parent
            match self.style.height {
                DIMEN::PERCENT(percent) => {
                    // if parent dimension is not defined i.e. depends on child itself then error
                    if parent_height < 0 {
                        panic!(
                            "Circular dependence on dimensions: Parent does not have a dimension, while child depends on it. <Some Debug Info>{:p}",
                            self
                        )
                    }

                    // calculate the dimensions
                    // it may be either percentage or flex
                    // if flex then parent will have converted the height to PERCEN
                    self.height = (parent_height as f32 * percent).floor() as i32;
                }
                DIMEN::INT(h) => {
                    if h < FIT_CONTENT {
                        panic!("Invalid Height : {}", h)
                    }
                    self.height = h;
                }
            }
            match self.style.width {
                DIMEN::PERCENT(percent) => {
                    // if parent dimension is not defined i.e. depends on child itself then error
                    if parent_width < 0 {
                        panic!(
                            "Circular dependence on dimensions: Parent does not have a dimension, while child depends on it. <Some Debug Info> {:p}",
                            self
                        )
                    }

                    // calculate the dimensions
                    // it may be either percentage or flex
                    // if flex then parent will have converted the width to PERCEN
                    self.width = (parent_width as f32 * percent).floor() as i32;
                }
                DIMEN::INT(w) => {
                    if w < FIT_CONTENT {
                        panic!("Invalid Width : {}", w)
                    }
                    self.width = w;
                }
            }
            // content dimension will be same as parent if not border_box
            self.content_height = self.height;
            self.content_width = self.width;

            if matches!(self.style.boxsizing, BOXSIZING::BORDERBOX) {
                // then calculation of padding is required right now.

                // if height is FITCONTENT, we will leave the padding to zero. it will be calculated after calculate_child_dimension(treat height as CONTENTBOX)
                // similar for width
                self.fill_box_infos();
                if self.content_height != FIT_CONTENT {
                    self.content_height -=
                        self.paddingbottom + self.paddingtop + (self.style.border * 2);
                    self.content_height = self.content_height.max(0);
                }
                if self.content_width != FIT_CONTENT {
                    self.content_width -=
                        self.paddingleft + self.paddingright + (self.style.border * 2);

                    self.content_width = self.content_width.max(0);
                }
                // LOGLn!("{} {}", self.content_width, parent_width);
            }
        }

        let (cheight, cwidth, changed) = self.calculate_child_dimensions(changed);
        // content dimensions would have been updated if depend on child
        if changed {
            // if previously padding was not calculated (due to content box), then it will be calculated now
            self.fill_box_infos();

            // update height and width
            self.height = self.content_height;
            self.width = self.content_width;

            // if visibility set to VISIBLE then update the content dimensions
            if matches!(self.style.overflow, OVERFLOWBEHAVIOUR::VISIBLE) {
                self.content_height = cheight;
                self.content_width = cwidth;
            }

            self.extrax = self.paddingleft + self.paddingright + (self.style.border * 2);
            self.extray = self.paddingbottom + self.paddingtop + (self.style.border * 2);

            // update the height and width with padding
            self.height += self.extray;
            self.width += self.extrax;

            self.children_height = cheight + self.extray;
            self.children_width = cwidth + self.extrax;
            // LOGLn!(
            //     "{:p} {} {} : {} {} {} {} {} {}",
            //     self, self.height, self.width, self.content_height, self.content_width ,self.extrax, self.extray, cheight, cwidth
            // );
        }

        (self.height, self.width, changed)
    }

    /**
     * given child box returns the parents box where to render this child
     * NOTE: Margin, top and left all these are expected to be added by the caller
     */
    fn corrected_render_box(
        &self,
        child_render_box: &mut RenderBox,
        top_left: &(i32, i32),
        last_cusor: &(i32, i32),
    ) -> RenderBox {
        let mut curr_render_box = RenderBox {
            toplefty: child_render_box.toplefty + top_left.0 - self.scrolly,
            topleftx: child_render_box.topleftx + top_left.1 - self.scrollx,
            bottomrighty: child_render_box.bottomrighty + top_left.0 - self.scrolly,
            bottomrightx: child_render_box.bottomrightx + top_left.1 - self.scrollx,
        };

        if curr_render_box.toplefty < 0 {
            // means we need to cut some top portion from the child
            child_render_box.toplefty += -curr_render_box.toplefty; // shift it down by as much as negative
            child_render_box.toplefty =
                child_render_box.toplefty.min(child_render_box.bottomrighty);
            curr_render_box.toplefty = 0;
        }
        if curr_render_box.topleftx < 0 {
            // same for x direction
            child_render_box.topleftx += -curr_render_box.topleftx; // shift it right by as much as negative
            child_render_box.topleftx =
                child_render_box.topleftx.min(child_render_box.bottomrightx); // clamp it by bottomright
            curr_render_box.topleftx = 0;
        }
        // bottom may also go above curr scroll
        if curr_render_box.bottomrighty < 0 {
            // means we need to cut some top portion from the child
            child_render_box.bottomrighty += -curr_render_box.bottomrighty; // shift it down by as much as negative
            curr_render_box.bottomrighty = 0;
        }
        if curr_render_box.bottomrightx < 0 {
            // same for x direction
            child_render_box.bottomrightx += -curr_render_box.bottomrightx; // shift it right by as much as negative
            curr_render_box.bottomrightx = 0;
        }

        // no point must cross the lastcursor
        curr_render_box.toplefty = curr_render_box.toplefty.max(0).min(last_cusor.0);
        curr_render_box.topleftx = curr_render_box.topleftx.max(0).min(last_cusor.1);
        curr_render_box.bottomrighty = curr_render_box.bottomrighty.max(0).min(last_cusor.0);
        curr_render_box.bottomrightx = curr_render_box.bottomrightx.max(0).min(last_cusor.1);

        curr_render_box
    }

    /** Walks through the children while maintaining their position inside this component
     * if event is not None, then calls the handler else renders the children
     * renders the children which have their:  min_z_index <= z_index <= max_z_index
     **/
    fn render_children(
        &self,
        win: &WINDOW,
        icomponents: &Vec<Arc<Mutex<IView>>>,
        topleft: &mut (i32, i32),
        last_cursor: &mut (i32, i32),
        max_z_index: i32,
        min_z_index: i32,
        mut event_opt: Option<&mut EVENT>,
    ) -> RenderBox {
        let scroll_end_cursor = (
            self.scrolly + self.content_height + self.extray - (self.style.border * 2),
            self.scrollx + self.content_width + self.extrax - (self.style.border * 2),
        );

        let mut last_cursor_with_border = last_cursor.clone();
        last_cursor_with_border.0 += self.style.border * 2;
        last_cursor_with_border.1 += self.style.border * 2;

        let direction = &self.style.flex_direction;

        let mut curr_render_box = RenderBox {
            topleftx: 0,
            toplefty: 0,
            bottomrightx: 0,
            bottomrighty: 0,
        };

        let mut cheight_wrap = 0;
        let mut cwidth_wrap = 0;

        let mut actualx = 0;
        let mut actualy = 0;
        match &event_opt {
            Some(e) => {
                actualx = e.clientx;
                actualy = e.clienty;
                // LOGLn!("EVENT: {}", icomponents.len());
            },
            None => {},
        }

        // loop over the children
        icomponents.iter().for_each(|child_lk| {
            // calls the render function of child if it's bounds are within the view port of this window
            // gets the width covered by the child
            let is_static = {
                let child = child_lk.lock().unwrap();
                if child.style.z_index > max_z_index || child.style.z_index < min_z_index {
                    return;
                }
                matches!(child.style.position, POSITION::STATIC)
            };

            if is_static {
                // if event_opt.is_some() {
                //     LOGLn!("{:p} {:?} {:?}", self, topleft, scroll_end_cursor);
                // }
                if topleft.0 >= scroll_end_cursor.0 || topleft.1 >= scroll_end_cursor.1 {
                    return;
                }

                let mut prevtopleft = topleft.clone();
                let considerh;
                let considerw; // height and width if this child is considered
                let margin = {
                    let child = child_lk.lock().unwrap();
                    match direction {
                        FLEXDIRECTION::VERTICAL => {
                            cwidth_wrap = max(
                                cwidth_wrap,
                                child.width + child.marginleft + child.marginright,
                            );
                            if self.flex_wrap_on
                                && topleft.0 + child.height + child.marginbottom + child.margintop
                                    >= scroll_end_cursor.0
                            {
                                topleft.0 = self.paddingtop;
                                topleft.1 += cwidth_wrap;
                                prevtopleft = topleft.clone();
                            }
                            topleft.0 += child.height;
                            considerh = topleft.0 + child.margintop;
                            considerw = topleft.1 + child.marginleft + child.width;
                        }
                        FLEXDIRECTION::HORIZONTAL => {
                            cheight_wrap = max(
                                cheight_wrap,
                                child.height + child.marginbottom + child.margintop,
                            );
                            if self.flex_wrap_on
                                && topleft.1 + child.width + child.marginleft + child.marginright
                                    >= scroll_end_cursor.1
                            {
                                topleft.1 = self.paddingleft;
                                topleft.0 += cheight_wrap;
                                prevtopleft = topleft.clone();
                            }
                            topleft.1 += child.width;
                            considerh = topleft.0 + child.height + child.margintop;
                            considerw = topleft.1 + child.marginleft;
                        }
                    }
                    topleft.0 += child.margintop;
                    topleft.1 += child.marginleft;
                    (
                        child.margintop,
                        child.marginbottom,
                        child.marginleft,
                        child.marginright,
                        child.top,
                        child.left,
                    )
                };

                if prevtopleft.0 >= scroll_end_cursor.0 || prevtopleft.1 >= scroll_end_cursor.1 {
                    return;
                }

                if !(considerh + self.style.border < self.scrolly
                    || considerw + self.style.border < self.scrollx)
                {
                    // if visible is set true then its scrollx and scrolly will already be 0

                    prevtopleft.0 += margin.0 + margin.4;
                    prevtopleft.1 += margin.2 + margin.5;

                    // either within the limits or is not static
                    let (mut render_box, child_win) = {
                        let mut child = child_lk.lock().unwrap();
                        if event_opt.is_some() {
                            let render_box = RenderBox {
                                topleftx: 0,
                                toplefty: 0,
                                bottomrightx: child.width - 1,
                                bottomrighty: child.height - 1,
                            };
                            (render_box, 0 as WINDOW)
                        } else {
                            child.__render__()
                        }
                    };
                    
                    // update the render box
                    let mut curr_box =
                        self.corrected_render_box(&mut render_box, &prevtopleft, &last_cursor);

                    curr_box.add_to_all(self.style.border);

                    // LOGLn!(
                    //     "{:p} {:?} {:?} {:?}",
                    //     self,
                    //     render_box,
                    //     curr_box,
                    //     last_cursor
                    // );

                    // need to consider the flex direction
                    // place the child at current top and left position
                    if let Some(event) = &mut event_opt {
                        // LOGLn!("{:?} {:?}", event, curr_box);
                        // now check whether this box fells under the event constraints
                        if curr_box.is_inside((event.clienty, event.clientx)) {
                            event.clientx -= curr_box.topleftx - self.style.border;
                            event.clienty -= curr_box.toplefty - self.style.border;
                            let mut child = child_lk.lock().unwrap();
                            if matches!(child.style.overflow, OVERFLOWBEHAVIOUR::SCROLL) {
                                DOCUMENT.lock().unwrap().set_active(child_lk.clone());
                            }
                            child.__handle_mouse_event__(event);
                            // now call child's event_handler
                            event.clientx = actualx;
                            event.clienty = actualy;
                        }
                    } else {
                        copywin(
                            child_win,
                            *win,
                            render_box.toplefty,
                            render_box.topleftx,
                            curr_box.toplefty,
                            curr_box.topleftx,
                            curr_box.bottomrighty,
                            curr_box.bottomrightx,
                            0,
                        );

                        child_lk.lock().unwrap().destroy_basic_struct();
                    }
                    curr_render_box.update(&curr_box);
                }
                match direction {
                    FLEXDIRECTION::VERTICAL => {
                        topleft.0 += margin.1;
                        topleft.1 -= margin.2; // since this was added but we are not going in this direction
                        // hence remove the left margin
                    }
                    FLEXDIRECTION::HORIZONTAL => {
                        topleft.1 += margin.3;
                        topleft.0 -= margin.0; // since this was added but we are not going in this direction
                        // hence remove the top margin
                    }
                }
            } else {
                let (margin, (mut render_box, child_win)) = {
                    let mut child = child_lk.lock().unwrap();
                    let res = {
                        if event_opt.is_some() {
                            let render_box = RenderBox {
                                topleftx: 0,
                                toplefty: 0,
                                bottomrightx: child.width - 1,
                                bottomrighty: child.height - 1,
                            };
                            (render_box, 0 as WINDOW)
                        } else {
                            child.__render__()
                        }
                    };
                    (
                        (
                            child.margintop,
                            child.marginbottom,
                            child.marginleft,
                            child.marginright,
                            child.top,
                            child.left,
                        ),
                        res,
                    )
                };

                // update the render box
                let curr_box = self.corrected_render_box(
                    &mut render_box,
                    &(self.scrolly + margin.4, self.scrollx + margin.5), // current scroll and the top and left, scroll will be substracted out inside function
                    &last_cursor_with_border,
                );

                if let Some(event) = &mut event_opt {
                    // now check whether this box fells under the event constraints
                    if curr_box.is_inside((event.clienty, event.clientx)) {
                        let mut child = child_lk.lock().unwrap();
                        if matches!(child.style.overflow, OVERFLOWBEHAVIOUR::SCROLL) {
                            DOCUMENT.lock().unwrap().set_active(child_lk.clone());
                        }
                        // now call child's event_handler
                        child.__handle_mouse_event__(event);
                    }
                } else {
                    copywin(
                        child_win,
                        *win,
                        render_box.toplefty,
                        render_box.topleftx,
                        curr_box.toplefty,
                        curr_box.topleftx,
                        curr_box.bottomrighty,
                        curr_box.bottomrightx,
                        0,
                    );

                    child_lk.lock().unwrap().destroy_basic_struct();
                }
                curr_render_box.update(&curr_box);
            }
        });

        curr_render_box
    }

    /**
     * Get important parameter of thepad screen and call render on its children
     * returns:
     *      rendered toplefty, topleftx
     *      botomrighty and bottomrightx changed(rendered),
     *      its window (which should be rendered by the parent)
     *
     * uses DOCUMENT.lock()
     */
    pub(crate) fn __render__(&mut self) -> (RenderBox, WINDOW) {
        let mut topleft = (self.paddingtop, self.paddingleft); // virtual screen
        let mut last_cursor = (
            self.content_height + self.extray - (self.style.border * 2) - 1, // do not consider the borderwidth in the lastcursor of this window
            self.content_width + self.extrax - (self.style.border * 2) - 1,
        );
        // do not consider the padding along the direction

        last_cursor.0 = last_cursor.0.max(0);
        last_cursor.1 = last_cursor.1.max(0);
        /*   __ .  .  .
        topleft->|
                 |           ___ . . .
                 | cursor ->|
                 |          |
              */
        self.init_basic_struct();

        let Some(basicstr) = &self.basic_struct else {
            panic!("NO WINDOW found for View")
        };

        let mut curr_render_box = RenderBox {
            topleftx: 0,
            toplefty: 0,
            bottomrightx: 0,
            bottomrighty: 0,
        };
        let win: &WINDOW;

        let border_color = {
            DOCUMENT
                .lock()
                .unwrap()
                .get_color_pair(self.style.border_color, self.style.background_color)
        };

        match &self.content {
            IViewContent::CHIDREN(icomponents) => {
                win = {
                    let BASICSTRUCT::WIN(win_t) = &basicstr else {
                        panic!("NO WINDOW found for View")
                    };
                    win_t
                };
                // LOGLn!("{}", icomponents.len());

                curr_render_box.update(&self.render_children(
                    win,
                    icomponents,
                    &mut topleft,
                    &mut last_cursor,
                    -1,
                    MIN,
                    None
                ));

                if self.style.render {
                    // then we need to render this window itself
                    // so background must be updated
                    wbkgd(*win, ' ' as u32 | COLOR_PAIR(border_color));
                    if self.style.border > 0 {
                        // LOG!("{} {}", self.style.border_color, self.style.background_color);
                        wattron(*win, COLOR_PAIR(border_color)); // setting border_pair
                        box_(*win, 0, 0);
                        wattroff(*win, COLOR_PAIR(border_color)); // setting border_pair
                    }
                }

                curr_render_box.update(&self.render_children(
                    win,
                    icomponents,
                    &mut topleft,
                    &mut last_cursor,
                    0,
                    0,
                    None
                ));
            }
            IViewContent::TEXT(txt) => {
                let BASICSTRUCT::WIN(win_t) = &basicstr else {
                    panic!("NO WINDOW found for View")
                };

                win = win_t;

                if self.style.render {
                    let text_color = {
                        DOCUMENT
                            .lock()
                            .unwrap()
                            .get_color_pair(self.style.color, self.style.background_color)
                    };

                    // LOGLn!("{} {}", txt, self.style.color));

                    wbkgd(*win, ' ' as u32 | COLOR_PAIR(border_color));
                    if self.style.border > 0 {
                        wattron(*win, COLOR_PAIR(border_color)); // setting border_pair
                        box_(*win, 0, 0);
                        wattroff(*win, COLOR_PAIR(border_color)); // setting off border_pair
                    }
                    // LOGLn!("{} {} {:?} {:?}", self.children_height, self.children_width, topleft, last_cursor);
                    let pad = newpad(self.children_height, self.children_width);

                    // then we need to render this window itself
                    // so background must be updated
                    wbkgd(pad, ' ' as u32 | COLOR_PAIR(border_color));

                    wattron(pad, COLOR_PAIR(text_color)); // setting text_pair
                    // display the text at curootrrent top and left
                    let res = mvwprintw(pad, 0, 0, &txt);
                    if let Err(_) = res {
                        LOGLn!("Warning: NULL Error while rendering Text View");
                    };
                    wattroff(pad, COLOR_PAIR(text_color)); // setting off text_pair

                    copywin(
                        pad,
                        *win,
                        self.scrolly,
                        self.scrollx,
                        topleft.0 + self.style.border,
                        topleft.1 + self.style.border,
                        last_cursor.0 - self.paddingbottom + self.style.border,
                        last_cursor.1 - self.paddingright + self.style.border,
                        0,
                    );

                    delwin(pad);
                    // wrefresh(*win);
                }
            }
        }

        if self.style.render {
            curr_render_box.toplefty = 0;
            curr_render_box.topleftx = 0;
            curr_render_box.bottomrighty = (self.content_height + self.extray - 1).max(0);
            curr_render_box.bottomrightx = (self.content_width + self.extrax - 1).max(0);
        }

        (curr_render_box, *win)
    }

    pub(crate) fn handle_default(&mut self, event: &mut EVENT) {
        let mut scroll_direction = -1;
        let is_scroll_vertical = matches!(self.style.flex_direction, FLEXDIRECTION::VERTICAL);
        let vertical = (!is_scroll_vertical & self.style.flex_wrap)
            | (is_scroll_vertical & !self.style.flex_wrap);
        if let Some(mevent) = &event.mevent {
            if mevent.bstate & BUTTON1_PRESSED as u32 > 0 {
                // left mouse clicked
                if self.style.taborder >= 0 {
                    // generate a tab event which will change focus and call handler itself
                    DOCUMENT.lock().unwrap().next_tab_id = self.id;
                    ungetch('\t' as i32);
                }
            } else if (mevent.bstate & BUTTON2_PRESSED as u32 == 0)
                && matches!(self.style.overflow, OVERFLOWBEHAVIOUR::SCROLL)
            {
                if mevent.bstate & BUTTON4_PRESSED as u32 > 0 {
                    if mevent.bstate & BUTTON_SHIFT as u32 > 0 {
                        // scroll right
                        scroll_direction = 3;
                    } else {
                        // scroll down
                        scroll_direction = 1;
                    }
                } else if mevent.bstate & BUTTON5_PRESSED as u32 > 0 {
                    if mevent.bstate & BUTTON_SHIFT as u32 > 0 {
                        // scroll left
                        scroll_direction = 2;
                    } else {
                        // scroll up
                        scroll_direction = 0;
                    }
                }
            }
        } else {
            match event.key {
                // natural scrolling
                KEY_UP => scroll_direction = 1,
                KEY_RIGHT => scroll_direction = 3,
                KEY_LEFT => scroll_direction = 2,
                KEY_DOWN => scroll_direction = 0,
                _ => {}
            }
        };
        // LOGLn!("{:p} {} {} {} {}",self, self.scrolly, self.children_height,self.content_height, self.extray);
        match scroll_direction {
            0 => {
                // scroll up
                if vertical
                    && self.scrolly < self.children_height - self.content_height - self.extray
                {
                    self.scrolly += 1;
                    self.style.render = true;
                }
            }
            1 => {
                if vertical && self.scrolly > 0 {
                    self.scrolly -= 1;
                    self.style.render = true;
                }
            }
            2 => {
                if !vertical
                    && self.scrollx < self.children_width - self.content_width - self.extrax
                {
                    self.scrollx += 1;
                    self.style.render = true;
                }
            }
            3 => {
                // scroll right
                if !vertical && self.scrollx > 0 {
                    self.scrollx -= 1;
                    self.style.render = true;
                }
            }
            _ => {}
        }
        if self.style.render {
            DOCUMENT.lock().unwrap().changed = true;
        }
    }

    /** Finds the child under the event and transfers it to the child */
    fn transfer_event(&mut self, event: &mut EVENT) {
        let mut topleft = (self.paddingtop, self.paddingleft); // virtual screen
        let mut last_cursor = (
            self.content_height + self.extray - (self.style.border * 2) - 1, // do not consider the borderwidth in the lastcursor of this window
            self.content_width + self.extrax - (self.style.border * 2) - 1,
        );
        // do not consider the padding along the direction

        last_cursor.0 = last_cursor.0.max(0);
        last_cursor.1 = last_cursor.1.max(0);
        let win: &WINDOW = &(0 as WINDOW);

        match &self.content {
            IViewContent::CHIDREN(icomponents) => {
                self.render_children(
                    win,
                    icomponents,
                    &mut topleft,
                    &mut last_cursor,
                    MAX,
                    MIN,
                    Some(event)
                );
            }
            IViewContent::TEXT(_) => {}
        }
    }

    /**
     * handles the given mouse event. Do not pass a non mouse event
     * returns whether to propogate bubbling or not
     *          true: do not bubble
     * Uses DOCUMENT lock()
     */
    pub(crate) fn __handle_mouse_event__(&mut self, event: &mut EVENT) {
        {
            let Some(_) = &event.mevent else {
                panic!("Invalid Handler")
            };
        }

        // LOGLn!("{:p} {:?}", self, event);
        // handle capture
        self.style.handle_event(event, true);
        if !event.propogate {
            return;
        }

        if event.default {
            self.handle_default(event);
        }

        // find the child under the event
        self.transfer_event(event);

        if self.style.render {
            DOCUMENT.lock().unwrap().changed = true;
        }

        // handle bubble
        if event.propogate {
            self.style.handle_event(event, false);
        }
    }
}
