use makepad_widgets::*;
use crate::ui_runner::UiRunner;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <Label> {
            text: "Not blessed yet"
            draw_text: {
                text_style: {font_size: 12.0},
            }
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,

    #[rust]
    ui_runner: UiRunner,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.ui_runner.handle(cx, event, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, _cx:&mut Cx) {
        let ui = self.ui_runner;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(5));
            ui.run(|s: &mut Self, cx| {
                s.label(id!(body)).set_text("A dragonfly came here and blessed your text :)");
                s.redraw(cx);
            });
        });
    }
}