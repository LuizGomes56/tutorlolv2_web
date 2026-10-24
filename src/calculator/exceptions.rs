use std::marker::PhantomData;
use tutorlolv2::{ValueId, bitset::BitSet, model::ValueException};

/// Packed exceptions in ascending ID order. T supplies the item/rune domain,
/// which ValueException itself does not encode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExceptionMap<T: ValueId> {
    keys: BitSet,
    values: Vec<ValueException>,
    marker: PhantomData<T>,
}

impl<T: ValueId> ExceptionMap<T> {
    pub fn values(&self) -> &[ValueException] {
        &self.values
    }

    pub fn get(&self, key: &T) -> Option<&ValueException> {
        let index = key.index() as u32;
        self.keys.contains_const(index).then(|| {
            let position = self.keys.count_lesser_elements_const(index) as usize;
            &self.values[position]
        })
    }

    pub fn insert(&mut self, key: T, stacks: u32) {
        let index = key.index() as u32;

        // The rank of the ID is also its offset into the packed values slice.
        let position = self.keys.count_lesser_elements_const(index) as usize;
        let value = key.pack_exc(stacks);

        if self.keys.contains_const(index) {
            self.values[position] = value;
        } else {
            self.values.insert(position, value);
            self.keys.insert_const(index);
        }
    }

    pub fn push_item(&mut self, value: T, ally: bool, mut insert: impl FnMut(T)) {
        if T::exceptions(ally).contains_const(value.index() as _) {
            self.insert(value, 0);
        }

        insert(value);
    }
}

#[cfg(test)]
mod tests {
    use crate::calculator::ExceptionMap;
    use tutorlolv2::{CastId, ItemId, RuneId, model::ValueException};

    #[test]
    fn replacing_stacks_keeps_a_single_exception_per_key() {
        let mut exceptions = ExceptionMap::<ItemId>::default();
        assert!(exceptions.values().is_empty());
        assert!(exceptions.get(&ItemId::DarkSeal).is_none());

        exceptions.insert(ItemId::DarkSeal, 4);
        exceptions.insert(ItemId::DarkSeal, 9);

        let values: &[ValueException] = exceptions.values();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].stacks(), 9);
        assert_eq!(values[0].get_item_id(), Some(ItemId::DarkSeal));
        assert_eq!(exceptions.get(&ItemId::DarkSeal).unwrap().stacks(), 9);
    }

    #[test]
    fn key_order_and_equality_are_independent_of_insertion_order() {
        let mut ascending = ExceptionMap::<ItemId>::default();
        let mut descending = ExceptionMap::<ItemId>::default();
        // Include zero and IDs on both sides of multiple bitset word boundaries.
        let indices = [0_u16, 1, 63, 64, 65, 127, 128];
        for index in indices {
            ascending.insert(ItemId::try_from_u16(index).unwrap(), u32::from(index) + 2);
        }
        for index in indices.into_iter().rev() {
            descending.insert(ItemId::try_from_u16(index).unwrap(), u32::from(index) + 2);
        }
        assert_eq!(ascending, descending);
        let values: &[ValueException] = ascending.values();
        assert_eq!(
            values
                .iter()
                .map(ValueException::stacks)
                .collect::<Vec<_>>(),
            [2, 3, 65, 66, 67, 129, 130]
        );
        for index in indices {
            let id = ItemId::try_from_u16(index).unwrap();
            assert_eq!(ascending.get(&id).unwrap().stacks(), u32::from(index) + 2);
        }
        assert!(ascending.get(&ItemId::try_from_u16(62).unwrap()).is_none());
        descending.insert(ItemId::try_from_u16(64).unwrap(), 99);
        assert_ne!(ascending, descending);
        assert_eq!(descending.values().len(), 7);
    }

    #[test]
    fn typed_maps_pack_overlapping_item_and_rune_ids_in_their_own_domain() {
        let item = ItemId::try_from_u16(1).unwrap();
        let rune = RuneId::try_from_u16(1).unwrap();
        assert_ne!(item.entity(), rune.entity());
        let mut items = ExceptionMap::<ItemId>::default();
        let mut runes = ExceptionMap::<RuneId>::default();
        items.insert(item, 5);
        runes.insert(rune, 8);
        assert_eq!(items.get(&item).unwrap().get_item_id(), Some(item));
        assert_eq!(runes.get(&rune).unwrap().get_rune_id(), Some(rune));
        assert_eq!(items.values()[0].stacks(), 5);
        assert_eq!(runes.values()[0].stacks(), 8);
    }

    #[test]
    fn packing_truncates_stacks_without_changing_the_key() {
        let mut exceptions = ExceptionMap::<RuneId>::default();
        let rune = RuneId::try_from_u16(0).unwrap();
        exceptions.insert(rune, u32::MAX);
        assert_eq!(
            exceptions.get(&rune).unwrap().stacks(),
            ValueException::VAL_MASK
        );
        assert_eq!(exceptions.values()[0].get_rune_id(), Some(rune));
        exceptions.insert(rune, 0);
        assert_eq!(exceptions.values().len(), 1);
        assert_eq!(exceptions.values()[0].stacks(), 0);
    }

    #[test]
    fn adding_an_exception_item_resets_stacks_and_calls_the_inserter() {
        let mut exceptions = ExceptionMap::<ItemId>::default();
        exceptions.insert(ItemId::DarkSeal, 8);
        let mut inserted = Vec::new();
        exceptions.push_item(ItemId::DarkSeal, true, |item| inserted.push(item));
        assert_eq!(inserted, [ItemId::DarkSeal]);
        assert_eq!(exceptions.values().len(), 1);
        assert_eq!(exceptions.get(&ItemId::DarkSeal).unwrap().stacks(), 0);
    }
}
