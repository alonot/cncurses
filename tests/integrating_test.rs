use std::{rc::Rc, sync::Mutex};

use cncurses::{components::{button::Button, text::Text, view::View}, interfaces::Component, run};

struct P {
    i : i32
}
impl Component for P {
    fn __call__(&mut self) -> Rc<Mutex<dyn Component>> {
        let p = Text::new("Namaste!!".to_string(), vec![]);
        p.build()
    }
}

fn main() {
    let p = Text::new("Hwll".to_string(), vec![]).build();   

    let v = View::new(vec![p], vec![]).onclick(|| {}).onscroll(|| {});

    run(P{i:10});
}