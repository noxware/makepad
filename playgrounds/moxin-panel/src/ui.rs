use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <Splitter> {
            axis: Horizontal,
            align: FromA(150.0),
            min_horizontal: 0.0,
            min_vertical: 0.0,
            //split_bar_size: 0.2,
            a: <View> {
                show_bg: true,
                draw_bg: {color: #f00},
            },
            b: <Splitter> {
                axis: Horizontal,
                align: FromB(150.0),
                min_horizontal: 0.0,
                min_vertical: 0.0,
                a: <View> {
                    show_bg: true,
                    draw_bg: {color: #0f0},
                },
                b: <View> {
                    show_bg: true,
                    draw_bg: {color: #00f},
                },
            },
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
