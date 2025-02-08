use {
    crate::{
        cx_2d::Cx2d,
        draw_list_2d::ManyInstances,
        geometry::GeometryQuad2D,
        makepad_platform::*,
        text::{
            font::{AtlasKind, GlyphImage},
            geom::{Point, Rect, Size, Transformation},
            layouter::{LaidoutGlyph, LaidoutRow, LaidoutText, LayoutOptions, LayoutParams, Text},
            non_nan::NonNanF32,
        },
    },
    std::rc::Rc,
};

live_design! {
    use link::shaders::*;

    pub DrawText2 = {{DrawText2}} {
        color: #fff

        texture grayscale_texture: texture2d
        texture color_texture: texture2d

        varying tex_coord1: vec2
        varying clipped: vec2
        varying pos: vec2

        fn vertex(self) -> vec4 {
            let min_pos = self.rect_pos;
            let max_pos = self.rect_pos + self.rect_size;

            self.clipped = clamp(
                mix(min_pos, max_pos, self.geom_pos),
                self.draw_clip.xy,
                self.draw_clip.zw
            )

            let normalized: vec2 = (self.clipped - min_pos) / self.rect_size;

            self.tex_coord1 = mix(
                vec2(self.font_t1.x, self.font_t1.y),
                vec2(self.font_t2.x, self.font_t2.y),
                normalized.xy
            )
            self.pos = normalized;
            return self.camera_projection * (self.camera_view * (self.view_transform * vec4(
                self.clipped.x,
                self.clipped.y,
                self.char_depth + self.draw_zbias,
                1.
            )))
        }

        fn sdf(self, p: vec2) -> float {
            let dxt = length(dFdx(p));
            let dyt = length(dFdy(p));
            let scale = (dxt + dyt) * 512.0 * 0.5;
            
            let radius = 3.0;
            let cutoff = 0.25;
            let s = sample2d(self.grayscale_texture, p).x;
            s = clamp((s - (1.0 - cutoff)) * radius / scale + 0.5, 0.0, 1.0);
            return s;
        }

        fn pixel(self) -> vec4 {
            if self.tex_index == 0 {
                let c = vec4(1.0, 1.0, 1.0, 1.0);
                return self.sdf(self.tex_coord1.xy) * vec4(c.rgb * c.a, c.a);
            } else {
                let c = sample2d(self.color_texture, self.tex_coord1.xy);
                return vec4(c.rgb * c.a, c.a);
            }
        }
    }
}

#[derive(Live, LiveRegister)]
#[repr(C)]
pub struct DrawText2 {
    #[rust]
    pub many_instances: Option<ManyInstances>,

    #[live]
    pub geometry: GeometryQuad2D,

    #[deref]
    pub draw_vars: DrawVars,
    // these values are all generated
    #[live]
    pub color: Vec4,
    #[calc]
    pub tex_index: f32,
    #[calc]
    pub font_t1: Vec2,
    #[calc]
    pub font_t2: Vec2,
    #[calc]
    pub rect_pos: Vec2,
    #[calc]
    pub rect_size: Vec2,
    #[calc]
    pub draw_clip: Vec4,
    #[calc]
    pub char_depth: f32,
}

impl LiveHook for DrawText2 {
    fn before_apply(&mut self, cx: &mut Cx, apply: &mut Apply, index: usize, nodes: &[LiveNode]) {
        self.draw_vars
            .before_apply_init_shader(cx, apply, index, nodes, &self.geometry);
    }

    fn after_apply(&mut self, cx: &mut Cx, apply: &mut Apply, index: usize, nodes: &[LiveNode]) {
        self.draw_vars
            .after_apply_update_self(cx, apply, index, nodes, &self.geometry);
    }
}

impl DrawText2 {
    pub fn draw(&mut self, cx: &mut Cx2d, p: Point<f32>, text: Rc<Text>) {
        let laidout_text = cx.fonts.borrow_mut().get_or_layout(LayoutParams {
            options: LayoutOptions {
                max_width_in_lpxs: NonNanF32::new(128.0).unwrap(),
            },
            text,
        });
        self.draw_laidout_text(cx, p, laidout_text);
    }

    fn draw_laidout_text(&mut self, cx: &mut Cx2d<'_>, p: Point<f32>, text: Rc<LaidoutText>) {
        let fonts = cx.fonts.borrow_mut();
        self.draw_vars.texture_slots[0] = Some(fonts.grayscale_texture().clone());
        self.draw_vars.texture_slots[1] = Some(fonts.color_texture().clone());
        drop(fonts);
        let mut many_instances = cx.begin_many_aligned_instances(&self.draw_vars).unwrap();
        let mut p = p;
        let mut is_first_row = true;
        for row in &text.rows {
            if !is_first_row {
                p.y += row.ascender_in_lpxs;
            }
            let x = p.x;
            self.draw_laidout_row(cx, &mut p, row, &mut many_instances.instances);
            p.x = x;
            p.y += -row.descender_in_lpxs + row.line_gap_in_lpxs;
            is_first_row = false;
        }
        let new_area = cx.end_many_instances(many_instances);
        self.draw_vars.area = cx.update_area_refs(self.draw_vars.area, new_area);
    }

    fn draw_laidout_row(
        &mut self,
        cx: &mut Cx2d<'_>,
        p: &mut Point<f32>,
        row: &LaidoutRow,
        output: &mut Vec<f32>,
    ) {
        for glyph in &row.glyphs {
            self.draw_laidout_glyph(cx, p, glyph, output);
            p.x += glyph.advance_in_lpxs;
        }

        cx.cx.debug.rect(
            makepad_platform::Rect {
                pos: dvec2(p.x as f64, p.y as f64),
                size: dvec2(1000.0, 1.0),
            },
            vec4(1.0, 0.0, 0.0, 1.0),
        );
    }

    fn draw_laidout_glyph(
        &mut self,
        cx: &mut Cx2d<'_>,
        p: &mut Point<f32>,
        laidout_glyph: &LaidoutGlyph,
        output: &mut Vec<f32>,
    ) {
        let lpxs_per_dpx = cx.current_dpi_factor() as f32;
        let font_size_in_dpxs = laidout_glyph.font_size_in_lpxs * lpxs_per_dpx;
        if let Some(allocated_glyph) = laidout_glyph.allocate(font_size_in_dpxs) {
            self.draw_glyph_image(
                cx,
                Point::new(p.x + laidout_glyph.offset_in_lpxs, p.y),
                allocated_glyph,
                laidout_glyph.font_size_in_lpxs,
                output,
            );
        }
    }

    fn draw_glyph_image(
        &mut self,
        cx: &mut Cx2d<'_>,
        p: Point<f32>,
        image: GlyphImage,
        font_size_in_lpxs: f32,
        output: &mut Vec<f32>,
    ) {
        use crate::text::sdf;

        fn tex_coord(point: Point<usize>, size: Size<usize>) -> Point<f32> {
            Point::new(
                (2 * point.x + 1) as f32 / (2 * size.width) as f32,
                (2 * point.y + 1) as f32 / (2 * size.height) as f32,
            )
        }

        fn point_to_vec2(point: Point<f32>) -> Vec2 {
            vec2(point.x, point.y)
        }

        fn size_to_vec2(point: Size<f32>) -> Vec2 {
            vec2(point.width, point.height)
        }

        let transform = Transformation::scaling_uniform(font_size_in_lpxs / image.dpxs_per_em)
            .translate(p.x, p.y);
        let bounds_in_lpxs = Rect::new(
            Point::new(image.bounds_in_dpxs.min().x, -image.bounds_in_dpxs.max().y),
            image.bounds_in_dpxs.size,
        )
        .transform(transform);
        self.rect_pos = point_to_vec2(bounds_in_lpxs.origin - Size::new(sdf::PADDING as f32 / cx.current_dpi_factor() as f32, sdf::PADDING as f32 / cx.current_dpi_factor() as f32));
        self.rect_size = size_to_vec2(bounds_in_lpxs.size + Size::new(2.0 * sdf::PADDING as f32 / cx.current_dpi_factor() as f32, 2.0 * sdf::PADDING as f32 / cx.current_dpi_factor() as f32)); 
        self.tex_index = match image.atlas_kind {
            AtlasKind::Grayscale => 0.0,
            AtlasKind::Color => 1.0,
        };
        self.font_t1 = point_to_vec2(tex_coord(image.atlas_bounds.min(), image.atlas_size));
        self.font_t2 = point_to_vec2(tex_coord(image.atlas_bounds.max(), image.atlas_size));
        self.char_depth += 1.0; // TODO

        output.extend_from_slice(self.draw_vars.as_slice());
        /*
        println!("RECT POS {:?}", self.rect_pos);
        println!("RECT SIZE {:?}", self.rect_size);
        println!("FONT T1 {:?}", self.font_t1);
        println!("FONT T2 {:?}", self.font_t2);
        */
    }
}
