mod selector;
mod table;

use crate::utils::random_u64;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use tutorlolv2_gen::{AbilityId, ChampionId, ComboElement, ItemId, RuneId, TypeMetadata};
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
pub struct Stack {
    champion_id: ChampionId,
    values: Vec<StackEntry>,
}

impl Deref for Stack {
    type Target = Vec<StackEntry>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl Stack {
    pub fn new(
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        fn chain_meta<T: Copy>(
            meta: &[TypeMetadata<T>],
            f: fn(usize, T) -> StackValue,
        ) -> impl IntoIterator<Item = StackEntry> {
            meta.iter()
                .enumerate()
                .map(move |(i, v)| StackEntry::new(f(i, v.kind)))
        }

        let combos = champion_id.combos();
        let i = random_u64(0..combos.len() as _) as usize;

        Stack {
            champion_id,
            values: combos
                .get(i)
                .into_iter()
                .flat_map(|list| list.iter())
                .filter_map(|&element| match element {
                    ComboElement::Ability(ability_id) => champion_id
                        .index_of_ability(ability_id)
                        .map(|slot| StackEntry::new(StackValue::Ability { slot, ability_id })),
                    ComboElement::Attack => Some(StackEntry::new(StackValue::BasicAttack)),
                })
                .chain(chain_meta(items_meta, StackValue::Item))
                .chain(chain_meta(runes_meta, StackValue::Rune))
                .collect(),
        }
    }

    pub fn reconcile(
        &self,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        let items_allowed = items_meta.iter().map(|m| m.kind).collect::<HashSet<_>>();
        let runes_allowed = runes_meta.iter().map(|m| m.kind).collect::<HashSet<_>>();

        let mut values = Vec::with_capacity(self.len());

        for entry in self.iter() {
            if match entry.value {
                StackValue::Item(_, item_id) => items_allowed.contains(&item_id),
                StackValue::Rune(_, rune_id) => runes_allowed.contains(&rune_id),
                StackValue::Ability { .. } => champion_id == self.champion_id,
                _ => true,
            } {
                values.push(*entry);
            }
        }

        Self {
            champion_id,
            values,
        }
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
            StackAction::Insert(entry) => new.push(entry),
            StackAction::RemoveById(id) => new.retain(|e| e.id != id),
            StackAction::Replace(stack) => new = stack,
            StackAction::Clear => new.clear(),
        }
        Rc::new(new)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum StackValue {
    Ability { slot: usize, ability_id: AbilityId },
    Item(usize, ItemId),
    Rune(usize, RuneId),
    BasicAttack,
    CritStrike,
    OnhitMin,
    OnhitMax,
    Ignite,
}
