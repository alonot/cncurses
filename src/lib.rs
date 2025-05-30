// We'll have a Component that takes in many things as Input works on it and return other component
//  

/*
TODO:
 1. Rendering
 2. On Click
 3. On Scroll
*/

use std::{any::Any, fmt::Debug, rc::Rc, sync::Mutex};
use dyn_clone::clone;
use interfaces::{Component, Fiber, Stateful};
use nmodels::IView::IView;
use once_cell::sync::Lazy;

pub mod components;
mod nmodels;
pub mod interfaces;

/**
 * Checks and run IView, if Component can be downcasted to IView
 */
fn convert_to_icomponent(v: Rc<Mutex<dyn Component>>) -> Option<Rc<Mutex<IView>>>
{   
    if let Some(base) = v.lock().unwrap().__base__() {
        return Some(base)
    }
    None
}

/**
 * Initalize the window
 */
fn initialize() {
    
}

/**
 * Create tree, Keep recursing till we remove all non-base Components.
 * View and Buttons' children are expanded to get their IComponents.
 */
fn create_tree(mut node : Rc<Mutex<dyn Component>>) -> Rc<Mutex<IView>> {
    // in recuursion

    if let Some(base_lk) = convert_to_icomponent(node.clone()) {
        let base = base_lk.lock().unwrap();
        let content = &base.content;

        match content {
            interfaces::IViewContent::CHIDREN(iviews) => {
                
                // iterate over the children of node
                base.children.iter().for_each(|child| {
                    let new_iview = create_tree(child.clone()); 
                    // assign them as child to the base_lk node 
                });

            },
            interfaces::IViewContent::TEXT(_) => {
                // DO Nothing
            },
        }
        return base_lk.clone();
    } else {
        let new_node = node.lock().unwrap().__call__();
        return create_tree(new_node);
    }
    todo!()
}

static GLOBAL_VEC: Lazy<Mutex<Vec<Fiber>>> = Lazy::new(|| Mutex::new(vec![]));
static CURR_FIBER: Mutex<u32>              = Mutex::new(0);


/************  Public Function  ********** */

/**
 * Takes a clonable value and stores its clone 
 * on subsequent calls the value is cloned and then sent back to the user
 */
pub fn set_state<T: Stateful + Debug>(init_val:T) -> (T, impl Fn(T) ) {

    // extracting the Components Fiber
    let curr_fiber_idx = *CURR_FIBER.lock().unwrap();
    let mut global_vec = GLOBAL_VEC.lock().unwrap();
    let Some(currfiber) = global_vec.get_mut(curr_fiber_idx as usize) else {
        todo!()
    };

    let curr_hook = currfiber.current_idx;
    
    // add new entry if required
    if currfiber.current_idx == currfiber.state.len() as u32 {
        currfiber.state.push(Box::new(clone(&init_val)));
        currfiber.current_idx += 1;
    }
    
    let box_value = &currfiber.state[curr_hook as usize];

    let Some(downcasted_val) = box_value.as_any().downcast_ref::<T>() else {
        todo!()
    };

    // create the closure
    let set_value  = move |val: T| { // move to get ownership of `curr_hook` variable

        // extracting the Components Fiber
        let curr_fiber_idx = *CURR_FIBER.lock().unwrap();
        let mut global_vec = GLOBAL_VEC.lock().unwrap();
        let Some(currfiber) = global_vec.get_mut(curr_fiber_idx as usize) else {
            todo!()
        };
        
        if curr_hook == currfiber.state.len() as u32 {
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
pub fn run(
    app : impl Component
) {

    let mut global_vec = GLOBAL_VEC.lock().unwrap();
    global_vec.push(Fiber { current_idx: 0, state: vec![], changed: false }); 
    loop {

        // if change, get the tree from the app.
        
        // diff the tree to get the changed components
    
        // if changes, render the changed portion
        
        // handle click and scroll

    }

}

/**
 * Do not run these paralelly because they are working on same global variable. 
 * Hence Will result in undefined behaviour.
 */
#[cfg(test)]
mod test {
    use crate::{set_state, Fiber, GLOBAL_VEC};
    fn clear() {
        let mut global_vec = GLOBAL_VEC.lock().unwrap();
        global_vec.iter_mut().for_each(|v| {
            v.current_idx = 0;
            v.state.clear();
        });
        global_vec.clear();
    }

    fn setup() {
        let mut global_vec = GLOBAL_VEC.lock().unwrap();
        global_vec.push(Fiber { current_idx: 0, state: vec![], changed: false }); 
        global_vec.iter_mut().for_each(|v| {
            v.current_idx = 0;
        });
    }

    fn demo_app(val: i32) {
        
        let p = 0;
        
        let (p1, setp1) = set_state::<i32>(p);

        assert_eq!(p1, val);
        setp1(10);
    }

    fn demo_app_string(val: &str) {
        let (p1, setp1) = set_state("Namaste".to_string());
        assert_eq!(p1, val);
        setp1("Ram Ram Bhai Sare Ne".to_string());
    }

    #[test]
    fn test_set_state_i32() {
        clear();
        setup();
        demo_app(0);   
        
        setup();
        demo_app(10);
        
    }
    
    #[test]
    fn test_set_state_string() {
        
        clear();
        
        setup();
        demo_app_string("Namaste");   
        
        setup();
        demo_app_string("Ram Ram Bhai Sare Ne");

        {

            let mut global_vec = GLOBAL_VEC.lock().unwrap();
            let curr_fiber = &mut global_vec[0];
            
            assert!(curr_fiber.changed);
            
            curr_fiber.changed = false;
        }

        setup();
        demo_app_string("Ram Ram Bhai Sare Ne");

        {
            let mut global_vec = GLOBAL_VEC.lock().unwrap();
            let curr_fiber = &mut global_vec[0];
            assert!(!curr_fiber.changed);
        }

    }


}