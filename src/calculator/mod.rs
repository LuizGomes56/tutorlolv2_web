use {
    crate::utils::{EnumCast, tray::Tray},
    std::{num::NonZeroU8, rc::Rc},
    tutorlolv2::{
        ChampionId, ItemId, L_MSTR, L_TWRD, RuneId, TypeMetadata,
        model::{AbilityLevels, Damages, OutputCurrentPlayer, OutputEnemy, PlayerStats},
    },
};

mod components;
mod exceptions;
mod page;
mod reducer;

pub use exceptions::ExceptionMap;
pub use page::Calculator;

#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerData<T> {
    pub stats: T,
    pub items: Tray<ItemId>,
    pub item_exceptions: ExceptionMap<ItemId>,
    pub stacks: u32,
    pub level: NonZeroU8,
    pub champion_id: ChampionId,
    pub is_mega_gnar: bool,
    pub infer_stats: bool,
}

impl<T: Default> Default for PlayerData<T> {
    fn default() -> Self {
        let champion_id = ChampionId::random();
        let recommended_items = champion_id.recommended_items(champion_id.main_position());
        let mut items = Vec::with_capacity(recommended_items.len());

        let mut item_exceptions = ExceptionMap::default();

        for &item in recommended_items {
            item_exceptions.push_item(item, true, |item| items.push(item));
        }

        Self {
            stats: T::default(),
            items: items.into_iter().collect(),
            item_exceptions,
            stacks: 0,
            level: const { NonZeroU8::new(18).unwrap() },
            is_mega_gnar: false,
            champion_id,
            infer_stats: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub monster_damages: [Damages; L_MSTR],
    pub current_player: OutputCurrentPlayer,
    pub enemies: Rc<[OutputEnemy]>,
    pub tower_damages: [i32; L_TWRD],
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

impl From<tutorlolv2::model::OutputGame> for Game {
    fn from(output: tutorlolv2::model::OutputGame) -> Self {
        Self {
            monster_damages: output.monster_damages,
            current_player: output.current_player,
            enemies: output.enemies.into(),
            tower_damages: output.tower_damages,
            items_meta: output.items_meta.into(),
            runes_meta: output.runes_meta.into(),
        }
    }
}
