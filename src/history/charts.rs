use super::source::{HistorySource, IpcHistory};
use crate::replay::cache::CachedChunk;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    time::Duration,
};
use tutorlolv2::riot::RiotRealtime;
use tutorlolv2_protocol::ChunkInfo;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

pub const LABELS: [&str; 19] = [
    "Level",
    "Attack damage",
    "Ability power",
    "Armor",
    "Magic resistance",
    "Max health",
    "Attack speed (attacks/s)",
    "Flat armor penetration",
    "Flat magic penetration",
    "Armor penetration (%)",
    "Magic penetration (%)",
    "Crit chance (%)",
    "Crit damage (%)",
    "Kills",
    "Deaths",
    "Assists",
    "CS",
    "Occupied item slots",
    "Game time",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub time: f32,
    pub captured: i64,
    pub seq: u32,
    pub values: [Option<f32>; 19],
}

// Equivalent to PlayerStats::base100, also available without the calculation engine.
pub const fn penetration_percent(raw: f32) -> f32 {
    if raw.is_finite() {
        (1.0 - raw).clamp(0.0, 1.0) * 100.0
    } else {
        f32::NAN
    }
}

pub fn extract(frame: &RiotRealtime<'_>, captured: i64, seq: u32) -> Sample {
    let active = &frame.active_player;
    let host = frame
        .all_players
        .iter()
        .find(|p| !active.riot_id.is_empty() && p.riot_id == active.riot_id);

    let mut values = [None; 19];

    if let Some(host) = host {
        let s = &active.champion_stats;

        let raw = [
            active.level.get() as f32,
            s.attack_damage,
            s.ability_power,
            s.armor,
            s.magic_resist,
            s.max_health,
            s.attack_speed,
            s.armor_penetration_flat,
            s.magic_penetration_flat,
            penetration_percent(s.armor_penetration_percent),
            penetration_percent(s.magic_penetration_percent),
            s.crit_chance,
            s.crit_damage,
            host.scores.kills as f32,
            host.scores.deaths as f32,
            host.scores.assists as f32,
            host.scores.creep_score as f32,
            host.items
                .iter()
                .filter(|i| tutorlolv2::ItemId::from_riot_id(i.item_id).is_some())
                .count() as f32,
            frame.game_data.game_time,
        ];

        for (out, value) in values.iter_mut().zip(raw) {
            *out = value.is_finite().then_some(value)
        }
    }

    Sample {
        time: frame.game_data.game_time,
        captured,
        seq,
        values,
    }
}

pub fn normalize(mut points: Vec<Sample>) -> Vec<Sample> {
    points.retain(|p| p.time.is_finite());
    points.sort_by(|a, b| {
        a.time
            .total_cmp(&b.time)
            .then(a.captured.cmp(&b.captured))
            .then(a.seq.cmp(&b.seq))
    });

    let mut out: Vec<Sample> = vec![];

    for point in points {
        if out.last().is_some_and(|p| p.time == point.time) {
            out.pop();
        }

        out.push(point)
    }

    out
}
pub fn segments(points: &[Sample], metric: usize) -> Vec<Vec<(f32, f32)>> {
    let mut result = vec![];
    let mut current = vec![];
    let mut previous = None;

    for point in points {
        if let Some(value) = point.values[metric] {
            if previous.is_some_and(|t: f32| point.time - t > 2.) && !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }

            current.push((point.time, value));
            previous = Some(point.time);
        } else {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }

            previous = None;
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[derive(Properties, PartialEq)]
pub struct ChartsProps {
    pub id: String,
    pub chunks: Vec<ChunkInfo>,
    pub onseek: Callback<f32>,
    pub revision: u32,
}

#[component]
pub fn Charts(props: &ChartsProps) -> Html {
    let samples = use_state(Vec::<Sample>::new);
    let status = use_state(String::new);
    let selected = use_state(|| 0usize);
    let cache = use_memo(props.id.clone(), |_| {
        RefCell::new(BTreeMap::<u32, (ChunkInfo, Vec<Sample>)>::new())
    });

    {
        let samples = samples.clone();
        let status = status.clone();
        let cache = cache.clone();

        use_effect_with(
            (props.id.clone(), props.chunks.clone(), props.revision),
            move |(id, chunks, _)| {
                let active = Rc::new(Cell::new(true));
                let running = active.clone();
                let (id, mut chunks) = (id.clone(), chunks.clone());

                chunks.sort_by(|a, b| a.first_game_time.total_cmp(&b.first_game_time));

                spawn_local(async move {
                    let mut all = vec![];
                    let mut failures = 0;

                    status.set("Reading observed stats…".into());

                    for metadata in chunks {
                        if !running.get() {
                            return;
                        }

                        let cached = cache
                            .borrow()
                            .get(&metadata.seq)
                            .filter(|(old, _)| *old == metadata)
                            .map(|(_, p)| p.clone());

                        let points = if let Some(points) = cached {
                            points
                        } else {
                            match IpcHistory::frames(&id, metadata.seq).await {
                                Ok(bytes) => {
                                    match CachedChunk::decode(bytes, metadata.frames_format) {
                                        Ok(chunk) if chunk.validate(&metadata).is_ok() => chunk
                                            .borrow_dependent()
                                            .frames
                                            .iter()
                                            .map(|(captured, frame)| {
                                                extract(frame, *captured, metadata.seq)
                                            })
                                            .collect(),
                                        _ => {
                                            failures += 1;
                                            vec![]
                                        }
                                    }
                                }
                                Err(_) => {
                                    failures += 1;
                                    vec![]
                                }
                            }
                        };

                        if !running.get() {
                            return;
                        }

                        if !points.is_empty() {
                            cache
                                .borrow_mut()
                                .insert(metadata.seq, (metadata, points.clone()));
                        }

                        all.extend(points);
                        sleep(Duration::from_millis(0)).await;
                    }

                    if running.get() {
                        samples.set(normalize(all));
                        status.set(if failures>0{format!("Observed stats incomplete: {failures} chunks unavailable. Refresh details to retry.")}else{"Observed input only. Gaps indicate missing capture; viewer item simulations do not affect these values.".into()});
                    }
                });

                move || active.set(false)
            },
        );
    }

    let end = samples.last().map_or(1., |p| p.time.max(1.));
    let point = samples.get((*selected).min(samples.len().saturating_sub(1)));
    let onselect = {
        let selected = selected.clone();
        let samples = samples.clone();
        let onseek = props.onseek.clone();

        Callback::from(move |event: InputEvent| {
            if let Ok(index) = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value()
                .parse()
            {
                selected.set(index);

                if let Some(point) = samples.get(index) {
                    onseek.emit(point.time);
                }
            }
        })
    };

    let figures = LABELS[..18]
        .iter()
        .enumerate()
        .map(|(metric, label)| render_chart(&samples, metric, label, end));

    let table_rows = LABELS[..18].iter().enumerate().map(|(index, label)| {
        let value = point
            .and_then(|point| point.values[index])
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "Unavailable".into());

        html! {
            <tr>
                <th scope={"row"} class={classes!("py-1", "px-2", "border-b", "border-zinc-600")}>
                    { *label }
                </th>
                <td class={classes!("py-1", "px-2", "border-b", "border-zinc-600")}>{ value }</td>
            </tr>
        }
    });

    html! {
        <section class={classes!("history-charts")} aria-label={"Observed host stats"}>
            <h2>{ "Observed host stats" }</h2>
            <p role={"status"}>{ &*status }</p>

            if samples.is_empty() {
                <p>{ "No observed stats available." }</p>
            } else {
                <label for={"observed-time"}>{ "Selected captured sample" }</label>
                <input
                    id={"observed-time"}
                    type={"range"}
                    min={"0"}
                    max={(samples.len()-1).to_string()}
                    value={selected.to_string()}
                    oninput={onselect}
                />
                <p>
                    { point.map(|p|format!("Captured game time {}",super::list::duration(p.time))).unwrap_or_default() }
                </p>
                <div
                    class={classes!("history-chart-grid", "grid", "grid-cols-[repeat(auto-fit,minmax(min(100%,260px),1fr))]", "gap-3")}
                >
                    { for figures }
                </div>
                <table class={classes!("w-full", "text-left", "my-4")}>
                    <caption>{ "Selected-time observed values" }</caption>
                    <tbody>{ for table_rows }</tbody>
                </table>
            }

        </section>
    }
}

fn render_chart(samples: &[Sample], metric: usize, label: &str, end: f32) -> Html {
    let maximum = samples
        .iter()
        .filter_map(|point| point.values[metric])
        .fold(1f32, f32::max);

    let paths = segments(samples, metric).into_iter().map(|segment| {
        let mut path = String::new();

        for (index, (time, value)) in segment.iter().enumerate() {
            let x = time / end * 400.;
            let y = 95. - value / maximum * 90.;

            if index == 0 {
                path.push_str(&format!("M{x},{y}"));
            } else {
                path.push_str(&format!("H{x}V{y}"));
            }
        }

        html! { <path d={path} fill={"none"} stroke={"#38bdf8"} stroke-width={"2"} /> }
    });

    html! {
        <figure class={classes!("min-w-0", "p-2", "bg-zinc-900", "border", "border-zinc-600")}>
            <figcaption>{ label }</figcaption>
            <svg
                class={classes!("w-full", "h-auto")}
                viewBox={"0 0 400 100"}
                role={"img"}
                aria-label={format!("{label} over game time, 0 to {end:.0} seconds. Values also available in the selected-time table.")}
            >
                { for paths }
            </svg>
        </figure>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charts_split_invalid_metrics_and_missing_capture() {
        let p = |time, value| Sample {
            time,
            captured: 0,
            seq: 0,
            values: [value; 19],
        };
        assert_eq!(
            segments(
                &[
                    p(1., Some(2.)),
                    p(2., None),
                    p(3., Some(3.)),
                    p(8., Some(4.))
                ],
                0
            )
            .len(),
            3
        );
    }
    #[test]
    fn latest_capture_wins_equal_game_time() {
        let mut a = Sample {
            time: 1.,
            captured: 1,
            seq: 1,
            values: [Some(1.); 19],
        };
        let mut b = a.clone();
        b.captured = 2;
        b.values[0] = Some(2.);
        a.seq = 9;
        assert_eq!(normalize(vec![b.clone(), a]), vec![b]);
    }
    #[test]
    fn fixture_host_uses_active_identity_and_base100() {
        let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.bincode"));
        let (mut frame, _): (RiotRealtime<'_>, _) =
            bincode::borrow_decode_from_slice(bytes, crate::utils::BINCODE_CONFIG).unwrap();
        frame.active_player.champion_stats.armor_penetration_percent = 0.7;
        frame.active_player.champion_stats.crit_chance = 25.;
        let sample = extract(&frame, 0, 0);
        assert!((sample.values[9].unwrap() - 30.).abs() < 0.001);
        assert_eq!(sample.values[11], Some(25.));
        #[cfg(feature = "local-calculation")]
        assert_eq!(
            penetration_percent(0.7),
            frame
                .active_player
                .champion_stats
                .base100()
                .armor_penetration_percent
        );
        let host_index = frame
            .all_players
            .iter()
            .position(|player| player.riot_id == frame.active_player.riot_id)
            .unwrap();
        let expected_cs = frame.all_players[host_index].scores.creep_score;
        frame.all_players.swap(host_index, 1);
        frame.all_players[0].scores.creep_score = 999;
        frame.active_player.champion_stats.attack_damage = f32::NAN;
        frame.active_player.champion_stats.magic_penetration_percent = f32::INFINITY;
        let sample = extract(&frame, 0, 0);
        assert_eq!(sample.values[16], Some(expected_cs as f32));
        assert_eq!(sample.values[1], None);
        assert_eq!(sample.values[10], None);

        frame.active_player.riot_id = "missing";
        assert!(extract(&frame, 0, 0).values.iter().all(Option::is_none));
    }
}
