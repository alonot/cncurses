use std::{mem::take, sync::{Arc, Mutex}};

use crate::{interfaces::{ComponentBuilder, Style, STYLE}, Component};
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
        let mut binding = View::new_key(
            self.key.clone(),
            vec![self.child.clone()], 
            vec![]
        );

        let view = binding.assign_style(take(&mut self.style));
        
        let mview = take(view).build();
        
        mview
    }
}


impl Button {
    pub fn new<T: FnMut() + Send +'static>(key: Option<String>,child: Arc<Mutex<dyn Component>>, style: Vec<STYLE>,  onclick: T) -> Button {
        let style_obj = Style::from_style(style);
        
        let  mut btn = Button {
            key: key,
            child: child,
            style: style_obj
        };
        btn.onclick(onclick);

        btn
    }
    pub fn onclick<T: FnMut() + Send + 'static>(&mut self, onclick: T) -> &mut Self {
        self.style.onclick = Some(Arc::new(Mutex::new(onclick)));
        self
    }
    pub fn onscroll<S: FnMut() + Send + 'static>(&mut self, onscroll: S) -> &mut Self {
        self.style.onscroll = Some(Arc::new(Mutex::new(onscroll)));
        self
    }
}