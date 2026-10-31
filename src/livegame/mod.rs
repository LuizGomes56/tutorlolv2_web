use {
    crate::{
        components::image::{Image, ImageType},
        hover::Docs,
    },
    std::{num::NonZeroU8, rc::Rc},
    tutorlolv2::{
        AdaptiveType, ChampionId, GameMap, ItemId, Position, RuneId, TypeMetadata,
        model::{
            AbilityLevels, BasicStats, Damages, Dragons, EnemyStats, PlayerStats, SimpleStats, Team,
        },
    },
    yew::{html::IntoPropValue, prelude::*},
};

mod components;
mod page;

pub use components::*;
pub use page::{Livegame, LivegameView};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentPlayer {
    pub riot_id: Rc<str>,
    pub base_stats: BasicStats,
    pub bonus_stats: BasicStats,
    pub current_stats: PlayerStats,
    pub level: NonZeroU8,
    pub team: Team,
    pub adaptive_type: AdaptiveType,
    pub position: Position,
    pub champion_id: ChampionId,
    pub game_map: GameMap,
}

#[derive(Debug, PartialEq)]
pub struct Scoreboard {
    pub riot_id: Rc<str>,
    pub assists: u8,
    pub creep_score: u16,
    pub deaths: u8,
    pub kills: u8,
    pub champion_id: ChampionId,
    pub position: Position,
    pub team: Team,
}

impl IntoPropValue<Html> for &Scoreboard {
    fn into_prop_value(self) -> Html {
        self.into()
    }
}

impl From<&Scoreboard> for Html {
    fn from(scoreboard: &Scoreboard) -> Html {
        let Scoreboard {
            riot_id,
            assists,
            creep_score,
            deaths,
            kills,
            champion_id,
            position,
            ..
        } = scoreboard;

        html! {
            <td>
                <div class={classes!(
                    "grid", "grid-cols-[auto_1fr_auto]",
                    "gap-2", "items-center"
                )}>
                    <div data_docs={Docs::global(*champion_id).encode()} class={classes!("relative", "shrink-0")}>
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
            </td>
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Enemy {
    pub riot_id: Rc<str>,
    pub damages: Damages,
    pub siml_items: [Damages; ItemId::L_SIML],
    pub base_stats: SimpleStats,
    pub bonus_stats: SimpleStats,
    pub current_stats: EnemyStats,
    pub real_armor: i32,
    pub real_magic_resist: i32,
    pub level: NonZeroU8,
    pub champion_id: ChampionId,
    pub team: Team,
    pub position: Position,
}

impl Enemy {
    pub fn total_damage(&self) -> i32 {
        self.damages.sum()
    }

    pub fn item_scores(&self) -> [(i32, ItemId); ItemId::L_SIML] {
        let mut list = core::array::from_fn(|i| (self.siml_items[i].sum(), ItemId::SIML[i]));

        list.sort_unstable_by(|left, right| right.cmp(left));
        list
    }
}

impl From<tutorlolv2::model::Realtime<'_>> for Game {
    fn from(data: tutorlolv2::model::Realtime<'_>) -> Self {
        Self {
            current_player: CurrentPlayer {
                riot_id: data.current_player.riot_id.into(),
                base_stats: data.current_player.base_stats,
                bonus_stats: data.current_player.bonus_stats,
                current_stats: data.current_player.current_stats,
                level: data.current_player.level,
                team: data.current_player.team,
                adaptive_type: data.current_player.adaptive_type,
                position: data.current_player.position,
                champion_id: data.current_player.champion_id,
                game_map: data.current_player.game_map,
            },
            enemies: data
                .enemies
                .into_iter()
                .map(|enemy| Enemy {
                    riot_id: enemy.riot_id.into(),
                    damages: enemy.damages,
                    siml_items: enemy.siml_items,
                    base_stats: enemy.base_stats,
                    bonus_stats: enemy.bonus_stats,
                    current_stats: enemy.current_stats,
                    real_armor: enemy.real_armor,
                    real_magic_resist: enemy.real_magic_resist,
                    level: enemy.level,
                    champion_id: enemy.champion_id,
                    team: enemy.team,
                    position: enemy.position,
                })
                .collect(),
            scoreboard: data
                .scoreboard
                .into_iter()
                .map(|score| Scoreboard {
                    riot_id: score.riot_id.into(),
                    assists: score.assists,
                    creep_score: score.creep_score,
                    deaths: score.deaths,
                    kills: score.kills,
                    champion_id: score.champion_id,
                    position: score.position,
                    team: score.team,
                })
                .collect(),
            items_meta: data.items_meta.into(),
            runes_meta: data.runes_meta.into(),
            game_time: data.game_time,
            ability_levels: data.ability_levels,
            dragons: data.dragons,
        }
    }
}
