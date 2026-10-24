use {
    crate::utils::random_u64,
    std::{
        ops::{Deref, DerefMut},
        rc::Rc,
    },
    yew::Reducible,
};

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct Tray<T: Copy> {
    pub inner: Vec<TrayEntry<T>>,
}

impl<T: Copy> Default for Tray<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T: Copy> Tray<T> {
    pub fn new(inner: Vec<TrayEntry<T>>) -> Self {
        Self { inner }
    }

    pub fn values<U: FromIterator<T>>(&self) -> U {
        self.inner.iter().map(|e| e.value).collect()
    }
}

impl<T: Copy> Deref for Tray<T> {
    type Target = Vec<TrayEntry<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Copy> DerefMut for Tray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
        Tray::new(iter.into_iter().map(TrayEntry::new).collect())
    }
}

impl<'a, U: Copy> FromIterator<&'a U> for Tray<U> {
    fn from_iter<T: IntoIterator<Item = &'a U>>(iter: T) -> Self {
        Tray::new(iter.into_iter().copied().map(TrayEntry::new).collect())
    }
}

impl<U: Copy> FromIterator<TrayEntry<U>> for Tray<U> {
    fn from_iter<T: IntoIterator<Item = TrayEntry<U>>>(iter: T) -> Self {
        Tray::new(iter.into_iter().collect())
    }
}

impl<T, U: Copy> TrayAction<T, U>
where
    T: Deref<Target = Tray<U>> + DerefMut,
{
    pub fn apply(self, container: &mut T) {
        match self {
            TrayAction::Insert(entry) => container.push(TrayEntry::new(entry)),
            TrayAction::RemoveById(id) => container.retain(|e| e.id != id),
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
            TrayAction::RemoveById(id) => container.retain(|e| e.id != id),
            TrayAction::Replace(stack) => *container = stack,
            TrayAction::Clear => container.clear(),
        }
    }
}

impl<T: Copy> Reducible for Tray<T> {
    type Action = TrayAction<Self, T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = (*self).clone();
        action.apply(&mut new);
        Rc::new(new)
    }
}
