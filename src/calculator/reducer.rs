use crate::{
    calculator::{ExceptionMap, Player, PlayerData},
    model::{AbilityLevelsAction, EnemyStats, PlayerStats, ValueException},
    utils::{
        Print, ReduceApply,
        tray::{Tray, TrayAction, TrayEntry},
    },
};
use std::rc::Rc;
use tutorlolv2_gen::{ChampionId, ItemId, RuneId};
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
    Level(u8),
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

#[derive(Clone, PartialEq)]
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

impl core::ops::Deref for Enemies {
    type Target = Vec<Rc<PlayerData<EnemyStats>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Enemies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn push_item(
    item_exceptions: &mut ExceptionMap<ItemId>,
    v: ItemId,
    ally: bool,
    mut f: impl FnMut(ItemId),
) {
    if ItemId::exceptions(ally).contains(v.index()) {
        let value = ValueException::pack_item_id(v, 0);
        item_exceptions.inner.insert(v, value);
    }
    f(v);
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
                push_item(&mut self.item_exceptions, v, ally, |v| {
                    c.push(TrayEntry::new(v))
                })
            }),
            DataAction::ModifyItemExc((item_id, stacks)) => {
                let value = ValueException::pack_item_id(item_id, stacks);
                self.item_exceptions.inner.insert(item_id, value);
            }
        }
    }
}

impl Reducible for Player {
    type Action = PlayerAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            Self::Action::Tray(v) => v.apply(&mut new.runes),
            Self::Action::AbilityLevels(v) => new.abilities.apply(v),
            Self::Action::ModifyRuneExc((rune_id, stacks)) => {
                let value = ValueException::pack_rune_id(rune_id, stacks);
                new.rune_exceptions.inner.insert(rune_id, value);
            }
            Self::Action::Data(v) => {
                let mut data = new.data;
                data.reduce_mut(true, v);
                new.data = data;
            }
        }
        Rc::new(new)
    }
}

impl<T: ReduceApply> Reducible for PlayerData<T> {
    type Action = DataAction<T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        new.reduce_mut(false, action);
        Rc::new(new)
    }
}

impl Reducible for Enemies {
    type Action = EnemyAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            EnemyAction::Insert(champion_id) => match new.len() < Self::MAX_ENEMIES {
                true => {
                    new.push(Rc::new(PlayerData {
                        champion_id,
                        ..Default::default()
                    }));
                }
                false => "Max enemies reached".log(),
            },
            EnemyAction::Change(v, action) => new[v] = new[v].clone().reduce(action),
            EnemyAction::Remove(v) => match !new.is_empty() {
                true => {
                    new.swap_remove(v);
                }
                false => "At least one champion required".log(),
            },
        }
        Rc::new(new)
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
