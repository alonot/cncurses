use crate::Component;

/* Text 
 Basic Text which can hold an string
*/
pub struct Text <'a> {
    text: &'a str, // Lives as long as the object itself
    height: u32,
    width: u32,
    render: bool
}

impl Component for Text <'_> {
    fn __call__(&self) -> Box<dyn Component>  {
        todo!()
    }
}

impl <'a> Text <'a>  {
    fn new(str: &'a str) -> Text<'a> {
        todo!()
    }
}