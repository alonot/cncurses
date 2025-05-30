use crate::{Component, Style};

/* View 
Basic Block of screen which can contain multiple child
*/
pub struct View<T: FnMut() -> (), S: FnMut() -> ()>{
    children:               Vec<Box<dyn Component>>,
    style:                  Style<T, S>
}

impl <T: FnMut(), S: FnMut()> Component for View<T, S> {
    fn __call__(&self) -> Box<dyn Component>  {
        todo!()
    }
}

impl <T: FnMut(), S: FnMut()> View <T,S> {
    fn new(children: Vec<impl Component>) -> View<T, S> {
        todo!()
    }
}