use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    bubble = <RoundedView> {
        padding: 8,
        width: Fit,
        height: Fit,
        draw_bg: {
            color: #317,
        }
    }


    label = <Label> {
        text: "Hello, world!"
        draw_text: {
            text_style: {font_size: 12.0},
        }
    }

    BubbleLabel = {{BubbleLabel}} {
        width: Fill,
        <View> {
            width: 0,
            height: 0,
            meassure_bubble = <bubble> {
                width: Fit,
                <label> {
                    width: Fit,
                }
            }
        }
        displayed_bubble = <bubble> {
            width: 0,
            <label> {
                width: Fill,
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
        while !self.deref.draw_walk(cx, scope, walk).is_done() {}
        let available_width = self.deref.area().rect(&cx.cx).size.x;
        let requested_width = self.deref.view(id!(meassure_bubble)).area().rect(&cx.cx).size.x;

        dbg!(requested_width);
        dbg!(available_width);

        if available_width < requested_width {
            self.deref.view(id!(displayed_bubble)).apply_over(cx, live!(
                width: (available_width)
            ));
        } else {
            self.deref.view(id!(displayed_bubble)).apply_over(cx, live!(
                width: (requested_width)
            ));
        }

        if !self.did_redraw {
            self.did_redraw = true;
            self.deref.redraw(cx);
        }

        DrawStep::done()
    }
}
