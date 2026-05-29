mod components;
mod page;

use crate::{
    components::image::{Image, ImageType},
    model::{
        AbilityLevels, BasicStats, Damages, Dragons, EnemyStats, PlayerStats, SimpleStats, Team,
    },
    utils::encode_offset,
};
use bincode::Decode;
use std::rc::Rc;
use tutorlolv2_gen::{
    AdaptiveType, CastId, ChampionId, GameMap, ItemId, ItemsBitSet, L_SIML, Position, RuneId,
    SIMULATED_ITEMS_ENUM, SIMULATED_ITEMS_METADATA, TypeMetadata,
};
use yew::prelude::*;

pub use components::*;
pub use page::Livegame;

#[derive(Debug, Decode)]
pub struct Game {
    pub current_player: CurrentPlayer,
    pub enemies: Rc<[Enemy]>,
    pub scoreboard: Rc<[Scoreboard]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
    pub game_time: u32,
    pub ability_levels: AbilityLevels,
    pub dragons: Dragons,
}

#[derive(Debug, Decode)]
pub struct CurrentPlayer {
    pub riot_id: Rc<str>,
    pub base_stats: BasicStats,
    pub bonus_stats: BasicStats,
    pub current_stats: PlayerStats,
    pub level: u8,
    pub team: Team,
    pub adaptive_type: AdaptiveType,
    pub position: Position,
    pub champion_id: ChampionId,
    pub game_map: GameMap,
}

#[derive(Debug, Decode, PartialEq)]
pub struct Scoreboard {
    pub riot_id: Box<str>,
    pub assists: u8,
    pub creep_score: u16,
    pub deaths: u8,
    pub kills: u8,
    pub champion_id: ChampionId,
    pub position: Position,
    pub team: Team,
}

impl Scoreboard {
    pub fn to_html(&self) -> Html {
        let Scoreboard {
            riot_id,
            assists,
            creep_score,
            deaths,
            kills,
            champion_id,
            position,
            ..
        } = self;

        let data_offset = encode_offset(core::array::from_ref(&champion_id.formula()));

        html! {
            <div class={classes!(
                "grid", "grid-cols-[auto_1fr_auto]",
                "gap-2", "items-center"
            )}>
                <div {data_offset} class={classes!("relative", "shrink-0")}>
                    <Image
                        class={classes!("w-8", "h-8", "overflow-hidden")}
                        src={ImageType::from(champion_id)}
                    />
                </div>
                <div class={classes!(
                    "flex", "min-w-0", "flex-col", "gap-0.5",
                )}>
                    <span class={classes!(
                        "text-left",
                        "text-xs",
                        "text-std-100",
                        "truncate"
                    )}>
                        {riot_id.split_once('#').map(|(left, _)| left).unwrap_or(riot_id)}
                    </span>
                    <div class={classes!(
                        "flex", "items-center", "gap-1.5", "min-w-0",
                        "justify-between",
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "gap-1.5", "min-w-0",
                        )}>
                            <Image
                                class={classes!("w-3.5", "h-3.5", "shrink-0")}
                                src={ImageType::Position(*position)}
                            />
                            <span class={classes!(
                                "truncate",
                                "text-xs",
                                "text-std-400"
                            )}>
                                {champion_id.name()}
                            </span>
                        </div>
                        <span class={classes!(
                            "shrink-0",
                            "text-xs",
                            "text-std-400"
                        )}>
                            {"(CS: "}{creep_score}{")"}
                        </span>
                    </div>
                </div>
                <div class={classes!(
                    "shrink-0",
                    "w-24",
                    "text-sm",
                    "text-std-200",
                    "whitespace-nowrap"
                )}>
                    {kills}
                    <span class={classes!("text-std-400")}>{" / "}</span>
                    {deaths}
                    <span class={classes!("text-std-400")}>{" / "}</span>
                    {assists}
                </div>
            </div>
        }
    }
}

#[derive(Debug, Decode, PartialEq)]
pub struct Enemy {
    pub riot_id: Box<str>,
    pub damages: Damages,
    pub siml_items: [Damages; L_SIML],
    pub base_stats: SimpleStats,
    pub bonus_stats: SimpleStats,
    pub current_stats: EnemyStats,
    pub real_armor: i32,
    pub real_magic_resist: i32,
    pub level: u8,
    pub champion_id: ChampionId,
    pub team: Team,
    pub position: Position,
}

impl Enemy {
    pub fn total_damage(&self) -> i32 {
        self.damages.sum()
    }

    pub fn item_scores(&self) -> Vec<(i32, ItemId)> {
        let mut list = self
            .siml_items
            .iter()
            .zip(SIMULATED_ITEMS_ENUM)
            .map(|(s, item)| (s.sum(), item))
            .collect::<Vec<_>>();

        list.sort_unstable();
        list.reverse();
        list
    }
}
