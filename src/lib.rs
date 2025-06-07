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
    ALL_MOUSE_EVENTS, BUTTON4_PRESSED, BUTTON5_PRESSED, COLOR_PAIRS, COLORS, KEY_BTAB, KEY_MOUSE,
    KEY_MOVE, KEY_RESIZE, MEVENT, OK, REPORT_MOUSE_POSITION, cbreak, curs_set, endwin, getch,
    getmaxyx, getmouse, has_colors, initscr, keypad, mmask_t, mouseinterval, mousemask, nodelay,
    noecho, pair_content, refresh, start_color, stdscr, use_default_colors, wrefresh,
};
use nmodels::IView::IView;
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    i32,
    sync::{Arc, LazyLock, Mutex},
};

use crate::interfaces::{BASICSTRUCT, EVENT};
use crate::interfaces::Document;
use crate::styles::STYLE;
use crate::styles::CSSStyle;
use crate::styles::DIMEN;

pub mod components;
pub mod interfaces;
pub mod styles;
mod nmodels;

#[macro_export]
macro_rules! LOGLn {
    ($val:expr) => {
        let var = std::fs::read_to_string("debug.txt").map_or(format!(""), |f| f);
        let _ = std::fs::write("debug.txt", format!("{var}{}\n", $val));
    };
}

#[macro_export]
macro_rules! LOG {
    ($val:expr) => {
        let var = std::fs::read_to_string("debug.txt").map_or(format!(""), |f| f);
        let _ = std::fs::write("debug.txt", format!("{var}{}", $val));
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
        LOGLn!(format!("WARNING: Terminal does not support color"));
    }
    refresh();
    
    CSSStyle{
        border_color : "",
        flex_direction: "",
        padding: "",
        width: "",
        ..Default::default()
    };
}

fn debug_tree(node: Arc<Mutex<IView>>, tabs: i32) {
    let iview = node.lock().unwrap();
    for _ in 0..tabs {
        LOG!("\t");
    }
    LOG!("|-");
    match &iview.content {
        interfaces::IViewContent::CHIDREN(items) => {
            LOGLn!(format!(
                "IView_{}({}, ::{:p} {:?})",
                iview.id, iview.style.render, &*iview, iview.style.height
            ));
            items.iter().for_each(|child| {
                debug_tree(child.clone(), tabs + 1);
            });
        }
        interfaces::IViewContent::TEXT(txt) => {
            LOGLn!(format!(
                "IView_{}({}, {txt}, ::{:p})",
                iview.id, iview.style.render, &*iview
            ));
        }
    }
}

fn debug_fiber_tree(node: Arc<Mutex<Fiber>>, tabs: i32) {
    for _ in 0..tabs {
        LOG!("\t");
    }
    LOG!("|-");
    let inode = {
        let Some(inode_p) = node.lock().unwrap().iview.clone() else {
            panic!("DEBUG: No IView")
        };
        inode_p
    };

    let fiber = node.lock().unwrap();
    LOGLn!(format!(
        "Fiber({}, {},{:?}, ::{:p} {:?})",
        fiber.key,
        fiber.changed,
        get_typeid(fiber.component.clone()),
        &*inode.lock().unwrap(),
        Arc::as_ptr(&node)
    ));

    fiber.children.iter().for_each(|child| {
        debug_fiber_tree(child.clone(), tabs + 1);
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

                // LOGLn!(format!("{:?} {}", Arc::as_ptr(&fiber_lk), children.len()));
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

                // // add this new fiber as child_lk
                fiber.children.clear(); // destroys the previous sub-tree from this node
                fiber.children.push(child_fiber_lk); // adds the new sub_tree
            } else {
                // preserve the state
                // just change the component of the child_lk and changed to true
                {
                    let mut child = child_lk.lock().unwrap();
                    child.changed = true;
                    // LOGLn!(format!(
                    //     "{:?} ___ {:p}",
                    //     Arc::as_ptr(&fiber_lk),
                    //     &*iview.lock().unwrap()
                    // ));
                    child.iview = None;
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
        // // add this iview to this fiber
        iview.lock().unwrap().style.render = true;
        fiber.iview = Some(iview);
    } else {
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
    getmaxyx(stdscr(), y, x);
    {
        let mut document = DOCUMENT.lock().unwrap();
        document.clear_tab_order();
        document.clear_color_pairs();
    }
    let res = root.lock().unwrap().__init__(*y, *x);
    if res.2 {
        DOCUMENT.lock().unwrap().create_tab_order();

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

/**
 * returns whether to exit the program
 */
fn handle_keyboard_event(ch: i32) -> bool {
    let focused_iview = {
        let document = DOCUMENT.lock().unwrap();
        let iview = document.active_element();
        iview
    };
    let mut event = EVENT::new(ch);
    if let Some(iview) = focused_iview {
        handle_event(iview, &mut event);
    }
    // handle regular functionality if default is on

    if event.default {
        match ch {
            KEY_BTAB => {
                let mut document = DOCUMENT.lock().unwrap();
                document.advance_tab();
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

/************  Public Functions  ********** */

pub static DOCUMENT: Mutex<Document> = Mutex::new(Document {
    curr_fiber: None,
    tabindex: 0,
    taborder: vec![],
    unique_id: 0,
    changed: true,
    color_pairs: LazyLock::new(|| Mutex::new(HashMap::<(i16, i16), u16>::new())),
    total_allowed_pairs: 0,
    curr_color_pair: 0,
});

/**
 * Takes a clonable value and stores its clone
 * on subsequent calls the value is cloned and then sent back to the user
 */
pub fn use_state<T: Stateful + Debug>(init_val: T) -> (T, impl Fn(T)) {
    // extracting the Components Fiber

    let currfiber_lk = {
        let document = DOCUMENT.lock().unwrap();
        let Some(currfib_lk) = document.curr_fiber.clone() else {
            panic!("SET STATE: No fiber associated with the component")
        };
        currfib_lk
    };

    let mut currfiber = currfiber_lk.lock().unwrap();

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

        // extracting the Components Fiber
        let currfiber_lk = {
            let document = DOCUMENT.lock().unwrap();
            let Some(currfib_lk) = document.curr_fiber.clone() else {
                panic!("SET STATE: No fiber associated with the component")
            };
            currfib_lk
        };

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
    };

    return (clone(downcasted_val), set_value);
}

/**
 * Takes in a Component as input and call it
*/
pub fn run(app: impl Component) {
    // let mut global_vec = GLOBAL_VEC.lock().unwrap();
    // global_vec.push(Fiber { current_idx: 0, state: vec![], changed: false });

    let node: Arc<Mutex<dyn Component>> = Arc::new(Mutex::new(app));

    let root = create_render_tree(node);

    // debug_tree(root.clone(), 0);
    initialize();

    loop {
        // if change, get the tree from the app.
        // diff the tree to get the changed components
        let mut changed = diff_n_update(root.clone());
        changed |= DOCUMENT.lock().unwrap().changed;

        // if changes, render the changed portion
        if changed {
            let _ = tree_refresh(root.clone());
        }

        // handle click and scroll
        if handle_events(root.clone()) {
            break;
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
        io::{Write, stdout},
        panic,
        sync::{Arc, Mutex},
    };

    use ncurses::{COLOR_MAGENTA, COLOR_RED, endwin, getch};

    use crate::{
        components::{text::Text, view::View}, create_render_tree, debug_fiber_tree, debug_tree, diff_n_update, handle_events, initialize, interfaces::{Component, ComponentBuilder}, styles::{BOXSIZING, DIMEN, FLEXDIRECTION, OVERFLOWBEHAVIOUR, STYLE}, tree_refresh, use_state, DOCUMENT
    };

    struct DemoApp1 {
        pub val: i32,
    }

    impl Component for DemoApp1 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p1, setp1) = use_state::<i32>(self.val);

            View::new_style_vec(vec![], vec![]).build()
        }
    }

    struct DemoApp2 {
        pub val: String,
    }

    impl Component for DemoApp2 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p1, setp1) = use_state("Namaste".to_string());

            // assert_eq!(p1, self.val);
            // LOGLn!(format!("{} {}", self.val, p1));

            setp1("Ram Ram Bhai Sare Ne".to_string());

            let color = DOCUMENT.lock().unwrap().get_color(255, 60, 0);

            if p1 == "Ram Ram Bhai Sare Ne" {
                View::new_style_vec(
                    vec![
                        View::new_style_vec(
                            vec![
                                Text::new_style_vec("Shiv Shambo".to_string(), vec![STYLE::TEXTCOLOR(color)])
                                    .build(),
                            ],
                            vec![],
                        )
                        .build(),
                        Text::new_style_vec("Shiv Shambo".to_string(), vec![]).build(),
                        Text::new_style_vec("Shiv Shambo".to_string(), vec![]).build(),
                    ],
                    vec![STYLE::FLEXDIRECTION(FLEXDIRECTION::HORIZONTAL)],
                )
                .build()
            } else {
                View::new_key_style_vec(
                    Some("P".to_string()),
                    vec![Text::new_style_vec("Shiv Shambo".to_string(), vec![]).build()],
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

            LOGLn!(format!("{p}, {color}"));
            View::new_style_vec(
                vec![
                    DemoApp1 { val: 0 }.build(),
                    DemoApp2 {
                        val: self.v1.clone(),
                    }
                    .build(),
                    Text::new_style_vec("Hello asdnaksjdnakjsc ajs cjsd cjasdcjsadjcaskjdcnjasdncjasbdjchasbdjcasjdcnaksjdnclkasncalskjdnckalsnclksandckjansdlkcnaskjdcnaksndcasjkndsjsdajkdnjjsvabhjcnjcnjsdjlsdajxcnxcnkxcmnxcmnxcmnxcmnxcmnxcmnxcm,xcm,xcmnxcm,xcm,xcm,xcmnxcaskbkdjscbasdjcbjasbcjcbkasjbdcajcbashcjbaksjcbsajdchbasdj".to_string(), vec![STYLE::WIDTH(DIMEN::INT(10)), STYLE::OVERFLOW(OVERFLOWBEHAVIOUR::SCROLL),STYLE::HIEGHT(DIMEN::INT(10)), STYLE::TEXTCOLOR(color), STYLE::MARGINTOP(DIMEN::INT(5))]).onclick(move |_e| {
                            LOGLn!(format!("I was Called"));
                            setp(10);
                    }, true).build()
                ],
                vec![
                    STYLE::TABORDER(0),
                    STYLE::HIEGHT(DIMEN::PERCENT(50.)),
                    STYLE::PADDINGLEFT(DIMEN::INT(10)),
                    STYLE::PADDINGTOP(DIMEN::INT(10)),
                    STYLE::PADDINGBOTTOM(DIMEN::INT(10)),
                    STYLE::PADDINGRIGHT(DIMEN::INT(10)),
                    STYLE::BOXSIZING(BOXSIZING::BORDERBOX),
                    STYLE::BORDER(true),
                    STYLE::BORDERCOLOR(COLOR_RED)
                ],
            )
            .build()
        }
    }

    fn clear() {
        DOCUMENT.lock().unwrap().clear_fiber();
    }

    #[test]
    fn test_set_state_i32() {
        clear();
        let dm = DemoApp1 { val: 0 };
        // let root = create_render_tree(dm);

        // debug_tree(root, 0);

        // now move idx to start
        // reset();

        // {
        //     let mut global_vec = GLOBAL_VEC.lock().unwrap();
        //     let curr_fiber = &mut global_vec[0];

        //     assert!(curr_fiber.changed);
        // }

        // let (v, _) = set_state(0);

        // assert_eq!(v, 10);
    }

    #[test]
    fn test_set_state_string() {
        // clear();
        let dm = DemoApp2 {
            val: "Namaste".to_string(),
        };
        // create_render_tree(dm);

        // now move idx to start
        // reset();

        // {
        //     let mut global_vec = GLOBAL_VEC.lock().unwrap();
        //     let curr_fiber = &mut global_vec[0];

        //     assert!(curr_fiber.changed);
        // }

        let (v, setv) = use_state("".to_string());

        assert_eq!(v, "Ram Ram Bhai Sare Ne");

        // {
        //     let mut global_vec = GLOBAL_VEC.lock().unwrap();
        //     let curr_fiber = &mut global_vec[0];

        //     assert!(curr_fiber.changed);

        //     curr_fiber.changed = false;
        // }

        // to check the changed flag
        setv("Ram Ram Bhai Sare Ne".to_string());

        assert_eq!(v, "Ram Ram Bhai Sare Ne");

        // {
        //     let mut global_vec = GLOBAL_VEC.lock().unwrap();
        //     let curr_fiber = &mut global_vec[0];

        //     // since value was not changed, and flag was set to false(mimicing that rendering is complete), this flag must still
        //     // be off because the value actually didn't changed after calling setv() above
        //     assert!(!curr_fiber.changed);
        // }
    }

    #[test]
    fn test_create_tree1() {
        let _ = std::fs::write("debug.txt", "");
        panic::set_hook(Box::new(|info| {
            // Disable SGR mouse mode
            // LOGLn!(format!("\033[?1006l"));
            // stdout().flush();
            endwin();
            LOGLn!(format!("{}", info));
        }));

        // Enable extended mouse reporting if available
        // LOGLn!(format!("\033[?1006h")); // Enable SGR mouse mod)e
        // stdout().flush();

        initialize();
        // clear();
        let dm = DemoApp3 {
            v1: format!("Namaste"),
        };
        let node: Arc<Mutex<dyn Component>> = Arc::new(Mutex::new(dm));
        let root = create_render_tree(node);

        loop {
            // if change, get the tree from the app.
            // diff the tree to get the changed components
            let mut changed = diff_n_update(root.clone());
            changed |= DOCUMENT.lock().unwrap().changed;

            // if changes, render the changed portion
            if changed {
                // LOGLn!("_________________________________________________-");
                // debug_tree(root.clone(), 0);
                let _ = tree_refresh(root.clone());
                // {
                //     let Some(fiber) = DOCUMENT.lock().unwrap().curr_fiber.clone() else {
                //         panic!("No fiber")
                //     };

                //     debug_fiber_tree(fiber.clone(), 0);
                // }
            }

            // handle click and scroll
            if handle_events(root.clone()) {
                break;
            }
        }
        // LOGLn!(format!("\033[?1006l"));
        endwin();

        // debug_tree(root.clone(), 0);

        // let res = tree_refresh(root.clone());
        // LOG!("{} {} {}", res.0, res.1, res.2);
    }
}
