use std::{any::Any, cell::RefCell, clone, rc::Rc, sync::{Arc, Mutex}};

use dyn_clone::DynClone;
use ncurses::{newwin, MENU, PANEL, WINDOW};

use crate::{nmodels::IView::IView};


pub trait Stateful: DynClone + Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn Stateful) -> bool;
}
impl<T: Clone + Any + Send + PartialEq> Stateful for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn Stateful) -> bool {
        other.as_any().downcast_ref::<T>().map_or(false, |other| self == other)
    }
}

dyn_clone::clone_trait_object!(Stateful);

pub(crate) enum IViewContent {
    CHIDREN(Vec<Arc<Mutex<IView>>>),
    TEXT(String)
}

impl Default for IViewContent {
    fn default() -> Self {
        IViewContent::TEXT("".to_string())
    }
}

/**
 * Never use this multi-threaded
 */
pub(crate) enum BASICSTRUCT {
    WIN(WINDOW),
    PANEL(PANEL),
    MENU(MENU),
}

unsafe impl Send for BASICSTRUCT {}
unsafe impl Sync for BASICSTRUCT {}


impl Default for BASICSTRUCT {
    fn default() -> Self {
        BASICSTRUCT::WIN(newwin(0, 0, 0, 0))
    }
}

pub trait Component : Any + Send {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>;
    fn __base__(&self) -> Option<Arc<Mutex<IView>>> {
        None
    }
    fn __key__(&self) -> Option<String> {
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

#[derive(Default)]
pub(crate) struct Style
{
    pub(crate) height:                 i32,
    pub(crate) width:                  i32,
    pub(crate) top:                    i32,
    pub(crate) bottom:                 i32,
    pub(crate) left:                   i32,
    pub(crate) right:                  i32,
    pub(crate) background_color:       i32,
    pub(crate) z_index:                i32,
    pub(crate) onclick:  Option<Arc<Mutex<dyn FnMut() + Send>>> ,   // should be a clousure
    pub(crate) onscroll: Option<Arc<Mutex<dyn FnMut() + Send>>> ,  // should be a clousure
    pub(crate) render:                 bool,
    pub(crate) scroll:                 OVERFLOWBEHAVIOUR,
}

const FIT_CONTENT:i32 = -1;
const MAX_CONTENT:i32 = -2;

impl Style
{
    pub(crate) fn default() -> Style {
        Style { 
            height: FIT_CONTENT,
            width: FIT_CONTENT,
            top: 0, 
            bottom: 0, 
            left: 0, 
            right: 0, 
            background_color: 0, 
            z_index: 0, 
            onclick: None, 
            onscroll: None, 
            render: true, 
            scroll: OVERFLOWBEHAVIOUR::HIDDEN
        }
    }
    pub(crate) fn from_style(styles: Vec<STYLE>) -> Style {

        let mut style_obj = Style::default();

        styles.iter().for_each(|v| {

            match v {
                STYLE::HIEGHT(h) => style_obj.height = *h,
                STYLE::WIDTH(w) => style_obj.width = *w,
                STYLE::TOP(t) => style_obj.top = *t,
                STYLE::LEFT(l) => style_obj.left = *l,
                STYLE::BOTTOM(b) => style_obj.bottom = *b,
                STYLE::RIGHT(r) => style_obj.right = *r,
                STYLE::BACKGROUNDCOLOR(bg) => style_obj.background_color = *bg,
                STYLE::ZINDEX(z) => style_obj.z_index = *z,
                STYLE::OVERFLOW(overflow_behaviour) => style_obj.scroll = *overflow_behaviour,
            }

        });

        style_obj
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OVERFLOWBEHAVIOUR {
    VISIBLE,
    HIDDEN,
    SCROLL
}

impl Default for OVERFLOWBEHAVIOUR {
    fn default() -> Self {
        OVERFLOWBEHAVIOUR::HIDDEN
    }
}

pub enum STYLE {
    HIEGHT(i32),
    WIDTH(i32),
    TOP(i32),
    LEFT(i32),
    BOTTOM(i32),
    RIGHT(i32),
    BACKGROUNDCOLOR(i32),
    ZINDEX(i32),
    OVERFLOW(OVERFLOWBEHAVIOUR),
}


/**
 * Hooks struct. Each Component will have its own object of this struct
 */
pub(crate) struct Fiber {
    pub(crate) key : String,
    pub(crate) head: usize,
    pub(crate) state: Vec<Box<dyn Stateful>>,
    pub(crate) changed: bool,
    pub(crate) component: Arc<Mutex<dyn Component>>, // for rendering and re-rendering
    pub(crate) iview:     Option<Arc<Mutex<IView>>>, // Corresponding IView this Component yields
    pub(crate) children:  Vec<Arc<Mutex<Fiber>>>
}

impl Fiber {
    /**
     * Adds the fiber to the global fiber list and returns the index
     */
    pub(crate) fn new(key: String, component: Arc<Mutex<dyn Component>>, changed: bool) -> Arc<Mutex<Fiber>> {
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
