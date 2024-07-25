use makepad_widgets::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub trait Notify {
    /// Notify `self` that the subject with the given id has been updated.
    fn notify(&mut self, id: usize);
}

pub trait Notified {
    /// Check if the subject with the given id has been updated.
    fn notified(&self, id: usize) -> bool;
}

impl Notify for Cx {
    fn notify(&mut self, id: usize) {
        self.action(SubjectChanged { id });
    }
}

impl Notified for Event {
    fn notified(&self, id: usize) -> bool {
        match self {
            Event::Actions(actions) => actions
                .iter()
                .find_map(|action| action.downcast_ref::<SubjectChanged>())
                .map_or(false, |subject_changed| subject_changed.id == id),
            _ => false,
        }
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

/// Action dispatched when a subject is set.
#[derive(Debug)]
pub struct SubjectChanged {
    id: usize,
}

/// A minimalistic value container that notifies makepad when its value is set.
///
/// Provides reactive workflows useful for handling app-level state.
/// This is a bit inspired on Flutter's `ValueNotifier`.
pub struct Subject<T> {
    id: usize,
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
        Self { id, value }
    }

    /// Getter for the internal id of this subject.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Gets a immutable reference to the current value of this subject.
    ///
    /// Panics if the value has been taken out without replacing it before calling this.
    pub fn get(&self) -> ReadGuard<T> {
        self.value.read().unwrap().into()
    }

    /// Sets the value of this subject and notifies makepad about this subject update.
    pub fn set<N: Notify>(&self, notifiable: &mut N, value: T) {
        *self.value.write().unwrap() = value;
        notifiable.notify(self.id);
    }

    pub fn update<N: Notify>(&self, notifiable: &mut N, f: impl FnOnce(&mut T)) {
        f(&mut *self.value.write().unwrap());
        notifiable.notify(self.id);
    }

    /// Check if this subject has been changed.
    pub fn changed<N: Notified>(&mut self, notified: &N) -> bool {
        notified.notified(self.id)
    }
}
