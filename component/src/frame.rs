use {
    crate::{
        makepad_platform::*,
        component_map::*,
        frame_component::*
    }
};

live_register!{
    
    Frame: {{Frame}} {}
    Solid: Frame {bg:{shape: Solid}}
    Rect: Frame {bg:{shape: Rect}}
    Box: Frame {bg:{shape: Box}}
    BoxX: Frame {bg:{shape: BoxX}}
    BoxY: Frame {bg:{shape: BoxY}}
    BoxAll: Frame {bg:{shape: BoxAll}}
    GradientY: Frame {bg:{shape: GradientY}}
    Circle: Frame {bg:{shape: Circle}}
    Hexagon: Frame {bg:{shape: Hexagon}}
    GradientX: Frame {bg:{shape: Solid, fill:GradientX}}
    GradientY: Frame {bg:{shape: Solid, fill:GradientY}}
    UserDraw: Frame {user_draw: true}
    Clip: Frame {clip: true,}
    Scroll: Frame {clip: true,}
}

// ClipFrame
// ScrollFrame
// Frame

#[derive(Live)]
#[live_register(register_as_frame_component!(Frame))]
pub struct Frame { // draw info per UI element
    /*#[alias(color, bg.color)]
    #[alias(color2, bg.color2)]
    #[alias(border_width, bg.border_width)]
    #[alias(border_color, bg.border_color)]
    #[alias(inset, bg.inset)]
    #[alias(radius, bg.radius)]
    #[alias(shape, bg.shape)]
    #[alias(fill, bg.fill)]*/
    bg: DrawShape,
    
    layout: Layout,
    
    #[alias(width, walk.width)]
    #[alias(height, walk.height)]
    #[alias(margin, walk.margin)]
    pub walk: Walk,
    
    clip: bool,
    hidden: bool,
    user_draw: bool,
    
    #[rust] view: Option<View>,
    
    scroll_x: FrameComponentRef,
    scroll_y: FrameComponentRef,
    
    #[rust] self_id: LiveId,
    
    #[rust] defer_walks: Vec<(LiveId, DeferWalk)>,
    #[rust] draw_state: DrawStateWrap<DrawState>,
    
    #[rust] children: ComponentMap<LiveId, FrameComponentRef>,
    #[rust] create_order: Vec<LiveId>
}

impl LiveHook for Frame {
    fn before_apply(&mut self, _cx: &mut Cx, from: ApplyFrom, _index: usize, _nodes: &[LiveNode]) -> Option<usize> {
        if let ApplyFrom::ApplyClear = from {
            self.create_order.clear();
        }
        None
    }
    
    fn after_apply(&mut self, cx: &mut Cx, _from: ApplyFrom, _index: usize, _nodes: &[LiveNode]) {
        if self.clip && self.view.is_none() {
            self.view = Some(View::new(cx));
        }
        //self.self_id = nodes[index].id;
        //if let Some(file_id) = from.file_id() {
        //self.live_ptr = Some(LivePtr::from_index(file_id, index, cx.live_registry.borrow().file_id_to_file(file_id).generation));
        //}
    }
    
    fn apply_value_unknown(&mut self, cx: &mut Cx, from: ApplyFrom, index: usize, nodes: &[LiveNode]) -> usize {
        let id = nodes[index].id;
        match from{
            ApplyFrom::Animate=>{
                if let Some(component) = self.children.get_mut(&nodes[index].id){
                    component.apply(cx, from, index, nodes)
                }
                else{
                    nodes.skip_node(index)
                }
            }
            _=>{
                if nodes[index].origin.id_non_unique() {
                    self.create_order.push(id);
                    return self.children.get_or_insert(cx, id, | cx | {FrameComponentRef::new(cx)})
                        .apply(cx, from, index, nodes);
                }
                else {
                    cx.apply_error_no_matching_field(live_error_origin!(), index, nodes);
                    nodes.skip_node(index)
                }
            }
        }
    }
}

impl FrameComponent for Frame {
    fn handle_component_event(&mut self, cx: &mut Cx, event: &mut Event, _self_id: LiveId) -> FrameComponentActionRef {
        self.handle_event(cx, event).into()
    }
    
    fn get_walk(&self) -> Walk {
        self.walk
    }
    
    fn draw_component(&mut self, cx: &mut Cx2d, walk: Walk) -> Result<(), LiveId> {
        self.draw_walk(cx, walk)
    }
    
    fn redraw(&mut self, cx: &mut Cx) {
        if self.clip {
            self.view.as_mut().unwrap().redraw(cx);
        }
    }
}

#[derive(Clone)]
enum DrawState {
    Drawing(usize),
    DeferWalk(usize)
}

impl Frame {
    pub fn find_child(&self, id: LiveId) -> Option<&Box<dyn FrameComponent >> {
        if let Some(child) = self.children.get(&id) {
            return child.as_ref();
        }
        for child in self.children.values() {
            if let Some(c) = child.as_ref().unwrap().find_child(id) {
                return Some(c)
            }
        }
        None
    }
    
    pub fn child<T: 'static + FrameComponent>(&self, id: LiveId) -> Option<&T> {
        if let Some(child) = self.find_child(id) {
            child.cast::<T>()
        }
        else {
            None
        }
    }
    
    pub fn find_child_mut(&mut self, id: LiveId) -> Option<&mut Box<dyn FrameComponent >> {
        if self.children.get(&id).is_some() {
            return self.children.get_mut(&id).unwrap().as_mut()
        }
        for child in self.children.values_mut() {
            if let Some(c) = child.as_mut().unwrap().find_child_mut(id) {
                return Some(c)
            }
        }
        None
    }
    
    pub fn child_mut<T: 'static + FrameComponent>(&mut self, id: LiveId) -> Option<&mut T> {
        if let Some(child) = self.find_child_mut(id) {
            child.cast_mut::<T>()
        }
        else {
            None
        }
    }
    
    pub fn handle_event(&mut self, cx: &mut Cx, event: &mut Event) -> FrameActions {
        let mut actions = Vec::new();
        for id in &self.create_order {
            if let Some(child) = self.children.get_mut(id).unwrap().as_mut() {
                actions.merge(*id, child.handle_component_event(cx, event, *id));
            }
        }
        FrameActions::from_vec(actions)
    }
    
    pub fn draw(&mut self, cx: &mut Cx2d) -> Result<(), LiveId> {
        self.draw_walk(cx, self.get_walk())
    }
    
    pub fn draw_walk(&mut self, cx: &mut Cx2d, mut walk: Walk) -> Result<(), LiveId> {
        if self.hidden {
            return Ok(())
        }
        // the beginning state
        if self.draw_state.begin(cx, DrawState::Drawing(0)) {
            self.defer_walks.clear();
            
            if self.clip {
                if self.view.as_mut().unwrap().begin(cx, walk, self.layout).is_err() {
                    return Ok(())
                };
                walk = Walk::default();
            }
            
            // ok so.. we have to keep calling draw till we return LiveId(0)
            if self.bg.shape != Shape::None {
                self.bg.begin(cx, walk, self.layout);
            }
            else {
                cx.begin_turtle(walk, self.layout);
            }
            
            if self.user_draw {
                return Err(self.self_id)
            }
        }
        
        while let DrawState::Drawing(step) = self.draw_state.get() {
            if step < self.create_order.len() {
                let id = self.create_order[step];
                if let Some(child) = self.children.get_mut(&id).unwrap().as_mut() {
                    let walk = child.get_walk();
                    if let Some(fw) = cx.defer_walk(walk) {
                        self.defer_walks.push((id, fw));
                    }
                    else {
                        child.draw_component(cx, walk) ?;
                    }
                }
                self.draw_state.set(DrawState::Drawing(step + 1));
            }
            else {
                self.draw_state.set(DrawState::DeferWalk(0));
            }
        }
        
        while let DrawState::DeferWalk(step) = self.draw_state.get() {
            if step < self.defer_walks.len() {
                let (id, dw) = &self.defer_walks[step];
                if let Some(child) = self.children.get_mut(&id).unwrap().as_mut() {
                    let walk = dw.resolve(cx);
                    child.draw_component(cx, walk) ?;
                }
                self.draw_state.set(DrawState::DeferWalk(step + 1));
            }
            else {
                if self.bg.shape != Shape::None {
                    self.bg.end(cx);
                }
                else {
                    cx.end_turtle();
                }
                if self.clip {
                    self.view.as_mut().unwrap().end(cx);
                }
                self.draw_state.end();
                break;
            }
        }
        
        return Ok(());
    }
}

