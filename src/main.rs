#![allow(static_mut_refs)]
use crate::{
    calculator::Calculator,
    components::{hoverdocs::HoverDocs, sidebar::Sidebar},
    formulas::Formulas,
    livegame::Livegame,
    overlay::Overlay,
    utils::init_cache,
};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

mod calculator;
mod components;
mod formulas;
mod livegame;
mod model;
mod overlay;
mod utils;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[not_found]
    #[at("/")]
    Homepage,
    #[at("/overlay")]
    Overlay,
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
                    Route::Livegame => html!(<Livegame />),
                    Route::Overlay => return html!(<Overlay />),
                    _ => html!(<Calculator />),
                };
                html! {
                    <>
                        <div class={classes!("bg-std-900")}>
                            <div class={classes!(
                                "flex", "max-h-screen", "h-full"
                            )}>
                                <Sidebar />
                                <div class={classes!(
                                    "flex-1", "overflow-y-auto",
                                    "overflow-x-hidden"
                                )}>
                                    {component}
                                </div>
                            </div>
                        </div>
                        <HoverDocs />
                    </>
                }
            }} />
        </BrowserRouter>
    };
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
