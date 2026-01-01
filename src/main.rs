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
            <div class={classes!("bg-[#1f1f1f]")}>
                <Documentation />
                // <Calculator />
            </div>
        }
    }

    #[cfg(feature = "overlay")]
    html!(<div class={classes!("bg-transparent")}><Overlay /></div>)
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
