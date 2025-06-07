use std::{sync::{Arc, Mutex}};

use cncurses::{components::{ text::Text, view::View}, interfaces::{Component, ComponentBuilder}, run};

struct P {
    _i : i32
}
impl Component for P {
    fn __call__(&mut self) -> Arc<Mutex<(dyn Component + 'static)>> {
        let p = Text::new_style_vec("Namaste!!".to_string(), vec![]);
        p.build()
    }
}

fn main() {
    let p = Text::new_style_vec("Hwll".to_string(), vec![]).build();   

    let mut _t = 10;

    let _v = View::new_style_vec(vec![p], vec![]).onclick(|_e| {}, true).onscroll(
        move |_e| {
            _t +=1;
        }, false
    );

    run(P{_i:10});
}