/***
 * The internal View model
 */

use std::{sync::{Arc, Mutex}};

use crate::{interfaces::{Component, IViewContent, Style, STYLE}};

#[derive(Clone, Default)]
pub(crate) struct IView
{
    pub(crate) content:                IViewContent,
    pub(crate) style:                  Style,
    pub(crate) children:               Vec<Arc<Mutex<dyn Component>>>, // will be neglected if TViewContent::TEXT
    pub(crate) parent:                 Option<Arc<Mutex<IView>>>,
}

impl Component for IView {
    /**
     * Panics
     */
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>>  {
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
        parent: None
    }
}
pub(crate) fn from_text(text: String, styles: Vec<STYLE>) -> IView {
    IView { 
        content: IViewContent::TEXT(text), 
        style: Style::from_style(styles),
        children: vec![],
        parent: None
    }
}
pub(crate) fn with_style_vec(styles: Vec<STYLE>, content : IViewContent, children: Vec<Arc<Mutex<dyn Component>>>) -> IView{
    IView { 
        content: content, 
        style: Style::from_style(styles) ,
        children: children,
        parent: None
        }
    }
    
    pub(crate) fn build(self) -> Arc<Mutex<IView>> {
        Arc::new(Mutex::new(self))
    }

    pub(crate) fn from_(p:&IView) -> IView {
        IView { 
            content: p.content.clone(), 
            style: p.style.clone(),
            children: p.children.clone(),
            parent: p.parent.clone()
        }
    }

    pub(crate) fn attach_parent(&mut self, parent:Arc<Mutex<IView>>) -> &Self {
        self.parent = Some(parent);
        self
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
                    let width = child.lock().unwrap().__render__();
                    
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
