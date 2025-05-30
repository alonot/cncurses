use crate::Component;

/* Button 
 Basic Button which can hold other Component
*/
pub struct Button{
    child: Box<dyn Component>,
    height: u32,
    width: u32,
    render: bool
}

impl Component for Button {
    fn __call__(&self) -> Box<dyn Component>  {
        todo!()
    }
}

impl Button {
    fn new(child: impl Component) -> Button {
        todo!()
    }
}