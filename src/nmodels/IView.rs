/***
 * The internal View model
 */

use crate::{IComponent, Style};

struct IView<T, S>
where 
T: FnMut(),
S: FnMut()
{
    children:               Vec<Box<dyn IComponent>>,
    style:                  Style<T, S>
}

impl <T, S>IComponent for IView<T, S>
where 
T: FnMut(), 
S: FnMut() 
{
    fn get_children(&self) -> &Vec<Box<dyn IComponent>> {
        &self.children
    }
}
