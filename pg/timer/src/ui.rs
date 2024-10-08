use crate::ui_runner::UiRunner;
use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <Label> {
            text: "0"
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
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let ui = self.ui_runner;
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            ui.defer(|s: &mut Self, cx| {
                let label = s.label(id!(body));
                let current = label.text().parse::<i32>().unwrap();
                label.set_text(&(current + 1).to_string());
                s.redraw(cx);
            });
        });
    }
}
