use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    BubbleLabel = {{BubbleLabel}} {
        bubble = <RoundedView> {
            padding: 8,
            width: Fit,
            height: Fit,
            draw_bg: {
                color: #317,
            }
            label = <Label> {
                text: "Hello, worlddddddddddd,dsfgjkldfshgjkdfhjghdfjghdkjghdfkjhfdkhkjdf f,shj,d!"
                draw_text: {
                    text_style: {font_size: 12.0},
                }
            }
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct BubbleLabel {
    #[deref]
    deref: View,

    #[rust]
    did_redraw: bool,
}

impl Widget for BubbleLabel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.apply_over(cx, live!(
            width: 0.0,
        ));

        self.deref.label(id!(label)).apply_over(cx, live!(
            width: Fit,
        ));

        self.deref.view(id!(bubble)).apply_over(cx, live!(
            width: Fit,
        ));

        while !self.deref.draw_walk(cx, scope, walk).is_done() {}
        let available_width = self.deref.area().rect(&cx.cx).size.x;
        let requested_width = self.deref.view(id!(bubble)).area().rect(&cx.cx).size.x;

        dbg!(requested_width);
        dbg!(available_width);

        self.deref.apply_over(cx, live!(
            width: Fill,
        ));

        self.deref.label(id!(label)).apply_over(cx, live!(
            width: Fill,
        ));

        if available_width < requested_width {
            self.deref.view(id!(bubble)).apply_over(cx, live!(
                width: (available_width)
            ));
        } else {
            self.deref.view(id!(bubble)).apply_over(cx, live!(
                width: (requested_width)
            ));
        }

        self.deref.draw_walk(cx, scope, walk)
    }
}
