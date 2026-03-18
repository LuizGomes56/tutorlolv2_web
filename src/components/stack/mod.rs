mod insert;
mod remover;
mod table;

use crate::{
    components::tray::{Tray, TrayAction, TrayEntry},
    utils::random_u64,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use tutorlolv2_gen::{AbilityId, ChampionId, ComboElement, ItemId, RuneId, TypeMetadata};
use yew::{Callback, Reducible};

pub use insert::StackInsert;
pub use remover::StackRemover;
pub use table::StackTable;

impl Deref for Stack {
    type Target = Tray<StackValue>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Stack {
    champion_id: ChampionId,
    values: Tray<StackValue>,
}

impl Stack {
    #[yew::hook]
    pub fn use_push(stack: &yew::UseReducerHandle<Stack>) -> Callback<StackValue> {
        let stack = stack.clone();
        yew::use_callback((), move |value, _| {
            stack.dispatch(TrayAction::Insert(value))
        })
    }

    pub fn new(
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        fn chain_meta<T: Copy>(
            meta: &[TypeMetadata<T>],
            f: fn(usize, T) -> StackValue,
        ) -> impl IntoIterator<Item = TrayEntry<StackValue>> {
            meta.iter()
                .enumerate()
                .map(move |(i, v)| TrayEntry::new(f(i, v.kind)))
        }

        let combos = champion_id.combos();
        let i = random_u64(0..combos.len() as _) as usize;

        Stack {
            champion_id,
            values: Tray::new(
                combos
                    .get(i)
                    .into_iter()
                    .flat_map(|list| list.iter())
                    .filter_map(|&element| match element {
                        ComboElement::Ability(ability_id) => champion_id
                            .index_of_ability(ability_id)
                            .map(|slot| TrayEntry::new(StackValue::Ability { slot, ability_id })),
                        ComboElement::Attack => Some(TrayEntry::new(StackValue::BasicAttack)),
                    })
                    .chain(chain_meta(items_meta, StackValue::Item))
                    .chain(chain_meta(runes_meta, StackValue::Rune))
                    .collect(),
            ),
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

        let mut values = Tray::new(Vec::with_capacity(self.len()));

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

impl Reducible for Stack {
    type Action = TrayAction<Self, StackValue>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        action.apply(&mut new);
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
