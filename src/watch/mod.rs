pub(crate) mod api;
use crate::replay::timeline;

use crate::{
    Route,
    livegame::{Game, LivegameView},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    time::Duration,
};
use tutorlolv2_protocol::{ShareInfo, ShareStatus, display_code, normalize_code};
use wasm_bindgen::JsCast;
use web_sys::{AbortController, HtmlInputElement};
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WatchProps {
    #[prop_or_default]
    pub code: String,
}

#[component]
pub fn Watch(props: &WatchProps) -> Html {
    let input = use_state(|| props.code.clone());
    let invalid = use_state(|| false);

    {
        let input = input.clone();
        let invalid = invalid.clone();
        use_effect_with(props.code.clone(), move |code| {
            input.set(code.clone());
            invalid.set(false);
        });
    }

    let navigator = use_navigator().unwrap();
    let code = normalize_code(&props.code);
    let submit = {
        let input = input.clone();
        let invalid = invalid.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if let Some(code) = normalize_code(&input) {
                invalid.set(false);
                navigator.push(&Route::WatchCode { code });
            } else {
                invalid.set(true);
            }
        })
    };

    let change = {
        let input = input.clone();

        Callback::from(move |event: InputEvent| {
            input.set(event.target_unchecked_into::<HtmlInputElement>().value())
        })
    };

    html! {
        <main class={classes!("watch-page")}>
            <header class={classes!("watch-entry")}>
                <h1>{ "Watch a match" }</h1>
                <form onsubmit={submit} novalidate={true}>
                    <label for={"share-code"}>{ "Share code" }</label>
                    <input
                        id={"share-code"}
                        value={(*input).clone()}
                        oninput={change}
                        placeholder={"ABCD-EFGH"}
                        autocomplete={"off"}
                        spellcheck={"false"}
                        aria-invalid={invalid.to_string()}
                        aria-describedby={"code-help"}
                    />
                    <button type={"submit"}>{ "Watch" }</button>
                    <p id={"code-help"} role={if *invalid {"alert"} else {"status"}}>
                        { if *invalid {
                            "Enter the eight-character code from the host. Spaces and dashes are optional."
                            } else {
                                "No account needed. Ask the host for their share code."
                            } }
                    </p>
                </form>
            </header>
            if let Some(code) = code {
                <Viewer key={code.clone()} code={code.clone()} />
            } else if !props.code.is_empty() {
                <p role={"alert"}>
                    { "Invalid share code. Enter the eight-character code above." }
                </p>
            }
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct ViewerProps {
    code: String,
}

#[component]
fn Viewer(props: &ViewerProps) -> Html {
    let metadata = use_state(|| None::<ShareInfo>);
    let message = use_state(|| "Loading match...".to_string());
    let live_message = use_state(String::new);
    let fatal = use_state(|| false);
    let retry = use_state(|| 0u32);
    let live = use_state(|| None::<Rc<Game>>);
    let replay = use_state(|| None::<Rc<Game>>);
    let follow = use_state(|| true);
    let time = use_state(|| 0f32);
    let playing = use_state(|| false);
    let speed = use_state(|| 1u32);
    let replay_message = use_state(String::new);
    let cache = use_memo((), |_| {
        RefCell::new(BTreeMap::<u32, Rc<api::CachedChunk>>::new())
    });

    // Revalidate even ended Public replays so revocation clears an open viewer.
    {
        let metadata = metadata.clone();
        let time = time.clone();
        let message = message.clone();
        let fatal = fatal.clone();
        let live = live.clone();
        let replay = replay.clone();
        let cache = cache.clone();

        use_effect_with((props.code.clone(), *retry), move |(code, _)| {
            let code = code.clone();
            let abort = AbortController::new().unwrap();
            let task_abort = abort.clone();
            let active = Rc::new(Cell::new(true));
            let running = active.clone();

            spawn_local(async move {
                let mut initial = true;

                while running.get() {
                    match api::info(&code, &task_abort).await {
                        Ok(info) if running.get() => {
                            if initial {
                                time.set(
                                    info.chunks
                                        .first()
                                        .map_or(info.summary.first_game_time, |chunk| {
                                            chunk.first_game_time
                                        }),
                                );
                                initial = false;
                            }

                            fatal.set(false);
                            message.set(String::new());
                            metadata.set(Some(info));
                        }

                        Err(error) if running.get() => {
                            let revoked = error.starts_with("This match")
                                || error.starts_with("Code not found");

                            message.set(error);

                            if revoked {
                                fatal.set(true);
                                metadata.set(None);
                                live.set(None);
                                replay.set(None);
                                cache.borrow_mut().clear();
                            }

                            break;
                        }
                        _ => break,
                    }

                    sleep(Duration::from_secs(5)).await;
                }
            });

            move || {
                active.set(false);
                abort.abort();
            }
        });
    }

    // Cancel fetch on route change, retry, revocation or transition to ended.
    {
        let live = live.clone();
        let message = live_message.clone();
        let fatal = fatal.clone();
        let metadata = metadata.clone();
        let replay = replay.clone();
        let cache = cache.clone();
        let enabled = metadata
            .as_ref()
            .is_some_and(|info| !matches!(info.status, ShareStatus::Ended { .. }))
            && !*fatal;

        use_effect_with(
            (props.code.clone(), *retry, enabled),
            move |(code, _, enabled)| {
                let abort = AbortController::new().unwrap();
                let task_abort = abort.clone();
                let active = Rc::new(Cell::new(true));
                let running = active.clone();
                let code = code.clone();
                let enabled = *enabled;

                spawn_local(async move {
                    let mut seq = 0u64;

                    while enabled && running.get() {
                        let result = async {
                            let response =
                                api::get(&format!("{code}/live?after={seq}"), &task_abort).await?;

                            if response.status() == 204 {
                                return Ok(None);
                            }

                            if response.status() == 410 {
                                return Err(api::status_error(410));
                            }

                            if !response.ok() {
                                return Err(api::status_error(response.status()));
                            }

                            let header = |name| {
                                response
                                    .headers()
                                    .get(name)
                                    .map_err(|_| "Missing live metadata".to_string())?
                                    .ok_or_else(|| "Missing live metadata".to_string())
                            };

                            let next = header("X-Frame-Seq")?
                                .parse::<u64>()
                                .map_err(|_| "Invalid live sequence")?;

                            let _captured = header("X-Captured-At")?
                                .parse::<i64>()
                                .map_err(|_| "Invalid capture time")?;

                            let _game_time = header("X-Game-Time")?
                                .parse::<f32>()
                                .map_err(|_| "Invalid game time")?;

                            if next <= seq {
                                return Err(
                                    "The live stream returned an old frame. Retry to reconnect."
                                        .to_string(),
                                );
                            }

                            let bytes = api::bytes(&response).await?;
                            let game = api::frame(&bytes).await?;

                            seq = next;
                            Ok(Some(game))
                        }
                        .await;

                        if !running.get() {
                            break;
                        }

                        match result {
                            Ok(Some(game)) => {
                                live.set(Some(Rc::new(game)));
                                message.set(String::new());
                            }
                            Ok(None) => {}
                            Err(error) => {
                                // A live 410 can mean Public ended. Revalidate before clearing replay access.
                                if error.starts_with("This match") {
                                    match api::info(&code, &task_abort).await {
                                        Ok(info) if running.get() => {
                                            metadata.set(Some(info));
                                            message.set(String::new());
                                        }
                                        Err(error) if running.get() => {
                                            fatal.set(true);
                                            metadata.set(None);
                                            live.set(None);
                                            replay.set(None);
                                            cache.borrow_mut().clear();
                                            message.set(error);
                                        }
                                        _ => {}
                                    }
                                } else {
                                    message.set(error);
                                }
                                break;
                            }
                        }
                    }
                });

                move || {
                    active.set(false);
                    abort.abort();
                }
            },
        );
    }

    let ended = metadata
        .as_ref()
        .is_some_and(|m| matches!(m.status, ShareStatus::Ended { .. }));

    let end = metadata
        .as_ref()
        .map_or(0., |m| m.summary.last_game_time)
        .max(live.as_ref().map_or(0., |g| g.game_time as f32));

    let viewing_live = *follow && !ended;

    // Access lifecycle stays here; the source-neutral controller owns bounded replay work.
    let access_error = {
        let message = message.clone();
        let fatal = fatal.clone();
        let metadata = metadata.clone();
        let live = live.clone();
        let replay = replay.clone();
        let cache = cache.clone();

        Callback::from(move |error: String| {
            message.set(error);
            fatal.set(true);
            metadata.set(None);
            live.set(None);
            replay.set(None);
            cache.borrow_mut().clear();
        })
    };
    crate::replay::controller::use_replay_controller(
        crate::replay::controller::ReplayRequest {
            source: crate::replay::cache::ReplaySource::Capability(props.code.clone()),
            chunks: metadata
                .as_ref()
                .map(|value| value.chunks.clone())
                .unwrap_or_default(),
            time: *time,
            enabled: !viewing_live && !*fatal && metadata.is_some(),
            revision: *retry,
        },
        cache.clone(),
        replay.clone(),
        replay_message.clone(),
        access_error,
    );

    {
        let time = time.clone();
        let playing = playing.clone();

        use_effect_with(
            (
                *playing,
                *speed,
                *time,
                end,
                replay_message.starts_with("Loading replay"),
            ),
            move |(play, speed, position, end, buffering)| {
                let active = Rc::new(Cell::new(true));
                let running = active.clone();
                let (play, speed, position, end) = (*play && !*buffering, *speed, *position, *end);

                spawn_local(async move {
                    if play {
                        sleep(Duration::from_millis(250)).await;

                        if running.get() {
                            let next = (position + 0.25 * speed as f32).min(end);
                            time.set(next);

                            if next >= end {
                                playing.set(false);
                            }
                        }
                    }
                });
                move || active.set(false)
            },
        );
    }

    let seek = {
        let time = time.clone();
        let follow = follow.clone();

        Callback::from(move |value: f32| {
            follow.set(false);
            time.set(value.clamp(0., end));
        })
    };

    let onseek = {
        let seek = seek.clone();

        Callback::from(move |event: InputEvent| {
            if let Ok(value) = event
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .parse()
            {
                seek.emit(value);
            }
        })
    };

    let onkey = {
        let seek = seek.clone();
        let position = if viewing_live { end } else { *time };

        Callback::from(move |event: KeyboardEvent| {
            if !event.is_composing()
                && !event
                    .target()
                    .and_then(|value| value.dyn_into::<web_sys::Element>().ok())
                    .is_some_and(|element| {
                        matches!(element.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                            || element.get_attribute("contenteditable").is_some()
                    })
            {
                match event.key().as_str() {
                    "ArrowLeft" => {
                        event.prevent_default();
                        seek.emit(position - 10.);
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        seek.emit(position + 10.);
                    }
                    _ => {}
                }
            }
        })
    };

    let toggle_play = {
        let playing = playing.clone();
        let follow = follow.clone();

        Callback::from(move |_| {
            follow.set(false);
            playing.set(!*playing);
        })
    };

    let go_live = {
        let follow = follow.clone();
        let playing = playing.clone();
        let time = time.clone();

        Callback::from(move |_| {
            if *follow {
                time.set(end);
            }

            follow.set(!*follow);
            playing.set(false);
        })
    };

    let retry_click = {
        let retry = retry.clone();

        Callback::from(move |_| retry.set(*retry + 1))
    };

    let selected = if viewing_live
        || (!*playing
            && live
                .as_ref()
                .is_some_and(|game| (*time - game.game_time as f32).abs() < 0.5))
    {
        (*live).clone()
    } else {
        (*replay).clone()
    };

    let coverage = metadata
        .as_ref()
        .map(|m| timeline::gaps(&m.chunks, end))
        .unwrap_or_default();

    let coverage_markers = coverage.iter().map(|(first, last)| {
        html! {
            <span
                class={classes!("watch-gap")}
                style={format!(
                    "left:{}%;width:{}%",
                    first / end.max(1.0) * 100.0,
                    (last - first) / end.max(1.0) * 100.0)
                }
                title={format!("Not recorded: {first:.0}–{last:.0} s")}
            />
        }
    });

    let speed_buttons = [1, 4, 16].into_iter().map(|value| {
        let speed = speed.clone();

        html! {
            <button
                aria-pressed={(*speed==value).to_string()}
                onclick={Callback::from(move |_|speed.set(value))}
            >
                { format!("{value}×") }
            </button>
        }
    });

    html! {
        <section aria-label={"Match viewer"}>
            <div class={classes!("watch-toolbar")}>
                <strong class={classes!("share-code")}>{ display_code(&props.code) }</strong>
                if let Some(info) = metadata.as_ref() {
                    <span>
                        { format!("{} · {}",info.summary.champion, info.summary.active_riot_id) }
                    </span>
                    <strong>
                        { match &info.status {
                                ShareStatus::Live { .. } => {
                                    if viewing_live {
                                        "● LIVE"
                                    } else {
                                        "Viewing replay · live continues"
                                    }
                                }
                                ShareStatus::HostDisconnected => "Waiting for host",
                                ShareStatus::Ended { .. } => "Match ended · replay",
                            } }
                    </strong>
                }
            </div>
            if !message.is_empty() {
                <div class={classes!("watch-status")} role={"status"}>
                    { &*message }
                    <button onclick={retry_click.clone()}>{ "Retry" }</button>
                </div>
            }
            if !live_message.is_empty() {
                <div class={classes!("watch-status")} role={"status"}>
                    { &*live_message }
                    <button onclick={retry_click.clone()}>{ "Retry" }</button>
                </div>
            }
            if metadata.is_some() && !*fatal {
                <div class={classes!("watch-timeline")} onkeydown={onkey}>
                    <label for={"replay-time"}>
                        { format!(
                                "Replay time: {:.0} / {:.0} s",
                                if viewing_live { end } else { *time },
                                end
                            ) }
                    </label>
                    <div class={classes!("watch-coverage")} aria-label={"Recording coverage"}>
                        { for coverage_markers }
                    </div>
                    <input
                        id={"replay-time"}
                        type={"range"}
                        min={"0"}
                        max={end.to_string()}
                        step={"1"}
                        value={if viewing_live { end.to_string() } else { time.to_string() }}
                        oninput={onseek}
                        aria-describedby={"timeline-help"}
                    />
                    <div class={classes!("watch-actions")}>
                        <button
                            onclick={toggle_play}
                            disabled={end <= 0.0 || metadata.as_ref().is_none_or(|info| info.chunks.is_empty())}
                        >
                            { if *playing { "Pause" } else { "Play" } }
                        </button>
                        { for speed_buttons }
                        if !ended {
                            <button aria-pressed={viewing_live.to_string()} onclick={go_live}>
                                { "Follow live" }
                            </button>
                        }
                    </div>
                    <p id={"timeline-help"}>
                        { "← / → seek 10 seconds. Hatched areas were not recorded. Sealed replay may trail live by up to a minute." }
                    </p>
                    if !viewing_live && !replay_message.is_empty() {
                        <p role={"status"}>{ &*replay_message }</p>
                    }
                    if !coverage.is_empty() {
                        <p class={classes!("watch-gap-text")}>
                            { "Missing coverage: " }
                            { coverage.iter().map(|(a,b)|format!("{a:.0}–{b:.0}s")).collect::<Vec<_>>().join(", ") }
                        </p>
                    }
                </div>
                <LivegameView
                    data={selected}
                    error={if viewing_live {
                        "Waiting for the host’s next frame...".to_string()
                    } else {
                        String::new()
                    }}
                />
            }
        </section>
    }
}
