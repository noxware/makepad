use makepad_rustybuzz::UnicodeBuffer;

pub use {
    std::{
        borrow::{Borrow, Cow},
        collections::VecDeque,
        env,
        hash::{Hash, Hasher},
        rc::Rc,
        cell::RefCell,
        io,
        io::prelude::*,
        fs::{File, OpenOptions},
        collections::HashMap,
        mem,
        path::Path,
    },
    crate::{
        font_cache,
        font_cache::FontCache,
        font_loader::FontLoader,
        makepad_platform::*,
        cx_2d::Cx2d,
        turtle::{Walk, Layout},
        draw_list_2d::{ManyInstances, DrawList2d, RedrawingApi},
        geometry::GeometryQuad2D,
        makepad_vector::font::Glyph,
        makepad_vector::trapezoidator::Trapezoidator,
        makepad_vector::geometry::{AffineTransformation, Transform, Vector},
        makepad_vector::internal_iter::ExtendFromInternalIterator,
        makepad_vector::path::PathIterator,
    },
    fxhash::FxHashMap,
    makepad_rustybuzz::{Direction, GlyphBuffer},
    makepad_vector::ttf_parser::GlyphId,
    unicode_segmentation::UnicodeSegmentation
};

pub(crate) const ATLAS_WIDTH: usize = 4096;
pub(crate) const ATLAS_HEIGHT: usize = 4096;

#[derive(Debug)]
pub struct CxFontAtlas {
    pub font_loader: FontLoader,
    pub texture_sdf: Texture,
    pub texture_svg: Texture,
    pub clear_buffer: bool,
    pub alloc: CxFontsAtlasAlloc,
    pub font_cache: Option<FontCache>,
}

pub struct CxShapeCache {
    shape_keys: VecDeque<OwnedShapeKey>,
    shapes: FxHashMap<OwnedShapeKey, Box<[GlyphInfo]>>,
}

impl CxShapeCache {
    pub fn new() -> Self {
        Self {
            shape_keys: VecDeque::new(),
            shapes: FxHashMap::default(),
        }
    }

    pub fn shape<'a>(
        &'a mut self,
        is_secret: bool,
        direction: Direction,
        text: &str,
        font_ids: &[usize],
        font_atlas: &mut CxFontAtlas,
    ) -> Cow<'a, [GlyphInfo]> {
        if is_secret {
            Cow::Owned(self.shape_secret(direction, text, font_ids, font_atlas))
        } else {
            Cow::Borrowed(self.shape_full(direction, text, font_ids, font_atlas))
        }
    }

    fn shape_secret(
        &mut self,
        _direction: Direction,
        text: &str,
        font_ids: &[usize],
        font_atlas: &mut CxFontAtlas,
    ) -> Vec<GlyphInfo> {
        let Some((font_id, glyph_id)) = font_ids.iter().copied().find_map(|font_id| {
            let font = font_atlas.font_loader[font_id].as_mut().unwrap();
            let glyph_id = font.glyph_id('•').0 as usize;
            if glyph_id == 0 {
                None
            } else {
                Some((font_id, glyph_id))
            }
        }) else {
            return Vec::new();
        };
        text.grapheme_indices(true).map(|(index, _)| {
            GlyphInfo {
                font_id,
                glyph_id,
                cluster: index,
            }
        }).collect()
    }

    fn shape_full<'a>(
        &'a mut self,
        direction: Direction,
        text: &str,
        font_ids: &[usize],
        font_atlas: &CxFontAtlas,
    ) -> &'a [GlyphInfo] {
        if !self.shapes.contains_key(&(direction, text, font_ids) as &(dyn ShapeKey)) {
            let shape_key = (direction, text.into(), font_ids.into());
            let mut glyph_infos = Vec::new();
            let _ = self.shape_full_recursive(
                text,
                font_ids,
                font_atlas,
                &mut glyph_infos,
                0,
            );
            if self.shape_keys.len() == 4096 {
                let shape_key = self.shape_keys.pop_front().unwrap();
                self.shapes.remove(&shape_key as &(dyn ShapeKey));
            }
            self.shape_keys.push_back(shape_key.clone());
            self.shapes.insert(shape_key, glyph_infos.into());
        }
        &self.shapes[&(direction, text, font_ids) as &(dyn ShapeKey)]
    }

    fn shape_full_recursive(
        &mut self,
        text: &str,
        font_ids: &[usize],
        font_atlas: &CxFontAtlas,
        glyph_infos: &mut Vec<GlyphInfo>,
        // Used to pass the current cluster index value after the font switch.
        base_cluster: usize,
    ) -> Result<(), ()> {

        // Get the preferred font to be used currently.
        let Some((&font_id, remaining_font_ids)) = font_ids.split_first() else {
            return Err(());
        };

        // Verify if the font is available, and if not, try the fallback font.
        let Some(font) = &font_atlas.font_loader[font_id] else {
            return self.shape_full_recursive(
                text,
                remaining_font_ids,
                font_atlas,
                glyph_infos,
                base_cluster
            );
        };

        // Create and configure the HarfBuzz buffer.
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);

        // Shape the text using HarfBuzz.
        let buffer = font.owned_font_face.with_ref(|face| {
            makepad_rustybuzz::shape(face, &[], buffer)
        });

        let infos = buffer.glyph_infos();

        // Track the processed text position to avoid reprocessing characters
        // that have already been handled through font fallback
        let mut skip_to = 0;

        let mut info_iter = infos.iter();
        while let Some(info) = info_iter.next() {
            // If this position has already been processed, skip it.
            if (info.cluster as usize) < skip_to {
                continue;
            }
            // Calculate the absolute cluster position.
            let absolute_cluster = base_cluster + info.cluster as usize;

            // Handle valid glyphs.
            if info.glyph_id != 0 {
                glyph_infos.push(GlyphInfo {
                    font_id,
                    glyph_id: info.glyph_id as usize,
                    cluster: absolute_cluster,
                });
                continue;
            }

            // Handle missing glyphs.
            let start = info.cluster as usize;

            // Find the position of the next valid glyph.
            let next_cluster = {
                let mut preview_iter = info_iter.clone();
                let next_valid = preview_iter
                    .find(|next| next.glyph_id != 0)
                    .map(|next| next.cluster as usize)
                    .unwrap_or(text.len());
                next_valid
            };

            // Allow cluster values to remain the same or increase.
            debug_assert!(
                start <= next_cluster,
                "HarfBuzz guarantees monotonic cluster values"
            );

            // Recursively call,
            // trying to process the current character with the fallback font.
            if let Ok(()) = self.shape_full_recursive(
                &text[start..next_cluster],
                remaining_font_ids,
                font_atlas,
                glyph_infos,
                base_cluster + start,
            ) {
                skip_to = next_cluster;
                continue;
            }

            glyph_infos.push(GlyphInfo {
                font_id,
                glyph_id: info.glyph_id as usize,
                cluster: absolute_cluster,
            });
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub font_id: usize,
    pub glyph_id: usize,
    pub cluster: usize,
}

trait ShapeKey {
    fn direction(&self) -> Direction;
    fn text(&self) -> &str;
    fn font_ids(&self) -> &[usize];
}

impl<'a> Borrow<dyn ShapeKey + 'a> for (Direction, Rc<str>, Rc<[usize]>) {
    fn borrow(&self) -> &(dyn ShapeKey + 'a) {
        self
    }
}

impl Eq for dyn ShapeKey + '_ {}

impl Hash for dyn ShapeKey + '_ {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.direction().hash(hasher);
        self.text().hash(hasher);
        self.font_ids().hash(hasher);
    }
}

impl PartialEq for dyn ShapeKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        if self.direction() != other.direction() {
            return false;
        }
        if self.text() != other.text() {
            return false;
        }
        true
    }
}

type OwnedShapeKey = (Direction, Rc<str>, Rc<[usize]>);

impl ShapeKey for OwnedShapeKey {
    fn direction(&self) -> Direction {
        self.0
    }

    fn text(&self) -> &str {
        &self.1
    }

    fn font_ids(&self) -> &[usize] {
        &self.2
    }
}

type BorrowedShapeKey<'a> = (Direction, &'a str, &'a [usize]);

impl<'a> ShapeKey for BorrowedShapeKey<'a> {
    fn direction(&self) -> Direction {
        self.0
    }

    fn text(&self) -> &str {
        &self.1
    }

    fn font_ids(&self) -> &[usize] {
        &self.2
    }
}

#[derive(Debug, Default)]
pub struct CxFontsAtlasAlloc {
    pub texture_size: DVec2,
    pub full: bool,
    pub xpos: usize,
    pub ypos: usize,
    pub hmax: usize,
    pub todo: Vec<CxFontsAtlasTodo>,
    pub sdf: Option<CxFontsAtlasSdfConfig>,
}

#[derive(Debug)]
pub struct CxFontsAtlasSdfConfig {
    pub params: sdfer::esdt::Params,
}

impl CxFontAtlas {
    pub fn new(texture_sdf: Texture, texture_svg: Texture, os_type: &OsType) -> Self {
        Self {
            font_loader: FontLoader::new(),
            texture_sdf,
            texture_svg,
            clear_buffer: false,
            alloc: CxFontsAtlasAlloc {
                full: false,
                texture_size: DVec2 {
                    x: ATLAS_WIDTH as f64,
                    y: ATLAS_HEIGHT as f64
                },
                xpos: 0,
                ypos: 0,
                hmax: 0,
                todo: Vec::new(),
                // Set this to `None` to use CPU-rasterized glyphs instead of SDF.
                sdf: Some(CxFontsAtlasSdfConfig {
                    params: sdfer::esdt::Params {
                        pad: 4,
                        radius: 8.0,
                        cutoff: 0.25,
                        ..Default::default()
                    },
                })
            },
            font_cache: Some(FontCache::new(os_type.get_cache_dir())),
        }
    }
}
impl CxFontsAtlasAlloc {
    pub fn alloc_atlas_glyph(&mut self, w: f64, h: f64, todo: CxFontsAtlasTodo) -> CxFontAtlasGlyph {
        // In SDF mode, leave enough room around each glyph (i.e. padding).
        let pad = self.sdf.as_ref().map_or(0, |sdf| sdf.params.pad);

        // Preserve the aspect ratio, while still scaling up at least one side
        // to a power of 2, and that side has to the larger side, due to the
        // potential for extreme aspect ratios massively increasing the size.
        let max = w.max(h);
        // NOTE(eddyb) the `* 1.5` ensures that sizes which are already close to
        // a power of 2, still get scaled up by at least 50%.
        // FIXME(eddyb) the choice of pow2 here should probably be used as the
        // atlas page (so that similar enough font sizes reuse the same pow2),
        // but that's currently complicated by the `w`/`h` computation inside
        // `DrawText::draw_inner` (and duplicated by `swrast_atlas_todo` below),
        // which adds its own padding, relative to the font size (and DPI).
        let scale = ((max * 1.5).ceil() as usize).next_power_of_two().max(64) as f64 / max;

        let (w, h) = (
            (w * scale).ceil() as usize + pad * 2,
            (h * scale).ceil() as usize + pad * 2,
        );

        if w + self.xpos >= self.texture_size.x as usize {
            self.xpos = 0;
            self.ypos += self.hmax;
            self.hmax = 0;
        }
        if h + self.ypos >= self.texture_size.y as usize {
            // ok so the fontatlas is full..
            self.full = true;
            println!("FONT ATLAS FULL, TODO FIX THIS {} > {},", h + self.ypos, self.texture_size.y);
        }
        if h > self.hmax {
            self.hmax = h;
        }

        let x_range = self.xpos..(self.xpos + w);
        let y_range = self.ypos..(self.ypos + h);

        self.xpos += w;

        self.todo.push(todo);

        CxFontAtlasGlyph {
            t1: (dvec2(
                (x_range.start + pad) as f64,
                (y_range.start + pad) as f64,
            ) / self.texture_size).into(),

            // NOTE(eddyb) `- 1` is because the texture coordinate rectangle
            // formed by `t1` and `t2` is *inclusive*, while the integer ranges
            // (i.e. `x_range` and `y_range`) are (inherently) *exclusive*.
            t2: (dvec2(
                (x_range.end - pad - 1) as f64,
                (y_range.end - pad - 1) as f64,
            ) / self.texture_size).into(),
        }
    }
}

#[derive(Debug, Clone, Live, LiveRegister)]
pub struct Font {
    #[rust] pub font_id: Option<usize>,
    #[live] pub path: LiveDependency
}

#[derive(Clone)]
pub struct CxFontsAtlasRc(pub Rc<RefCell<CxFontAtlas>>);

#[derive(Clone)]
pub struct ShapeCacheRc(pub Rc<RefCell<CxShapeCache>>);

impl LiveHook for Font {
    fn after_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        Cx2d::lazy_construct_font_atlas(cx);
        let atlas = cx.get_global::<CxFontsAtlasRc>().clone();
        self.font_id = Some(atlas.0.borrow_mut().get_or_load_font(cx, self.path.as_str()));
    }
}

impl CxFontAtlas {
    pub fn get_or_load_font(&mut self, cx: &mut Cx, path: &str) -> usize {
        self.font_loader.get_or_load(cx, path)
    }

    pub fn reset_fonts_atlas(&mut self) {
        for (_, _, cxfont) in &mut self.font_loader {
            if let Some(cxfont) = cxfont {
                cxfont.atlas_pages.clear();
            }
        }
        self.alloc.todo.clear();
        self.alloc.full = false;
        self.alloc.xpos = 0;
        self.alloc.ypos = 0;
        self.alloc.hmax = 0;
        self.clear_buffer = true;
    }

    pub fn get_internal_font_atlas_texture_id(&self) -> Texture {
        self.texture_sdf.clone()
    }
}

impl<'a> Cx2d<'a> {
    pub fn lazy_construct_font_atlas(cx: &mut Cx){
        // ok lets fetch/instance our CxFontsAtlasRc
        if !cx.has_global::<CxFontsAtlasRc>() {

            let texture_sdf = Texture::new_with_format(cx, TextureFormat::VecRu8 {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                data: Some(vec![]),
                unpack_row_length: None,
                updated: TextureUpdated::Empty,
            });

            let texture_svg = Texture::new_with_format(cx, TextureFormat::VecBGRAu8_32 {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                data: Some(vec![]),
                updated: TextureUpdated::Full,
            });

            let fonts_atlas = CxFontAtlas::new(texture_sdf, texture_svg, cx.os_type());
            cx.set_global(CxFontsAtlasRc(Rc::new(RefCell::new(fonts_atlas))));
        }
    }

    pub fn lazy_construct_shape_cache(cx: &mut Cx) {
        if !cx.has_global::<ShapeCacheRc>() {
            cx.set_global(ShapeCacheRc(Rc::new(RefCell::new(CxShapeCache::new()))));
        }
    }

    pub fn reset_fonts_atlas(cx:&mut Cx){
        if cx.has_global::<CxFontsAtlasRc>() {
            let mut fonts_atlas = cx.get_global::<CxFontsAtlasRc>().0.borrow_mut();
            fonts_atlas.reset_fonts_atlas();
        }
    }

    pub fn draw_font_atlas(&mut self) {
        let fonts_atlas_rc = self.fonts_atlas_rc.clone();
        let mut fonts_atlas = fonts_atlas_rc.0.borrow_mut();
        let fonts_atlas = &mut*fonts_atlas;

        if fonts_atlas.alloc.full {
            fonts_atlas.reset_fonts_atlas();
        }

        // Will be automatically filled after the first use.
        let mut reuse_sdfer_bufs = None;

        for todo in mem::take(&mut fonts_atlas.alloc.todo) {
            self.swrast_atlas_todo(fonts_atlas, todo, &mut reuse_sdfer_bufs);
        }
    }

    fn swrast_atlas_todo(
        &mut self,
        fonts_atlas: &mut CxFontAtlas,
        todo: CxFontsAtlasTodo,
        reuse_sdfer_bufs: &mut Option<sdfer::esdt::ReusableBuffers>,
    ) {
        let cxfont = fonts_atlas.font_loader[todo.font_id].as_mut().unwrap();
        let _atlas_page = &cxfont.atlas_pages[todo.atlas_page_id];
        let _glyph = cxfont.owned_font_face.with_ref(|face| cxfont.ttf_font.get_glyph_by_id(face, todo.glyph_id).unwrap());

        self.swrast_atlas_todo_sdf(fonts_atlas, todo, reuse_sdfer_bufs);
    }

    fn swrast_atlas_todo_sdf(
        &mut self,
        font_atlas: &mut CxFontAtlas,
        todo: CxFontsAtlasTodo,
        reuse_sdfer_bufs: &mut Option<sdfer::esdt::ReusableBuffers>,
    ) {
        let font_path = font_atlas.font_loader.path(todo.font_id).unwrap().clone();
        let font = font_atlas.font_loader[todo.font_id].as_mut().unwrap();
        let atlas_page = &font.atlas_pages[todo.atlas_page_id];

        if ['\t', '\n', '\r'].iter().any(|&c| {
            Some(todo.glyph_id) == font.owned_font_face.with_ref(|face| face.glyph_index(c).map(|id| id.0 as usize))
        }) {
            return;
        }

        let glyph_id = todo.glyph_id;
        let font_size = atlas_page.font_size_in_device_pixels;

        let font_cache_key = font_cache::Key::new(&font_path, glyph_id, font_size);

        let mut font_cache = font_atlas.font_cache.take().unwrap();

        let font_cache::Entry {
            size,
            bytes,
        } = font_cache.get_or_insert_with(font_cache_key, |bytes| {
            self.rasterize_sdf(
                font_atlas,
                todo,
                reuse_sdfer_bufs,
                bytes
            )
        });

        let font = font_atlas.font_loader[todo.font_id].as_mut().unwrap();
        let atlas_page = &font.atlas_pages[todo.atlas_page_id];
        let atlas_glyph = atlas_page.atlas_glyphs.get(&todo.glyph_id).unwrap();

        let mut atlas_data = font_atlas.texture_sdf.take_vec_u8(self.cx);
        let (atlas_w, atlas_h) = font_atlas.texture_sdf.get_format(self.cx).vec_width_height().unwrap();
        if atlas_data.is_empty() {
            atlas_data = vec![0; atlas_w * atlas_h];
        } else {
            assert_eq!(atlas_data.len(), atlas_w * atlas_h);
        }

        let sdf_pad = font_atlas.alloc.sdf.as_ref().map_or(0, |sdf| sdf.params.pad);
        let atlas_x0 = (atlas_glyph.t1.x as f64 * font_atlas.alloc.texture_size.x) as usize - sdf_pad;
        let atlas_y0 = (atlas_glyph.t1.y as f64 * font_atlas.alloc.texture_size.y) as usize - sdf_pad;

        let mut index = 0;
        for y in 0..size.height {
            let dst = &mut atlas_data[(atlas_h - atlas_y0 - 1 - y) * atlas_w..][..atlas_w][atlas_x0..][..size.width];
            for dst in dst {
                *dst = bytes[index];
                index += 1;
            }
        }

        font_atlas.texture_sdf.put_back_vec_u8(self.cx, atlas_data, Some(RectUsize::new(
            PointUsize::new(atlas_x0, atlas_h - atlas_y0 - size.height),
            size,
        )));

        font_atlas.font_cache = Some(font_cache);
    }

    fn rasterize_sdf(
        &mut self,
        fonts_atlas: &mut CxFontAtlas,
        todo: CxFontsAtlasTodo,
        reuse_sdfer_bufs: &mut Option<sdfer::esdt::ReusableBuffers>,
        bytes: &mut Vec<u8>
    ) -> SizeUsize {
        let font = fonts_atlas.font_loader[todo.font_id].as_mut().unwrap();
        let atlas_page = &font.atlas_pages[todo.atlas_page_id];
        let glyph = font.owned_font_face.with_ref(|face| font.ttf_font.get_glyph_by_id(face, todo.glyph_id).unwrap());
        let atlas_glyph = atlas_page.atlas_glyphs.get(&todo.glyph_id).unwrap();

        let font_scale_pixels = atlas_page.font_size_in_device_pixels;

        // HACK(eddyb) ideally these values computed by `DrawText::draw_inner`
        // would be kept in each `CxFontsAtlasTodo`, to avoid recomputation here.
        let render_pad_dpx = 2.0;
        let render_wh = dvec2(
            ((glyph.bounds.p_max.x - glyph.bounds.p_min.x) * font_scale_pixels).ceil() + render_pad_dpx * 2.0,
            ((glyph.bounds.p_max.y - glyph.bounds.p_min.y) * font_scale_pixels).ceil() + render_pad_dpx * 2.0,
        );

        // NOTE(eddyb) `+ 1.0` is because the texture coordinate rectangle
        // formed by `t1` and `t2` is *inclusive*, see also the comment in
        // `alloc_atlas_glyph` (about its `- 1` counterpart to this `+ 1.0`).
        let atlas_alloc_wh = dvec2(
            (atlas_glyph.t2.x - atlas_glyph.t1.x) as f64 * fonts_atlas.alloc.texture_size.x + 1.0,
            (atlas_glyph.t2.y - atlas_glyph.t1.y) as f64 * fonts_atlas.alloc.texture_size.y + 1.0,
        );

        // HACK(eddyb) because `render_wh` can be larger than the `glyph.bounds`
        // scaled by `font_scale_pixels`, and `alloc_atlas_glyph` performs some
        // non-trivial scaling on `render_wh` to get better SDF quality, this
        // division is required to properly map the glyph outline into atlas
        // space, *without* encroaching into the extra space `render_wh` added.
        let atlas_scaling = atlas_alloc_wh / render_wh;

        let transform = AffineTransformation::identity()
            .translate(Vector::new(-glyph.bounds.p_min.x, -glyph.bounds.p_min.y))
            .uniform_scale(font_scale_pixels)
            .translate(Vector::new(render_pad_dpx, render_pad_dpx))
            .scale(Vector::new(atlas_scaling.x, atlas_scaling.y));
        let commands = glyph
            .outline
            .iter()
            .map(move |command| command.transform(&transform));

        // FIXME(eddyb) try reusing this buffer.
        let mut glyph_rast = sdfer::Image2d::<_, Vec<_>>::new(
            atlas_alloc_wh.x.ceil() as usize,
            atlas_alloc_wh.y.ceil() as usize,
        );

        let mut cur = ab_glyph_rasterizer::point(0.0, 0.0);
        let to_ab = |p: makepad_vector::geometry::Point| ab_glyph_rasterizer::point(p.x as f32, p.y as f32);
        commands
        .fold(ab_glyph_rasterizer::Rasterizer::new(
            glyph_rast.width(),
            glyph_rast.height()
        ), |mut rasterizer, cmd| match cmd {
            makepad_vector::path::PathCommand::MoveTo(p) => {
                cur = to_ab(p);
                rasterizer
            }
            makepad_vector::path::PathCommand::LineTo(p1) => {
                let (p0, p1) = (cur, to_ab(p1));
                rasterizer.draw_line(p0, p1);
                cur = p1;
                rasterizer
            }
            makepad_vector::path::PathCommand::ArcTo(..) => {
                unreachable!("font glyphs should not use arcs");
            }
            makepad_vector::path::PathCommand::QuadraticTo(p1, p2) => {
                let (p0, p1, p2) = (cur, to_ab(p1), to_ab(p2));
                rasterizer.draw_quad(p0, p1, p2);
                cur = p2;
                rasterizer
            }
            makepad_vector::path::PathCommand::CubicTo(p1, p2, p3) => {
                let (p0, p1, p2, p3) = (cur, to_ab(p1), to_ab(p2), to_ab(p3));
                rasterizer.draw_cubic(p0, p1, p2, p3);
                cur = p3;
                rasterizer
            }
            makepad_vector::path::PathCommand::Close => rasterizer
        })
        .for_each_pixel_2d(|x, y, a| {
            glyph_rast[(x as usize, y as usize)] = sdfer::Unorm8::encode(a);
        });

        let glyph_out = if let Some(sdf_config) = &fonts_atlas.alloc.sdf {
            let (glyph_sdf, new_reuse_bufs) = sdfer::esdt::glyph_to_sdf(
                &mut glyph_rast,
                sdf_config.params,
                reuse_sdfer_bufs.take(),
            );
            *reuse_sdfer_bufs = Some(new_reuse_bufs);
            glyph_sdf
        } else {
            glyph_rast
        };

        for y in 0..glyph_out.height() {
            for x in 0..glyph_out.width() {
                bytes.push(glyph_out[(x, y)].to_bits());
            }
        }

        SizeUsize::new(glyph_out.width(), glyph_out.height())
    }
}

#[derive(Debug)]
pub struct CxFont {
    pub ttf_font: makepad_vector::font::TTFFont,
    pub owned_font_face: crate::owned_font_face::OwnedFace,
    pub glyph_ids: Box<[Option<GlyphId>]>,
    pub atlas_pages: Vec<CxFontAtlasPage>,
}

#[derive(Clone, Debug)]
pub struct CxFontAtlasPage {
    pub font_size_in_device_pixels: f64,
    pub atlas_glyphs: HashMap<usize, CxFontAtlasGlyph>
}

#[derive(Clone, Copy, Debug)]
pub struct CxFontAtlasGlyph {
    pub t1: Vec2,
    pub t2: Vec2,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct CxFontsAtlasTodo {
    pub font_id: usize,
    pub atlas_page_id: usize,
    pub glyph_id: usize,
}

impl CxFont {
    pub fn load_from_ttf_bytes(bytes: Rc<Vec<u8>>) -> Result<Self, crate::owned_font_face::FaceParsingError> {
        let owned_font_face = crate::owned_font_face::OwnedFace::parse(bytes, 0)?;
        let ttf_font = owned_font_face.with_ref(|face| makepad_vector::ttf_parser::from_ttf_parser_face(face));
        Ok(Self {
            ttf_font,
            owned_font_face,
            glyph_ids: vec![None; 0x10FFFF].into_boxed_slice(),
            atlas_pages: Vec::new(),
        })
    }

    pub fn get_atlas_page_id(&mut self, font_size_in_device_pixels: f64) -> usize {
        for (index, sg) in self.atlas_pages.iter().enumerate() {
            if sg.font_size_in_device_pixels == font_size_in_device_pixels {
                return index
            }
        }
        self.atlas_pages.push(CxFontAtlasPage {
            font_size_in_device_pixels,
            atlas_glyphs: HashMap::new(),
        });
        self.atlas_pages.len() - 1
    }

    pub fn glyph_id(&mut self, c: char) -> GlyphId {
        if let Some(id) = self.glyph_ids[c as usize] {
            id
        } else {
            let id = self.owned_font_face.with_ref(|face| {
                face.glyph_index(c).unwrap_or(GlyphId(0))
            });
            self.glyph_ids[c as usize] = Some(id);
            id
        }
    }

    pub fn get_glyph(&mut self, c:char)->Option<&Glyph>{
        if c < '\u{10000}' {
            let id = self.glyph_id(c);
            Some(self.get_glyph_by_id(id.0 as usize).unwrap())
        } else {
            None
        }
    }

    pub fn get_glyph_by_id(&mut self, id: usize) -> makepad_vector::ttf_parser::Result<&Glyph> {
        self.owned_font_face.with_ref(|face| self.ttf_font.get_glyph_by_id(face, id))
    }

    pub fn get_advance_width_for_char(&mut self, c: char) -> Option<f64> {
        let id = self.glyph_id(c);
        self.get_advance_width_for_glyph(id)
    }

    pub fn get_advance_width_for_glyph(&mut self, id: GlyphId) -> Option<f64> {
        self.owned_font_face.with_ref(|face| face.glyph_hor_advance(id).map(|advance_width| advance_width as f64))
    }
}
