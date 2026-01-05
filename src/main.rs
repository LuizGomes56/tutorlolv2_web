use crate::{
    calculator::Calculator, components::header::Header, docs::Docs, overlay::Overlay,
    utils::cache::init_cache,
};
use yew::prelude::*;

mod calculator;
mod components;
mod docs;
mod model;
mod overlay;
mod utils;

#[component]
fn App() -> Html {
    #[cfg(not(feature = "overlay"))]
    return html! {
        <div class={classes!("bg-std-900")}>
            <Header />
            <Docs />
            // <Calculator />
        </div>
    };

    #[cfg(feature = "overlay")]
    html!(<div class={classes!("bg-transparent")}><Overlay /></div>)
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
