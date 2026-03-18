mod components;
mod page;

use crate::model::{
    AbilityLevels, BasicStats, Damages, Dragons, EnemyStats, PlayerStats, SimpleStats, Team,
};
use bincode::Decode;
use std::rc::Rc;
use tutorlolv2_gen::{
    AdaptiveType, ChampionId, GameMap, ItemId, ItemsBitSet, L_SIML, Position, RuneId,
    SIMULATED_ITEMS_METADATA, TypeMetadata,
};

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
    pub fn item_scores(&self, champion_id: ChampionId) -> Vec<(i32, ItemId)> {
        let array: [i32; L_SIML] = core::array::from_fn(|i| {
            let damage = &self.siml_items[i];
            damage.attacks.basic_attack
                + damage.attacks.onhit_damage.minimum_damage
                + damage.attacks.onhit_damage.maximum_damage
                + damage.attacks.critical_strike
                + damage.abilities.iter().sum::<i32>()
                + damage.items.iter().sum::<i32>()
                + damage.runes.iter().sum::<i32>()
        });

        let mut seen = ItemsBitSet::EMPTY;

        let mut list = Position::ARRAY
            .into_iter()
            .flat_map(|position| champion_id.recommended_items(position))
            .filter_map(|&item| {
                SIMULATED_ITEMS_METADATA
                    .iter()
                    .position(|m| m.kind == item)
                    .map(|index| (array[index], item))
            })
            .filter(|&(_, item)| seen.insert(item.index()))
            .collect::<Vec<_>>();

        list.sort_unstable();

        list
    }
}
