use crate::{subject::Subject, subject_makepad_impl::GlobalNotify};
use makepad_widgets::*;

pub struct AppState {
    pub counter: Subject<i32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: Subject::new(0),
        }
    }
}

impl AppState {
    pub fn increment_counter(&mut self, cx: &mut Cx) {
        // Update through update method.
        self.counter.update(cx, |value| *value += 1);
    }

    pub fn decrement_counter(&mut self, cx: &mut Cx) {
        // Update using set method (would require clone if not copy).
        let value = *self.counter.get();
        self.counter.set(cx, value - 1);
    }

    pub fn increment_counter_async(&mut self, _cx: &mut Cx, delay_secs: f64) {
        let counter = self.counter.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs_f64(delay_secs));
            counter.update(&mut GlobalNotify::new(), |value| *value += 1);
        });
    }
}
