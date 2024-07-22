use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Wea = {{Wea}} {
        align: {x: 0.5, y: 0.5}
        label = <Label> {}

    }
);

#[derive(Live, Widget)]
pub struct Wea {
    #[deref]
    deref: View,

    #[live]
    something: Option<LivePtr>,

    #[rust]
    pub flag: bool,

    #[rust]
    initialized: bool,
}

impl Widget for Wea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.initialized {
            self.initialized = true;
            cx.get_nodes_from_live_ptr(self.something.unwrap(), |cx, file_id, index, nodes| {
                //println!("Got nodes: {:?}", nodes);
                std::fs::write(
                    "/Users/wyeworks/nodes.txt",
                    format!(
                        "{:?}",
                        nodes
                            .iter()
                            .filter(|n| n.id == live_id!(text))
                            .collect::<Vec<_>>()
                    ),
                )
                .unwrap();
                index
            });
        }

        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.label(id!(label)).apply_from_ptr(cx, self.something);
        self.label(id!(label))
            .apply_over(cx, live! {text: "forced"});
        cx.redraw_all();
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Wea {}
