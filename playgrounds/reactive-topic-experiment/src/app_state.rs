use crate::subject::Subject;
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
        let value = self.counter.take();
        self.counter.set(cx, value + 1);
    }

    pub fn decrement_counter(&mut self, cx: &mut Cx) {
        let value = self.counter.take();
        self.counter.set(cx, value - 1);
    }
}
