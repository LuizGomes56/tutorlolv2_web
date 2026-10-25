use crate::{
    calculator::{Player, PlayerData},
    utils::{
        Print,
        tray::{Tray, TrayAction, TrayEntry},
    },
};
use derive_more::{Deref, DerefMut};
use std::{num::NonZeroU8, rc::Rc};
use tutorlolv2::{
    ChampionId, ItemId, RuneId,
    model::{EnemyStats, PlayerStats},
    yew::{ReduceApply, ability_levels::AbilityLevelsAction},
};
use yew::Reducible;

pub type EnemyDataAction = DataAction<EnemyStats>;
pub type PlayerDataAction = DataAction<PlayerStats>;

pub enum PlayerAction {
    Tray(TrayAction<Tray<RuneId>, RuneId>),
    ModifyRuneExc((RuneId, u32)),
    Data(PlayerDataAction),
    AbilityLevels(AbilityLevelsAction),
}

pub enum DataAction<T: ReduceApply> {
    Level(NonZeroU8),
    ReplaceStats(*const T),
    Stats(T::Action),
    Stacks(u32),
    InferStats(bool),
    IsMegaGnar(bool),
    Tray(TrayAction<Tray<ItemId>, ItemId>),
    ChampionId(ChampionId),
    ModifyItemExc((ItemId, u32)),
}

pub enum EnemyAction {
    Insert(ChampionId),
    Remove(usize),
    Change(usize, EnemyDataAction),
}

#[derive(Clone, PartialEq, Deref, DerefMut)]
#[repr(transparent)]
pub struct Enemies(Vec<Rc<PlayerData<EnemyStats>>>);

impl Enemies {
    pub const MAX_ENEMIES: usize = 5;
}

impl Default for Enemies {
    fn default() -> Self {
        let mut vector = Vec::with_capacity(Self::MAX_ENEMIES);

        (0..3).for_each(|_| vector.push(Default::default()));

        Self(vector)
    }
}

impl<T: ReduceApply> PlayerData<T> {
    pub fn reduce_mut(&mut self, ally: bool, action: DataAction<T>) {
        match action {
            DataAction::Level(v) => self.level = v,
            DataAction::ReplaceStats(v) => self.stats = unsafe { *v },
            DataAction::Stats(v) => self.stats.apply(v),
            DataAction::Stacks(v) => self.stacks = v,
            DataAction::InferStats(v) => self.infer_stats = v,
            DataAction::IsMegaGnar(v) => self.is_mega_gnar = v,
            DataAction::ChampionId(v) => self.champion_id = v,
            DataAction::Tray(v) => v.custom_apply(&mut self.items, |c, v| {
                self.item_exceptions
                    .push_item(v, ally, |v| c.push(TrayEntry::new(v)))
            }),
            DataAction::ModifyItemExc((item_id, stacks)) => {
                self.item_exceptions.insert(item_id, stacks);
            }
        }
    }
}

impl Reducible for Player {
    type Action = PlayerAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        match action {
            Self::Action::Tray(v) => v.apply(&mut new_mut.runes),
            Self::Action::AbilityLevels(v) => new_mut.abilities.apply(v),
            Self::Action::ModifyRuneExc((rune_id, stacks)) => {
                new_mut.rune_exceptions.insert(rune_id, stacks);
            }
            Self::Action::Data(v) => {
                new_mut.data.reduce_mut(true, v);
            }
        }

        new
    }
}

impl<T: ReduceApply> Reducible for PlayerData<T> {
    type Action = DataAction<T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        new_mut.reduce_mut(false, action);
        new
    }
}

impl Reducible for Enemies {
    type Action = EnemyAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        match action {
            EnemyAction::Insert(champion_id) => match new_mut.len() < Self::MAX_ENEMIES {
                true => {
                    new_mut.push(Rc::new(PlayerData {
                        champion_id,
                        ..Default::default()
                    }));
                }
                false => "Max enemies reached".log(),
            },
            EnemyAction::Change(v, action) => new_mut[v] = new_mut[v].clone().reduce(action),
            EnemyAction::Remove(v) => match !new_mut.is_empty() {
                true => {
                    new_mut.swap_remove(v);
                }
                false => "At least one champion required".log(),
            },
        }

        new
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LastAction {
    Init,
    Any,
    CurrentPlayer,
    EnemyPlayer(usize),
    Replace,
}

impl<T: ReduceApply> DataAction<T> {
    pub fn action(&self, default: LastAction) -> LastAction {
        match self {
            Self::ReplaceStats(_) => LastAction::Replace,
            _ => default,
        }
    }
}
