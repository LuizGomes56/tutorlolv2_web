use crate::{
    calculator::page::Calculator, documentation::page::Documentation, utils::cache::init_cache,
};
use yew::prelude::*;

mod calculator;
mod components;
mod documentation;
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
}
