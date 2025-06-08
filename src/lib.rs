// We'll have a Component that takes in many things as Input works on it and return other component
//

/*
TODO:
 1. Rendering
 2. On Click
 3. On Scroll
*/

use dyn_clone::clone;
use interfaces::{Component, Fiber, IViewContent, Stateful};
use ncurses::{
    ALL_MOUSE_EVENTS, BUTTON1_PRESSED, BUTTON2_PRESSED, COLOR_PAIRS, KEY_BTAB, KEY_CATAB, KEY_CTAB,
    KEY_DOWN, KEY_LEFT, KEY_MOUSE, KEY_RESIZE, KEY_RIGHT, KEY_STAB, KEY_UP, MEVENT, OK, cbreak,
    curs_set, endwin, getch, getmaxyx, getmouse, has_colors, initscr, keypad, mmask_t,
    mouseinterval, mousemask, nodelay, noecho, refresh, start_color, stdscr, use_default_colors,
    wrefresh,
};
use nmodels::iview::IView;
use std::{
    any::TypeId, collections::HashMap, fmt::Debug, i32, mem::take, panic, sync::{Arc, LazyLock, Mutex}
};

use crate::{interfaces::Document, nmodels::iview, styles::Style};
use crate::interfaces::{BASICSTRUCT, EVENT};
use crate::styles::DIMEN;
use crate::styles::STYLE;

pub mod components;
pub mod interfaces;
mod nmodels;
pub mod styles;

#[macro_export]
macro_rules! LOGLn {
    ($val:expr $(,$other: expr)* ) => {
        let var = std::fs::read_to_string("debug.txt").map_or(format!(""), |f| f);
        let _ = std::fs::write("debug.txt", format!(concat!("{}", $val, "\n"), var,$($other, )*));
    };
}

#[macro_export]
macro_rules! LOG {
    ($val:expr $(, $other:expr)* ) => {
        let var = std::fs::read_to_string("debug.txt").map_or(format!(""), |f| f);
        let _ = std::fs::write("debug.txt", format!(concat!("{}", $val), var,$($other, )*));
    };
}

/**
 * Checks and run IView, if Component can be downcasted to IView
 */
fn convert_to_icomponent(v: &Arc<Mutex<dyn Component>>) -> Option<Arc<Mutex<IView>>> {
    if let Some(base) = v.lock().unwrap().__base__() {
        return Some(base);
    }
    None
}

fn get_typeid(node: Arc<Mutex<dyn Component>>) -> TypeId {
    let cn = node.clone();
    let p = cn.lock().unwrap();
    // Cast to Any explicitly to bypass any Component trait issues
    let as_any: &dyn std::any::Any = &*p;
    as_any.type_id()
}

fn get_key(v: &Arc<Mutex<dyn Component>>) -> String {
    v.lock()
        .unwrap()
        .__key__()
        .map_or(format!(""), |f| f.clone())
}

/**
 * Initalize the window
 * uses DOCUMENT.lock()
 */
fn initialize() {
    initscr();
    noecho();
    keypad(stdscr(), true);
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    start_color();
    cbreak();
    nodelay(stdscr(), true); // make getch non-blocking
    use_default_colors();
    mousemask((ALL_MOUSE_EVENTS) as mmask_t, None);
    mouseinterval(0);
    if has_colors() {
        start_color();
        {
            let mut document = DOCUMENT.lock().unwrap();
            document.total_allowed_pairs = COLOR_PAIRS();
        }
        use_default_colors();
        // DOCUMENT.lock().unwrap().has_color = true;
    } else {
        LOGLn!("WARNING: Terminal does not support color");
    }
    refresh();
}

fn _debug_iview(iview:&std::sync::MutexGuard<'_, IView>) {
    LOGLn!(
        "IView_{}({}, ::{:p} {:?} {})",
        iview.id,
        iview.style.render,
        &**iview,
        iview.style.background_color,
        iview.children.len()
    );
}

fn _debug_tree(node: Arc<Mutex<IView>>, tabs: i32) {
    let iview: std::sync::MutexGuard<'_, IView> = node.lock().unwrap();
    for _ in 0..tabs {
        LOG!("\t");
    }
    LOG!("|-");
    match &iview.content {
        interfaces::IViewContent::CHIDREN(items) => {
            _debug_iview(&iview);
            items.iter().for_each(|child| {
                _debug_tree(child.clone(), tabs + 1);
            });
        }
        interfaces::IViewContent::TEXT(txt) => {
            LOGLn!(
                "IView_{}({}, {}, ::{:p})",
                iview.id,
                iview.style.render,
                txt,
                &*iview
            );
        }
    }
}

fn _debug_fiber(node: Arc<Mutex<Fiber>>) {
    let inode = {
        let Some(inode_p) = node.lock().unwrap().iview.clone() else {
            panic!("DEBUG: No IView")
        };
        inode_p
    };
    let fiber = node.lock().unwrap();
    LOGLn!(
        "Fiber({}, {},{:?}, ::{:p} {:?}):",
        fiber.key,
        fiber.changed,
        get_typeid(fiber.component.clone()),
        &*inode.lock().unwrap(),
        Arc::as_ptr(&node)
        // fiber.state.len()
    );
}

fn _debug_fiber_tree(node: Arc<Mutex<Fiber>>, tabs: i32) {
    for _ in 0..tabs {
        LOG!("\t");
    }
    LOG!("|-");
    _debug_fiber(node.clone());

    let fiber = node.lock().unwrap();

    fiber.children.iter().for_each(|child| {
        _debug_fiber_tree(child.clone(), tabs + 1);
    });
}

/**
* Create tree, Keep recursing till we remove all non-base Components.
* View and Buttons' children are expanded to get their IComponents.
* push new Fiber for each component
1. Convert this component to IView
2. Adds parent to this IView
3. **Always** creates a new fiber while parsing the Component. Should only be called if want to create new tree/sub-tree
4. Updates the fiber with iview created.
5. returns this Fiber back to get assigned to its parent

* NO-SIDE EFFECTS
*/
fn create_tree(
    node: Arc<Mutex<dyn Component>>,
    parent: Arc<Mutex<IView>>,
    changed: bool,
) -> Arc<Mutex<Fiber>> {
    // we'll get the current fiber set

    // LOG!(format!("{:?}", as_any(node.clone()).type_id()));
    let currfiber_lk = Fiber::new(get_key(&node), node.clone(), changed);

    call_n_create_with_fiber(node, currfiber_lk.clone(), parent, changed)
}

/**
 * Returns the root IView
 * Root Fiber will be in CURRFIBER
 *
 * **HAS** SIDE-EFFECTS
 */
fn create_render_tree(node: Arc<Mutex<dyn Component>>) -> Arc<Mutex<IView>> {
    let parent = IView::new()
        .set_style(STYLE::HIEGHT(DIMEN::PERCENT(100.)))
        .set_style(STYLE::WIDTH(DIMEN::PERCENT(100.)))
        .build();

    let fiber = create_tree(node, parent.clone(), true);

    let Some(iview) = fiber.lock().unwrap().iview.clone() else {
        panic!("CREATERENDERTREEE: no iview in given Componenet")
    };

    parent.lock().unwrap().content = IViewContent::CHIDREN(vec![iview]);

    let _ = DOCUMENT.lock().unwrap().re_assign_fiber(Some(fiber));

    parent
}

/**
1. calls the given node with given fiber
2. Assigns the parent IView created
3. assigns the Iview created to given fiber
4. Creates new Fibers for child. Must be called only when new sub-tree/ tree is required

No SIDE EFFECTS
*/
fn call_n_create_with_fiber(
    node: Arc<Mutex<dyn Component>>,
    fiber_lk: Arc<Mutex<Fiber>>,
    parent: Arc<Mutex<IView>>,
    changed: bool,
) -> Arc<Mutex<Fiber>> {
    let iview = if let Some(base_lk) = convert_to_icomponent(&node) {
        let mut base = base_lk.lock().unwrap();
        // we need to assign parent to this base
        base.parent = Some(parent);

        {
            let mut curr_fiber = fiber_lk.lock().unwrap();
            curr_fiber.children.clear();
        }

        // iterate over the children of node
        let children: Vec<Arc<Mutex<IView>>> = base
            .children
            .iter()
            .map(|child| {
                let fiber = create_tree(child.clone(), base_lk.clone(), changed);

                let Some(iview) = fiber.lock().unwrap().iview.clone() else {
                    panic!("CREATETREEE: no iview in given Componenet")
                };

                // adds this new fiber as child
                let mut curr_fiber = fiber_lk.lock().unwrap();
                curr_fiber.children.push(fiber);

                iview
            })
            .collect();

        let content = &mut base.content;

        match content {
            interfaces::IViewContent::CHIDREN(iviews) => {
                children.iter().for_each(|child| {
                    iviews.push(child.clone());
                });
            }
            interfaces::IViewContent::TEXT(_) => {
                // DO Nothing
            }
        }

        base_lk.clone()
    } else {
        let prev_fiber = DOCUMENT
            .lock()
            .unwrap()
            .re_assign_fiber(Some(fiber_lk.clone()));

        let new_node = node.lock().unwrap().__call__();

        // restore actual fiber back
        DOCUMENT.lock().unwrap().assign_fiber(prev_fiber);

        let child_fiber = create_tree(new_node, parent, changed);

        let Some(iview) = child_fiber.lock().unwrap().iview.clone() else {
            panic!("CREATETREEE: no iview in given Componenet")
        };

        // add this new fiber as child
        let mut curr_fiber = fiber_lk.lock().unwrap();
        curr_fiber.children.clear();
        curr_fiber.children.push(child_fiber);

        iview
    };

    let cfiber_lk = fiber_lk.clone();
    let mut fiber = cfiber_lk.lock().unwrap();
    fiber.add_iview(iview);

    fiber_lk
}

fn is_not_same(
    new_node: &Arc<Mutex<dyn Component + 'static>>,
    child_lk: &Arc<Mutex<Fiber>>,
) -> bool {
    let new_key = &get_key(&new_node);

    // check if key is different or type is different
    let child = child_lk.lock().unwrap();
    let prev_key = &child.key;

    new_key != prev_key || get_typeid(child.component.clone()) != get_typeid(new_node.clone())
}

/**
 * Update the given base component with fiber's chidlren
 */
fn update_child(fiber_lk: Arc<Mutex<Fiber>>) {
    let fiber = fiber_lk.lock().unwrap();
    if let Some(base_lk) = convert_to_icomponent(&fiber.component) {
        // update the parent
        let cbase_lk = base_lk.clone();
        {
            let mut base = cbase_lk.lock().unwrap();
            let content = &mut base.content;

            match content {
                interfaces::IViewContent::CHIDREN(iviews) => {
                    // extract the possibly changed iviews
                    let iview_children: Vec<Arc<Mutex<IView>>> = fiber
                        .children
                        .iter()
                        .map(|child_fiber_lk| {
                            let child_fiber = child_fiber_lk.lock().unwrap();
                            let Some(iview) = child_fiber.iview.clone() else {
                                panic!("CREATETREE: no iview in given Component")
                            };
                            iview
                        })
                        .collect();

                    iviews.clear();
                    iview_children.into_iter().for_each(|child| {
                        iviews.push(child);
                    });
                }
                interfaces::IViewContent::TEXT(_) => {
                    // DO Nothing
                    // because this would have no children
                }
            }
        }
    }
}

/**
 * Takes a fiber as input and if its change is on
 * then recreates the component but with a twist than `create_tree`
 * This time it checks if fiber needs to be changed or not using key and type of Component
 * If yes the creates a new Fiber else returns the same
 */
fn check_for_change(fiber_lk: Arc<Mutex<Fiber>>, parent: Arc<Mutex<IView>>) -> bool {
    let mut changed;

    let component;
    {
        let mut fiber = fiber_lk.lock().unwrap();

        changed = fiber.changed;

        fiber.changed = false;

        component = fiber.component.clone();
    }
    let mut newly_created = false;

    if changed {
        let iview = if let Some(base_lk) = convert_to_icomponent(&component) {
            // if changed then fetch the new children from the base component

            // parent would be in fiber's iview

            let mut base = base_lk.lock().unwrap();

            base.parent = Some(parent);
            let mut new_children = vec![];
            {
                let mut fiber = fiber_lk.lock().unwrap();
                let curr_fiber_children = &mut fiber.children;

                let mut i = 0;

                // iterate over the children of node
                let children: Vec<Arc<Mutex<IView>>> = base
                    .children
                    .iter()
                    .map(|new_node| {
                        // we have to re render all the children or change their states if different
                        let is_not_same_child = {
                            if i <= curr_fiber_children.len() {
                                let child_lk = curr_fiber_children[i].clone();
                                is_not_same(new_node, &child_lk)
                            } else {
                                true
                            }
                        };

                        let iview = if is_not_same_child {
                            // since parent will unmount hence children would also unmount
                            let fiber: Arc<Mutex<Fiber>> =
                                create_tree(new_node.clone(), base_lk.clone(), true);

                            let Some(iview) = fiber.lock().unwrap().iview.clone() else {
                                panic!("CREATETREEE: no iview in given Componenet")
                            };
                            // adds this new fiber as child
                            new_children.push(fiber);

                            iview
                        } else {
                            // code never reaches here if i >= len
                            // just update this fiber
                            // however the parent to this fiber will be base_lk
                            let child_fiber_lk = &curr_fiber_children[i];
                            {
                                let mut fiber = child_fiber_lk.lock().unwrap();
                                fiber.changed = true;
                                fiber.component = new_node.clone();
                            }

                            check_for_change(child_fiber_lk.clone(), base_lk.clone());
                            // adds this new fiber as child
                            new_children.push(child_fiber_lk.clone());

                            let Some(child_iview) = child_fiber_lk.lock().unwrap().iview.clone()
                            else {
                                panic!("CHECKFORCHANGE: No IView")
                            };
                            child_iview
                        };
                        i += 1;

                        iview
                    })
                    .collect();

                // LOGLn!(
                //     "{:?} {} {:p}",
                //     Arc::as_ptr(&fiber_lk),
                //     children.len(),
                //     &*base
                // );
                let content = &mut base.content;

                match content {
                    interfaces::IViewContent::CHIDREN(iviews) => {
                        iviews.clear();
                        children.iter().for_each(|child| {
                            iviews.push(child.clone());
                        });
                    }
                    interfaces::IViewContent::TEXT(_) => {
                        // DO Nothing
                        // already fiber.iview is updated
                    }
                }
            }

            {
                let mut fiber = fiber_lk.lock().unwrap();
                fiber.children = new_children;
            }
            drop(base);
            // debug_tree(base_lk.clone(), 0);
            base_lk.clone()
        } else {
            // this is not base component.

            // only 1 child
            let prev_fiber = {
                DOCUMENT
                    .lock()
                    .unwrap()
                    .re_assign_fiber(Some(fiber_lk.clone()))
            };

            let new_node = component.lock().unwrap().__call__();

            DOCUMENT.lock().unwrap().assign_fiber(prev_fiber);

            let cfiber_lk = fiber_lk.clone();
            let mut fiber = cfiber_lk.lock().unwrap();

            let child_lk = fiber.children[0].clone();

            let is_not_same_child = is_not_same(&new_node, &child_lk);

            if is_not_same_child {
                // create new tree
                let child_fiber_lk = create_tree(new_node, parent, false); // somewhere inside the IView would get filled by its child_lk

                newly_created = true;
                // // add this new fiber as child_lk
                fiber.children.clear(); // destroys the previous sub-tree from this node
                fiber.children.push(child_fiber_lk); // adds the new sub_tree
            } else {
                // preserve the state
                // just change the component of the child_lk and changed to true
                {
                    let mut child = child_lk.lock().unwrap();
                    child.changed = true;
                    // let Some(iview) = child.iview.clone() else {
                    //     panic!("asd")
                    // };
                    // LOGLn!(
                    //     "{:?} ___ {:p}",
                    //     Arc::as_ptr(&fiber_lk),
                    //     &*iview.lock().unwrap()
                    // );
                    child.component = new_node.clone();
                }

                check_for_change(child_lk.clone(), parent);
            }

            let child_lk = fiber.children[0].clone();
            let child_fiber = child_lk.lock().unwrap();
            let Some(iview) = child_fiber.iview.clone() else {
                panic!("CREATETREEE: no iview in given Componenet")
            };
            iview
            // if same then change will be called below
        };

        let mut fiber = fiber_lk.lock().unwrap();

        if let Some(prev_iview) = &fiber.iview {
            let mut document = DOCUMENT.lock().unwrap();
            if document.is_active(prev_iview) {
                document.set_active(iview.clone());
            }
            if let Some(focus) = document.focused_element() {
                if Arc::as_ptr(&focus).eq(&Arc::as_ptr(&prev_iview)) {
                    // if focused then update 
                    document.update_focused_iview(iview.clone(), iview.lock().unwrap().id);
                }

            }
            REMOVEINDEX.lock().unwrap().push(prev_iview.lock().unwrap().id);
            
            if !newly_created {
                let mut iv = iview.lock().unwrap();
                iv.fill_box_infos_from_other(&prev_iview.lock().unwrap());
            }

        }

        // // add this iview to this fiber
        let mut iv = iview.lock().unwrap();
        iv.style.render = true;
        fiber.iview = Some(iview.clone());
    } else {
        {
            let fiber = fiber_lk.lock().unwrap();
            // decide the parent...
            let child_parent = if let Some(base_lk) = convert_to_icomponent(&component) {
                // if this is base component then parent will be this IView
                base_lk
            } else {
                // if this is Component then the parent is incoming parent
                parent
            };

            fiber.children.iter().for_each(|child| {
                changed |= check_for_change(child.clone(), child_parent.clone());
            });
        }
        // if changed then we need to update the parent list
        if changed {
            update_child(fiber_lk.clone());
        }
    }

    changed
}

/**
 * takes root IView as input and updates the IView tree
 * Assumes the root fiber is in global CURR_FIBER.
 * HAS SIDE_EFFECTS
 */
fn diff_n_update(root: Arc<Mutex<IView>>) -> bool {
    let fiber_lk = {
        let document = DOCUMENT.lock().unwrap();
        let Some(currfib_lk) = document.curr_fiber.clone() else {
            panic!("DIFFTREE: No Fiber found")
        };
        currfib_lk
    };

    let changed = check_for_change(fiber_lk.clone(), root.clone());
    if changed {
        // update the root
        let Some(iview) = fiber_lk.lock().unwrap().iview.clone() else {
            panic!("DIFFTREE: No Iview")
        };

        root.lock().unwrap().content = IViewContent::CHIDREN(vec![iview]);
    }

    changed
}

/**
 * Init the IView tree's structure
 */
fn tree_refresh(root: Arc<Mutex<IView>>) -> (i32, i32, bool) {
    let x = &mut 0;
    let y = &mut 0;
    refresh();
    getmaxyx(stdscr(), y, x);
    {
        let mut document = DOCUMENT.lock().unwrap();
        document.clear_color_pairs();
        // document._clear_tab_order();
    };
    let res = root.lock().unwrap().__init__(*y, *x);
    if res.2 {
        {
            let mut document = DOCUMENT.lock().unwrap();
            REMOVEINDEX.lock().unwrap().iter().for_each(|id| {
                // remove this id and its children
                document.remove_id(id);
            });
            document.create_tab_order();
        }

        let _ = root.lock().unwrap().__render__();
        let Some(basic_struct) = &root.lock().unwrap().basic_struct else {
            panic!("NO window at root");
        };
        let BASICSTRUCT::WIN(win) = basic_struct else {
            panic!("NO window at root");
        };
        wrefresh(*win);
        refresh();
    }
    DOCUMENT.lock().unwrap().changed = false;
    res
}

/**
 * Bubbles up from current active to the parent
 */
fn handle_event(iview_lk: Arc<Mutex<IView>>, event: &mut EVENT) {
    let iview = iview_lk.lock().unwrap();

    iview.style.handle_event(event, false);

    let Some(parent) = iview.parent.clone() else {
        return;
    };

    if event.propogate {
        handle_event(parent, event);
    }
}

pub(crate) fn handle_focus_change(
    prev_iview: Option<Arc<Mutex<IView>>>,
    new_iview: Option<Arc<Mutex<IView>>>,
) {
    if {
        if let Some(iview_lk) = prev_iview.clone() {
            if let Some(iview_lk_new) = new_iview.clone() {
                Arc::as_ptr(&iview_lk).eq(&Arc::as_ptr(&iview_lk_new))
            } else {
                false
            }
        } else {
            false
        }
    } {
        return;
    }
    if let Some(iview_lk) = prev_iview {
        let mut iview = iview_lk.lock().unwrap();
        iview.focused = false;
        if let Some(onunfocus) = iview.style.onunfocus.clone() {
            onunfocus.lock().unwrap()();
        }
    }
    if let Some(iview_lk) = new_iview {
        let mut iview = iview_lk.lock().unwrap();
        iview.focused = true;
        if let Some(onfocus) = iview.style.onfocus.clone() {
            onfocus.lock().unwrap()();
        }
        if matches!(iview.style.overflow, styles::OVERFLOWBEHAVIOUR::SCROLL) {
            DOCUMENT.lock().unwrap().set_active(iview_lk.clone());
        }
    }
}

/**
 * returns whether to exit the program
 */
fn handle_keyboard_event(ch: i32) -> bool {
    let focused_iview = {
        let document = DOCUMENT.lock().unwrap();
        let iview = document.focused_element();
        iview
    };
    let mut event = EVENT::new(ch);
    if let Some(iview) = focused_iview.clone() {
        handle_event(iview, &mut event);
    }
    // handle regular functionality if default is on
    const TAB: i32 = '\t' as i32;

    if event.default {
        
        match ch {
            KEY_BTAB | KEY_CTAB | KEY_STAB | KEY_CATAB | TAB => {
                let (prev_iview, new_iview) = {
                    let mut document = DOCUMENT.lock().unwrap();
                    document.advance_tab()
                };
                handle_focus_change(prev_iview, new_iview);
            }
            KEY_UP | KEY_DOWN | KEY_RIGHT | KEY_LEFT => {
                // scroll on current active element
                if let Some(iview) = {
                    let document = DOCUMENT.lock().unwrap();
                    let iview = document.curr_active.clone();
                    iview
                } {
                    iview.lock().unwrap().handle_default(&mut event);
                }
            }
            val if val == 'q' as i32 => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/**
 * returns true if to exit the app
 */
fn handle_events(root: Arc<Mutex<IView>>) -> bool {
    let ch = getch();
    match ch {
        KEY_RESIZE => {
            initialize();
            tree_refresh(root.clone());
        }
        KEY_MOUSE => {
            let mut mevent = MEVENT {
                id: 0,
                x: 0,
                y: 0,
                z: 0,
                bstate: 0,
            };

            if getmouse(&mut mevent) == OK {
                let mut event = EVENT::new(ch);
                event.mevent = Some(mevent);
                event.clientx = mevent.x;
                event.clienty = mevent.y;
                if mevent.bstate & BUTTON1_PRESSED as u32 > 0
                    || mevent.bstate & BUTTON2_PRESSED as u32 > 0
                {
                    // if button clicked the active will be set by `__handle_mouse_event__`
                    DOCUMENT.lock().unwrap().clear_active();
                }

                root.lock().unwrap().__handle_mouse_event__(&mut event);
            }
        }
        val => {
            // call the keyboard handler
            if handle_keyboard_event(val) {
                return true;
            }
        }
    }
    return false;
}

pub(crate) static REMOVEINDEX: Mutex<Vec<i32>> = Mutex::new(vec![]);

/************  Public Functions  ********** */

pub static DOCUMENT: Mutex<Document> = Mutex::new(Document {
    curr_fiber: None,
    tabindex: 0,
    taborder: vec![],
    unique_id: 0,
    next_tab_id: -1,
    changed: true,
    curr_active: None,
    color_pairs: LazyLock::new(|| Mutex::new(HashMap::<(i16, i16), u16>::new())),
    total_allowed_pairs: 0,
    curr_color_pair: 0,
});

/**
 * Takes a clonable value and stores its clone
 * on subsequent calls the value is cloned and then sent back to the user
 */
pub fn use_state<T: Stateful + Debug>(init_val: T) -> (T, Arc<dyn Fn(T)>) {
    // extracting the Components Fiber

    let currfiber_lk = {
        let document = DOCUMENT.lock().unwrap();
        let Some(currfib_lk) = document.curr_fiber.clone() else {
            panic!("SET STATE: No fiber associated with the component")
        };
        currfib_lk
    };

    let curr_fiber_lk_clone = currfiber_lk.clone();

    let mut currfiber = curr_fiber_lk_clone.lock().unwrap();

    let curr_hook = currfiber.head;

    // add new entry if required
    if currfiber.head == currfiber.state.len() {
        currfiber.state.push(Box::new(clone(&init_val)));
        currfiber.head += 1;
    }

    let box_value = &currfiber.state[curr_hook as usize];

    let Some(downcasted_val) = box_value.as_any().downcast_ref::<T>() else {
        panic!("SET STATE: Unable to downcast to correct type")
    };

    // create the closure
    let set_value = move |val: T| {
        // move to get ownership of `curr_hook` variable

        {
            // debug_fiber(currfiber_lk.clone());
            let mut currfiber = currfiber_lk.lock().unwrap();

            if curr_hook == currfiber.state.len() {
                return;
            }

            let box_value = &mut currfiber.state[curr_hook as usize];
            if val.eq(&**box_value) {
                return;
            }
            *box_value = Box::new(clone(&val));
            currfiber.changed = true; // to re render this section
        }
        // LOGLn!("{:?}", Arc::as_ptr(&currfiber_lk));
    };

    return (clone(downcasted_val), Arc::new(set_value));
}

/**
 * Takes in a Component as input and call it
*/
pub fn run(app: impl Component) {
    let _ = std::fs::write("debug.txt", "");

    initialize();

    let node: Arc<Mutex<dyn Component>> = Arc::new(Mutex::new(app));

    let root = create_render_tree(node);
    // let rootc = root.clone();
    panic::set_hook(Box::new(move |info| {
        endwin();
        println!("{}", info);
    }));

    loop {
        // if change, get the tree from the app.
        // diff the tree to get the changed components
        let mut changed = diff_n_update(root.clone());
        // handle click and scroll
        if handle_events(root.clone()) {
            break;
        }

        changed |= DOCUMENT.lock().unwrap().changed;

        
        // if changes, render the changed portion
        if changed {
            // _debug_tree( root.clone(), 0);
            let _ = tree_refresh(root.clone());
            // {
            //     let Some(fiber) = DOCUMENT.lock().unwrap().curr_fiber.clone() else {
            //         panic!("No fiber")
            //     };

            //     _debug_fiber_tree(fiber.clone(), 0);
            // }
        }
    }

    endwin();
}

/**
 * Do not run these paralelly because they are working on same global variable.
 * Hence Will result in undefined behaviour.
 */
#[cfg(test)]
mod test {
    use std::{
        panic,
        sync::{Arc, Mutex},
        vec,
    };

    use ncurses::{COLOR_MAGENTA, COLOR_RED, endwin};

    use crate::{
        DOCUMENT,
        components::{text::Text, view::View},
        initialize,
        interfaces::{Component, ComponentBuilder},
        run,
        styles::{CSSStyle, DIMEN, FLEXDIRECTION, OVERFLOWBEHAVIOUR, STYLE},
        use_state,
    };

    struct DemoApp1 {
        pub val: i32,
    }

    impl Component for DemoApp1 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (_p1, _setp1) = use_state::<i32>(self.val);

            View::new_style_vec(vec![], vec![]).build()
        }
    }

    struct DemoApp2 {
        pub _val: String,
    }

    impl Component for DemoApp2 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p1, setp1) = use_state("Namaste".to_string());

            setp1("Namaste hai bhai Sare ne!".to_string());

            let color = DOCUMENT.lock().unwrap().get_color(255, 60, 0);

            if p1 == "Namaste hai bhai Sare ne!" {
                View::new_style_vec(
                    vec![
                        View::new_style_vec(
                            vec![
                                Text::new_style_vec(
                                    "Rama".to_string(),
                                    vec![STYLE::TEXTCOLOR(color)],
                                )
                                .build(),
                            ],
                            vec![],
                        )
                        .build(),
                        Text::new_style_vec("Vadakam".to_string(), vec![]).build(),
                        Text::new_style_vec("Vadakam".to_string(), vec![]).build(),
                    ],
                    vec![STYLE::FLEXDIRECTION(FLEXDIRECTION::HORIZONTAL)],
                )
                .build()
            } else {
                View::new_key_style_vec(
                    Some("P".to_string()),
                    vec![Text::new_style_vec("Vadakam".to_string(), vec![]).build()],
                    vec![],
                )
                .build()
            }
        }
    }

    struct DemoApp3 {
        pub v1: String,
    }

    impl Component for DemoApp3 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p, setp) = use_state(0);
            let color = if p == 0 {
                COLOR_MAGENTA
            } else {
                DOCUMENT.lock().unwrap().get_color(255, 120, 0)
            };
            let p1 = setp.clone();

            View::new(
                vec![
                    DemoApp1 { val: 0 }.build(),
                    DemoApp2 {
                        _val: self.v1.clone(),
                    }
                    .build(),
                    Text::new_style_vec("Hello askdjnakjsncasjcas aasmdkancaksjdncjasdckjasdbcjasbcdjsbdchjsbdj ppp ooo ooo lll iii asdnakjsdlnc jasdncljlnasdcjans ljdc asjkdc nljaskd cja sndjckasnlcjk qqq asncjkasnciunsciuasndcjnasdvcabsdjbcajsdbcjasdcjasbxcnxbccasdbciausnaskdnvjkasbcmn xcjknizxjn kasnuijcnw www".to_string(), vec![STYLE::WIDTH(DIMEN::INT(10)), STYLE::OVERFLOW(OVERFLOWBEHAVIOUR::SCROLL),STYLE::HIEGHT(DIMEN::INT(8)), STYLE::TEXTCOLOR(color), STYLE::MARGINTOP(DIMEN::INT(5))]).onclick(move |_e| {
                            LOGLn!("I was Called");
                            setp(10);
                    }, true).build()
                ],
                CSSStyle {
                    taborder: 0,
                    height: "50%",
                    padding: "10 10 10 10",
                    boxsizing: "border-box",
                    border: 1,  // assuming true means border width of 1
                    border_color: COLOR_RED,  // assuming COLOR_RED is an i16 constant
                    ..Default::default()
                },
            ).onfocus(move || {
                            LOGLn!("I was Called");
                            p1(10);
                    })
            .build()
        }
    }

    fn _clear() {
        DOCUMENT.lock().unwrap()._clear_fiber();
    }

    #[test]
    fn test_create_tree1() {
        let _ = std::fs::write("debug.txt", "");
        panic::set_hook(Box::new(|info| {
            endwin();
            LOGLn!("{}", info);
        }));

        initialize();
        // clear();
        let dm = DemoApp3 {
            v1: format!("Namaste"),
        };
        run(dm);
    }
}
