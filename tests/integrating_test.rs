use std::{sync::{Arc, Mutex}};

use cncurses::{components::{button::Button, text::Text, view::View}, interfaces::{Component, ComponentBuilder}, run};

struct P {
    i : i32
}
impl Component for P {
    fn __call__(&mut self) -> Arc<Mutex<(dyn Component + 'static)>> {
        let p = Text::new_style_vec("Namaste!!".to_string(), vec![]);
        p.build()
    }
}

fn main() {
    let p = Text::new_style_vec("Hwll".to_string(), vec![]).build();   

    let mut t = 10;

    let v = View::new_style_vec(vec![p], vec![]).onclick(|e| {}, true).onscroll(
        move |e| {
            t +=1;
        }, false
    );

    run(P{i:10});
}