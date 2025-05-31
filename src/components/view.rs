use std::{sync::{Arc, Mutex}};

use crate::{interfaces::{IViewContent, Style, STYLE}, nmodels::IView::IView, Component};

/* View 
Basic Block of screen which can contain multiple child
*/
#[derive(Default, Clone)]
pub struct View{
    base_component:                    Arc<Mutex<IView>>
}

impl Component for View {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>  {
        panic!("Invalid call to BaseComponent")
    }
    fn __base__(&self) -> Option<Arc<Mutex<IView>>> {
        Some(self.base_component.clone())
    }
}


impl View {
    pub fn new(children: Vec<Arc<Mutex<dyn Component>>>, style: Vec<STYLE>) -> View {
        View {
            base_component: IView::with_style_vec(style, IViewContent::CHIDREN(vec![]), children).build()
        }
    }
    pub fn onclick<T: FnMut() + Send + 'static>(&mut self, onclick: T) -> &mut Self {
        self.base_component.lock().unwrap().style.onclick = Some(Arc::new(Mutex::new(onclick)));
        self
    }
    pub fn onscroll<S: FnMut() + Send + 'static>(&mut self, onscroll: S) -> &mut Self {
        self.base_component.lock().unwrap().style.onscroll = Some(Arc::new(Mutex::new(onscroll)));
        self
    }
    pub(crate) fn assign_style(&mut self, style_obj: Style) -> &mut Self {
        self.base_component.lock().unwrap().style = style_obj;
        self
    }
}