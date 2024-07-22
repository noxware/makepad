use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        padding: {top: 30}
        body =  <View> {
            padding: 40
            <RoundedView> {
                flow: Down
                show_bg: true,
                spacing: 30
                scroll_bars: <ScrollBars> {}
                draw_bg: {
                    color: #a00,
                    radius: 10.0
                }
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
                <View> {width: Fill, height: 50, show_bg: true, draw_bg: {color: #800}}
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
