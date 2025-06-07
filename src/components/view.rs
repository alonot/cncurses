use std::{sync::{Arc, Mutex}};

use crate::{interfaces::{Component, EVENT}, styles::{CSSStyle, Style, STYLE}, IView, IViewContent};



/* View 
Basic Block of screen which can contain multiple child
*/
#[derive(Default, Clone)]
pub struct View{
    base_component:                    Arc<Mutex<IView>>,
    key:                               Option<String>
}

impl Component for View {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>  {
        panic!("Invalid call to BaseComponent")
    }
    fn __base__(&self) -> Option<Arc<Mutex<IView>>> {
        Some(self.base_component.clone())
    }
    fn __key__(&self) -> Option<&String> {
        self.key.as_ref()
    }
}


impl View {
    pub fn new(children: Vec<Arc<Mutex<dyn Component>>>, style: CSSStyle) -> View {
        View {
            key: None,
            base_component: IView::with_style(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn new_key(key: Option<String>,children: Vec<Arc<Mutex<dyn Component>>>, style: CSSStyle) -> View {
        View {
            key: key,
            base_component: IView::with_style(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn new_style_vec(children: Vec<Arc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            key: None,
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn new_key_style_vec(key: Option<String>,children: Vec<Arc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            key: key,
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn is_focused(&self) -> bool {
        self.base_component.lock().unwrap().focused
    }
    pub fn onclick<T: FnMut(&mut EVENT) + 'static>(self, onclick: T, capture:bool) -> Self {
        if capture {
            self.base_component.lock().unwrap().style.onclick_capture = Some(Arc::new(Mutex::new(onclick)));
        } else {
            self.base_component.lock().unwrap().style.onclick_bubble = Some(Arc::new(Mutex::new(onclick)));
        }
        self
    }
    pub fn onscroll<S: FnMut(&mut EVENT) + 'static>(self, onscroll: S, capture:bool) -> Self {
        if capture {
            self.base_component.lock().unwrap().style.onscroll_capture = Some(Arc::new(Mutex::new(onscroll)));
        } else {   
            self.base_component.lock().unwrap().style.onscroll_bubble = Some(Arc::new(Mutex::new(onscroll)));
        }
        self
    }
    pub fn onfocus<T: FnMut() + 'static>(self, onfocus: T) -> Self {
        self.base_component.lock().unwrap().style.onfocus = Some(Arc::new(Mutex::new(onfocus)));
        self
    }
    pub fn onunfocus<S: FnMut() + 'static>(self, onunfocus: S) -> Self {
        self.base_component.lock().unwrap().style.onunfocus = Some(Arc::new(Mutex::new(onunfocus)));
        self
    }
    pub(crate) fn assign_style(&mut self, style_obj: Style) -> &mut Self {
        self.base_component.lock().unwrap().style = style_obj;
        self
    }
}