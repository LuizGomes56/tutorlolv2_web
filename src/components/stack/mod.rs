mod selector;
mod table;

use std::{ops::Deref, rc::Rc};
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, RuneId};
use yew::Reducible;

pub use selector::StackSelector;
pub use table::StackTable;

#[derive(Clone, Default, PartialEq)]
pub struct Stack(Vec<StackValue>);

impl Deref for Stack {
    type Target = Vec<StackValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum StackAction {
    Insert(StackValue),
    Remove(usize),
    Clear,
}

impl Reducible for Stack {
    type Action = StackAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            StackAction::Insert(value) => new.0.push(value),
            StackAction::Remove(index) => {
                new.0.swap_remove(index);
            }
            StackAction::Clear => new.0.clear(),
        }
        Rc::new(new)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackValue {
    Ability(usize, ChampionId, AbilityId),
    Item(usize, ItemId),
    Rune(usize, RuneId),
    BasicAttack,
    CriticalStrike,
    OnhitMin,
    OnhitMax,
    Ignite(u8),
}
