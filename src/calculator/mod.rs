use crate::model::{Attacks, BasicStats, Damages, Dragons, SimpleStats, Stats, ValueException};
use bincode::{Decode, Encode};
use std::rc::Rc;
use tutorlolv2_gen::{AbilityId, AdaptativeType, ChampionId, ItemId, RuneId, TypeMetadata};

mod components;
pub mod page;
mod reducer;

/// Exact number of resistence variations for jungle monsters
pub const L_MSTR: usize = 7;

/// Number of different plates a tower can have. Each tower can have `0..=5` plates
pub const L_TWRD: usize = 6;

#[derive(Clone, Debug, Encode, PartialEq)]
pub struct InputGame {
    pub active_player: Player,
    pub enemy_players: Vec<Rc<PlayerData<SimpleStats>>>,
    pub dragons: Dragons,
}

#[derive(Clone, Debug, Default, Encode, PartialEq)]
pub struct Player {
    pub runes: Vec<RuneId>,
    pub rune_exceptions: Vec<ValueException>,
    pub abilities: AbilityLevels,
    pub data: PlayerData<Stats>,
}

/// Minimum required data to qualify a valid enemy player, and calculate
/// damages against this target. Field `stats` is required, but if `infer_stats`
/// is set to true, the enemy's stats will be inferred and this field will be ignored.
/// The same happens with `is_mega_gnar`, which can be set to true, but will only
/// have effect if field `champion_id` is also of type [`ChampionId::Gnar`].
/// Field `stacks` is useless if the associated champion does not have any special
/// characteristics that are related to stack-scaling
#[derive(Clone, Debug, Default, Encode, PartialEq)]
pub struct PlayerData<T> {
    pub stats: T,
    pub items: Vec<ItemId>,
    pub item_exceptions: Vec<ValueException>,
    pub stacks: u32,
    pub level: u8,
    pub infer_stats: bool,
    pub is_mega_gnar: bool,
    pub champion_id: ChampionId,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct FinalEnemy {
    pub damages: Damages,
    pub base_stats: SimpleStats,
    pub bonus_stats: SimpleStats,
    pub current_stats: SimpleStats,
    pub real_armor: i32,
    pub real_magic_resist: i32,
    pub level: u8,
    pub champion_id: ChampionId,
}

#[derive(Clone, Copy, Debug, Decode, PartialEq)]
pub struct FinalPlayer {
    pub current_stats: Stats,
    pub base_stats: BasicStats,
    pub bonus_stats: BasicStats,
    pub level: u8,
    pub adaptative_type: AdaptativeType,
    pub champion_id: ChampionId,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct MonsterDamage {
    pub attacks: Attacks,
    pub abilities: Box<[i32]>,
    pub items: Box<[i32]>,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct Game {
    pub monster_damages: [MonsterDamage; L_MSTR],
    pub current_player: FinalPlayer,
    pub enemies: Box<[FinalEnemy]>,
    pub tower_damages: [i32; L_TWRD],
    pub abilities_meta: Box<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Box<[(usize, usize)]>,
    pub items_meta: Box<[TypeMetadata<ItemId>]>,
    pub runes_meta: Box<[TypeMetadata<RuneId>]>,
}

/// Holds the levels of the abilities of a champion
#[derive(Clone, Copy, Debug, Default, Encode, PartialEq)]
pub struct AbilityLevels {
    pub q: u8,
    pub w: u8,
    pub e: u8,
    pub r: u8,
}
