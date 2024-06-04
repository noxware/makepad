use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::bubble_label::*;

    Ui = {{Ui}} {
        body = <View> {
            padding: {left: 10.0, right: 10.0, top: 28.0, bottom: 10.0},
            width: Fill,
            height: Fill,

            <RoundedView> {
                padding: 8,
                width: Fill,
                height: Fill,
                draw_bg: {
                    color: #066,
                }
                <BubbleLabel> {
                    bubble: {
                        padding: 8,
                        draw_bg: {
                            color: #317,
                        }
                    }
                    label: {
                        text: "dslkfnjdsklfjnsdkljfdslkjflsdj ldsfjklsdjflsjf",
                        draw_text: {
                            text_style: {font_size: 30.0},
                            wrap: Word,
                        }
                    }
                }
            }
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
