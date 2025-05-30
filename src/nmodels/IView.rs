/***
 * The internal View model
 */

use std::{iter::TakeWhile, mem::take, rc::Rc, sync::Mutex};

use crate::{interfaces::{Component, IViewContent, Style, STYLE}};

#[derive(Clone, Default)]
pub(crate) struct IView
{
    pub(crate) content:                IViewContent,
    pub(crate) children:               Vec<Rc<Mutex<dyn Component>>>, // we be neglected if TViewContent::TEXT
    pub(crate) style:                  Style
}

impl Component for IView {
    /**
     * Panics
     */
    fn __call__(&mut self) -> Rc<Mutex<dyn Component>>  {
        panic!("Invalid Call")
    }
}

impl IView
{
    pub(crate) fn new() -> IView {
        IView { 
        content: IViewContent::TEXT    ("".to_string()), 
        children: vec![],
        style: Style::default(),
        }
    }
    pub(crate) fn from_text(text: String, styles: Vec<STYLE>) -> IView {
        IView { 
        content: IViewContent::TEXT(text), 
        style: Style::from_style(styles),
        children: vec![]
        }
    }
    pub(crate) fn with_style_vec(styles: Vec<STYLE>, content : IViewContent, children: Vec<Rc<Mutex<dyn Component>>>) -> IView{
        IView { 
        content: content, 
        style: Style::from_style(styles) ,
        children: children
        }
    }
    pub(crate) fn build(self) -> Rc<Box<dyn Component>> {
        Rc::new(Box::new(self))
    }
    
    pub(crate) fn build_rciview(self) -> Rc<Mutex<IView>> {
        Rc::new(Mutex::new(self))
    }

    pub(crate) fn from_(p:&IView) -> IView {
        IView { 
            content: p.content.clone(), 
            style: p.style.clone(),
            children: p.children.clone()
        }
    }

    /**
     * Get important parameter of the screen and call render on its children
     */
    fn __render__(&self) -> i32 {

        let content = &self.content;
        let style = &self.style;

        match content {
            IViewContent::CHIDREN(icomponents) => {
                // loop over the children
                icomponents.iter().for_each(|child| {
                    // calls the render function of child
                    // gets the width covered by the child
                    let width = child.__render__();
                    
                    // TODO: fill left width with background color.
                });
            },
            IViewContent::TEXT(txt) => {
                // display the text
            },
        }

        0
    }
}
