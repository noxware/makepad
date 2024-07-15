use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    FadeView = <CachedView> {
        draw_bg: {
            instance opacity: 1.0

            fn pixel(self) -> vec4 {
                let color = sample2d_rt(self.image, self.pos * self.scale + self.shift) + vec4(self.marked, 0.0, 0.0, 0.0);
                return Pal::premul(vec4(color.xyz, color.w * self.opacity))
            }
        }
    }

    Panel = {{Panel}} {
        flow: Overlay,
        width: 300
        open_content = <FadeView> {
            <Label> {text: "Open content"}
        }
        persistent_content = <View> {
            <Label> {text: "Persistent content"}
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Panel {
    #[deref]
    parent: View,
    #[live]
    open: bool,
    #[live]
    open_width: f32,
    #[rust]
    initialized: bool,
    /*
    #[live]
    persistent_content: View,
    #[live]
    open_content: View,
    */
}

impl Widget for Panel {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.initialized {
            self.initialized = true;
            self.update_width(cx);
        }

        self.parent.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.parent.handle_event(cx, event, scope)
    }
}

impl Panel {
    fn update_width(&mut self, cx: &mut Cx) {
        if self.view(id!(open_content)).is_visible() {
            self.apply_over(
                cx,
                live! {
                    width: 300
                },
            );
        } else {
            self.apply_over(
                cx,
                live! {
                    width: 50
                },
            );
        }
    }
}

impl PanelRef {
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) {
        self.view(id!(open_content)).set_visible(open);
        self.borrow_mut().and_then(|mut panel| {
            panel.update_width(cx);
            Some(())
        });
    }

    pub fn is_open(&self) -> bool {
        self.view(id!(open_content)).is_visible()
    }
}
