#[cfg(not(feature = "overlay"))]
use crate::components::sidebar::Sidebar;
use crate::{
    calculator::Calculator, components::header::Header, docs::Docs, overlay::Overlay,
    utils::cache::init_cache,
};
use yew::prelude::*;
#[cfg(not(feature = "overlay"))]
use yew_router::Switch;
use yew_router::{BrowserRouter, Routable};

mod calculator;
mod components;
mod docs;
mod model;
mod overlay;
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
    #[cfg(not(feature = "overlay"))]
    return html! {
        <BrowserRouter>
            <Switch<Route> render={|route| {
                let component = match route {
                    Route::Calculator => html!(<Calculator />),
                    _ => html!(<Calculator />),
                };
                html! {
                    <div class={classes!("bg-std-900")}>
                        // <Header />
                        // <Sidebar />
                        <div class={classes!("flex", "w-full")}>
                            {component}
                        </div>
                    </div>
                }
            }} />
        </BrowserRouter>
    };

    #[cfg(feature = "overlay")]
    html!(<div class={classes!("bg-transparent")}><Overlay /></div>)
}

fn main() {
    yew::Renderer::<App>::new().render();
    init_cache();
}
