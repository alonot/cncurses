use std::{mem::take, rc::Rc, sync::Mutex};

use crate::{interfaces::{Style, STYLE}, Component};
use super::view::View;

/* Button 
 Basic Button which can hold other Component
*/
pub struct Button{
    child: Rc<Mutex<dyn Component>>,
    style: Style
}

impl Component for Button {
    fn __call__(&mut self) -> Rc<Mutex<dyn Component>>  {
        let mut binding = View::new(
            vec![self.child.clone()], 
            vec![]
        );

        let view = binding.assign_style(take(&mut self.style));
        
        let mview = take(view).build();
        
        mview
    }
}


impl Button {
    pub fn new<T: FnMut() + 'static>(child: Rc<Mutex<dyn Component>>, style: Vec<STYLE>,  onclick: T) -> Button {
        let style_obj = Style::from_style(style);
        
        let  mut btn = Button {
            child: child,
            style: style_obj
        };
        btn.onclick(onclick);

        btn
    }
    pub fn build(&self) -> Rc<Mutex<&dyn Component>> {
        Rc::new(Mutex::new(self))
    }
    pub fn onclick<T: FnMut() + 'static>(&mut self, onclick: T) -> &mut Self {
        self.style.onclick = Some(Rc::new(Box::new(onclick)));
        self
    }
    pub fn onscroll<S: FnMut() + 'static>(&mut self, onscroll: S) -> &mut Self {
        self.style.onscroll = Some(Rc::new(Box::new(onscroll)));
        self
    }
}