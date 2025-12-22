use crate::{
    calculator::page::Calculator, documentation::page::Documentation, utils::cache::init_cache,
};
use yew::prelude::*;

mod calculator;
mod documentation;
mod model;
mod utils;

#[component]
fn App() -> Html {
    html! {
        <Documentation />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
    println!("Hello, world!");
}
