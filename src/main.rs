#![allow(static_mut_refs)]
use crate::{
    calculator::Calculator, components::sidebar::Sidebar, formulas::Formulas,
    utils::cache::init_cache,
};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

mod calculator;
mod components;
mod formulas;
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
    #[at("/formulas")]
    Formulas,
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
                    Route::Formulas => html!(<Formulas />),
                    _ => html!(<Calculator />),
                };
                html! {
                    <div class={classes!("bg-std-900")}>
                        <div class={classes!(
                            "grid", "grid-cols-[auto_1fr]",
                            "max-h-screen", "h-full"
                        )}>
                            <Sidebar />
                            <div class={classes!(
                                "w-full", "overflow-auto"
                            )}>
                                {component}
                            </div>
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
