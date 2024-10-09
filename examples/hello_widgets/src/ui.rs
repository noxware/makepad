use makepad_widgets::*;

use crate::{list::ListWidgetExt, meta::MetaWidgetRefExt};

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::meta::Meta;
    import crate::list::List;

    Ui = {{Ui}} {
        item_template: <View> {
            button = <Button> { width: Fill }
            fruit = <Meta> {}
        }
        body = <View> {
            flow: Down,
            align: {x: 0.5, y: 0.5},
            spacing: 32,
            <Label> { text: "Which fruit do you like the most?" }
            list = <List> {flow: Down, height: Fit, width: Fill}
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
            if let Some(list) = self.list(id!(list)).borrow() {
                let clicked_fruit = list
                    .items()
                    .find(|item: &&WidgetRef| item.button(id!(button)).clicked(actions))
                    .map(|item| item.meta(id!(fruit)).get_value::<Fruit>().unwrap().clone());

                if let Some(fruit) = clicked_fruit {
                    println!(
                        "You liked the {} {}, with id {}",
                        fruit.color, fruit.name, fruit.id
                    );
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        let items = fetch_fruits()
            .iter()
            .map(|fruit| {
                let widget = WidgetRef::new_from_ptr(cx, self.item_template);
                widget.button(id!(button)).set_text(&fruit.name);
                widget.meta(id!(fruit)).set_value(fruit.clone());
                widget
            })
            .collect::<Vec<_>>();

        self.list(id!(list)).set_items(items);
    }
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