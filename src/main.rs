use crate::{calculator::page::Calculator, utils::cache::init_cache};
use yew::prelude::*;

mod calculator;
mod model;
mod utils;

#[component]
fn App() -> Html {
    html! {
        <Calculator />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
    println!("Hello, world!");
}
