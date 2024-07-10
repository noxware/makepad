use makepad_widgets::*;

use crate::app_state::AppState;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <View> {
            height: Fit
            width: Fit
            flow: Down
            align: {x: 0.5, y: 0.5}
            increment = <Button> {
                text: "+"
            }
            value = <Label> {
                text: "0"
            }
            decrement = <Button> {
                text: "-"
            }
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,

    #[rust]
    initialized: bool,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let app_state = scope.data.get_mut::<AppState>().unwrap();

        if app_state.counter.changed(event) || !self.initialized {
            self.label(id!(value))
                .set_text(&app_state.counter.get().to_string());
            self.redraw(cx);
        }

        self.initialized = true;

        self.widget_match_event(cx, event, scope);
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for Ui {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let app_state = scope.data.get_mut::<AppState>().unwrap();

        if self.button(id!(increment)).clicked(actions) {
            app_state.increment_counter(cx);
        }

        if self.button(id!(decrement)).clicked(actions) {
            app_state.decrement_counter(cx);
        }
    }
}
