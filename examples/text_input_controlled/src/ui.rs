use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        show_bg: true,
        draw_bg: {
            color: #aaa,
        }
        align: {x: 0.5, y: 0.5}
        body = <TextInput> {
            empty_text: "Enter something...",
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,

    #[rust]
    bind: String,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Event::Actions(actions) = event {
            if let Some(new) = self.text_input(id!(body)).changed(actions) {
                self.bind = new;
            }
        }
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.text_input(id!(body)).set_text(&self.bind);
        self.deref.draw_walk(cx, scope, walk)
    }
}
