use makepad_widgets::*;
use crate::dragonfly::{self, Dragonfly, Mutator};

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <Label> {
            text: "Hello, world!"
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

    #[rust(Dragonfly::new())]
    dragonfly: Dragonfly,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.dragonfly.clone().handle(self, cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, cx:&mut Cx) {
        self.dragonfly.spawn(|mutator: Mutator| {
            std::thread::sleep(std::time::Duration::from_secs(3));
            mutator.mutate(|ui: &mut Self, cx: &mut Cx| {
                println!("Mutating...");
                ui.label(id!(body)).set_text("Hello, Dragonfly!");
                ui.redraw(cx);
            });
        });
    }
}