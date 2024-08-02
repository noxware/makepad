use crate::subject::*;
use makepad_widgets::*;

impl Notify for Cx {
    fn notify(&mut self, id: Id) {
        self.action(id);
    }
}

impl Notified for Event {
    fn notified(&self, id: Id) -> bool {
        match self {
            Event::Actions(actions) => actions
                .iter()
                .find_map(|action| action.downcast_ref::<Id>())
                .map_or(false, |changed_id| *changed_id == id),
            // Workaround: If we receive a signal, we check the change event from the global set.
            Event::Signal => GLOBAL_NOTIFICATIONS.lock().unwrap().contains(&id),
            _ => false,
        }
    }
}

use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

/// Workaround: Implements `Notify` calling `global_notify`.
///
/// You can use this when you need to notify a change from a different thread.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalNotify;

impl GlobalNotify {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Notify for GlobalNotify {
    fn notify(&mut self, id: Id) {
        global_notify(id);
    }
}

/// Workaround: Holds change notifications that came from different threads through `global_notify`.
static GLOBAL_NOTIFICATIONS: LazyLock<Mutex<HashSet<Id>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Workaround: Buffers a change event globally, and sends a signal to Makepad.
pub fn global_notify(id: Id) {
    GLOBAL_NOTIFICATIONS.lock().unwrap().insert(id);
    SignalToUI::set_ui_signal();
}

/// Workaround: Clears all global notifications.
pub fn clear_global_notifications() {
    GLOBAL_NOTIFICATIONS.lock().unwrap().clear();
}
