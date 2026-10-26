mod shortcut;
pub use shortcut::ShortcutSettings;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::js_sys::Function;
use yew::prelude::*;

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn toggle_overlay() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn close_overlay() -> Result<JsValue, JsValue>;

    pub fn watch_overlay_focus(callback: &Function, error: &Function) -> Function;
    pub fn watch_overlay_change(callback: &Function, error: &Function) -> Function;
}

#[component]
pub fn Settings() -> Html {
    let error = use_state(String::new);
    let open = {
        let error = error.clone();

        Callback::from(move |_| {
            let error = error.clone();

            yew::platform::spawn_local(async move {
                match toggle_overlay().await {
                    Ok(_) => error.set(String::new()),
                    Err(value) => error.set(format!("Could not open the overlay: {value:?}")),
                }
            });
        })
    };

    html! {
        <main class={classes!("desktop-settings")}>
            <h1>{"Overlay settings"}</h1>
            <p>
                {"Use one shortcut to open the overlay, arrange your widgets, and return to your game."}
            </p>
            <ShortcutSettings />
            <section class={classes!("overlay-settings-section")}>
                <h2>{"Widgets and layout"}</h2>
                <p>
                    {"Open the overlay to move and resize widgets. Choose which widgets stay visible when you close it. Your choices and positions are saved on this device."}
                </p>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button")}
                    onclick={open}
                >
                    {"Toggle overlay"}
                </button>

                if !error.is_empty() {
                    <p role={"alert"}>{&*error}</p>
                }
            </section>
        </main>
    }
}

// Both subscriptions return synchronous cleanup functions, including during async registration.
#[hook]
pub fn use_overlay_focus() -> (bool, String) {
    let focused = use_state(|| false);
    let error = use_state(String::new);

    {
        let focused = focused.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            let callback = Closure::<dyn FnMut(bool)>::new(move |value| focused.set(value));
            let failure = Closure::<dyn FnMut(String)>::new(move |value| error.set(value));

            let stop = watch_overlay_focus(
                callback.as_ref().unchecked_ref(),
                failure.as_ref().unchecked_ref(),
            );

            move || {
                let _ = stop.call0(&JsValue::NULL);
                drop((callback, failure));
            }
        });
    }

    (*focused, (*error).clone())
}
