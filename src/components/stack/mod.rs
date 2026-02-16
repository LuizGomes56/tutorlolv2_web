mod selector;
mod table;

use std::rc::Rc;
use tutorlolv2_gen::ItemId;
use yew::Reducible;

pub use selector::{StackSelector, StackSelectorProps};
pub use table::{StackTable, StackTableProps};

#[derive(Clone, Default, PartialEq)]
pub struct Stack(Vec<StackValue>);

impl Stack {
    pub fn boxed(&self) -> Box<[StackValue]> {
        self.0.clone().into_boxed_slice()
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
    Ability(usize),
    Item(usize),
    Rune(usize),
    BasicAttack,
    CriticalStrike,
    OnhitMin,
    OnhitMax,
    Ignite(u8),
}
