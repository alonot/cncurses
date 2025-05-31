use std::sync::{Arc, Mutex};

use crate::{interfaces::{STYLE}, nmodels::IView::IView, Component};

/* Text 
 Basic Text which can hold an string
*/
pub struct Text{
    base_component:         Arc<Mutex<IView>>
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
        Text {
            base_component: IView::from_text(text, style).build()
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
}