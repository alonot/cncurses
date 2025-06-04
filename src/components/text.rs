use std::{mem::take, sync::{Arc, Mutex}};

use crate::{interfaces::{EVENT, STYLE}, nmodels::IView::IView, Component};

/* Text 
 Basic Text which can hold an string
*/
pub struct Text{
    base_component:         Arc<Mutex<IView>>,
    key:                    Option<String>
}

impl Component for Text {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>  {
        panic!("Invalid call to BaseComponent")
    }
    fn __base__(&self) -> Option<Arc<Mutex<IView>>> {
        Some(self.base_component.clone())
    }
}

impl Text {
    pub fn new(text: String, style: Vec<STYLE>) -> Text {
        let iview = IView::from_text(text, style);
        Text {
            key: None,
            base_component: iview.build()
        }
    }
    pub fn new_key(key: Option<String>,text: String, style: Vec<STYLE>) -> Text {
        let iview = IView::from_text(text, style);
        Text {
            key: key,
            base_component: iview.build()
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
}