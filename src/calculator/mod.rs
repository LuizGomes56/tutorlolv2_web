use crate::{
    calculator::reducer::push_item,
    model::{
        AbilityLevels, BasicStats, Damages, Dragons, EnemyStats, PlayerStats, SimpleStats,
        ValueException,
    },
    utils::{EnumCast, tray::Tray},
};
use bincode::{Decode, Encode};
use std::{collections::HashMap, hash::Hash, rc::Rc};
use tutorlolv2_gen::{AdaptiveType, ChampionId, ItemId, L_MSTR, L_TWRD, RuneId, TypeMetadata};

mod components;
mod page;
mod reducer;

pub use page::Calculator;

#[derive(Clone, Debug, Default, PartialEq)]
#[repr(transparent)]
pub struct ExceptionMap<T: Default + Eq + Hash> {
    pub inner: HashMap<T, ValueException>,
}

impl<T> Encode for ExceptionMap<T>
where
    T: Default + Encode + Eq + Hash,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.inner.len().encode(encoder)?;
        for value in self.inner.values() {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Encode, PartialEq)]
pub struct InputGame<'a> {
    pub active_player: &'a Player,
    pub enemy_players: &'a [Rc<PlayerData<EnemyStats>>],
    pub dragons: &'a Dragons,
}

#[derive(Clone, Debug, Encode, PartialEq)]
pub struct Player {
    pub runes: Tray<RuneId>,
    pub rune_exceptions: ExceptionMap<RuneId>,
    pub abilities: AbilityLevels,
    pub data: PlayerData<PlayerStats>,
}

impl Default for Player {
    fn default() -> Self {
        let data = PlayerData::default();
        let champion_id = data.champion_id;

        Self {
            runes: champion_id
                .recommended_runes(champion_id.main_position())
                .iter()
                .copied()
                .collect(),
            rune_exceptions: Default::default(),
            abilities: AbilityLevels {
                q: 5,
                w: 5,
                e: 5,
                r: 3,
            },
            data,
        }
    }
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
    pub items: Tray<ItemId>,
    pub item_exceptions: ExceptionMap<ItemId>,
    pub stacks: u32,
    pub level: u8,
    pub infer_stats: bool,
    pub is_mega_gnar: bool,
    pub champion_id: ChampionId,
}

impl<T: Default> Default for PlayerData<T> {
    fn default() -> Self {
        let champion_id = ChampionId::random();
        let recommended_items = champion_id.recommended_items(champion_id.main_position());
        let mut items = Vec::with_capacity(recommended_items.len());
        let mut item_exceptions = ExceptionMap {
            inner: HashMap::with_capacity(recommended_items.len()),
        };
        for &item in recommended_items {
            push_item(&mut item_exceptions, item, true, |item| items.push(item));
        }
        Self {
            stats: T::default(),
            items: items.into_iter().collect(),
            item_exceptions,
            stacks: 0,
            level: 18,
            infer_stats: true,
            is_mega_gnar: false,
            champion_id,
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
    pub adaptive_type: AdaptiveType,
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
