// use crate::model::{AbilityLevels, BasicStats, Damages, Dragons, PlayerStats, SimpleStats, Team};
// use bincode::Decode;
// use std::rc::Rc;
// use tutorlolv2_gen::{
//     AbilityId, AdaptativeType, ChampionId, Ctx, GameMap, ItemId, L_SIML, MergeData, Position,
//     RuneId, TypeMetadata,
// };

// mod components;
// mod glue;
// mod page;

// pub use page::Overlay;

// #[derive(Decode)]
// pub struct Game {
//     pub current_player: CurrentPlayer,
//     pub enemies: Rc<[Enemy]>,
//     pub scoreboard: Box<[Scoreboard]>,
//     pub items_meta: Rc<[TypeMetadata<ItemId>]>,
//     pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
//     pub game_time: u32,
//     pub ability_levels: AbilityLevels,
//     pub dragons: Dragons,
// }

// #[derive(Decode)]
// pub struct CurrentPlayer {
//     pub riot_id: Box<str>,
//     pub base_stats: BasicStats,
//     pub bonus_stats: BasicStats,
//     pub current_stats: PlayerStats,
//     pub level: u8,
//     pub team: Team,
//     pub adaptative_type: AdaptativeType,
//     pub position: Position,
//     pub champion_id: ChampionId,
//     pub game_map: GameMap,
// }

// #[derive(Decode)]
// pub struct Scoreboard {
//     pub riot_id: Box<str>,
//     pub assists: u8,
//     pub creep_score: u16,
//     pub deaths: u8,
//     pub kills: u8,
//     pub champion_id: ChampionId,
//     pub position: Position,
//     pub team: Team,
// }

// #[derive(Decode, PartialEq)]
// pub struct Enemy {
//     pub riot_id: Box<str>,
//     pub damages: Damages,
//     pub siml_items: [Damages; L_SIML],
//     pub base_stats: SimpleStats,
//     pub bonus_stats: SimpleStats,
//     pub current_stats: SimpleStats,
//     pub real_armor: i32,
//     pub real_magic_resist: i32,
//     pub level: u8,
//     pub champion_id: ChampionId,
//     pub team: Team,
//     pub position: Position,
//     pub eval_ctx: Ctx,
// }
