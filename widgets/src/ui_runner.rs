use crate::*;
use std::sync::Mutex;
use std::marker::PhantomData;

/// Run code on the UI thread from another thread.
///
/// Allows you to mix non-blocking threaded code, with code that reads and updates
/// your widget in the UI thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRunner<W: Widget + 'static> {
    /// Trick to later distinguish actions sent globally thru `Cx::post_action`.
    id: usize,
    widget: PhantomData<W>
}

impl<W: Widget + 'static> UiRunner<W> {
    /// Create a new isolated instance.
    ///
    /// Functions scheduled thru this instance will not interfere with functions
    /// scheduled thru other instances.
    ///
    /// The instance itself can be copied and passed around.
    pub fn new(id: usize) -> Self {
        Self { id, widget: PhantomData }
    }

    /// Handle all functions scheduled thru this instance.
    ///
    /// You should call this once from your `handle_event` method, like:
    ///
    /// ```rust
    /// fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
    ///    // ... handle other stuff ...
    ///    self.ui_runner.handle(cx, event, self);
    /// }
    /// ```
    ///
    /// Once a function has been handled, it will never run again.
    pub fn handle(self, cx: &mut Cx, event: &Event, widget: &mut W) {
        let mut redraw = false;

        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(action) = action.downcast_ref::<UiRunnerAction<W>>() {
                    if action.id != self.id {
                        continue;
                    }

                    redraw |= action.redraw;

                    if let Some(f) = action.f.lock().unwrap().take() {
                        (f)(widget, cx);
                    }
                }
            }
        }

        if redraw {
            widget.redraw(cx);
        }
    }

    /// Schedule the provided closure to run on the UI thread.
    ///
    /// Note: You will need to specify the type of the target widget, and it should
    /// match the target type being handled by the `UiRunner::handle` method, or it
    /// will be ignored.
    pub fn defer(self, f: impl DeferCallback<W>) {
        let action = UiRunnerAction {
            f: Mutex::new(Some(Box::new(f))),
            redraw: false,
            id: self.id,
        };

        Cx::post_action(action);
    }

    /// Same as `defer`, but also redraws the widget after the closure has run.
    pub fn defer_with_redraw(
        self,
        f: impl DeferCallback<W>,
    ) {
        let action = UiRunnerAction {
            f: Mutex::new(Some(Box::new(f))),
            redraw: true,
            id: self.id,
        };

        Cx::post_action(action);
    }
}

struct UiRunnerAction<W: Widget> {
    f: Mutex<Option<Box<dyn DeferCallback<W>>>>,
    redraw: bool,
    id: usize,
}

impl<W: Widget> std::fmt::Debug for UiRunnerAction<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiRunnerAction")
            .field("id", &self.id)
            .field("f", &"...")
            .finish()
    }
}

pub trait DeferCallback<W: Widget + 'static>: FnOnce(&mut W, &mut Cx) + Send + 'static {}

impl<W: Widget + 'static, F: FnOnce(&mut W, &mut Cx) + Send + 'static> DeferCallback<W> for F {}

pub trait UiRunnerExt: Widget + Sized {
    fn ui_runner(&self) -> UiRunner<Self>;
}

impl<W: Widget> UiRunnerExt for W {
    fn ui_runner(&self) -> UiRunner<Self> {
        UiRunner::new(self.widget_uid().0 as usize)
    }
}