mod insert;
mod remover;
pub mod storage;
mod table;

use crate::utils::tray::{Tray, TrayAction, TrayEntry};
use bincode::{Decode, Encode};
use derive_more::{Deref, DerefMut};
use std::rc::Rc;
use tutorlolv2::{AbilityId, ChampionId, ComboElement, ItemId, RuneId, TypeMetadata};
use yew::{Callback, Reducible, UseReducerHandle, UseStateHandle, hook, use_callback};

pub use insert::StackInsert;
pub use remover::StackRemover;
pub use table::StackTable;

#[derive(Clone, Default, Deref, DerefMut, PartialEq)]
pub struct Stack {
    champion_id: ChampionId,

    #[deref]
    #[deref_mut]
    values: Tray<StackValue>,
}

impl Stack {
    #[hook]
    pub fn use_push(stack: &UseReducerHandle<Stack>) -> Callback<StackValue> {
        let stack = stack.clone();

        use_callback((), move |value, _| {
            stack.dispatch(TrayAction::Insert(value))
        })
    }

    fn default_metadata_values<T: Copy>(
        metadata: &[TypeMetadata<T>],
        value: fn(usize, T) -> StackValue,
    ) -> impl Iterator<Item = StackValue> + '_ {
        // Item and rune damage arrays reserve a min/max pair per metadata
        // entry, including entries whose maximum column stays zero.
        metadata
            .iter()
            .enumerate()
            .map(move |(i, m)| value(i * 2, m.kind))
    }

    pub fn new(
        combo_index: &UseStateHandle<usize>,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        let combos = champion_id.combos();
        let i = **combo_index % combos.len().max(1);

        combo_index.set(i);

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
                    .chain(
                        Self::default_metadata_values(items_meta, StackValue::Item)
                            .map(TrayEntry::new),
                    )
                    .chain(
                        Self::default_metadata_values(runes_meta, StackValue::Rune)
                            .map(TrayEntry::new),
                    ),
            ),
        }
    }

    pub fn reconcile(
        &self,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Self {
        let valid = |entry: &TrayEntry<StackValue>| match entry.value {
            StackValue::Item(slot, item_id) => {
                items_meta.get(slot / 2).is_some_and(|m| m.kind == item_id)
                    && (slot % 2 == 0 || item_id.deals_max_damage())
            }
            StackValue::Rune(slot, rune_id) => {
                runes_meta.get(slot / 2).is_some_and(|m| m.kind == rune_id)
                    && (slot % 2 == 0 || rune_id.deals_max_damage())
            }
            StackValue::Ability { slot, ability_id } => {
                champion_id == self.champion_id
                    && champion_id.index_of_ability(ability_id) == Some(slot)
            }
            _ => true,
        };

        if champion_id == self.champion_id && self.iter().all(|entry| valid(&entry)) {
            return self.clone();
        }

        let values = self.iter().filter(valid).collect();

        Self {
            champion_id,
            values,
        }
    }
}

impl Reducible for Stack {
    type Action = TrayAction<Self, StackValue>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        action.apply(new_mut);

        new
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_reconciliation_shares_values_and_changed_metadata_removes_stale_entries() {
        let champion_id = ChampionId::default();
        let stack = Stack {
            champion_id,
            values: Tray::new([
                TrayEntry {
                    id: 10,
                    value: StackValue::BasicAttack,
                },
                TrayEntry {
                    id: 20,
                    value: StackValue::Item(0, ItemId::NashorsTooth),
                },
            ]),
        };
        let metadata = [ItemId::NashorsTooth.data().metadata];
        let same = stack.reconcile(champion_id, &metadata, &[]);
        assert_eq!(same.values().as_ptr(), stack.values().as_ptr());
        let changed = same.reconcile(champion_id, &[], &[]);
        assert_eq!(changed.values(), &[StackValue::BasicAttack]);
        assert_eq!(changed.iter().next().unwrap().id, 10);
        assert_eq!(stack.len(), 2);
    }
}
