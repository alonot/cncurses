use std::{any::Any, cell::RefCell, clone, mem::take, rc::Rc, sync::{Arc, Mutex}};

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

pub(crate) struct MENUSTRUCT {
    menu: MENU,
    win: WINDOW
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

pub trait Component : Any + Send {
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
    PERCENT(f32)
}

impl Default for DIMEN {
    fn default() -> Self {
        DIMEN::INT(0)
    }
}

#[derive(Default)]
pub(crate) struct Style
{
    pub(crate) height:                  DIMEN,
    pub(crate) width:                   DIMEN,
    pub(crate) top:                     DIMEN,
    pub(crate) left:                    DIMEN,
    pub(crate) paddingleft:                DIMEN,
    pub(crate) paddingtop:                DIMEN,
    pub(crate) paddingright:                DIMEN,
    pub(crate) paddingbottom:                DIMEN,

    pub(crate) boxsizing:               BOXSIZING,
    pub(crate) flex:                    u32,
    pub(crate) flex_direction:          FLEXDIRECTION,
    pub(crate) background_color:        i32,
    pub(crate) z_index:                 i32,
    pub(crate) onclick:                 Option<Arc<Mutex<dyn FnMut() + Send>>> ,   // should be a clousure
    pub(crate) onscroll:                Option<Arc<Mutex<dyn FnMut() + Send>>> ,  // should be a clousure
    pub(crate) render:                  bool,
    pub(crate) scroll:                  OVERFLOWBEHAVIOUR,
}

pub const FIT_CONTENT:i32 = -1;
pub const MAX_CONTENT:i32 = -2;

impl Style
{
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
            z_index: 0, 
            onclick: None, 
            onscroll: None, 
            render: true, 
            scroll: OVERFLOWBEHAVIOUR::HIDDEN
        }
    }
    pub(crate) fn set_style(&mut self, v: STYLE) {
        match v {
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

pub enum FLEXDIRECTION {
    VERTICAL,
    HORIZONTAL
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
    CONTENTBOX
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
