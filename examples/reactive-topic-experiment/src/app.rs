use makepad_widgets::*;

use crate::app_state::AppState;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::ui::*;

    App = {{App}} {
        ui: <Ui> {}
    }
);

#[derive(Live, LiveHook)]
struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    app_state: AppState,
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui
            .handle_event(cx, event, &mut Scope::with_data(&mut self.app_state));
    }
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        crate::ui::live_design(cx);
    }
}

app_main!(App);
