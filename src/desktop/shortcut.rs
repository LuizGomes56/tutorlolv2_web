use wasm_bindgen::{JsCast, prelude::*};
use web_sys::js_sys::{Function, Reflect};
use yew::{platform::spawn_local, prelude::*};

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    fn watch_overlay_shortcut(callback: &Function, error: &Function) -> Function;

    fn start_shortcut_recording(
        captured: &Function,
        error: &Function,
        finished: &Function,
        ready: &Function,
    ) -> Function;

    fn format_shortcut(shortcut: &JsValue) -> String;

    #[wasm_bindgen(catch)]
    async fn save_overlay_shortcut(shortcut: &JsValue) -> Result<JsValue, JsValue>;
}

fn field(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &name.into()).unwrap_or(JsValue::UNDEFINED)
}

#[component]
pub fn ShortcutSettings() -> Html {
    let saved = use_state(|| None::<JsValue>);
    let draft = use_state(|| None::<JsValue>);
    let recording = use_state(|| false);
    let recording_ready = use_state(|| false);
    let saving = use_state(|| false);
    let error = use_state(String::new);
    let warning = use_state(String::new);
    let status = use_state(String::new);

    {
        let saved = saved.clone();
        let warning = warning.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            let callback = Closure::<dyn FnMut(JsValue)>::new(move |settings: JsValue| {
                saved.set(Some(field(&settings, "shortcut")));
                warning.set(field(&settings, "warning").as_string().unwrap_or_default());
            });

            let failure = Closure::<dyn FnMut(String)>::new(move |message| error.set(message));

            let stop = watch_overlay_shortcut(
                callback.as_ref().unchecked_ref(),
                failure.as_ref().unchecked_ref(),
            );

            move || {
                let _ = stop.call0(&JsValue::NULL);
                drop((callback, failure));
            }
        });
    }

    {
        let draft = draft.clone();
        let error = error.clone();
        let recording = recording.clone();
        let recording_ready = recording_ready.clone();

        use_effect_with(*recording, move |active| {
            let callbacks = if *active {
                let captured =
                    Closure::<dyn FnMut(JsValue)>::new(move |value| draft.set(Some(value)));

                let failure = Closure::<dyn FnMut(String)>::new(move |message| error.set(message));
                let recording_ready_done = recording_ready.clone();

                let finished = Closure::<dyn FnMut()>::new(move || {
                    recording_ready_done.set(false);
                    recording.set(false);
                });

                let ready = Closure::<dyn FnMut()>::new(move || recording_ready.set(true));
                let stop = start_shortcut_recording(
                    captured.as_ref().unchecked_ref(),
                    failure.as_ref().unchecked_ref(),
                    finished.as_ref().unchecked_ref(),
                    ready.as_ref().unchecked_ref(),
                );

                Some((stop, captured, failure, finished, ready))
            } else {
                None
            };

            move || {
                if let Some((stop, captured, failure, finished, ready)) = callbacks {
                    let _ = stop.call0(&JsValue::NULL);

                    drop((captured, failure, finished, ready));
                }
            }
        });
    }

    let record = {
        let recording = recording.clone();
        let recording_ready = recording_ready.clone();
        let error = error.clone();
        let status = status.clone();

        Callback::from(move |_| {
            error.set(String::new());
            status.set(String::new());
            recording_ready.set(false);
            recording.set(!*recording);
        })
    };

    let save = {
        let saved = saved.clone();
        let draft = draft.clone();
        let saving = saving.clone();
        let error = error.clone();
        let status = status.clone();
        let warning = warning.clone();

        Callback::from(move |reset: bool| {
            let shortcut = if reset {
                web_sys::js_sys::JSON::parse(
                    r#"{"code":"KeyK","ctrl":true,"alt":false,"shift":false,"meta":false}"#,
                )
                .ok()
            } else {
                (*draft).clone()
            };

            let Some(shortcut) = shortcut else {
                return;
            };

            saving.set(true);
            error.set(String::new());
            status.set(String::new());

            let saved = saved.clone();
            let draft = draft.clone();
            let saving = saving.clone();
            let error = error.clone();
            let status = status.clone();
            let warning = warning.clone();

            spawn_local(async move {
                match save_overlay_shortcut(&shortcut).await {
                    Ok(settings) => {
                        saved.set(Some(field(&settings, "shortcut")));
                        draft.set(None);
                        warning.set(String::new());
                        status.set("Shortcut saved".into());
                    }
                    Err(value) => error.set(
                        value
                            .as_string()
                            .unwrap_or_else(|| format!("Could not save shortcut: {value:?}")),
                    ),
                }

                saving.set(false);
            });
        })
    };

    let label = draft
        .as_ref()
        .or(saved.as_ref())
        .map(format_shortcut)
        .unwrap_or_else(|| "Loading...".into());

    html! {
        <section
            class={classes!("overlay-settings-section")}
            aria-label={"Overlay shortcut"}
            aria-busy={saving.to_string()}
        >
            <h2>{"Overlay shortcut"}</h2>
            <p>
                {"Press once to open, then press again to close. Escape also closes the overlay. Ctrl+O cycles enemies."}
            </p>
            <div class={classes!("overlay-shortcut-controls")}>
                <output
                    class={classes!("overlay-shortcut-key")}
                    aria-label={"Current shortcut"}
                >
                    {label}
                </output>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button")}
                    onclick={record}
                    disabled={*saving || saved.is_none()}
                >
                    {if *recording {
                        "Cancel recording"
                    } else {
                        "Record shortcut"
                    }}
                </button>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button", "primary")}
                    onclick={save.reform(|_| false)}
                    disabled={*saving || *recording || draft.is_none()}
                >
                    {"Save shortcut"}
                </button>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button")}
                    onclick={save.reform(|_| true)}
                    disabled={*saving || *recording || saved.is_none()}
                >
                    {"Reset to Ctrl+K"}
                </button>
            </div>
            <p
                class={classes!("overlay-settings-feedback")}
                role={"status"}
                aria-live={"polite"}
            >
                {if *recording && !*recording_ready {
                    "Preparing to record…"
                } else if *recording {
                    "Press a key or key combination. Escape cancels."
                } else if *saving {
                    "Saving shortcut…"
                } else if draft.is_some() {
                    "Unsaved shortcut — select Save shortcut to apply."
                } else {
                    &*status
                }}
            </p>

            if !warning.is_empty() {
                <p class={classes!("overlay-settings-warning")} role={"status"}>
                    {&*warning}
                </p>
            }

            if !error.is_empty() {
                <p class={classes!("overlay-settings-error")} role={"alert"}>
                    {&*error}
                </p>
            }
        </section>
    }
}
