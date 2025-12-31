use crate::{
    calculator::Calculator, documentation::Documentation, overlay::Overlay,
    utils::cache::init_cache,
};
use yew::prelude::*;

mod calculator;
mod components;
mod documentation;
mod model;
mod overlay;
mod utils;

#[component]
fn App() -> Html {
    #[cfg(not(feature = "overlay"))]
    {
        html! {
            <Documentation />
            // <Calculator />
        }
    }

    #[cfg(feature = "overlay")]
    html!(<Overlay />)
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
