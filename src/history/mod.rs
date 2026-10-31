#[cfg(feature = "tauri")]
mod charts;
#[cfg(feature = "tauri")]
mod details;
#[cfg(feature = "tauri")]
pub(crate) mod list;
#[cfg(feature = "tauri")]
pub(crate) mod source;
#[cfg(not(feature = "tauri"))]
use crate::Route;
use yew::prelude::*;
#[cfg(not(feature = "tauri"))]
use yew_router::prelude::*;
#[derive(Properties, PartialEq)]
pub struct HistoryProps {
    #[prop_or_default]
    pub id: String,
}
#[component]
pub fn History(props: &HistoryProps) -> Html {
    #[cfg(feature = "tauri")]
    {
        if props.id.is_empty() {
            html! { <list::HistoryListView /> }
        } else {
            html! { <details::HistoryDetailsView key={props.id.clone()} id={props.id.clone()} /> }
        }
    }
    #[cfg(not(feature = "tauri"))]
    {
        let _ = props;
        html! {
            <main class={classes!("watch-page")}>
                <h1>{ "Match history" }</h1>
                <p>
                    { "Your local match history is available in the desktop app. Website account history is not available yet." }
                </p>
                <Link<Route> to={Route::Watch}>{ "Watch a shared match" }</Link<Route>>
            </main>
        }
    }
}
