use makepad_widgets::*;

use crate::wea::WeaWidgetExt;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::wea::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        body = <Wea> {
            something: <Label> {text: "I'm a label, that's what I am"}
            do = <Button> {text: "do"}
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
                println!("Button clicked!");
                self.wea(id!(body)).borrow_mut().unwrap().flag = true;
                self.apply_over(cx, live! {body = {something: {text: "clicked"}}});
            }
        }

        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
