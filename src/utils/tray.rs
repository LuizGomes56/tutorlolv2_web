use {
    crate::utils::random_u64,
    std::{
        ops::{Deref, DerefMut},
        rc::Rc,
    },
    yew::Reducible,
};

/// Stable entry identities alongside contiguous values. All mutations keep the
/// two vectors aligned so calculator inputs can borrow the values directly.
#[derive(Clone, Debug, PartialEq)]
pub struct Tray<T: Copy> {
    ids: Vec<u64>,
    values: Vec<T>,
}

impl<T: Copy> Default for Tray<T> {
    fn default() -> Self {
        Self {
            ids: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<T: Copy> Tray<T> {
    pub fn new(entries: impl IntoIterator<Item = TrayEntry<T>>) -> Self {
        entries.into_iter().collect()
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = TrayEntry<T>> + ExactSizeIterator + '_ {
        self.ids
            .iter()
            .copied()
            .zip(self.values.iter().copied())
            .map(|(id, value)| TrayEntry { id, value })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn push(&mut self, entry: TrayEntry<T>) {
        self.ids.push(entry.id);
        self.values.push(entry.value);
    }

    pub fn remove_by_id(&mut self, id: u64) {
        if let Some(index) = self.ids.iter().position(|&entry_id| entry_id == id) {
            self.ids.remove(index);
            self.values.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.values.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrayEntry<T: Copy> {
    pub id: u64,
    pub value: T,
}

impl<T: Copy> TrayEntry<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: random_u64(0..u64::MAX),
            value,
        }
    }
}

pub enum TrayAction<T, U: Copy> {
    Insert(U),
    RemoveById(u64),
    Replace(T),
    Clear,
}

impl<U: Copy> FromIterator<U> for Tray<U> {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        iter.into_iter().map(TrayEntry::new).collect()
    }
}

impl<'a, U: Copy> FromIterator<&'a U> for Tray<U> {
    fn from_iter<T: IntoIterator<Item = &'a U>>(iter: T) -> Self {
        iter.into_iter().copied().collect()
    }
}

impl<U: Copy> FromIterator<TrayEntry<U>> for Tray<U> {
    fn from_iter<T: IntoIterator<Item = TrayEntry<U>>>(iter: T) -> Self {
        let (ids, values) = iter
            .into_iter()
            .map(|entry| (entry.id, entry.value))
            .unzip();

        Self { ids, values }
    }
}

impl<T, U: Copy> TrayAction<T, U>
where
    T: Deref<Target = Tray<U>> + DerefMut,
{
    pub fn apply(self, container: &mut T) {
        match self {
            TrayAction::Insert(entry) => container.push(TrayEntry::new(entry)),
            TrayAction::RemoveById(id) => container.remove_by_id(id),
            TrayAction::Replace(stack) => *container = stack,
            TrayAction::Clear => container.clear(),
        }
    }
}

impl<T: Copy> TrayAction<Tray<T>, T> {
    pub fn apply(self, container: &mut Tray<T>) {
        self.custom_apply(container, |c, v| c.push(TrayEntry::new(v)));
    }

    pub fn custom_apply(self, container: &mut Tray<T>, mut f: impl FnMut(&mut Tray<T>, T)) {
        match self {
            TrayAction::Insert(v) => f(container, v),
            TrayAction::RemoveById(id) => container.remove_by_id(id),
            TrayAction::Replace(stack) => *container = stack,
            TrayAction::Clear => container.clear(),
        }
    }
}

impl<T: Copy> Reducible for Tray<T> {
    type Action = TrayAction<Self, T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        action.apply(new_mut);

        new
    }
}

#[cfg(test)]
mod tests {
    use super::{Tray, TrayAction, TrayEntry};

    #[test]
    fn removing_one_duplicate_preserves_other_identities_and_value_order() {
        let mut tray = Tray::new(vec![
            TrayEntry { id: 10, value: 7 },
            TrayEntry { id: 20, value: 7 },
            TrayEntry { id: 30, value: 9 },
        ]);

        TrayAction::RemoveById(10).custom_apply(&mut tray, |_, _| unreachable!());

        let values: &[i32] = tray.values();
        assert_eq!(values, &[7, 9]);
        assert_eq!(
            tray.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            [20, 30]
        );

        tray.push(TrayEntry { id: 40, value: 11 });
        TrayAction::RemoveById(999).custom_apply(&mut tray, |_, _| unreachable!());
        assert_eq!(tray.values(), &[7, 9, 11]);
        assert_eq!(
            tray.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            [20, 30, 40]
        );
    }

    #[test]
    fn replacement_clear_and_insertion_keep_ids_paired_with_values() {
        let mut tray = Tray::new(vec![TrayEntry { id: 1, value: 5 }]);
        let replacement = Tray::new(vec![TrayEntry { id: 2, value: 8 }]);
        TrayAction::Replace(replacement).custom_apply(&mut tray, |_, _| unreachable!());
        assert_eq!(tray.values(), &[8]);
        assert_eq!(tray.iter().next().unwrap().id, 2);

        TrayAction::Clear.custom_apply(&mut tray, |_, _| unreachable!());
        assert!(tray.values().is_empty());
        assert_eq!(tray.iter().count(), 0);

        TrayAction::Insert(13).custom_apply(&mut tray, |tray, value| {
            tray.push(TrayEntry { id: 3, value });
        });
        assert_eq!(tray.values(), &[13]);
        assert_eq!(tray.iter().next().unwrap().id, 3);
    }
}
