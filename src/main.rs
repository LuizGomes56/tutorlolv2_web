use {
    crate::{
        about::About,
        calculator::Calculator,
        components::{hoverdocs::HoverDocs, sidebar::Sidebar},
        formulas::Formulas,
        livegame::Livegame,
    },
    yew::prelude::*,
    yew_router::{BrowserRouter, Routable, Switch},
};

#[cfg(feature = "tauri")]
use crate::overlay::Overlay;

#[cfg(feature = "tauri")]
mod overlay;

#[cfg(feature = "tauri")]
mod desktop;

mod about;
mod calculator;
mod components;
mod formulas;
mod history;
mod hover;
mod livegame;
mod model;
mod replay;
mod utils;
mod watch;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[not_found]
    #[at("/")]
    Homepage,
    #[cfg(feature = "tauri")]
    #[at("/overlay")]
    Overlay,
    #[cfg(feature = "tauri")]
    #[at("/settings")]
    Settings,
    #[at("/calculator")]
    Calculator,
    #[at("/watch")]
    Watch,
    #[at("/watch/:code")]
    WatchCode { code: String },
    #[at("/history")]
    History,
    #[at("/history/:id")]
    HistoryDetails { id: String },
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
    html! {
        <BrowserRouter>
            <Switch<Route>
                render={|route| {
                #[cfg(feature = "tauri")]
                if route == Route::Overlay {
                    return html! {
                        <>
                            <Overlay />
                            <HoverDocs />
                        </>
                    };
                }

                let component = match route {
                    #[cfg(feature = "tauri")]
                    Route::Settings => html!(<desktop::Settings />),
                    Route::Calculator => html!(<Calculator />),
                    Route::Formulas => html!(<Formulas />),
                    Route::Watch => html!(<watch::Watch />),
                    Route::WatchCode { code } => html!(<watch::Watch code={code} />),
                    Route::History => html!(<history::History />),
                    Route::HistoryDetails { id } => html!(<history::History id={id} />),
                    Route::Livegame => html!(<Livegame />),
                    Route::About => html!(<About />),
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
            }}
            />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::Route;
    use yew_router::Routable;

    #[test]
    fn overlay_route_is_available_only_in_tauri_builds() {
        assert_eq!(
            Route::routes().contains(&"/overlay"),
            cfg!(feature = "tauri")
        );
        assert!(Route::routes().contains(&"/livegame"));
        assert!(Route::routes().contains(&"/history"));
        assert!(Route::routes().contains(&"/history/:id"));
        assert!(Route::routes().contains(&"/watch"));
        assert!(Route::routes().contains(&"/watch/:code"));
    }
}
