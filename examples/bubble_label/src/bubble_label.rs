use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    BubbleTemplate = <RoundedView> {
        height: Fit,
    }

    LabelTemplate = <Label> {}

    BubbleLabel = {{BubbleLabel}} {
        label: <LabelTemplate> {}
        bubble: <BubbleTemplate> {}

        width: Fill,
        <View> {
            width: 0,
            height: 0,
            meassure_bubble = <BubbleTemplate> {
                width: Fit,
                meassure_label = <LabelTemplate> {
                    width: Fit,
                }
            }
        }
        displayed_bubble = <BubbleTemplate> {
            width: 0,
            displayed_label = <LabelTemplate> {
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

    #[live]
    label: Option<LivePtr>,

    #[live]
    bubble: Option<LivePtr>,
}

impl Widget for BubbleLabel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.view(id!(meassure_bubble)).apply_from_ptr(cx, self.bubble);
        self.deref.view(id!(displayed_bubble)).apply_from_ptr(cx, self.bubble);
        self.deref.label(id!(meassure_label)).apply_from_ptr(cx, self.label);
        self.deref.label(id!(displayed_label)).apply_from_ptr(cx, self.label);

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
            dbg!("else");
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
