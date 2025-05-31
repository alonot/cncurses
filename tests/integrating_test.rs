use std::{sync::{Arc, Mutex}};

use cncurses::{components::{button::Button, text::Text, view::View}, interfaces::{Component, ComponentBuilder}, run};

struct P {
    i : i32
}
impl Component for P {
    fn __call__(&mut self) -> Arc<Mutex<(dyn Component + 'static)>> {
        let p = Text::new("Namaste!!".to_string(), vec![]);
        p.build()
    }
}

fn main() {
    let p = Text::new("Hwll".to_string(), vec![]).build();   

    let mut t = 10;

    let v = View::new(vec![p], vec![]).onclick(|| {}).onscroll(
        move || {
            t +=1;
        }
    );

    run(P{i:10});
}