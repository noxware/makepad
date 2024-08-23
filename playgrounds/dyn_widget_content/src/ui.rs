use makepad_widgets::*;

use crate::computed_list::ComputedListWidgetExt;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::computed_list::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <View> {
            flow: Down,
            <Label> {
                text: "Hello, worldddd!"
                draw_text: {
                    text_style: {font_size: 12.0},
                }
            }
            list = <ComputedList> {}
        }

        template: <Button> {
            width: Fill,
            text: "abcdefg"
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

    #[live]
    template: Option<LivePtr>,
}

const DATA: [&str; 3] = ["a", "b", "c"];

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.computed_list(id!(list))
            .compute_from(DATA.iter(),|data| {
                let widget = WidgetRef::new_from_ptr(cx, self.template);
                widget.set_text(data);
                widget
            });
    }
}
