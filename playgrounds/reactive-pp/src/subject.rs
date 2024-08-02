use std::sync::{Arc, RwLock, RwLockReadGuard};

/// Let a type be notified of changes in a subject.
pub trait Notify {
    /// Notify `self` that the subject with the given id has been updated.
    fn notify(&mut self, id: Id);
}

/// Let a type check if a subject has been updated.
pub trait Notified {
    /// Check if the subject with the given id has been updated.
    fn notified(&self, id: Id) -> bool;
}

/// Unique identifier for a subject.
///
/// - Abstracts away the internal id type of a subject.
/// - In Makepad, it can be dispatched directly as an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

/// Read-only guard returned by get.
// This is just to avoid exposing the RwLockReadGuard directly.
pub struct ReadGuard<'a, T: ?Sized> {
    guard: RwLockReadGuard<'a, T>,
}

impl<'a, T: ?Sized> std::ops::Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<'a, T: ?Sized> From<RwLockReadGuard<'a, T>> for ReadGuard<'a, T> {
    fn from(guard: RwLockReadGuard<'a, T>) -> Self {
        Self { guard }
    }
}

/// A minimalistic value container that notifies a `Notify` when its value changes.
///
/// Provides reactive workflows useful for handling app-level state.
/// This is a bit inspired on Flutter's `ValueNotifier`.
pub struct Subject<T> {
    id: Id,
    value: Arc<RwLock<T>>,
}

impl<T> Clone for Subject<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Arc::clone(&self.value),
        }
    }
}

impl<T> Subject<T> {
    pub fn new(initial_value: T) -> Self {
        let value = Arc::new(RwLock::new(initial_value));
        let id = Arc::as_ptr(&value) as usize;
        let id = id.into();
        Self { id, value }
    }

    /// Getter for the internal id of this subject.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Gets a immutable reference to the current value of this subject.
    pub fn get(&self) -> ReadGuard<T> {
        self.value.read().unwrap().into()
    }

    /// Sets the value of this subject and notifies a `Notify` about this subject update.
    pub fn set<N: Notify>(&self, notify: &mut N, value: T) {
        *self.value.write().unwrap() = value;
        notify.notify(self.id);
    }

    /// Updates the value hold by this subject and notifies a `Notify` about this subject update.
    pub fn update<N: Notify>(&self, notify: &mut N, f: impl FnOnce(&mut T)) {
        f(&mut *self.value.write().unwrap());
        notify.notify(self.id);
    }

    /// Check if this subject has been changed.
    pub fn changed<N: Notified>(&mut self, notified: &N) -> bool {
        notified.notified(self.id)
    }
}
