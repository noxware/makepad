use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    BubbleLabel = {{BubbleLabel}} {
        width: Fill,
        height: Fit,
        bubble = <RoundedView> {
            padding: 8,
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
    parent: View,

    #[rust]
    available_width: f64,
}

impl Widget for BubbleLabel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.parent.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while !self.parent.draw_walk(cx, scope, walk).is_done() {}
        let new_available_width = self.area().rect(cx).size.x;

        dbg!(new_available_width);

        if new_available_width != self.available_width {
            self.available_width = new_available_width;

            let bubble = self.view(id!(bubble));
            let label = self.label(id!(label));

            label.apply_over(
                cx,
                live!(
                    width: Fit
                ),
            );

            bubble.apply_over(
                cx,
                live!(
                    width: Fit
                ),
            );

            while !self.parent.draw_walk(cx, scope, walk).is_done() {}
            let requested_width = bubble.area().rect(cx).size.x;

            dbg!(requested_width);

            let new_width = requested_width.min(self.available_width);

            bubble.apply_over(
                cx,
                live!(
                    width: (new_width)
                ),
            );

            label.apply_over(
                cx,
                live!(
                    width: Fill
                ),
            );

            while !self.parent.draw_walk(cx, scope, walk).is_done() {}
        }

        DrawStep::done()
    }
}
