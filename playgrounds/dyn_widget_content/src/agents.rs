use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Agents = {{Agents}} {
        flow: Down,
        padding: 20,
        spacing: 10,
        height: Fit,
        draw_bg: {
            fn pixel() -> vec4 {return #19}
        }
        margin: {left: 20},
        //align: {x: 0.5, y: 0.5}
        item_template: <Button> {
            width: Fill,
            text: "abcdefg"
            draw_text: {
                text_style: {font_size: 12.0},
            }
        }
    }
);

#[derive(Live, Widget)]
pub struct Agents {
    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,


    #[live]
    item_template: Option<LivePtr>,

    #[rust]
    items: Vec<(usize, WidgetRef)>,

    #[rust]
    initialized: bool
}

const DATA: [&str; 3] = ["a", "b", "c"];

impl Widget for Agents {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.initialized {
            self.initialized = true;
            self.items = DATA.iter().enumerate().map(|(i, _)| {
                let item = WidgetRef::new_from_ptr(cx, self.item_template);
                (i, item)
            }).collect();
            dbg!(self.items.len());
        }

        for (_, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        // self.draw_bg.begin(cx, walk, self.layout);
        self.items.iter_mut().for_each(|(_, item)| {
            item.draw_all(cx, scope);
        });
        // self.draw_bg.end(cx);
        cx.end_turtle();
        DrawStep::done()
    }
}

impl LiveHook for Agents {
    fn after_new_from_doc(&mut self, cx:&mut Cx) {
        
    }
}