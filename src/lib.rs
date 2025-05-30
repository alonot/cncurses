// We'll have a Component that takes in many things as Input works on it and return other component
//  

/*
TODO:
 1. Rendering
 2. On Click
 3. On Scroll
*/

use std::{any::Any, fmt::Debug, sync::Mutex};
use dyn_clone::{clone, DynClone};
use once_cell::sync::Lazy;

mod components;
mod nmodels;

trait Stateful: DynClone + Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn Stateful) -> bool;
}
impl<T: Clone + Any + Send + PartialEq> Stateful for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn Stateful) -> bool {
        other.as_any().downcast_ref::<T>().map_or(false, |other| self == other)
    }
}

dyn_clone::clone_trait_object!(Stateful);

/**
 * Internal trait
 */

trait IComponent {
    fn get_children(&self) -> &Vec<Box<dyn IComponent>>;

    /**
     * Get important parameter of the screen and call render on its children
     */
    fn __render__(&self) -> i32 {

        // loop over the children
        self.get_children().iter().for_each(|child| {
            // calls the render function of child
            // gets the width covered by the child
            let width = child.__render__();
            
            // TODO: fill left width with background color.
        });

        0
    }
}


pub trait Component {
    fn __call__(&self) -> Box<dyn Component> ;
}

struct Style<T, S> 
where 
    T : FnMut(),
    S : FnMut()
{
    height:                 u32,
    width:                  u32,
    render:                 bool,
    scroll:                 bool,
    top:                    i32,
    bottom:                 i32,
    left:                   i32,
    right:                  i32,
    background_color:       i32,
    z_index:                i32,
    onclick: T ,   // should be a clousure
    onscroll: S ,  // should be a clousure
}


/**
 * Hooks struct. Each Component will have its own object of this struct
 */
struct Fiber {
    current_idx : u32,
    state: Vec<Box<dyn Stateful + 'static>>,
    changed: bool
}

/**
 * Initalize the window
 */
fn initialize() {
    
}

/**
 * Create tree, Keep recursing till we remove all non-base Components.
 * View and Buttons' children are expanded to get their base Components.
 * Base Components : View , Button, Text
 */
fn create_tree() {
    
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
    use crate::{set_state, Fiber, CURR_FIBER, GLOBAL_VEC};
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