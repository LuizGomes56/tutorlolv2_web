use super::{
    item_selector::{make_inventory, use_rune_tray_callback},
    selector_dialog::{Search, SelectorDialog},
};
use crate::{
    calculator::{
        page::PlayerProps,
        reducer::{LastAction, PlayerAction},
    },
    components::image::{Image, ImageType},
    hover::Docs,
    utils::tray::TrayAction,
};
use strum::VariantArray;
use tutorlolv2::RuneId;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum RunePath {
    Precision,
    Domination,
    Sorcery,
    Resolve,
    Inspiration,
    Other,
}

impl RunePath {
    const ALL: [Self; 6] = [
        Self::Precision,
        Self::Domination,
        Self::Sorcery,
        Self::Resolve,
        Self::Inspiration,
        Self::Other,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::Precision => "Precision",
            Self::Domination => "Domination",
            Self::Sorcery => "Sorcery",
            Self::Resolve => "Resolve",
            Self::Inspiration => "Inspiration",
            Self::Other => "Shards & legacy",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Precision => "path-precision",
            Self::Domination => "path-domination",
            Self::Sorcery => "path-sorcery",
            Self::Resolve => "path-resolve",
            Self::Inspiration => "path-inspiration",
            Self::Other => "path-other",
        }
    }
    fn icon(self) -> RuneId {
        match self {
            Self::Precision => RuneId::PressTheAttack,
            Self::Domination => RuneId::Electrocute,
            Self::Sorcery => RuneId::SummonAery,
            Self::Resolve => RuneId::GraspOfTheUndying,
            Self::Inspiration => RuneId::FirstStrike,
            Self::Other => RuneId::AdaptiveForce,
        }
    }
}

fn rune_path(rune: RuneId) -> RunePath {
    use RuneId::*;
    match rune {
        PressTheAttack | LethalTempo | FleetFootwork | Conqueror | AbsorbLife | Overheal
        | Triumph | PresenceOfMind | LegendAlacrity | LegendTenacity | LegendBloodline
        | LegendHaste | CoupDeGrace | CutDown | LastStand => RunePath::Precision,
        Electrocute | Predator | DarkHarvest | HailOfBlades | CheapShot | TasteOfBlood
        | SuddenImpact | ZombieWard | GhostPoro | EyeballCollection | GrislyMementos
        | SixthSense | DeepWard | RavenousHunter | IngeniousHunter | RelentlessHunter
        | UltimateHunter | TreasureHunter => RunePath::Domination,
        SummonAery | ArcaneComet | PhaseRush | NullifyingOrb | ManaflowBand | NimbusCloak
        | AxiomArcanist | Transcendence | Celerity | AbsoluteFocus | Scorch | Waterwalking
        | GatheringStorm | TheUltimateHat => RunePath::Sorcery,
        GraspOfTheUndying | Aftershock | Guardian | Demolish | FontOfLife | ShieldBash
        | Conditioning | SecondWind | BonePlating | Overgrowth | Revitalize | Unflinching
        | IronSkin | MirrorShell | Chrysalis => RunePath::Resolve,
        GlacialAugment | UnsealedSpellbook | FirstStrike | HextechFlashtraption
        | MagicalFootwear | PerfectTiming | FutureSMarket | MinionDematerializer
        | BiscuitDelivery | CosmicInsight | ApproachVelocity | TimeWarpTonic | CashBack
        | TripleTonic | JackOfAllTrades | Kleptomancy | PrototypeOmnistone | CelestialBody => {
            RunePath::Inspiration
        }
        _ => RunePath::Other,
    }
}

#[derive(PartialEq, Properties)]
pub struct RuneSelectorProps {
    pub player_props: PlayerProps,
    pub is_open: UseStateHandle<bool>,
}

#[component]
pub fn RuneSelector(props: &RuneSelectorProps) -> Html {
    let player_props = &props.player_props;
    let player = &player_props.player;
    let champion_id = player.data.champion_id;
    let query = use_state(String::new);
    let path = use_state(|| None::<RunePath>);
    let remove = use_rune_tray_callback(player_props, TrayAction::RemoveById);
    let replace = use_rune_tray_callback(player_props, TrayAction::Replace);
    let insert = {
        let player_props = player_props.clone();
        Callback::from(move |rune| {
            player_props.last_action.replace(LastAction::CurrentPlayer);
            player_props
                .player
                .dispatch(PlayerAction::Tray(TrayAction::Insert(rune)));
        })
    };
    let normalized_query = query.trim().to_ascii_lowercase();
    let visible = RuneId::VARIANTS
        .iter()
        .copied()
        .filter(|rune| {
            rune.name().to_ascii_lowercase().contains(&normalized_query)
                && path.is_none_or(|selected| rune_path(*rune) == selected)
        })
        .collect::<Vec<_>>();
    let reset = {
        let query = query.clone();
        let path = path.clone();
        Callback::from(move |_| {
            query.set(String::new());
            path.set(None);
        })
    };
    let all_paths = {
        let path = path.clone();
        Callback::from(move |_| path.set(None))
    };
    html! {
        <SelectorDialog title="Runes" kind="runes" is_open={props.is_open.clone()}>
            <div class="selector-workspace rune-workspace">
                <nav class="selector-filters rune-paths" aria-label="Filter runes by path">
                    <div class="filter-heading">{"Paths"}</div>
                    <button type="button" class={classes!("stat-filter", path.is_none().then_some("is-active"))} onclick={all_paths} aria-pressed={path.is_none().to_string()}><span class="all-filter-icon" aria-hidden="true">{"⊞"}</span><span>{"All runes"}</span></button>
                    {for RunePath::ALL.into_iter().map(|p| {
                        let path = path.clone();
                        html! {
                            <button type="button" class={classes!("stat-filter", "rune-path-filter", p.class(), (*path == Some(p)).then_some("is-active"))}
                                aria-pressed={(*path == Some(p)).to_string()} onclick={Callback::from(move |_| path.set(Some(p)))}>
                                <Image src={ImageType::Rune(p.icon())} /><span>{p.name()}</span>
                            </button>
                        }
                    })}
                </nav>
                <main class="selector-catalog">
                    <div class="catalog-toolbar"><h3>{path.map_or("All runes", |p| p.name())}<span class="catalog-count" role="status">{visible.len()}</span></h3><Search query={query.clone()} placeholder="Search runes…" /></div>
                    <div class="catalog-heading"><span class="catalog-hint">{"Choose any combination of paths"}</span>
                        if path.is_some() || !query.is_empty() { <button type="button" class="text-action" onclick={reset.clone()}>{"Reset filters"}</button> }
                    </div>
                    <div class="catalog-scroll">
                        if visible.is_empty() {
                            <div class="catalog-empty"><span aria-hidden="true">{"◇"}</span><h3>{"No runes found"}</h3><p>{"Try another name or choose a different path."}</p><button type="button" class="empty-reset" onclick={reset}>{"Clear search & filters"}</button></div>
                        } else {
                            {for RunePath::ALL.into_iter().filter_map(|p| {
                                let runes = visible.iter().copied().filter(|rune| rune_path(*rune) == p).collect::<Vec<_>>();
                                if runes.is_empty() { return None; }
                                Some(html! {
                                    <section class={classes!("rune-group", p.class())}>
                                        <h3><span class="rune-path-dot" />{p.name()}<span>{runes.len()}</span></h3>
                                        <div class="rune-catalog-grid">
                                            {for runes.into_iter().map(|rune| {
                                                let insert = insert.clone();
                                                let count = player.runes.iter().filter(|e| e.value == rune).count();
                                                html! {
                                                    <button type="button" key={rune as u8} class={classes!("rune-card", (count > 0).then_some("is-equipped"))}
                                                        aria-label={format!("Add {}", rune.name())} onclick={Callback::from(move |_| insert.emit(rune))} data_docs={Docs::global(rune).encode()}>
                                                        <span class="catalog-art"><Image src={ImageType::Rune(rune)} />
                                                            if count > 0 { <span class="equipped-badge" aria-label={format!("{count} selected")}>{count}</span> }
                                                        </span>
                                                        <span>{rune.name()}</span>
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    </section>
                                })
                            })}
                        }
                    </div>
                </main>
                <aside class="selector-builds rune-builds" aria-label="Selected runes">
                    <div class="shop-detail-scroll">
                        <div class="rune-profile"><Image src={ImageType::Champion(champion_id)} /><span>{"Rune page"}</span><h3>{champion_id.name()}</h3></div>
                        {make_inventory(champion_id, &player.runes, remove, replace)}
                        <div class="selected-rune-names">
                            {for player.runes.iter().map(|entry| html! {<div key={entry.id}><Image src={ImageType::Rune(entry.value)} /><span>{entry.value.name()}</span></div>})}
                        </div>
                        <p class="rune-page-help">{"Click a rune to add it. Your selections update the calculator instantly."}</p>
                    </div>
                </aside>
            </div>
        </SelectorDialog>
    }
}
