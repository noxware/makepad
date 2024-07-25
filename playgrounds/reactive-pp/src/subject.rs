use makepad_widgets::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

type ReadGuard<'a, T> = RwLockReadGuard<'a, T>;

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

    /// Gets a immutable reference to the current value of this subject.
    ///
    /// Panics if the value has been taken out without replacing it before calling this.
    pub fn get(&self) -> ReadGuard<T> {
        self.value.read().unwrap()
    }

    /// Sets the value of this subject and notifies makepad about this subject update.
    pub fn set(&self, cx: &mut Cx, value: T) {
        *self.value.write().unwrap() = value;
        cx.action(SubjectChanged { id: self.id })
    }

    pub fn update(&self, cx: &mut Cx, f: impl FnOnce(&mut T)) {
        f(&mut *self.value.write().unwrap());
        cx.action(SubjectChanged { id: self.id })
    }

    /// Check if this subject has been changed.
    pub fn changed(&mut self, event: &Event) -> bool {
        match event {
            Event::Actions(actions) => actions
                .iter()
                .find_map(|action| action.downcast_ref::<SubjectChanged>())
                .map_or(false, |subject_changed| subject_changed.id == self.id),
            _ => false,
        }
    }
}
