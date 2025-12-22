use crate::{
    calculator::{AbilityLevels, Player, PlayerData},
    model::{Dragons, SimpleStats, Stats, ValueException},
};
use std::rc::Rc;
use tutorlolv2_gen::{ChampionId, ItemId, RuneId};
use yew::Reducible;

pub type EnemyDataAction = DataAction<SimpleStats>;
pub type PlayerDataAction = DataAction<Stats>;

pub enum PlayerAction {
    InsertRune(RuneId),
    RemoveRune(usize),
    InsertRuneExc(RuneId, u32),
    RemoveRuneExc(usize),
    Data(PlayerDataAction),
    AbilityLevel(AbilityLevels),
}

pub enum DataAction<T> {
    Stats(*const T),
    Stacks(u32),
    InferStats(bool),
    IsMegaGnar(bool),
    InsertItem(ItemId),
    RemoveItem(usize),
    ChampionId(ChampionId),
    InsertItemExc(ItemId, u32),
    RemoveItemExc(usize),
}

pub enum EnemyAction {
    Push,
    Remove(usize),
    Edit(usize, EnemyDataAction),
}

#[derive(Clone, Default)]
#[repr(transparent)]
pub struct Enemies(Vec<Rc<PlayerData<SimpleStats>>>);

impl core::ops::Deref for Enemies {
    type Target = Vec<Rc<PlayerData<SimpleStats>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Enemies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Copy> PlayerData<T> {
    pub fn reduce_mut(&mut self, action: DataAction<T>) {
        match action {
            DataAction::Stats(v) => self.stats = unsafe { *v },
            DataAction::Stacks(v) => self.stacks = v,
            DataAction::InferStats(v) => self.infer_stats = v,
            DataAction::IsMegaGnar(v) => self.is_mega_gnar = v,
            DataAction::InsertItem(v) => self.items.push(v),
            DataAction::ChampionId(v) => self.champion_id = v,
            DataAction::RemoveItem(v) => {
                self.items.swap_remove(v);
            }
            DataAction::InsertItemExc(item_id, stacks) => {
                let value = ValueException::pack_item_id(item_id, stacks);
                self.item_exceptions.push(value)
            }
            DataAction::RemoveItemExc(v) => {
                self.item_exceptions.swap_remove(v);
            }
        }
    }
}

impl Reducible for Player {
    type Action = PlayerAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            Self::Action::InsertRune(v) => new.runes.push(v),
            Self::Action::AbilityLevel(v) => new.abilities = v,
            Self::Action::RemoveRune(v) => {
                new.runes.swap_remove(v);
            }
            Self::Action::InsertRuneExc(rune_id, stacks) => {
                let value = ValueException::pack_rune_id(rune_id, stacks);
                new.rune_exceptions.push(value);
            }
            Self::Action::RemoveRuneExc(v) => {
                new.rune_exceptions.swap_remove(v);
            }
            Self::Action::Data(v) => {
                let mut data = new.data;
                data.reduce_mut(v);
                new.data = data;
            }
        }
        Rc::new(new)
    }
}

impl<T: Copy> Reducible for PlayerData<T> {
    type Action = DataAction<T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        new.reduce_mut(action);
        Rc::new(new)
    }
}

impl Reducible for Enemies {
    type Action = EnemyAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            EnemyAction::Push => new.push(Default::default()),
            EnemyAction::Edit(v, action) => new[v] = new[v].clone().reduce(action),
            EnemyAction::Remove(v) => {
                new.swap_remove(v);
            }
        }
        Rc::new(new)
    }
}

pub enum DragonAction {
    AllyFire(u16),
    AllyEarth(u16),
    AllyChemtech(u16),
    EnemyEarth(u16),
}

impl Reducible for Dragons {
    type Action = DragonAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            DragonAction::AllyFire(v) => new.ally_fire_dragons = v,
            DragonAction::AllyEarth(v) => new.ally_earth_dragons = v,
            DragonAction::AllyChemtech(v) => new.ally_chemtech_dragons = v,
            DragonAction::EnemyEarth(v) => new.enemy_earth_dragons = v,
        }
        Rc::new(new)
    }
}
