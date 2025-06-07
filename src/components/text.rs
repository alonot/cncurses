use std::sync::{Arc, Mutex};

use crate::{interfaces::{Component, EVENT}, styles::{CSSStyle, STYLE}, IView};

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
    fn __key__(&self) -> Option<&String> {
        self.key.as_ref()
    }
}

impl Text {
    pub fn new(text: String, style: CSSStyle) -> Text {
        let iview = IView::with_style( style, crate::IViewContent::TEXT(text), vec![]);
        Text {
            key: None,
            base_component: iview.build()
        }
    }
    pub fn new_key(key: Option<String>,text: String, style: CSSStyle) -> Text {
        let iview = IView::with_style( style, crate::IViewContent::TEXT(text), vec![]);
        Text {
            key: key,
            base_component: iview.build()
        }
    }
    pub fn new_style_vec(text: String, style: Vec<STYLE>) -> Text {
        let iview = IView::from_text(text, style);
        Text {
            key: None,
            base_component: iview.build()
        }
    }
    pub fn new_key_style_vec(key: Option<String>,text: String, style: Vec<STYLE>) -> Text {
        let iview = IView::from_text(text, style);
        Text {
            key: key,
            base_component: iview.build()
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
}