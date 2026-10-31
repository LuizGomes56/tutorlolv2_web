pub(crate) mod cache;
pub(crate) mod controller;
pub(crate) mod extent;
pub(crate) mod timeline;

#[cfg(feature = "tauri")]
use {
    crate::livegame::{Game, LivegameView},
    cache::CachedChunk,
    std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        rc::Rc,
        time::Duration,
    },
    tutorlolv2_protocol::RecordingDetails,
    wasm_bindgen::JsCast,
    web_sys::HtmlInputElement,
    yew::{
        platform::{spawn_local, time::sleep},
        prelude::*,
    },
};

#[cfg(feature = "tauri")]
#[derive(Properties, PartialEq)]
pub struct OwnerReplayProps {
    pub id: String,
    pub recording: RecordingDetails,
    #[prop_or_default]
    pub selected_time: Option<(f32, u32)>,
}

#[cfg(feature = "tauri")]
#[component]
pub fn OwnerReplay(props: &OwnerReplayProps) -> Html {
    let time = use_state(|| {
        props
            .recording
            .chunks
            .first()
            .map_or(0., |c| c.first_game_time)
    });
    let playing = use_state(|| false);
    let speed = use_state(|| 1u32);
    let retry = use_state(|| 0u32);
    let game = use_state(|| None::<Rc<Game>>);
    let error = use_state(String::new);
    let cache = use_memo(
        (),
        |_| RefCell::new(BTreeMap::<u32, Rc<CachedChunk>>::new()),
    );

    {
        let time = time.clone();

        use_effect_with(props.selected_time, move |selected| {
            if let Some(selected) = selected {
                time.set(selected.0);
            }
        });
    }

    let mut chunks = props.recording.chunks.clone();
    chunks.sort_by(|a, b| a.first_game_time.total_cmp(&b.first_game_time));

    let end = props
        .recording
        .summary
        .recording
        .game_length_s
        .map(|v| v as f32)
        .unwrap_or(props.recording.summary.recording.last_game_time);

    controller::use_replay_controller(
        controller::ReplayRequest {
            source: cache::ReplaySource::Owner(props.id.clone()),
            chunks: chunks.clone(),
            time: *time,
            enabled: true,
            revision: *retry,
        },
        cache.clone(),
        game.clone(),
        error.clone(),
        Callback::noop(),
    );

    {
        let time = time.clone();
        let playing = playing.clone();

        use_effect_with(
            (*playing, *time, *speed, end),
            move |(play, position, speed, end)| {
                let active = Rc::new(Cell::new(true));
                let running = active.clone();
                let (play, position, speed, end) = (*play, *position, *speed, *end);

                spawn_local(async move {
                    if play {
                        sleep(Duration::from_millis(250)).await;

                        if running.get() {
                            let next = (position + 0.25 * speed as f32).min(end);
                            time.set(next);

                            if next >= end {
                                playing.set(false)
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
        Callback::from(move |value: f32| time.set(value.clamp(0., end.max(0.))))
    };

    let oninput = {
        let seek = seek.clone();

        Callback::from(move |e: InputEvent| {
            if let Ok(value) = e
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .parse()
            {
                seek.emit(value)
            }
        })
    };

    let onkey = {
        let seek = seek.clone();
        let position = *time;

        Callback::from(move |e: KeyboardEvent| {
            if e.is_composing() {
                return;
            }

            if e.target()
                .and_then(|v| v.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|v| {
                    matches!(v.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                        || v.get_attribute("contenteditable").is_some()
                })
            {
                return;
            }

            match e.key().as_str() {
                "ArrowLeft" => {
                    e.prevent_default();
                    seek.emit(position - 10.)
                }
                "ArrowRight" => {
                    e.prevent_default();
                    seek.emit(position + 10.)
                }
                _ => {}
            }
        })
    };

    html! {
        <section
            class={classes!("watch-timeline")}
            aria-label={"Owner replay"}
            onkeydown={onkey}
            tabindex={"0"}
        >
            <label for={"owner-time"}>
                { format!("Replay {} / {}",crate::history::list::duration(*time),crate::history::list::duration(end)) }
            </label>
            <div class={classes!("watch-coverage")}>
                { for timeline::gaps(&chunks,end).iter().map(|(a,b)|html!{<span class={classes!("watch-gap")} style={format!("left:{}%;width:{}%",a/end.max(1.)*100.,(b-a)/end.max(1.)*100.)} title={format!("Missing capture {a:.0}–{b:.0}s")}/>}) }
            </div>
            <input
                id={"owner-time"}
                type={"range"}
                min={"0"}
                max={end.max(0.).to_string()}
                step={"1"}
                value={time.to_string()}
                oninput={oninput}
                disabled={chunks.is_empty()}
            />
            <div class={classes!("watch-actions")}>
                <button
                    disabled={chunks.is_empty()}
                    onclick={{let playing=playing.clone();Callback::from(move |_|playing.set(!*playing))}}
                >
                    { if *playing{"Pause"}else{"Play"} }
                </button>
                { for [1,4,16].into_iter().map(|value|{let speed=speed.clone();html!{<button aria-pressed={(*speed==value).to_string()} onclick={Callback::from(move |_|speed.set(value))}>{format!("{value}×")}</button>}}) }
            </div>
            <p>
                { "← / → seek 10 seconds. Hatched regions and gaps between samples were not captured." }
            </p>

            if !error.is_empty() {
                <p role={"status"}>
                    { &*error }
                    <button
                        onclick={{let retry=retry.clone();Callback::from(move |_|retry.set(*retry+1))}}
                    >
                        { "Retry replay" }
                    </button>
                </p>
            }

            <LivegameView
                data={(*game).clone()}
                error={String::new()}
            />
        </section>
    }
}
