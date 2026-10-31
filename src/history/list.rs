use super::source::*;
use crate::{
    Route,
    components::image::{Image, ImageType},
};
use std::{cell::Cell, rc::Rc, str::FromStr, time::Duration};
use tutorlolv2::ChampionId;
use tutorlolv2_protocol::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{Function, Promise};
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};
use yew_router::prelude::*;

#[hook]
pub fn use_history_revision() -> UseStateHandle<u32> {
    let revision = use_state(|| 0u32);

    {
        let revision = revision.clone();

        use_effect_with((), move |_| {
            let active = Rc::new(Cell::new(true));
            let running = active.clone();
            let dirty = Rc::new(Cell::new(false));
            let changed = dirty.clone();
            let callback = Rc::new(Closure::<dyn FnMut(JsValue)>::new(move |_| {
                changed.set(true)
            }));
            let callback_task = callback.clone();
            let listener_active = active.clone();

            spawn_local(async move {
                let register = async {
                    let events = member(&tauri()?, "event")?;
                    let listen: Function = member(&events, "listen")?
                        .dyn_into()
                        .map_err(|_| ())
                        .map_err(|_| HistoryError {
                            kind: HistoryErrorKind::Other,
                            message: "Event connection unavailable".into(),
                            retryable: true,
                        })?;

                    let name: &str = super::source::Event::HistoryChanged.into();
                    let result = listen
                        .call2(&events, &name.into(), callback_task.as_ref().as_ref())
                        .map_err(|_| HistoryError {
                            kind: HistoryErrorKind::Other,
                            message: "Subscription failed".into(),
                            retryable: true,
                        })?;

                    let stop = JsFuture::from(Promise::resolve(&result))
                        .await
                        .map_err(|_| HistoryError {
                            kind: HistoryErrorKind::Other,
                            message: "Subscription failed".into(),
                            retryable: true,
                        })?;

                    Ok::<Function, HistoryError>(stop.unchecked_into())
                }
                .await;

                if let Ok(stop) = register {
                    while listener_active.get() {
                        sleep(Duration::from_millis(100)).await;
                    }

                    let _ = stop.call0(&JsValue::NULL);
                }

                drop(callback_task);
            });

            spawn_local(async move {
                let mut count = 0;

                while running.get() {
                    sleep(Duration::from_millis(250)).await;

                    if running.get() && dirty.replace(false) {
                        count += 1;
                        revision.set(count);
                    }
                }
            });

            move || {
                active.set(false);
                drop(callback);
            }
        });
    }

    revision
}
pub fn page_classes() -> Classes {
    classes!(
        "watch-page",
        "history-page",
        "min-w-0",
        "wrap-anywhere",
        "[&_:focus-visible]:outline-2",
        "[&_:focus-visible]:outline-sky-400",
        "[&_:focus-visible]:outline-offset-2",
        "[&_[role=status]]:min-h-[1.5em]"
    )
}

pub fn result(value: Option<GameResult>) -> &'static str {
    match value {
        Some(GameResult::Win) => "Win",
        Some(GameResult::Lose) => "Loss",
        Some(GameResult::Unknown) => "Unknown result",
        Some(GameResult::Remake) => "Remake",
        None => "Result pending",
    }
}

pub fn duration(seconds: f32) -> String {
    format!(
        "{}:{:02}",
        (seconds.max(0.) as u32) / 60,
        (seconds.max(0.) as u32) % 60
    )
}

pub fn upload(entry: &HistoryEntry) -> String {
    if entry.recording_id.is_none() {
        return "Waiting to upload".into();
    }

    if entry.summary.is_none() {
        return "Waiting for metadata".into();
    }

    let mut parts = vec![];

    if entry.lost_chunks > 0 {
        parts.push(format!("Capture loss: {} chunks", entry.lost_chunks));
    }

    if entry.pending_chunks > 0 || entry.pending_control {
        parts.push(format!(
            "Uploading: {} chunks{}",
            entry.pending_chunks,
            if entry.pending_control {
                " · control pending"
            } else {
                ""
            }
        ));
    }

    if parts.is_empty() {
        parts.push(
            if entry
                .summary
                .as_ref()
                .is_some_and(|s| s.ended_at_ms.is_none())
            {
                "Ongoing"
            } else {
                "Complete"
            }
            .into(),
        );
    }

    parts.join(" · ")
}

#[component]
pub fn HistoryListView() -> Html {
    let page = use_state(|| None::<HistoryList>);
    let offset = use_state(|| 0u32);
    let refresh = use_state(|| 0u32);
    let revision = use_history_revision();
    let error = use_state(String::new);
    let notice = use_state(String::new);
    let busy = use_state(|| false);
    let lifetime = use_mut_ref(|| true);

    {
        let lifetime = lifetime.clone();

        use_effect_with((), move |_| {
            *lifetime.borrow_mut() = true;
            move || *lifetime.borrow_mut() = false
        });
    }

    {
        let page = page.clone();
        let error = error.clone();

        use_effect_with((*offset, *refresh, *revision), move |(offset, _, _)| {
            let active = Rc::new(Cell::new(true));
            let running = active.clone();
            let offset = *offset;

            spawn_local(async move {
                match IpcHistory::list(offset).await {
                    Ok(value) if running.get() => {
                        page.set(Some(value));
                        error.set(String::new());
                    }
                    Err(value) if running.get() => error.set(format!("List: {}", value.message)),
                    _ => {}
                }
            });

            move || active.set(false)
        });
    }

    let retry = {
        let refresh = refresh.clone();

        Callback::from(move |_| refresh.set(*refresh + 1))
    };

    let repair = {
        let busy = busy.clone();
        let error = error.clone();
        let notice = notice.clone();
        let lifetime = lifetime.clone();
        let refresh = refresh.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }

            busy.set(true);

            let (busy, error, notice, lifetime, refresh) = (
                busy.clone(),
                error.clone(),
                notice.clone(),
                lifetime.clone(),
                refresh.clone(),
            );

            spawn_local(async move {
                let result = IpcHistory::repair().await;

                if !*lifetime.borrow() {
                    return;
                }

                busy.set(false);

                match result {
                    Ok(()) => {
                        notice.set("Repair queued. Server metadata and pending recordings are being reconciled. Hidden flags cannot be recovered from a lost index.".into());
                        refresh.set(*refresh + 1);
                    }
                    Err(e) => error.set(format!("Repair: {}", e.message)),
                }
            });
        })
    };

    html! {
        <main class={page_classes()}>
            <h1>{ "Match history" }</h1>
            <div class={classes!("watch-actions")}>
                <button onclick={retry.clone()}>{ "Refresh" }</button>
                <button disabled={*busy} onclick={repair}>{ "Repair index" }</button>
            </div>
            <p role={"status"}>{ &*notice }</p>

            if !error.is_empty() {
                <p role={"alert"}>
                    { &*error }
                    <button onclick={retry}>{ "Retry" }</button>
                </p>
            }

            if let Some(page)=page.as_ref() {
                if page.refreshing {
                    <p role={"status"}>{ "Refreshing cached history…" }</p>
                }

                if let Some(error)=&page.refresh_error {
                    <p role={"status"}>{ format!("Showing cached metadata. Refresh: {error}") }</p>
                }

                if page.entries.is_empty() {
                    <p>{ "No recorded matches on this page." }</p>
                }

                <div
                    class={classes!("history-rows", "grid", "gap-3", "my-4")}
                >
                    { for page.entries.iter().map(|entry| {
                        let refresh = refresh.clone();
                        html! { <HistoryRow key={identity(entry)} entry={entry.clone()} onhidden={Callback::from(move |_| refresh.set(*refresh + 1))} /> }
                    }) }
                </div>
                <div class={classes!("watch-actions")}>
                    <button
                        disabled={*offset==0}
                        onclick={{let offset=offset.clone();Callback::from(move |_|offset.set(offset.saturating_sub(50)))}}
                    >
                        { "Previous 50" }
                    </button>
                    <span>{ format!("Page {}",*offset/50+1) }</span>
                    <button
                        disabled={!page.has_more}
                        onclick={{let offset=offset.clone();Callback::from(move |_|offset.set(*offset+50))}}
                    >
                        { "Next 50" }
                    </button>
                </div>
            } else {
                <p role={"status"}>{ "Loading cached history…" }</p>
            }
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn provisional() -> HistoryEntry {
        HistoryEntry {
            local_id: Some([1; 16]),
            recording_id: None,
            summary: None,
            share: ShareState {
                mode: ShareMode::Private,
                code: None,
                url: None,
            },
            pending_chunks: 3,
            pending_control: true,
            lost_chunks: 1,
            last_error: None,
            hidden: false,
        }
    }
    #[test]
    fn provisional_identity_never_becomes_a_server_identity() {
        let mut entry = provisional();
        assert!(identity(&entry).starts_with("local:"));
        assert_eq!(upload(&entry), "Waiting to upload");
        entry.recording_id = Some([2; 16]);
        assert_eq!(identity(&entry), uuid([2; 16]));
        assert_eq!(upload(&entry), "Waiting for metadata");
    }

    #[test]
    fn unknown_results_and_champions_stay_honest() {
        use core::str::FromStr;

        assert_eq!(result(None), "Result pending");
        assert_eq!(result(Some(GameResult::Unknown)), "Unknown result");
        assert_eq!(
            ChampionId::from_str("aatrox").ok(),
            Some(ChampionId::Aatrox)
        );
        assert_eq!(ChampionId::from_str("future-champion").ok(), None);
    }
}

#[derive(Properties, PartialEq)]
struct HistoryRowProps {
    entry: HistoryEntry,
    onhidden: Callback<()>,
}

#[component]
fn HistoryRow(props: &HistoryRowProps) -> Html {
    let confirm = use_state(|| false);
    let busy = use_state(|| false);
    let pending = use_mut_ref(|| false);
    let error = use_state(String::new);
    let active = use_mut_ref(|| true);

    {
        let active = active.clone();
        use_effect_with((), move |_| move || *active.borrow_mut() = false);
    }

    let hide = {
        let id = props.entry.recording_id.map(uuid);
        let (busy, pending, error, active, onhidden) = (
            busy.clone(),
            pending.clone(),
            error.clone(),
            active.clone(),
            props.onhidden.clone(),
        );

        Callback::from(move |_| {
            let Some(id) = id.clone() else {
                return;
            };

            if *pending.borrow() {
                return;
            }

            *pending.borrow_mut() = true;
            busy.set(true);

            let (busy, pending, error, active, onhidden) = (
                busy.clone(),
                pending.clone(),
                error.clone(),
                active.clone(),
                onhidden.clone(),
            );
            spawn_local(async move {
                let result = IpcHistory::hide(&id).await;

                if !*active.borrow() {
                    return;
                }

                *pending.borrow_mut() = false;
                busy.set(false);

                match result {
                    Ok(()) => onhidden.emit(()),
                    Err(value) => error.set(format!("Hide: {}", value.message)),
                }
            });
        })
    };

    let request_confirmation = {
        let confirm = confirm.clone();

        Callback::from(move |_| confirm.set(true))
    };

    let cancel = {
        let confirm = confirm.clone();

        Callback::from(move |_| confirm.set(false))
    };

    let entry = &props.entry;

    html! {
        <article
            class={classes!("history-row", "bg-zinc-900", "border", "border-zinc-600", "p-3", "[&_button]:mr-3", "[&_a]:mr-3")}
        >
            if let Some(summary) = &entry.summary {
                <h2 class={classes!("font-semibold", "flex", "items-center", "gap-2")}>
                    if let Ok(champion) = ChampionId::from_str(&summary.recording.champion) {
                        <Image
                            class={classes!("history-champion", "size-8", "object-cover")}
                            src={ImageType::Champion(champion)}
                        />
                        { champion.name() }
                    } else {
                        <span aria-hidden={"true"}>{ "? " }</span>
                        { &summary.recording.champion }
                    }
                </h2>
                <p>
                    { format!("{:?} · Queue {} · {}", summary.recording.game_mode, summary.recording.queue_id.map(|queue|queue.to_string()).unwrap_or_else(||"—".into()), result(summary.recording.result)) }
                </p>
                <p>
                    { format!("{}/{}/{} · {} CS · {}{} · {} s covered", summary.kills, summary.deaths, summary.assists, summary.creep_score, duration(summary.recording.game_length_s.map(|v|v as f32).unwrap_or(summary.recording.last_game_time)), if summary.recording.game_length_s.is_none(){" (incomplete)"}else{""}, summary.coverage_s) }
                </p>
                <p>
                    { web_sys::js_sys::Date::new(&JsValue::from_f64(summary.recording.started_at_ms as f64)).to_locale_string("default", &JsValue::UNDEFINED).as_string().unwrap_or_default() }
                </p>
            } else {
                <h2 class={classes!("font-semibold")}>{ "Waiting for upload / metadata" }</h2>
                <p>{ "Result — · K/D/A — · CS — · Duration —" }</p>
            }
            <p>{ format!("{:?} · {}", entry.share.mode, upload(entry)) }</p>

            if let Some(error) = &entry.last_error {
                <p role={"status"}>{ error }</p>
            }

            if let Some(id) = entry.recording_id.map(uuid) {
                <Link<Route> to={Route::HistoryDetails { id }}>{ "Open replay" }</Link<Route>>

                if *confirm {
                    <p>
                        { "Remove only from this device’s history? Sharing and uploads continue." }
                    </p>
                    <button disabled={*busy} onclick={hide}>{ "Hide on this device" }</button>
                    <button disabled={*busy} onclick={cancel}>{ "Cancel" }</button>
                } else {
                    <button onclick={request_confirmation}>{ "Hide…" }</button>
                }
            }
            <p role={"status"}>{ if *busy {"Saving local hide…"}else{""} }</p>

            if !error.is_empty() {
                <p role={"alert"}>{ &*error }</p>
            }
        </article>
    }
}
