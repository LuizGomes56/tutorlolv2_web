mod selector;
mod table;

use crate::utils::traits::random_u64;
use std::{collections::HashSet, ops::Deref, rc::Rc};
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, RuneId, TypeMetadata};
use yew::Reducible;

pub use selector::StackSelector;
pub use table::StackTable;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StackEntry {
    pub id: u64,
    pub value: StackValue,
}

impl StackEntry {
    pub fn new(value: StackValue) -> Self {
        Self {
            id: random_u64(0..u64::MAX),
            value,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
#[repr(transparent)]
pub struct Stack(pub Vec<StackEntry>);

impl Deref for Stack {
    type Target = Vec<StackEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Stack {
    pub fn reconcile(
        &self,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        let items_allowed = items_meta.iter().map(|m| m.kind).collect::<HashSet<_>>();
        let runes_allowed = runes_meta.iter().map(|m| m.kind).collect::<HashSet<_>>();

        let mut out = Vec::with_capacity(self.len());

        for entry in self.iter() {
            if match entry.value {
                StackValue::Ability {
                    slot,
                    champion_id: cid,
                    ability_id,
                } => {
                    cid == champion_id
                        && slot < champion_id.number_of_abilities()
                        && champion_id
                            .abilities()
                            .get(slot)
                            .is_some_and(|m| m.kind == ability_id)
                }
                StackValue::Item(_, item_id) => items_allowed.contains(&item_id),
                StackValue::Rune(_, rune_id) => runes_allowed.contains(&rune_id),
                _ => true,
            } {
                out.push(*entry);
            }
        }

        Stack(out)
    }
}

pub enum StackAction {
    Insert(StackEntry),
    RemoveById(u64),
    Replace(Stack),
    Clear,
}

impl Reducible for Stack {
    type Action = StackAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        match action {
            StackAction::Insert(entry) => new.0.push(entry),

            StackAction::RemoveById(id) => {
                if let Some(pos) = new.0.iter().rposition(|e| e.id == id) {
                    new.0.swap_remove(pos);
                }
            }

            StackAction::Replace(stack) => new = stack,
            StackAction::Clear => new.0.clear(),
        }
        Rc::new(new)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackValue {
    Ability {
        slot: usize,
        champion_id: ChampionId,
        ability_id: AbilityId,
    },
    Item(usize, ItemId),
    Rune(usize, RuneId),
    BasicAttack,
    CritStrike,
    OnhitMin,
    OnhitMax,
    Ignite,
}
