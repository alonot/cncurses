use std::{mem::take, sync::{Arc, Mutex}};

        use crate::{interfaces::{Component, ComponentBuilder, EVENT}, styles::{CSSStyle, Style, STYLE}};
use super::view::View;

/* Button 
 Basic Button which can hold other Component
*/
pub struct Button{
    child: Arc<Mutex<dyn Component>>,
    style: Style,
    key: Option<String>
}

impl Component for Button {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>  {
        let mut binding = View::new_key_style_vec(
            self.key.clone(),
            vec![self.child.clone()], 
            vec![STYLE::TABORDER(0)]
        );

        let view = binding.assign_style(take(&mut self.style));
        
        let mview = take(view).build();
        
        mview
    }
}


impl Button {
    pub fn new<T: FnMut(&mut EVENT) + Send +'static>(key: Option<String>,child: Arc<Mutex<dyn Component>>, style: Vec<STYLE>,  onclick: T) -> Button {
        let style_obj = Style::from_style(style);
        
        let btn = Button {
            key: key,
            child: child,
            style: style_obj
        };
        btn.onclick(onclick, false)
    }
    pub fn new_style_vec<T: FnMut(&mut EVENT) + Send +'static>(key: Option<String>,child: Arc<Mutex<dyn Component>>, style: CSSStyle,  onclick: T) -> Button {
        let style_obj = style.create_style();
        
        let btn = Button {
            key: key,
            child: child,
            style: style_obj
        };
        btn.onclick(onclick, false)
    }
    pub fn onclick<T: FnMut(&mut EVENT) + 'static>(mut self, onclick: T, capture:bool) -> Self {
        if capture {
            self.style.onclick_capture = Some(Arc::new(Mutex::new(onclick)));
        } else {
            self.style.onclick_bubble = Some(Arc::new(Mutex::new(onclick)));
        }
        self
    }
    pub fn onscroll<S: FnMut(&mut EVENT) + 'static>(mut self, onscroll: S, capture:bool) -> Self {
        if capture {
            self.style.onscroll_capture = Some(Arc::new(Mutex::new(onscroll)));
        } else {   
            self.style.onscroll_bubble = Some(Arc::new(Mutex::new(onscroll)));
        }
        self
    }
    pub fn onfocus<T: FnMut() + 'static>(mut self, onfocus: T) -> Self {
        self.style.onfocus = Some(Arc::new(Mutex::new(onfocus)));
        self
    }
    pub fn onunfocus<S: FnMut() + 'static>(mut self, onunfocus: S) -> Self {
        self.style.onunfocus = Some(Arc::new(Mutex::new(onunfocus)));
        self
    }
}