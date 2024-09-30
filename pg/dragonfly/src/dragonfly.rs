use std::sync::{Arc, Mutex};

use makepad_widgets::*;

struct MutatorAction<T> {
    f: Arc<Mutex<Box<dyn FnMut(&mut T, &mut Cx) + Send + 'static>>>,
    id: usize,
}

impl<T> std::fmt::Debug for MutatorAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutator")
            .field("id", &self.id)
            .field("f", &"{irrelevant}")
            .finish()
    }
}

pub struct Mutator {
    id: usize,
}

impl Mutator {
    fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn mutate<T: 'static>(&self, f: impl FnMut(&mut T, &mut Cx) + Send + 'static) {
        let action = MutatorAction {
            f: Arc::new(Mutex::new(Box::new(f))),
            id: self.id,
        };

        Cx::post_action(action);
    }
}

#[derive(Debug, Clone)]
pub struct Dragonfly {
    id: usize,
}

impl Dragonfly {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { id }
    }

    pub fn handle<T: 'static>(&self, target: &mut T, cx: &mut Cx, event: &Event) {
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(action) = action.downcast_ref::<MutatorAction<T>>() {
                    if action.id != self.id {
                        continue;
                    }

                    (action.f.lock().unwrap())(target, cx);
                }
            }
        }
    }

    pub fn spawn(&self, f: impl FnOnce(Mutator) + Send + 'static) {
        let mutator = Mutator::new(self.id);
        std::thread::spawn(move || {
            f(mutator);
        });
    }
}
