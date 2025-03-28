use {
    std::{
        cell::RefCell,
        ops::Deref,
        ops::DerefMut,
        rc::Rc,
    },
    crate::{
        font_loader::FontLoader,
        makepad_math::{DVec2,Vec2Index},
        makepad_platform::{
            DrawEvent,
            Area,
            DrawListId,
            WindowId,
            PassId,
            Pass,
            CxPassParent,
            CxPassRect,
            Cx
        },
        nav::CxNavTreeRc,
        icon_atlas::CxIconAtlasRc,
        font_atlas::CxFontsAtlasRc,
        draw_list_2d::DrawList2d,
        glyph_rasterizer::GlyphRasterizer,
        text::{fonts::Fonts, layouter},
        text_shaper::TextShaper,
        turtle::{Turtle, TurtleWalk, Walk, AlignEntry},
    },
    makepad_rustybuzz::UnicodeBuffer,
};

pub struct PassStackItem {
    pub pass_id: PassId,
    dpi_factor: f64,
    draw_list_stack_len: usize,
    turtles_len: usize
}


pub struct Cx2d<'a> {
    pub cx: &'a mut Cx,
    pub draw_event: &'a DrawEvent,
    pub (crate) pass_stack: Vec<PassStackItem>,
    pub (crate) overlay_id: Option<DrawListId>,
    //pub (crate) overlay_sweep_lock: Option<Rc<RefCell<Area>>>,
    pub draw_list_stack: Vec<DrawListId>,
    pub (crate) turtles: Vec<Turtle>,
    pub (crate) turtle_walks: Vec<TurtleWalk>,
    pub (crate) turtle_clips: Vec<(DVec2, DVec2)>,
    pub (crate) align_list: Vec<AlignEntry>,
    pub fonts: Rc<RefCell<Fonts>>,
    pub font_loader: Rc<RefCell<FontLoader>>,
    pub text_shaper: Rc<RefCell<TextShaper>>,
    pub glyph_rasterizer: Rc<RefCell<GlyphRasterizer>>,
    pub fonts_atlas_rc: CxFontsAtlasRc,
    pub icon_atlas_rc: CxIconAtlasRc,
    pub nav_tree_rc: CxNavTreeRc,
    pub rustybuzz_buffer: Option<UnicodeBuffer>, 
}

impl<'a> Deref for Cx2d<'a> {type Target = Cx; fn deref(&self) -> &Self::Target {self.cx}}
impl<'a> DerefMut for Cx2d<'a> {fn deref_mut(&mut self) -> &mut Self::Target {self.cx}}

impl<'a> Drop for Cx2d<'a> {
    fn drop(&mut self) {
        if !self.fonts.borrow_mut().update_textures(&mut self.cx) {
            self.cx.redraw_all();
        }

        self.draw_font_atlas();
        self.draw_icon_atlas();
    }
}

impl<'a> Cx2d<'a> {
    
    pub fn get_current_window_id(&self)->Option<WindowId>{
        self.cx.get_pass_window_id(self.pass_stack.last().unwrap().pass_id)
    }
    /*pub fn set_sweep_lock(&mut self, lock:Area){
        *self.overlay_sweep_lock.as_ref().unwrap().borrow_mut() = lock;
    }
    
    pub fn clear_sweep_lock(&mut self, lock:Area){
        let mut sweep_lock = self.overlay_sweep_lock.as_ref().unwrap().borrow_mut();
        if *sweep_lock == lock{
            *sweep_lock = Area::Empty
        }
    }*/
    
    pub fn new(cx: &'a mut Cx, draw_event: &'a DrawEvent) -> Self {
        Self::lazy_construct_fonts(cx);
        Self::lazy_construct_font_loader(cx);
        Self::lazy_construct_text_shaper(cx);
        Self::lazy_construct_glyph_rasterizer(cx);
        Self::lazy_construct_font_atlas(cx);
        Self::lazy_construct_nav_tree(cx);
        Self::lazy_construct_icon_atlas(cx);
        cx.redraw_id += 1;
        let fonts = cx.get_global::<Rc<RefCell<Fonts>>>().clone();
        let font_loader = cx.get_global::<Rc<RefCell<FontLoader>>>().clone();
        let text_shaper=  cx.get_global::<Rc<RefCell<TextShaper>>>().clone();
        let glyph_rasterizer = cx.get_global::<Rc<RefCell<GlyphRasterizer>>>().clone();
        let fonts_atlas_rc = cx.get_global::<CxFontsAtlasRc>().clone();
        let nav_tree_rc = cx.get_global::<CxNavTreeRc>().clone();
        let icon_atlas_rc = cx.get_global::<CxIconAtlasRc>().clone();
        Self {
            overlay_id: None,
            fonts,
            font_loader,
            text_shaper,
            glyph_rasterizer,
            fonts_atlas_rc,
            cx: cx,
            draw_event,
            // overlay_sweep_lock: None,
            pass_stack: Vec::new(),
            draw_list_stack: Vec::with_capacity(64),
            turtle_clips: Vec::with_capacity(1024),
            turtle_walks: Vec::with_capacity(1024),
            turtles: Vec::with_capacity(64),
            align_list: Vec::with_capacity(4096),
            nav_tree_rc,
            icon_atlas_rc,
            rustybuzz_buffer: Some(UnicodeBuffer::new()),
        }
    }
    
    pub fn current_dpi_factor(&self) -> f64 {
        self.pass_stack.last().unwrap().dpi_factor
    }
    
    pub fn inside_pass(&self) -> bool {
        self.pass_stack.len()>0
    }
    
    pub fn make_child_pass(&mut self, pass: &Pass) {
        let pass_id = self.pass_stack.last().unwrap().pass_id;
        let cxpass = &mut self.passes[pass.pass_id()];
        cxpass.parent = CxPassParent::Pass(pass_id);
    }
    
    pub fn begin_pass(&mut self, pass: &Pass, dpi_override: Option<f64>) {
        let cxpass = &mut self.passes[pass.pass_id()];
        cxpass.main_draw_list_id = None;
        let dpi_factor = if let Some(dpi_override) = dpi_override {dpi_override}
        else {
            match cxpass.parent {
                CxPassParent::Window(window_id) => {
                    self.passes[pass.pass_id()].pass_rect = Some(CxPassRect::Size(self.windows[window_id].get_inner_size()));
                    self.get_delegated_dpi_factor(pass.pass_id())
                }
                CxPassParent::Pass(pass_id) => {
                    self.passes[pass.pass_id()].pass_rect = self.passes[pass_id].pass_rect.clone();
                    self.get_delegated_dpi_factor(pass_id)
                }
                _ => {
                    1.0
                }
            }
        };
        self.passes[pass.pass_id()].dpi_factor = Some(dpi_factor);
        
        self.pass_stack.push(PassStackItem {
            dpi_factor,
            draw_list_stack_len: self.draw_list_stack.len(),
            turtles_len: self.turtles.len(),
            pass_id: pass.pass_id()
        });
    }
    
    pub fn end_pass(&mut self, pass: &Pass) {
        let stack_item = self.pass_stack.pop().unwrap();
        if stack_item.pass_id != pass.pass_id() {
            panic!();
        }
        
        if self.draw_list_stack.len() != stack_item.draw_list_stack_len {
            panic!("Draw list stack disaligned, forgot an end_view(cx)");
        }
        if self.turtles.len() != stack_item.turtles_len {
            panic!("Turtle stack disaligned, forgot an end_turtle()");
        }
    }
    
    pub fn set_pass_area(&mut self, pass: &Pass, area: Area) {
        self.passes[pass.pass_id()].pass_rect = Some(CxPassRect::Area(area));
    }
        
    pub fn set_pass_area_with_origin(&mut self, pass: &Pass, area: Area, origin:DVec2) {
        self.passes[pass.pass_id()].pass_rect = Some(CxPassRect::AreaOrigin(area, origin));
    }
        
    pub fn set_pass_shift_scale(&mut self, pass: &Pass, shift: DVec2, scale: DVec2) {
        self.passes[pass.pass_id()].view_shift = shift;
        self.passes[pass.pass_id()].view_scale = scale;
    }
    
    /*
    pub fn set_pass_scaled_area(&mut self, pass: &Pass, area: Area, scale:f64) {
        self.passes[pass.pass_id()].pass_rect = Some(CxPassRect::ScaledArea(area, scale));
    }   */

    pub fn current_pass_size(&self) -> DVec2 {
        self.cx.get_pass_rect(self.pass_stack.last().unwrap().pass_id, self.current_dpi_factor()).unwrap().size
    }
    
    pub fn will_redraw(&self, draw_list_2d: &mut DrawList2d, walk: Walk) -> bool {
        // ok so we need to check if our turtle position has changed since last time.
        // if it did, we redraw
        let rect = self.peek_walk_turtle(walk);
        if draw_list_2d.dirty_check_rect != rect {
            draw_list_2d.dirty_check_rect = rect;
            return true;
        }
        self.draw_event.draw_list_will_redraw(self, draw_list_2d.draw_list.id())
    }

    pub fn will_redraw_check_axis(&self, draw_list_2d: &mut DrawList2d, size:f64, axis:Vec2Index) -> bool {
        // ok so we need to check if our turtle position has changed since last time.
        // if it did, we redraw
        if draw_list_2d.dirty_check_rect.size.index(axis) != size {
            draw_list_2d.dirty_check_rect.size.set_index(axis, size);
            return true;
        }
        self.draw_event.draw_list_will_redraw(self, draw_list_2d.draw_list.id())
    }
    
    pub fn append_sub_draw_list(&mut self, draw_list_2d: &DrawList2d)  {
        let dl = &mut self.cx.draw_lists[*self.draw_list_stack.last().unwrap()];
        dl.append_sub_list(self.cx.redraw_id, draw_list_2d.draw_list_id());
    }
    
    pub fn lazy_construct_fonts(cx: &mut Cx) {
        if !cx.has_global::<Rc<RefCell<Fonts>>>() {
            let fonts = Fonts::new(
                cx,
                layouter::Settings::default(),
            );
            cx.set_global(Rc::new(RefCell::new(fonts)));
        }
    }

    pub fn update_fonts_textures(&mut self) {
        self.fonts.borrow_mut().update_textures(&mut self.cx);
    }
}
