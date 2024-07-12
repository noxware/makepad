use makepad_widgets::*;
use std::sync::atomic::AtomicUsize;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
    value: Option<T>,
}

impl<T> Subject<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            value: Some(initial_value),
        }
    }

    /// Gets a immutable reference to the current value of this subject.
    ///
    /// Panics if the value has been taken out without replacing it before calling this.
    pub fn get(&self) -> &T {
        self.value
            .as_ref()
            .expect("Subject is empty. Did you took the value out without replacing it?")
    }

    /// Sets the value of this subject and notifies makepad about this subject update.
    pub fn set(&mut self, cx: &mut Cx, value: T) {
        self.value = Some(value);
        cx.action(SubjectChanged { id: self.id })
    }

    /// Takes the value out of this subject giving you the chance to modify before calling `set` again.
    ///
    /// Once you take the value out, you must call `set` before calling `get` again, or it will panic.
    pub fn take(&mut self) -> T {
        self.value
            .take()
            .expect("There is nothing to take from the Subject. Seems like it has been already taken and was never replaced.")
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
