use crate::{
    model::{
        AbilityLevels, BasicStats, Damages, Dragons, EnemyStats, PlayerStats, SimpleStats,
        ValueException,
    },
    utils::EnumCast,
};
use bincode::{Decode, Encode};
use std::rc::Rc;
use tutorlolv2_gen::{AdaptativeType, ChampionId, ItemId, L_MSTR, L_TWRD, RuneId, TypeMetadata};

mod components;
mod page;
mod reducer;

pub use page::Calculator;

#[derive(Clone, Debug, Encode, PartialEq)]
pub struct InputGame {
    pub active_player: Player,
    pub enemy_players: Vec<Rc<PlayerData<EnemyStats>>>,
    pub dragons: Dragons,
}

#[derive(Clone, Debug, Default, Encode, PartialEq)]
pub struct Player {
    pub runes: Vec<RuneId>,
    pub rune_exceptions: Vec<ValueException>,
    pub abilities: AbilityLevels,
    pub data: PlayerData<PlayerStats>,
}

/// Minimum required data to qualify a valid enemy player, and calculate
/// damages against this target. Field `stats` is required, but if `infer_stats`
/// is set to true, the enemy's stats will be inferred and this field will be ignored.
/// The same happens with `is_mega_gnar`, which can be set to true, but will only
/// have effect if field `champion_id` is also of type [`ChampionId::Gnar`].
/// Field `stacks` is useless if the associated champion does not have any special
/// characteristics that are related to stack-scaling
#[derive(Clone, Debug, Encode, PartialEq)]
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

impl<T: Default> Default for PlayerData<T> {
    fn default() -> Self {
        Self {
            stats: T::default(),
            items: Vec::new(),
            item_exceptions: Vec::new(),
            stacks: 0,
            level: 1,
            infer_stats: true,
            is_mega_gnar: false,
            champion_id: ChampionId::random(),
        }
    }
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct FinalEnemy {
    pub damages: Damages,
    pub base_stats: SimpleStats,
    pub bonus_stats: SimpleStats,
    pub current_stats: EnemyStats,
    pub real_armor: i32,
    pub real_magic_resist: i32,
    pub level: u8,
    pub champion_id: ChampionId,
}

#[derive(Clone, Copy, Debug, Decode, PartialEq)]
pub struct FinalPlayer {
    pub current_stats: PlayerStats,
    pub base_stats: BasicStats,
    pub bonus_stats: BasicStats,
    pub level: u8,
    pub adaptative_type: AdaptativeType,
    pub champion_id: ChampionId,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct Game {
    pub monster_damages: [Damages; L_MSTR],
    pub current_player: FinalPlayer,
    pub enemies: Rc<[FinalEnemy]>,
    pub tower_damages: [i32; L_TWRD],
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}
