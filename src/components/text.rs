use std::{rc::Rc, sync::Mutex};

use crate::{interfaces::{STYLE}, nmodels::IView::IView, Component};

/* Text 
 Basic Text which can hold an string
*/
pub struct Text{
    base_component:         Rc<Mutex<IView>>
}

impl Component for Text {
    fn __call__(&mut self) -> Rc<Mutex<dyn Component>>  {
        panic!("Invalid call to BaseComponent")
    }
    fn __base__(&self) -> Option<Rc<Mutex<IView>>> {
        Some(self.base_component.clone())
    }
}

impl Text {
    pub fn new(text: String, style: Vec<STYLE>) -> Text {
        Text {
            base_component: IView::from_text(text, style).build_rciview()
        }
    }
    pub fn build(self) -> Rc<Mutex<dyn Component>> {
        Rc::new(Mutex::new(self))
    }
    pub fn onclick<T: FnMut() + 'static>(&mut self, onclick: T) -> &mut Self {
        self.base_component.lock().unwrap().style.onclick = Some(Rc::new(Box::new(onclick)));
        self
    }
    pub fn onscroll<S: FnMut() + 'static>(&mut self, onscroll: S) -> &mut Self {
        self.base_component.lock().unwrap().style.onscroll = Some(Rc::new(Box::new(onscroll)));
        self
    }
}