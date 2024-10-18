use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    BImg = {{BImg}} {
        // needed to apply shaders from `base.rs` or will not work at all.
        // should be the same for other widgets.
        img: <Image> {}
    }

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <View> {
            align: {x: 0.5, y: 0.5}
        
            <BImg>{
                img: {
                    source: dep("crate://self/resources/img.jpg"),
                    height: 230.0,
                    width: 360.0,
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

#[derive(Live, LiveHook, Widget)]
pub struct BImg {
    #[redraw] area: Area,
    #[walk] walk: Walk,
    #[live] img: Image,
}

impl Widget for BImg {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.img.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.img.draw_walk(cx, walk)
    }
}