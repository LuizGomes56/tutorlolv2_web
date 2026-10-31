//! Main-window sharing controls. Credentials remain in the native recorder.
use std::{cell::Cell, rc::Rc, time::Duration};
use tutorlolv2_protocol::{RecordingState, RecordingStatus, ShareMode, ShareState};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{self, Function, Promise, Reflect};
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

fn member(value: &JsValue, name: &str) -> Result<JsValue, String> {
    Reflect::get(value, &JsValue::from_str(name))
        .map_err(|e| format!("Desktop connection unavailable: {e:?}"))
}

fn tauri() -> Result<JsValue, String> {
    member(
        &web_sys::window().ok_or("Browser unavailable")?.into(),
        "__TAURI__",
    )
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum Command {
    GetRecordingState,
    SetShareMode,
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
enum Event {
    RecordingStateChanged,
    ShareStateChanged,
}

impl Command {
    async fn invoke<T: serde::de::DeserializeOwned>(
        self,
        args: serde_json::Value,
    ) -> Result<T, String> {
        let core = member(&tauri()?, "core")?;
        let function: Function = member(&core, "invoke")?
            .dyn_into()
            .map_err(|_| "Desktop connection unavailable")?;

        let args = js_sys::JSON::parse(&args.to_string()).map_err(|_| "Invalid desktop request")?;
        let promise = function
            .call2(&core, &JsValue::from_str(self.into()), &args)
            .map_err(|e| format!("Desktop request failed: {e:?}"))?;

        let result = JsFuture::from(Promise::resolve(&promise))
            .await
            .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

        decode(result)
    }
}
fn decode<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, String> {
    let json = js_sys::JSON::stringify(&value)
        .map_err(|_| "Invalid desktop response")?
        .as_string()
        .ok_or("Invalid desktop response")?;

    serde_json::from_str(&json).map_err(|e| format!("Invalid desktop response: {e}"))
}

#[component]
pub fn SharingBar() -> Html {
    let state = use_state(|| None::<RecordingState>);
    let latest = use_mut_ref(|| None::<RecordingState>);

    *latest.borrow_mut() = (*state).clone();

    let revision = use_mut_ref(|| 0u64);
    let error = use_state(String::new);
    let notice = use_state(String::new);
    let busy = use_state(|| false);
    let refresh = use_state(|| 0u32);

    {
        let state = state.clone();
        let error = error.clone();
        let latest = latest.clone();
        let revision = revision.clone();

        use_effect_with(*refresh, move |_| {
            error.set(String::new());

            let active = Rc::new(Cell::new(true));
            let running = active.clone();
            let state_event = state.clone();
            let active_event = active.clone();
            let latest_event = latest.clone();
            let revision_event = revision.clone();

            let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
                if active_event.get()
                    && let Ok(payload) = member(&event, "payload")
                    && let Ok(value) = decode::<RecordingState>(payload)
                {
                    *revision_event.borrow_mut() += 1;
                    *latest_event.borrow_mut() = Some(value.clone());
                    state_event.set(Some(value));
                }
            });

            let state_share = state.clone();
            let active_share = active.clone();
            let latest_share = latest.clone();
            let revision_share = revision.clone();

            let share_callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
                if active_share.get()
                    && let Ok(payload) = member(&event, "payload")
                    && let Ok(share) = decode::<ShareState>(payload)
                {
                    let current = latest_share.borrow().clone();

                    if let Some(mut value) = current {
                        value.share = share;
                        *revision_share.borrow_mut() += 1;
                        *latest_share.borrow_mut() = Some(value.clone());
                        state_share.set(Some(value));
                    }
                }
            });

            let callbacks = Rc::new((callback, share_callback));
            let task_callbacks = callbacks.clone();

            // Register both events; if registration completes after unmount, immediately unlisten.
            for event in [Event::RecordingStateChanged, Event::ShareStateChanged] {
                let active = active.clone();
                let callbacks = task_callbacks.clone();

                spawn_local(async move {
                    let register = async {
                        let events = member(&tauri()?, "event")?;
                        let listen: Function = member(&events, "listen")?
                            .dyn_into()
                            .map_err(|_| "Event connection unavailable")?;

                        let callback = match event {
                            Event::RecordingStateChanged => callbacks.0.as_ref(),
                            Event::ShareStateChanged => callbacks.1.as_ref(),
                        };

                        let promise = listen
                            .call2(&events, &JsValue::from_str(event.into()), callback)
                            .map_err(|_| "Event subscription failed")?;

                        let stop = JsFuture::from(Promise::resolve(&promise))
                            .await
                            .map_err(|_| "Event subscription failed")?;

                        Ok::<Function, String>(stop.unchecked_into())
                    }
                    .await;

                    if let Ok(stop) = register {
                        while active.get() {
                            sleep(Duration::from_millis(250)).await;
                        }

                        let _ = stop.call0(&JsValue::NULL);
                    }

                    drop(callbacks);
                });
            }

            spawn_local(async move {
                while running.get() {
                    let before = *revision.borrow();

                    match Command::GetRecordingState
                        .invoke::<RecordingState>(serde_json::json!({}))
                        .await
                    {
                        Ok(value) if running.get() => {
                            if *revision.borrow() == before {
                                *latest.borrow_mut() = Some(value.clone());
                                state.set(Some(value));
                            }
                        }
                        Err(value) if running.get() => {
                            error.set(value);
                            break;
                        }
                        _ => break,
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            });

            move || {
                active.set(false);
                drop(callbacks);
            }
        });
    }

    let available = state.as_ref().is_some_and(|s| {
        s.recording_id.is_some()
            && matches!(
                s.status,
                RecordingStatus::Recording | RecordingStatus::Suspended
            )
    });

    let public_reason = state.as_ref().and_then(|s| {
        if s.is_practice {
            Some("Public is unavailable for Practice Tool games.")
        } else if s.is_custom == Some(true) {
            Some("Public is unavailable for custom games.")
        } else if s.is_custom.is_none() {
            Some("Public is unavailable until the game type is known.")
        } else {
            None
        }
    });

    let mode = state.as_ref().map_or(ShareMode::Private, |s| s.share.mode);
    let retry = {
        let refresh = refresh.clone();
        Callback::from(move |_| refresh.set(*refresh + 1))
    };

    let mode_buttons = [
        (ShareMode::Private, "Private"),
        (ShareMode::Live, "Live"),
        (ShareMode::Public, "Public"),
    ]
    .into_iter()
    .map(|(target, label)| {
        let state = state.clone();
        let busy = busy.clone();
        let error = error.clone();
        let notice = notice.clone();
        let latest = latest.clone();
        let revision = revision.clone();
        let disabled =
            *busy || !available || (target == ShareMode::Public && public_reason.is_some());

        let click = Callback::from(move |_| {
            if *busy {
                return;
            }

            busy.set(true);
            notice.set(String::new());

            let state = state.clone();
            let busy = busy.clone();
            let error = error.clone();
            let latest = latest.clone();
            let revision = revision.clone();

            *revision.borrow_mut() += 1;

            spawn_local(async move {
                match Command::SetShareMode
                    .invoke::<ShareState>(serde_json::json!({
                        "mode": target
                    }))
                    .await
                {
                    Ok(share) => {
                        let current = latest.borrow().clone();

                        if let Some(mut value) = current {
                            value.share = share;
                            *revision.borrow_mut() += 1;
                            *latest.borrow_mut() = Some(value.clone());
                            state.set(Some(value));
                        }

                        error.set(String::new());
                    }
                    Err(value) => error.set(value),
                }

                busy.set(false);
            });
        });

        html! {
            <button
                disabled={disabled}
                aria-pressed={(mode == target).to_string()}
                onclick={click}
            >
                { label }
            </button>
        }
    });

    html! {
        <section class={classes!("sharing-bar")} aria-label={"Share this match"}>
            <strong>{ "Share" }</strong>
            <div class={classes!("watch-actions")}>{ for mode_buttons }</div>
            <p>{
                match mode {
                    ShareMode::Private => "Private: only this device can view the recording.",
                    ShareMode::Live => "Live: anyone with the code can watch and rewind until the match ends.",
                    ShareMode::Public => {
                        "Public: anyone with the code can watch and replay after the match ends."
                    }
                }
            }</p>

            if !available {
                <p>{ "Start a match to share it." }</p>
            }

            if let Some(reason) = public_reason {
                <p>{ reason }</p>
            }

            if let Some(value) = state.as_ref() {
                if let Some(code) = value.share.code.as_ref() {
                    <strong class={classes!("share-code")}>
                        { tutorlolv2_protocol::display_code(code) }
                    </strong>
                }

                {
                    for [
                        ("Copy code", value.share.code.clone()),
                        ("Copy link", value.share.url.clone()),
                    ]
                    .into_iter()
                    .filter_map(|(label, text)| {
                        text.map(|text| {
                            let notice = notice.clone();
                            let onclick = Callback::from(move |_| {
                                let text = text.clone();
                                let notice = notice.clone();
                                spawn_local(async move {
                                    match JsFuture::from(
                                        web_sys::window()
                                            .unwrap()
                                            .navigator()
                                            .clipboard()
                                            .write_text(&text),
                                    )
                                    .await
                                    {
                                        Ok(_) => notice.set(format!("{label}: copied")),
                                        Err(_) => notice
                                            .set("Could not copy. Select the code and copy it manually.".into()),
                                    }
                                });
                            });
                            html! { <button onclick={onclick}>{ label }</button> }
                        })
                    })
                }

                if let Some(upload_error) = value.upload.last_error.as_ref() {
                    <p role={"status"}>
                        { format!("Recording upload will retry: {upload_error}") }
                    </p>
                }
            }
            <p
                class={classes!("sharing-feedback")}
                role={"status"}
                aria-live={"polite"}
            >
                { if *busy {
                        "Updating sharing..."
                    } else {
                        &*notice
                    } }
            </p>

            if !error.is_empty() {
                <p role={"alert"}>
                    { &*error }
                    <button onclick={retry}>{ "Retry connection" }</button>
                </p>
            }
        </section>
    }
}
