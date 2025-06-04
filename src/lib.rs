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
    ALL_MOUSE_EVENTS, KEY_BTAB, KEY_MOUSE, KEY_MOVE, KEY_RESIZE, MEVENT, OK, REPORT_MOUSE_POSITION,
    cbreak, curs_set, endwin, getch, getmaxyx, getmouse, initscr, keypad, mmask_t, mouseinterval,
    mousemask, nodelay, noecho, printw, refresh, start_color, stdscr, use_default_colors, wrefresh,
};
use nmodels::IView::IView;
use std::{
    any::TypeId,
    fmt::Debug,
    i32,
    sync::{Arc, Mutex},
};

use crate::interfaces::{BASICSTRUCT, Document, EVENT};

pub mod components;
pub mod interfaces;
mod nmodels;

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
    mousemask((ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION) as mmask_t, None);
    mouseinterval(0);
    refresh();
}

fn debug_tree(node: Arc<Mutex<IView>>, tabs: i32) {
    let iview = node.lock().unwrap();
    for _ in 0..tabs {
        print!("\t");
    }
    print!("|-");
    match &iview.content {
        interfaces::IViewContent::CHIDREN(items) => {
            println!("IView({}, {:p})", iview.style.render, &*iview);
            items.iter().for_each(|child| {
                debug_tree(child.clone(), tabs + 1);
            });
        }
        interfaces::IViewContent::TEXT(txt) => {
            println!(
                "IView({}, {txt}, {:p})",
                iview.style.render,
                &*iview
            );
        }
    }
}

fn debug_fiber_tree(node: Arc<Mutex<Fiber>>, tabs: i32) {
    for _ in 0..tabs {
        print!("\t");
    }
    print!("|-");
    let inode = {
        let Some(inode_p) = node.lock().unwrap().iview.clone() else {
            panic!("DEBUG: No IView")
        };
        Arc::as_ptr(&inode_p)
    };

    let fiber = node.lock().unwrap();
    println!(
        "Fiber({}, {},{:?}, {:?})",
        fiber.key,
        fiber.changed,
        get_typeid(fiber.component.clone()),
        inode
    );

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

    // println!("{:?}", as_any(node.clone()).type_id());
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
    let parent = IView::new().build();

    let fiber = create_tree(node, parent.clone(), false);

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

/**
 * Update the given base component with fiber's chidlren
 */
fn update_child(fiber_lk: Arc<Mutex<Fiber>>, base_lk: Arc<Mutex<IView>>) {
    let fiber = fiber_lk.lock().unwrap();

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

/**
 * Takes a fiber as input and if its change is on
 * then recreates the component but with a twist than `create_tree`
 * This time it checks if fiber needs to be changed or not using key and type of Component
 * If yes the creates a new Fiber else returns the same
 */
fn check_for_change(fiber_lk: Arc<Mutex<Fiber>>) -> bool {
    let mut changed;

    let component;
    {
        let cfiber_lk = fiber_lk.clone();
        let fiber = cfiber_lk.lock().unwrap();

        changed = fiber.changed;

        component = fiber.component.clone();
    }

    if changed && convert_to_icomponent(&component).is_none() {
        // this is not base component.

        // only 1 child
        let prev_fiber = DOCUMENT
            .lock()
            .unwrap()
            .re_assign_fiber(Some(fiber_lk.clone()));

        let new_node = component.lock().unwrap().__call__();

        DOCUMENT.lock().unwrap().assign_fiber(prev_fiber);

        let cfiber_lk = fiber_lk.clone();
        let mut fiber = cfiber_lk.lock().unwrap();

        let child_lk = fiber.children[0].clone();

        let is_not_same_child = {
            let new_key = &get_key(&new_node);

            // check if key is different or type is different
            let child = child_lk.lock().unwrap();
            let prev_key = &child.key;

            // true
            new_key != prev_key
                || get_typeid(child.component.clone()) != get_typeid(new_node.clone())
        };

        if is_not_same_child {
            let Some(iview) = fiber.iview.clone() else {
                panic!("CREATETREE: no iview in given Component")
            };

            let Some(parent) = iview.lock().unwrap().parent.clone() else {
                panic!("CREATETREE: no Parent for given IView")
            };

            // create new tree
            let child_fiber_lk = create_tree(new_node, parent, false); // somewhere inside the IView would get filled by its child_lk

            let cchild_fiber_lk = child_fiber_lk.clone();
            let mut child_fiber = cchild_fiber_lk.lock().unwrap();
            child_fiber.changed = false;
            let Some(iview) = child_fiber.iview.clone() else {
                panic!("CREATETREEE: no iview in given Componenet")
            };

            // // add this iview to this fiber
            fiber.iview = Some(iview);

            // // add this new fiber as child_lk
            fiber.children.clear(); // destroys the previous sub-tree from this node
            fiber.children.push(child_fiber_lk); // adds the new sub_tree

            fiber.changed = false;

            return changed;
        } else {
            // just change the component of the child_lk and changed to true
            let mut child = child_lk.lock().unwrap();
            child.changed = true;
            child.component = new_node;
        }
        // if same then change will be called below
    }

    {
        // to isolate and release the lock at end of block
        let mut fiber = fiber_lk.lock().unwrap();

        fiber.children.iter().for_each(|child| {
            {
                child.lock().unwrap().changed |= changed; //if parent is set to true
            }

            changed |= check_for_change(child.clone());
        });

        fiber.changed = false;
    }

    if changed {
        // then childs IView would have got updated

        // this time convert_to_component may return different tree because some child has changed

        if let Some(base_lk) = convert_to_icomponent(&component) {
            base_lk.lock().unwrap().style.render = true;
            update_child(fiber_lk.clone(), base_lk.clone());

            // update with new Iview
            fiber_lk.lock().unwrap().iview = Some(base_lk.clone());
        } else {
            // only 1 child
            // gets the child's iview
            let Some(iview) = fiber_lk.lock().unwrap().children[0]
                .lock()
                .unwrap()
                .iview
                .clone()
            else {
                panic!("DIFFTREE: No Iview")
            };

            fiber_lk.lock().unwrap().iview = Some(iview.clone());
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

    let changed = check_for_change(fiber_lk.clone());
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
    DOCUMENT.lock().unwrap().clear_tab_order();
    let res = root.lock().unwrap().__init__(*y, *x);
    if res.2 {
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
            },
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

static DOCUMENT: Mutex<Document> = Mutex::new(Document {
    curr_fiber: None,
    tabindex: 0,
    taborder: vec![],
});

/**
 * Takes a clonable value and stores its clone
 * on subsequent calls the value is cloned and then sent back to the user
 */
pub fn set_state<T: Stateful + Debug>(init_val: T) -> (T, impl Fn(T)) {
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
        let changed = diff_n_update(root.clone());

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
    use std::sync::{Arc, Mutex};

    use ncurses::{endwin, getch};

    use crate::{
        DOCUMENT,
        components::{text::Text, view::View},
        create_render_tree, debug_fiber_tree, debug_tree, diff_n_update, handle_events, initialize,
        interfaces::{
            BOXSIZING, Component, ComponentBuilder, DIMEN, FLEXDIRECTION, OVERFLOWBEHAVIOUR, STYLE,
        },
        set_state, tree_refresh,
    };

    struct DemoApp1 {
        pub val: i32,
    }

    impl Component for DemoApp1 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p1, setp1) = set_state::<i32>(self.val);

            setp1(10);

            View::new(vec![], vec![]).build()
        }
    }

    struct DemoApp2 {
        pub val: String,
    }

    impl Component for DemoApp2 {
        fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
            let (p1, setp1) = set_state("Namaste".to_string());

            // assert_eq!(p1, self.val);
            println!("{} {}", self.val, p1);

            setp1("Ram Ram Bhai Sare Ne".to_string());

            if p1 == "Ram Ram Bhai Sare Ne" {
                View::new(
                    vec![
                        View::new(
                            vec![Text::new("Shiv Shambo".to_string(), vec![]).build()],
                            vec![],
                        )
                        .build(),
                        Text::new("Shiv Shambo".to_string(), vec![]).build(),
                        Text::new("Shiv Shambo".to_string(), vec![]).build(),
                    ],
                    vec![STYLE::FLEXDIRECTION(FLEXDIRECTION::HORIZONTAL)],
                )
                .build()
            } else {
                View::new_key(
                    Some("P".to_string()),
                    vec![Text::new("Shiv Shambo".to_string(), vec![]).build()],
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
            View::new(
                vec![
                    DemoApp1 { val: 0 }.build(),
                    DemoApp2 {
                        val: self.v1.clone(),
                    }
                    .build(),
                    Text::new("Hello".to_string(), vec![STYLE::HIEGHT(DIMEN::INT(20))]).onclick(|e| {
                        println!("I was Called");
                    }, true).build(),
                ],
                vec![
                    // STYLE::WIDTH(DIMEN::INT(1)),
                    STYLE::PADDINGLEFT(DIMEN::INT(10)),
                    STYLE::PADDINGTOP(DIMEN::INT(10)),
                    STYLE::PADDINGBOTTOM(DIMEN::INT(10)),
                    STYLE::PADDINGRIGHT(DIMEN::INT(10)),
                    // STYLE::OVERFLOW(OVERFLOWBEHAVIOUR::VISIBLE),
                    // STYLE::BOXSIZING(BOXSIZING::BORDERBOX),
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

        let (v, setv) = set_state("".to_string());

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
        // clear();
        let dm = DemoApp3 {
            v1: format!("Namaste"),
        };
        let node: Arc<Mutex<dyn Component>> = Arc::new(Mutex::new(dm));
        let root = create_render_tree(node);
        debug_tree(root.clone(), 0);
        {
            let Some(fiber) = DOCUMENT.lock().unwrap().curr_fiber.clone() else {
                panic!("No fiber")
            };

            debug_fiber_tree(fiber.clone(), 0);
        }

        initialize();

        let res = tree_refresh(root.clone());

        while getch() != 'q' as i32 {}
        endwin();
        {
            let Some(fiber) = DOCUMENT.lock().unwrap().curr_fiber.clone() else {
                panic!("No fiber")
            };

            println!("LL {}", diff_n_update(root.clone()));

        }

        let Some(fiber) = DOCUMENT.lock().unwrap().curr_fiber.clone() else {
            panic!("No fiber")
        };

        debug_fiber_tree(fiber.clone(), 0);

        let Some(iview) = fiber.lock().unwrap().iview.clone() else {
            panic!("No IView")
        };

        debug_tree(root.clone(), 0);

        initialize();

        let res = tree_refresh(root.clone());

        loop {
            if handle_events(root.clone()) {
                break;
            }
        }
        endwin();

        // debug_tree(root.clone(), 0);

        // let res = tree_refresh(root.clone());
        // println!("{} {} {}", res.0, res.1, res.2);
    }
}
