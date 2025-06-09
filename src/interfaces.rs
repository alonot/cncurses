use std::{
    any::Any,
    clone,
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use dyn_clone::DynClone;
use ncurses::{MENU, MEVENT, PANEL, WINDOW, init_pair, newwin};

use crate::{_debug_iview, LOGLn, nmodels::iview::IView};

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
    _menu: MENU,
    _win: WINDOW,
}

/**
 * Never use this multi-threaded
 */
pub(crate) enum BASICSTRUCT {
    WIN(WINDOW),
    _PANEL(PANEL),
    _MENU(MENUSTRUCT),
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
        EVENT {
            mevent: None,
            key: ch,
            clientx: 0,
            clienty: 0,
            propogate: true,
            default: true,
        }
    }

    pub fn get_mevent(&self) -> &Option<MEVENT> {
        &self.mevent
    }

    pub fn get_key(&self) -> i32 {
        self.key
    }
    pub fn get_clientx(&self) -> i32 {
        self.clientx
    }
    pub fn get_clienty(&self) -> i32 {
        self.clienty
    }

    pub fn stop_propogation(&mut self) {
        self.propogate = false;
    }

    pub fn prevent_default(&mut self) {
        self.default = false;
    }
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

pub(crate) struct TabElement {
    id: i32,
    iview: Arc<Mutex<IView>>,
}

pub struct Document {
    pub(crate) curr_fiber: Option<Arc<Mutex<Fiber>>>,
    pub(crate) taborder: Vec<TabElement>,
    pub(crate) tabindex: usize,
    pub(crate) unique_id: i32,
    pub(crate) curr_active: Option<Arc<Mutex<IView>>>,

    /** If any iview calls focus then this is set to its id. used by focus */
    pub(crate) next_tab_id: i32,

    pub(crate) color_pairs: LazyLock<Mutex<HashMap<(i16, i16), u16>>>,
    // pub(crate) colors: LazyLock<Mutex<HashMap<(i16, i16, i16), u16>>>,
    pub(crate) total_allowed_pairs: i32,
    // pub(crate) max_supported_colors: i32,
    // pub(crate) curr_color: u16,
    /** Used when color_pairs goes above limit(COLOR_PAIRS) */
    pub(crate) curr_color_pair: u16,

    /** Used when colors goes above limit(COLORS) */
    /** turned true if some changed happen to any IView. (This may occur even though there was no state change in Fiber. Eg of such events. Scroll, Focus) */
    pub(crate) changed: bool,
}

impl Document {
    /**Using in testing only */
    pub(crate) fn _clear_fiber(&mut self) {
        self.curr_fiber = None;
    }

    pub(crate) fn set_active(&mut self, iview: Arc<Mutex<IView>>) {
        self.curr_active = Some(iview);
    }

    pub(crate) fn is_active(&mut self, iview: &Arc<Mutex<IView>>) -> bool {
        if let Some(act) = &self.curr_active {
            return Arc::as_ptr(act).eq(&Arc::as_ptr(iview));
        }
        false
    }

    pub(crate) fn clear_active(&mut self) {
        self.curr_active = None;
    }

    /**
     * returns pair Number
     *
     * if this pair already exists return the pair number
     * OR
     * initialize a new pair using the given pairs
     * if total pairs limit cross then ignores the give pairs and just returns previously initialized pairs in circular format
     *
     */
    pub(crate) fn get_color_pair(&mut self, color_foregorund: i16, color_background: i16) -> i16 {
        let mut c_pairs = self.color_pairs.lock().unwrap();
        let pair = (color_foregorund, color_background);
        if let Some(pair_no) = c_pairs.get(&pair) {
            return *pair_no as i16;
        }
        let pair_no = self.curr_color_pair;
        if c_pairs.len() as i32 >= self.total_allowed_pairs {
            self.curr_color_pair += 1;
            self.curr_color_pair = self.curr_color_pair % c_pairs.len() as u16;
            return pair_no as i16;
        }
        init_pair(pair_no as i16, color_foregorund, color_background);
        c_pairs.insert(pair, pair_no);
        self.curr_color_pair += 1;
        return pair_no as i16;
    }

    /**
     * returns color index
     *
     * red : 0 - 255
     * green: 0 - 255
     * blue: 0 - 255
     *
     * in box : (6 * 6 * 6)
     *
     */
    pub fn get_color(&mut self, red: i16, green: i16, blue: i16) -> i16 {
        let (r, g, b) = (red.min(255) / 51, green.min(255) / 51, blue.min(255) / 51);
        return 16 + (36 * r + 6 * g + b);
    }

    pub(crate) fn clear_color_pairs(&mut self) {
        let mut c_pairs = self.color_pairs.lock().unwrap();
        c_pairs.clear();
        self.curr_color_pair = 0;
    }

    /**
     * Assigns given fiber to global fiber
     * resets the head to 0
     * returns the previous fiber
     */
    pub(crate) fn re_assign_fiber(
        &mut self,
        fiber_lk_opt: Option<Arc<Mutex<Fiber>>>,
    ) -> Option<Arc<Mutex<Fiber>>> {
        if let Some(fiber_lk) = &fiber_lk_opt {
            fiber_lk.lock().unwrap().head = 0;
        }

        let prev_fiber = { self.curr_fiber.clone() };

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

    /** Uses element.lock() */
    pub(crate) fn insert_tab_element(&mut self, element: Arc<Mutex<IView>>) {
        let id = element.lock().unwrap().id;
        if let Some(_) = self.taborder.iter().find(|ielement| ielement.id == id) {
            return;
        }
        // else this would pick up an element after taborder length is increase due to push
        if self.tabindex == self.taborder.len() {
            self.tabindex += 1;
        }

        self.taborder.push(TabElement {
            id: id,
            iview: element,
        });
    }

    /**creates the tab order using the inserted elements so far */
    pub(crate) fn create_tab_order(&mut self) {
        let id = {
            if self.tabindex >= self.taborder.len() {
                -1
            } else {
                self.taborder[self.tabindex].id
            }
        };
        self.taborder
            .sort_by_key(|c| -c.iview.lock().unwrap().style.taborder);
        let prev_next_id = self.next_tab_id;
        self.next_tab_id = id;
        self.focus();
        self.next_tab_id = prev_next_id;
    }

    pub(crate) fn remove_id(&mut self, id: &i32) {
        if let Some(idx) = self.taborder.iter().position(|ielement| ielement.id == *id) {
            // //// removing the children
            let element = &self.taborder[idx];
            let iview = &element.iview;
            let mut to_remove = vec![];
            match &iview.lock().unwrap().content {
                IViewContent::CHIDREN(items) => {
                    items.iter().for_each(|child| {
                        to_remove.push( child.lock().unwrap().id );
                    });
                }
                IViewContent::TEXT(_) => {
                    // no children do nothing
                }
            }

            to_remove.iter().for_each(|id| {
                self.remove_id(id);
            });

            ///////
            // LOGLn!("REMOVING {}",id);
            self.taborder.remove(idx);

            if self.tabindex == idx {
                // going out
                self.tabindex = self.taborder.len();
            }
        }
    }

    /**returns the id of previous tab element
     * -1 if none.
     */
    pub(crate) fn _clear_tab_order(&mut self) -> i32 {
        let mut prev_id = -1;
        if let Some(prev_iview_lk) = self.focused_element() {
            prev_id = prev_iview_lk.lock().unwrap().id;
        };
        // self.tabindex = 0;
        self.taborder.clear();
        self.next_tab_id = -1;
        // LOGLn!("ZERO");
        prev_id
    }

    /**Locks the iview */
    pub(crate) fn advance_tab(&mut self) -> (Option<Arc<Mutex<IView>>>, Option<Arc<Mutex<IView>>>) {
        let prev_iview_lk = self.focused_element();
        if self.next_tab_id != -1 {
            // LOGLn!("START: {} {} {}", self.tabindex, self.taborder.len(), self.next_tab_id);
            self.focus()
        } else {
            self.tabindex += 1;
            self.next_tab_id = -1;
            if self.tabindex > self.taborder.len() {
                self.tabindex = 0;
            }
            // LOGLn!("{} {}", self.tabindex, self.taborder.len());
            (prev_iview_lk, self.focused_element())
        }
    }

    /** Change the focus to given current next_tab_id if available
     */
    pub(crate) fn focus(&mut self) -> (Option<Arc<Mutex<IView>>>, Option<Arc<Mutex<IView>>>) {
        let prev_iview_lk = self.focused_element();
        if let Some(idx) = self
            .taborder
            .iter()
            .position(|ielement| ielement.id == self.next_tab_id)
        {
            self.tabindex = idx;
        }
        self.next_tab_id = -1;
        (prev_iview_lk, self.focused_element())
    }

    pub(crate) fn update_focused_iview(&mut self, iview: Arc<Mutex<IView>>, id: i32) {
        if self.tabindex >= self.taborder.len() {
            return;
        }
        self.taborder[self.tabindex].iview = iview;
        self.taborder[self.tabindex].id = id;
    }

    pub(crate) fn focused_element(&self) -> Option<Arc<Mutex<IView>>> {
        if self.tabindex >= self.taborder.len() {
            return None;
        }
        Some(self.taborder[self.tabindex].iview.clone())
    }
}
