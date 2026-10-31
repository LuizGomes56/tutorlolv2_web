use super::{
    charts::Charts,
    list::{page_classes, upload, use_history_revision},
    source::*,
};
use crate::{Route, replay::OwnerReplay};
use std::{cell::Cell, rc::Rc, time::Duration};
use tutorlolv2_protocol::*;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HistoryDetailsViewProps {
    pub id: String,
}

#[component]
pub fn HistoryDetailsView(props: &HistoryDetailsViewProps) -> Html {
    let selected_time = use_state(|| None::<(f32, u32)>);
    let details = use_state(|| None::<HistoryDetails>);
    let latest = use_mut_ref(|| None::<HistoryDetails>);

    *latest.borrow_mut() = (*details).clone();

    let error = use_state(String::new);
    let retry = use_state(|| 0u32);
    let revision = use_history_revision();
    let mounted = use_mut_ref(|| true);
    let mutation_revision = use_mut_ref(|| 0u64);

    {
        let mounted = mounted.clone();

        use_effect_with((), move |_| {
            *mounted.borrow_mut() = true;
            move || *mounted.borrow_mut() = false
        });
    }

    {
        let details = details.clone();
        let error = error.clone();
        let mutation_revision = mutation_revision.clone();

        use_effect_with((props.id.clone(), *retry, *revision), move |(id, _, _)| {
            let active = Rc::new(Cell::new(true));
            let running = active.clone();
            let id = id.clone();

            spawn_local(async move {
                while running.get() {
                    let before = *mutation_revision.borrow();

                    match IpcHistory::details(&id).await {
                        Ok(value) if running.get() => {
                            let ongoing = value.recording.summary.ended_at_ms.is_none();

                            if *mutation_revision.borrow() == before {
                                details.set(Some(value));
                            }

                            error.set(String::new());

                            if !ongoing {
                                break;
                            }
                        }
                        Err(e) if running.get() => {
                            error.set(format!("Details: {}", e.message));
                            break;
                        }
                        _ => break,
                    }

                    sleep(Duration::from_secs(5)).await;
                }
            });
            move || active.set(false)
        });
    }

    let refresh = {
        let retry = retry.clone();
        Callback::from(move |_| retry.set(*retry + 1))
    };

    html! {
        <main class={page_classes()}>
            <Link<Route> to={Route::History}>{ "← Back to history" }</Link<Route>>
            <h1>{ "Recorded match" }</h1>
            <button onclick={refresh}>{ "Refresh details" }</button>
            if !error.is_empty() {
                <p role={"alert"}>{ &*error }</p>
            }
            if let Some(value)=details.as_ref() {
                <h2>
                    { format!("{} · {}",value.recording.summary.recording.champion,value.recording.summary.recording.active_riot_id) }
                </h2>
                <p>{ upload(&value.entry) }</p>
                if let Some(e)=&value.entry.last_error {
                    <p role={"status"}>{ e }</p>
                }
                <section
                    class={classes!("sharing-bar")}
                    aria-label={"Finished match sharing"}
                >
                    <h2>{ "Sharing" }</h2>
                    <p>{ format!("Confirmed: {:?}",value.recording.share.mode) }</p>
                    <ShareControls
                        key={props.id.clone()}
                        id={props.id.clone()}
                        recording={value.recording.clone()}
                        onchange={{
                            let details = details.clone();
                            let latest = latest.clone();
                            let mutation_revision = mutation_revision.clone();

                            Callback::from(move |share: ShareState| {
                                *mutation_revision.borrow_mut() += 1;
                                if let Some(mut value) = latest.borrow().clone() {
                                    value.recording.share = share.clone();
                                    value.entry.share = share;
                                    details.set(Some(value));
                                }
                            })
                        }}
                    />
                    if value.recording.summary.ended_at_ms.is_none() {
                        <p>
                            { "Sharing controls become available after the server confirms the match ended." }
                        </p>
                    }
                    if value.recording.summary.recording.is_practice||value.recording.summary.recording.is_custom!=Some(false) {
                        <p>
                            { "Public is unavailable for practice/custom games or while the game type is unknown." }
                        </p>
                    }
                    if let Some(code)=&value.recording.share.code {
                        <strong class={classes!("share-code")}>
                            { tutorlolv2_protocol::display_code(code) }
                        </strong>
                    }
                    if let Some(url)=&value.recording.share.url {
                        <a href={url.clone()}>{ "Open public viewer" }</a>
                    }
                </section>
                <OwnerReplay
                    key={props.id.clone()}
                    id={props.id.clone()}
                    recording={value.recording.clone()}
                    selected_time={*selected_time}
                />
                <Charts
                    key={props.id.clone()}
                    id={props.id.clone()}
                    chunks={value.recording.chunks.clone()}
                    revision={retry.wrapping_add(*revision)}
                    onseek={Callback::from(move |time| selected_time.set(Some((time, selected_time.map_or(0, |(_, generation)| generation.wrapping_add(1))))))}
                />
            } else if error.is_empty() {
                <p role={"status"}>{ "Loading owner recording…" }</p>
            }
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct ShareControlsProps {
    id: String,
    recording: RecordingDetails,
    onchange: Callback<ShareState>,
}

#[component]
fn ShareControls(props: &ShareControlsProps) -> Html {
    let busy = use_state(|| false);
    let pending = use_mut_ref(|| false);
    let error = use_state(String::new);
    let notice = use_state(String::new);
    let mounted = use_mut_ref(|| true);

    {
        let mounted = mounted.clone();
        use_effect_with((), move |_| move || *mounted.borrow_mut() = false);
    }

    let modes = [
        (HistoryShareMode::Private, "Private"),
        (HistoryShareMode::Public, "Public"),
    ]
    .into_iter()
    .map(|(mode, label)| {
        let summary = &props.recording.summary;
        let restricted =
            summary.recording.is_practice || summary.recording.is_custom != Some(false);

        let disabled = *busy
            || summary.ended_at_ms.is_none()
            || (mode == HistoryShareMode::Public && restricted);

        let selected = ShareMode::from(mode) == props.recording.share.mode;
        let (id, busy, pending, error, mounted, onchange) = (
            props.id.clone(),
            busy.clone(),
            pending.clone(),
            error.clone(),
            mounted.clone(),
            props.onchange.clone(),
        );

        let onclick = Callback::from(move |_| {
            if *pending.borrow() {
                return;
            }

            *pending.borrow_mut() = true;
            busy.set(true);

            let (id, busy, pending, error, mounted, onchange) = (
                id.clone(),
                busy.clone(),
                pending.clone(),
                error.clone(),
                mounted.clone(),
                onchange.clone(),
            );

            spawn_local(async move {
                let result = IpcHistory::share(&id, mode).await;

                if !*mounted.borrow() {
                    return;
                }

                *pending.borrow_mut() = false;
                busy.set(false);

                match result {
                    Ok(share) => {
                        onchange.emit(share);
                        error.set(String::new());
                    }
                    Err(value) => error.set(format!("Sharing: {}", value.message)),
                }
            });
        });

        html! {
            <button disabled={disabled} aria-pressed={selected.to_string()} onclick={onclick}>
                { label }
            </button>
        }
    });

    let copy_buttons = [
        ("Copy code", props.recording.share.code.clone()),
        ("Copy link", props.recording.share.url.clone()),
    ]
    .into_iter()
    .filter_map(|(label, text)| {
        text.map(|text| {
            let notice = notice.clone();
            let mounted = mounted.clone();
            let onclick = Callback::from(move |_| {
                let (text, notice, mounted) = (text.clone(), notice.clone(), mounted.clone());

                spawn_local(async move {
                    let result = wasm_bindgen_futures::JsFuture::from(
                        web_sys::window()
                            .unwrap()
                            .navigator()
                            .clipboard()
                            .write_text(&text),
                    )
                    .await;

                    if !*mounted.borrow() {
                        return;
                    }

                    notice.set(if result.is_ok() {
                        format!("{label}: copied")
                    } else {
                        "Could not copy. Select the code and copy it manually.".into()
                    });
                });
            });

            html! { <button onclick={onclick}>{ label }</button> }
        })
    });

    html! {
        <>
            <div class={classes!("watch-actions")}>{ for modes }{ for copy_buttons }</div>
            <p role={"status"} aria-live={"polite"}>
                { if *busy {
                        "Updating sharing..."
                    } else if !notice.is_empty() {
                        &*notice
                    } else {
                        "Owner access remains available with Private sharing."
                    } }
            </p>
            if !error.is_empty() {
                <p role={"alert"}>{ &*error }</p>
            }
        </>
    }
}
