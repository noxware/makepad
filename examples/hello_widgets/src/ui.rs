use makepad_widgets::*;

use crate::{list::ListWidgetExt, meta::MetaWidgetRefExt};

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::meta::Meta;

    Ui = {{Ui}} {
        body = <View> {
            flow: Down,
            align: {x: 0.5, y: 0.5},
            spacing: 32,
            <Label> { text: "Which fruit do you like the most?" }
            list = <FlatList> {
                flow: Down,
                height: Fill,
                width: Fill,
                Item = <View> {
                    width: Fill,
                    height: Fit,
                    button = <Button> { width: Fill }
                    fruit = <Meta> {}
                }
            }
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,

    #[live]
    item_template: Option<LivePtr>,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        if let Event::Actions(actions) = event {
            for (_, item) in self.flat_list(id!(list)).items_with_actions(actions) {
                if item.button(id!(button)).clicked(actions) {
                    let meta = item.meta(id!(fruit));
                    let fruit = meta.get_value::<Fruit>().unwrap();
                    dbg!(fruit);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(widget) = self.deref.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = widget.as_flat_list().borrow_mut() {
                for fruit in fetch_fruits() {
                    let widget = list.item(cx, LiveId::unique(), live_id!(Item)).unwrap();
                    widget.button(id!(button)).set_text(&fruit.name);
                    widget.meta(id!(fruit)).set_value(fruit.clone());
                    widget.draw_all(cx, scope);
                }
            }
        }

        DrawStep::done()
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {}
}

#[derive(Debug, Clone, PartialEq)]
struct Fruit {
    id: u64,
    name: String,
    color: String,
}

fn fetch_fruits() -> Vec<Fruit> {
    vec![
        Fruit {
            id: 1,
            name: "Apple".into(),
            color: "Red".into(),
        },
        Fruit {
            id: 2,
            name: "Banana".into(),
            color: "Yellow".into(),
        },
        Fruit {
            id: 3,
            name: "Grape".into(),
            color: "Purple".into(),
        },
    ]
}
