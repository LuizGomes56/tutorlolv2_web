#![allow(static_mut_refs)]
use crate::{calculator::Calculator, utils::cache::init_cache};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

mod calculator;
mod components;
mod docs;
mod model;
mod realtime;
mod utils;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[not_found]
    #[at("/")]
    Homepage,
    #[at("/calculator")]
    Calculator,
    #[at("/livegame")]
    Livegame,
    #[at("/docs")]
    Docs,
    #[at("/help")]
    Help,
    #[at("/about")]
    About,
    #[at("/faq")]
    FAQ,
    #[at("/github")]
    GitHub,
}

#[component]
fn App() -> Html {
    return html! {
        <BrowserRouter>
            <Switch<Route> render={|route| {
                let component = match route {
                    Route::Calculator => html!(<Calculator />),
                    _ => html!(<Calculator />),
                };
                html! {
                    <div class={classes!("bg-std-900")}>
                        <div class={classes!("flex", "w-full")}>
                            {component}
                        </div>
                    </div>
                }
            }} />
        </BrowserRouter>
    };
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
