/***
 * The internal View model
 */

use std::sync::{Arc, Mutex};

use crate::interfaces::{BASICSTRUCT, Component, IViewContent, STYLE, Style};

#[derive(Default)]
pub(crate) struct IView {
    pub(crate) content: IViewContent,
    pub(crate) style: Style,
    pub(crate) children: Vec<Arc<Mutex<dyn Component>>>, // will be neglected if TViewContent::TEXT
    pub(crate) parent: Option<Arc<Mutex<IView>>>,
    pub(crate) basic_struct: BASICSTRUCT
}

impl IView {
    pub(crate) fn new() -> IView {
        IView {
            content: IViewContent::TEXT("".to_string()),
            children: vec![],
            style: Style::default(),
            parent: None,
            basic_struct: BASICSTRUCT::default()
        }
    }
    pub(crate) fn from_text(text: String, styles: Vec<STYLE>) -> IView {
        IView {
            content: IViewContent::TEXT(text),
            style: Style::from_style(styles),
            children: vec![],
            parent: None,
            basic_struct: BASICSTRUCT::default()
        }
    }
    pub(crate) fn with_style_vec(
        styles: Vec<STYLE>,
        content: IViewContent,
        children: Vec<Arc<Mutex<dyn Component>>>,
    ) -> IView {
        IView {
            content: content,
            style: Style::from_style(styles),
            children: children,
            parent: None,
            basic_struct: BASICSTRUCT::default()
        }
    }

    pub(crate) fn build(self) -> Arc<Mutex<IView>> {
        Arc::new(Mutex::new(self))
    }

    pub(crate) fn attach_parent(&mut self, parent: Arc<Mutex<IView>>) -> &Self {
        self.parent = Some(parent);
        self
    }

    /**
     * Allocates actual ncurses window/panel/menu/form
     */
    pub(crate) fn __init__(&mut self) {

    }

    /**
     * Get important parameter of the screen and call render on its children
     */
    pub(crate) fn __render__(&mut self) -> i32 {
        let content = &self.content;
        let style = &self.style;

        match content {
            IViewContent::CHIDREN(icomponents) => {
                // loop over the children
                icomponents.iter().for_each(|child| {
                    // calls the render function of child
                    // gets the width covered by the child
                    let width = child.lock().unwrap().__render__();

                    if self.style.render {
                        // TODO: fill left width with background color.
                    }
                });
            }
            IViewContent::TEXT(txt) => {
                if self.style.render {
                    // display the text
                }
            }
        }
        self.style.render = false;

        0
    }
}
