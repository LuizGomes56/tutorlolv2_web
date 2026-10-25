//! Saved combo definitions. The browser record format is shared with other storage.
use super::StackValue;
use crate::utils::storage::{RecordStorage, encode_record, local_storage, read_record};
use std::collections::HashMap;
use tutorlolv2::ChampionId;

const COMBO_KIND: u8 = 1;
const COMBO_KEY: &str = "stack";

type StackStore = HashMap<ChampionId, Vec<StackValue>>;

fn update_combo(root: &mut StackStore, champion_id: ChampionId, entries: &[StackValue]) {
    root.insert(champion_id, entries.to_vec());
}

fn save_combo_record(
    storage: &mut impl RecordStorage,
    champion_id: ChampionId,
    entries: &[StackValue],
) -> bool {
    let mut root = read_record::<StackStore>(storage, COMBO_KEY, COMBO_KIND).unwrap_or_default();

    update_combo(&mut root, champion_id, entries);
    encode_record(COMBO_KIND, &root).is_some_and(|record| storage.put(COMBO_KEY, &record))
}

pub fn load_stack(champion_id: ChampionId) -> Option<Vec<StackValue>> {
    read_record::<StackStore>(&mut local_storage()?, COMBO_KEY, COMBO_KIND)?.remove(&champion_id)
}

pub fn save_stack(champion_id: ChampionId, entries: &[StackValue]) {
    if let Some(mut storage) = local_storage() {
        save_combo_record(&mut storage, champion_id, entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::storage::{MAX_BASE64_LENGTH, decode_record};
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use std::collections::HashMap;
    use strum::VariantArray;
    use tutorlolv2::{CastId, ItemId, RuneId};

    #[derive(Default)]
    struct MemoryStorage(HashMap<String, String>);

    impl RecordStorage for MemoryStorage {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }

        fn put(&mut self, key: &str, value: &str) -> bool {
            self.0.insert(key.to_owned(), value.to_owned());
            true
        }

        fn remove(&mut self, key: &str) {
            self.0.remove(key);
        }
    }

    #[test]
    fn combo_roundtrip_preserves_multiple_champions_and_every_action() {
        let champion = ChampionId::try_from_u16(0).unwrap();
        let other = ChampionId::try_from_u16(1).unwrap();
        let values = vec![
            StackValue::Ability {
                slot: 0,
                ability_id: champion.abilities()[0].kind,
            },
            StackValue::Item(1, ItemId::default()),
            StackValue::Rune(2, RuneId::default()),
            StackValue::BasicAttack,
            StackValue::CritStrike,
            StackValue::OnhitMin,
            StackValue::OnhitMax,
            StackValue::Ignite,
        ];
        let root = HashMap::from([(champion, values), (other, vec![StackValue::Ignite])]);
        let encoded = encode_record(COMBO_KIND, &root).unwrap();
        assert_eq!(
            decode_record::<StackStore>(COMBO_KIND, &encoded),
            Some(root)
        );
    }

    #[test]
    fn saving_one_champion_preserves_others_and_empty_combos() {
        let champion = ChampionId::try_from_u16(0).unwrap();
        let other = ChampionId::try_from_u16(1).unwrap();
        let mut storage = MemoryStorage::default();
        assert!(save_combo_record(
            &mut storage,
            other,
            &[StackValue::Ignite]
        ));
        assert!(save_combo_record(&mut storage, champion, &[]));
        let decoded = read_record::<StackStore>(&mut storage, COMBO_KEY, COMBO_KIND).unwrap();
        assert!(decoded[&champion].is_empty());
        assert_eq!(decoded[&other], [StackValue::Ignite]);
        assert!(save_combo_record(
            &mut storage,
            champion,
            &[StackValue::BasicAttack]
        ));
        let root = read_record::<StackStore>(&mut storage, COMBO_KEY, COMBO_KIND).unwrap();
        assert_eq!(root[&champion], [StackValue::BasicAttack]);
        assert_eq!(root.len(), 2);
    }

    #[test]
    fn invalid_old_and_trailing_records_are_discarded() {
        let valid = encode_record(COMBO_KIND, &StackStore::default()).unwrap();
        let mut trailing = BASE64.decode(&valid).unwrap();
        trailing.push(0);
        let mut future = BASE64.decode(&valid).unwrap();
        future[4] = future[4].wrapping_add(1);
        let mut truncated = BASE64.decode(&valid).unwrap();
        truncated.pop();
        let mut wrong_magic = BASE64.decode(&valid).unwrap();
        wrong_magic[0] ^= 1;
        for raw in [
            "{\"Aatrox\":[\"BasicAttack\"]}".to_owned(),
            "not base64!".to_owned(),
            String::new(),
            BASE64.encode(trailing),
            BASE64.encode(future),
            BASE64.encode(truncated),
            BASE64.encode(wrong_magic),
            encode_record(COMBO_KIND + 1, &StackStore::default()).unwrap(),
        ] {
            let mut storage = MemoryStorage(HashMap::from([(COMBO_KEY.to_owned(), raw)]));
            assert!(read_record::<StackStore>(&mut storage, COMBO_KEY, COMBO_KIND).is_none());
            assert!(!storage.0.contains_key(COMBO_KEY));
        }
    }

    #[test]
    fn rejected_data_can_be_replaced_with_a_new_valid_combo() {
        let mut storage = MemoryStorage(HashMap::from([(COMBO_KEY.to_owned(), "{}".to_owned())]));
        let champion = ChampionId::default();
        assert!(save_combo_record(
            &mut storage,
            champion,
            &[StackValue::CritStrike]
        ));
        let root = read_record::<StackStore>(&mut storage, COMBO_KEY, COMBO_KIND).unwrap();
        assert_eq!(root[&champion], [StackValue::CritStrike]);
        assert_eq!(root.len(), 1);
        assert!(storage.0.contains_key(COMBO_KEY));
    }

    #[test]
    fn invalid_stack_variant_and_unknown_champion_are_rejected() {
        let champion = ChampionId::default();
        // A raw Bincode record with the same envelope and map layout, but a
        // variant value that cannot decode as StackValue.
        let invalid_variant = HashMap::from([(champion, vec![99_u32])]);
        assert!(
            decode_record::<StackStore>(
                COMBO_KIND,
                &encode_record(COMBO_KIND, &invalid_variant).unwrap()
            )
            .is_none()
        );
        let invalid_champion = HashMap::from([(u16::MAX, Vec::<StackValue>::new())]);
        assert!(
            decode_record::<StackStore>(
                COMBO_KIND,
                &encode_record(COMBO_KIND, &invalid_champion).unwrap()
            )
            .is_none()
        );
    }

    #[test]
    fn oversized_and_forged_collection_lengths_are_rejected() {
        assert!(
            decode_record::<StackStore>(COMBO_KIND, &"A".repeat(MAX_BASE64_LENGTH + 4)).is_none()
        );
        // An encoded u64 occupies the collection-length position after the
        // envelope; decoding must respect the allocation limit.
        let forged = encode_record(COMBO_KIND, &u64::MAX).unwrap();
        assert!(decode_record::<StackStore>(COMBO_KIND, &forged).is_none());
    }

    #[test]
    fn stale_slots_are_removed_before_the_damage_table_indexes_them() {
        use crate::components::stack::{Stack, Tray, TrayEntry};
        let champion = ChampionId::default();
        let other = ChampionId::try_from_u16(1).unwrap();
        let ability_id = champion.abilities()[0].kind;
        let item = ItemId::NashorsTooth;
        let rune = RuneId::default();
        let items = [item.metadata()];
        let runes = [rune.metadata()];
        let values = [
            StackValue::Ability {
                slot: 0,
                ability_id,
            },
            StackValue::Ability {
                slot: usize::MAX,
                ability_id,
            },
            StackValue::Item(0, item),
            StackValue::Item(usize::MAX, item),
            StackValue::Rune(0, rune),
            StackValue::Rune(usize::MAX, rune),
            StackValue::BasicAttack,
        ];
        let stack = Stack {
            champion_id: champion,
            values: Tray::new(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(id, value)| TrayEntry { id: id as _, value }),
            ),
        };
        let safe = stack.reconcile(champion, &items, &runes);
        assert_eq!(safe.values(), [values[0], values[2], values[4], values[6]]);
        assert_eq!(safe.iter().map(|e| e.id).collect::<Vec<_>>(), [0, 2, 4, 6]);
        let changed_champion = stack.reconcile(other, &items, &runes);
        assert_eq!(changed_champion.values(), [values[2], values[4], values[6]]);
    }

    #[test]
    fn item_max_damage_slots_remain_valid_with_expanded_damage_columns() {
        use crate::components::stack::{Stack, Tray, TrayEntry};
        let first = ItemId::VARIANTS
            .iter()
            .copied()
            .find(|item| item.deals_max_damage())
            .unwrap();
        let second = ItemId::BladeOfTheRuinedKing;
        assert!(first.deals_max_damage());
        let metadata = [first.metadata(), second.metadata()];
        let values = [
            StackValue::Item(0, first),
            StackValue::Item(1, first),
            StackValue::Item(2, second),
        ];
        let stack = Stack {
            champion_id: ChampionId::default(),
            values: Tray::new(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(id, value)| TrayEntry { id: id as _, value }),
            ),
        };
        assert_eq!(
            stack
                .reconcile(ChampionId::default(), &metadata, &[])
                .values(),
            values
        );
    }

    #[test]
    fn default_item_entries_use_expanded_damage_slots_without_adding_actions() {
        use crate::components::stack::Stack;
        let first = ItemId::NashorsTooth;
        let second = ItemId::VARIANTS
            .iter()
            .copied()
            .find(|item| item.deals_max_damage())
            .unwrap();
        let third = ItemId::BladeOfTheRuinedKing;
        assert!(!first.deals_max_damage());
        assert!(!third.deals_max_damage());
        let metadata = [first.metadata(), second.metadata(), third.metadata()];
        let values =
            Stack::default_metadata_values(&metadata, StackValue::Item).collect::<Vec<_>>();
        assert_eq!(
            values,
            [
                StackValue::Item(0, first),
                StackValue::Item(2, second),
                StackValue::Item(4, third)
            ]
        );
        let stack = Stack {
            champion_id: ChampionId::default(),
            values: super::super::Tray::new(
                values
                    .iter()
                    .enumerate()
                    .map(|(id, &value)| super::super::TrayEntry { id: id as _, value }),
            ),
        };
        assert_eq!(
            stack
                .reconcile(ChampionId::default(), &metadata, &[])
                .values(),
            values
        );
    }

    #[test]
    fn default_rune_entries_use_fixed_damage_pairs() {
        use crate::components::stack::Stack;
        let runes = [
            RuneId::VARIANTS[0],
            RuneId::VARIANTS[1],
            RuneId::VARIANTS[2],
        ];
        let metadata = runes.map(|rune| rune.metadata());
        assert_eq!(
            Stack::default_metadata_values(&metadata, StackValue::Rune).collect::<Vec<_>>(),
            [
                StackValue::Rune(0, runes[0]),
                StackValue::Rune(2, runes[1]),
                StackValue::Rune(4, runes[2])
            ]
        );
    }
}
