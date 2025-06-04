use std::{sync::{Arc, Mutex}};

use crate::{interfaces::{IViewContent, Style, EVENT, STYLE}, nmodels::IView::IView, Component};

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
    pub fn new(children: Vec<Arc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            key: None,
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn new_key(key: Option<String>,children: Vec<Arc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            key: key,
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn onclick<T: FnMut(&mut EVENT) + Send + 'static>(self, onclick: T, capture:bool) -> Self {
        if capture {
            self.base_component.lock().unwrap().style.onclick_capture = Some(Arc::new(Mutex::new(onclick)));
        } else {
            self.base_component.lock().unwrap().style.onclick_bubble = Some(Arc::new(Mutex::new(onclick)));
        }
        self
    }
    pub fn onscroll<S: FnMut(&mut EVENT) + Send + 'static>(self, onscroll: S, capture:bool) -> Self {
        if capture {
            self.base_component.lock().unwrap().style.onscroll_capture = Some(Arc::new(Mutex::new(onscroll)));
        } else {   
            self.base_component.lock().unwrap().style.onscroll_bubble = Some(Arc::new(Mutex::new(onscroll)));
        }
        self
    }
    pub(crate) fn assign_style(&mut self, style_obj: Style) -> &mut Self {
        self.base_component.lock().unwrap().style = style_obj;
        self
    }
}