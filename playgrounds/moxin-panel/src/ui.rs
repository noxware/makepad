use makepad_widgets::*;

use crate::panel::PanelWidgetExt;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::panel::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <View> {
            padding: {top: 32}
            panel = <Panel> {
                /*persistent_content = {
                    <View> {
                        height: Fit
                        show_bg: true
                        draw_bg: {
                            fn pixel() -> vec4 {
                                return #a22
                            }
                        }
                        toggle = <Button> {text: "Toggle"}
                    }
                }
                open_content = {
                    <View> {
                        show_bg: true
                        draw_bg: {
                            fn pixel() -> vec4 {
                                return #22a
                            }
                        }
                        padding: {top: 40}
                    }

                }*/
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
        /*if let Event::Actions(actions) = event {
            if self.button(id!(toggle)).clicked(actions) {
                println!("Toggle button clicked");
                let mut panel = self.panel(id!(panel));
                panel.set_open(cx, !panel.is_open());
                self.redraw(cx);
            }
        }*/

        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
