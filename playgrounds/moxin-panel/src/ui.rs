use makepad_widgets::*;

use crate::panel::PanelWidgetExt;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::panel::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <View> {
            flow: Overlay
            padding: {top: 32}
            panel = <Panel> {
                persistent_content = {
                    toggle = <Button> {text: "Toggle"}
                }
                open_content = {
                    <View> {
                        show_bg: true
                        draw_bg: {
                            fn pixel() -> vec4 {
                                return #2a2
                            }
                        }
                    }
                }
            }
            do = <Button> {text: "do", margin: {left: 500}}
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
        if let Event::Actions(actions) = event {
            if self.button(id!(do)).clicked(actions) {
                println!("do");
                self.panel(id!(panel)).apply_over(cx, live! {width: 10});
                self.redraw(cx);
            }

            if self.button(id!(toggle)).clicked(actions) {
                let panel = self.panel(id!(panel));
                panel.set_open(cx, !panel.is_open(cx));
            }
        }

        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
