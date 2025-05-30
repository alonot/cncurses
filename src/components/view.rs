use std::{any::Any, cell::RefCell, rc::Rc, sync::Mutex};

use crate::{interfaces::{IViewContent, Style, STYLE}, nmodels::IView::IView, Component};

/* View 
Basic Block of screen which can contain multiple child
*/
#[derive(Default, Clone)]
pub struct View{
    base_component:                    Rc<Mutex<IView>>
}

impl Component for View {
    fn __call__(&mut self) -> Rc<Mutex<dyn Component>>  {
        panic!("Invalid call to BaseComponent")
    }
    fn __base__(&self) -> Option<Rc<Mutex<IView>>> {
        Some(self.base_component.clone())
    }
}


impl View {
    pub fn new(children: Vec<Rc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build_rciview()
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
    pub fn assign_style(&mut self, style_obj: Style) -> &mut Self {
        self.base_component.lock().unwrap().style = style_obj;
        self
    }
}