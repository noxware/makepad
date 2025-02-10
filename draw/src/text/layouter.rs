use {
    super::{
        font::{Font, GlyphId, RasterizedGlyph},
        font_atlas::{ColorAtlas, GrayscaleAtlas},
        font_family::FontFamily,
        font_loader,
        font_loader::{FontDefinitions, FontLoader},
        geom::{Point, Size},
        non_nan::NonNanF32,
        sdfer, shaper,
        shaper::{ShapedGlyph, ShapedText},
        substr::Substr,
        text::{Span, Text},
    },
    std::{
        cell::RefCell,
        collections::{HashMap, VecDeque},
        rc::Rc,
    },
};

#[derive(Debug)]
pub struct Layouter {
    font_loader: FontLoader,
    cache_size: usize,
    cached_params: VecDeque<LayoutParams>,
    cached_laidout_texts: HashMap<LayoutParams, Rc<LaidoutText>>,
}

impl Layouter {
    pub fn new(definitions: FontDefinitions, settings: Settings) -> Self {
        Self {
            font_loader: FontLoader::new(definitions, settings.font_loader),
            cache_size: settings.cache_size,
            cached_params: VecDeque::with_capacity(settings.cache_size),
            cached_laidout_texts: HashMap::with_capacity(settings.cache_size),
        }
    }

    pub fn sdfer(&self) -> &Rc<RefCell<sdfer::Sdfer>> {
        self.font_loader.sdfer()
    }

    pub fn grayscale_atlas(&self) -> &Rc<RefCell<GrayscaleAtlas>> {
        self.font_loader.grayscale_atlas()
    }

    pub fn color_atlas(&self) -> &Rc<RefCell<ColorAtlas>> {
        self.font_loader.color_atlas()
    }

    pub fn get_or_layout(&mut self, params: LayoutParams) -> Rc<LaidoutText> {
        if !self.cached_laidout_texts.contains_key(&params) {
            if self.cached_params.len() == self.cache_size {
                let params = self.cached_params.pop_front().unwrap();
                self.cached_laidout_texts.remove(&params);
            }
            let laidout_text = self.layout(params.clone());
            self.cached_params.push_back(params.clone());
            self.cached_laidout_texts
                .insert(params.clone(), Rc::new(laidout_text));
        }
        self.cached_laidout_texts.get(&params).unwrap().clone()
    }

    fn layout(&mut self, params: LayoutParams) -> LaidoutText {
        let mut text = LaidoutText::default();
        LayoutContext {
            loader: &mut self.font_loader,
            max_width_in_lpxs: params.options.max_width_in_lpxs.into_inner(),
            current_x_in_lpxs: 0.0,
            current_row: LaidoutRow::default(),
            output: &mut text,
        }
        .layout(&params.text);
        text
    }
}

#[derive(Debug)]
struct LayoutContext<'a> {
    loader: &'a mut FontLoader,
    max_width_in_lpxs: f32,
    current_x_in_lpxs: f32,
    current_row: LaidoutRow,
    output: &'a mut LaidoutText,
}

impl<'a> LayoutContext<'a> {
    fn remaining_width_on_current_row_in_lpxs(&self) -> f32 {
        self.max_width_in_lpxs - self.current_x_in_lpxs
    }

    fn layout(&mut self, text: &Text) {
        for span in &text.spans {
            self.layout_span(span);
        }
        self.finish_current_row();
    }

    fn layout_span(&mut self, span: &Span) {
        let font_family = self
            .loader
            .get_or_load_font_family(&span.style.font_family_id)
            .clone();
        self.layout_by_word(
            &font_family,
            span.style.font_size_in_lpxs.into_inner(),
            &span.text,
        );
    }

    fn layout_by_word(
        &mut self,
        font_family: &Rc<FontFamily>,
        font_size_in_lpxs: f32,
        text: &Substr,
    ) {
        use unicode_segmentation::UnicodeSegmentation;

        let mut fitter = Fitter::new(
            font_family,
            font_size_in_lpxs,
            text,
            text.split_word_bounds().map(|word| word.len()).collect(),
        );
        while !fitter.is_empty() {
            match fitter.fit(self.remaining_width_on_current_row_in_lpxs()) {
                Some(text) => {
                    for glyph in &text.glyphs {
                        self.push_glyph_to_current_row(font_size_in_lpxs, glyph);
                    }
                }
                None => {
                    self.finish_current_row();
                }
            }
        }
    }

    fn push_glyph_to_current_row(&mut self, font_size_in_lpxs: f32, glyph: &ShapedGlyph) {
        let advance_in_lpxs = glyph.advance_in_ems * font_size_in_lpxs;
        let offset_in_lpxs = glyph.offset_in_ems * font_size_in_lpxs;
        self.current_row.push_glyph(LaidoutGlyph {
            font: glyph.font.clone(),
            font_size_in_lpxs,
            id: glyph.id,
            advance_in_lpxs,
            offset_in_lpxs,
        });
        self.current_x_in_lpxs += advance_in_lpxs;
    }

    fn finish_current_row(&mut self) {
        use std::mem;

        self.output.push_row(mem::take(&mut self.current_row));
        self.current_x_in_lpxs = 0.0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub font_loader: font_loader::Settings,
    pub cache_size: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_loader: font_loader::Settings {
                shaper: shaper::Settings { cache_size: 4096 },
                sdfer: sdfer::Settings {
                    padding: 4,
                    radius: 8.0,
                    cutoff: 0.25,
                },
                grayscale_atlas_size: Size::new(512, 512),
                color_atlas_size: Size::new(512, 512),
            },
            cache_size: 4096,
        }
    }
}

#[derive(Debug)]
struct Fitter<'a> {
    font_family: &'a Rc<FontFamily>,
    font_size_in_lpxs: f32,
    text: Substr,
    segment_lens: Vec<usize>,
    text_width_in_lpxs: f32,
    segment_widths_in_lpxs: Vec<f32>,
}

impl<'a> Fitter<'a> {
    fn new(
        font_family: &'a Rc<FontFamily>,
        font_size_in_lpxs: f32,
        text: &'a Substr,
        segment_lens: Vec<usize>,
    ) -> Self {
        let segment_widths_in_lpxs: Vec<_> = segment_lens
            .iter()
            .copied()
            .scan(0, |segment_start, segment_len| {
                let segment_end = *segment_start + segment_len;
                let segment = text.substr(*segment_start..segment_end);
                let segment_width_in_ems = font_family.get_or_shape(segment).width_in_ems();
                let segment_width_in_lpxs = segment_width_in_ems * font_size_in_lpxs;
                *segment_start = segment_end;
                Some(segment_width_in_lpxs)
            })
            .collect();
        Self {
            font_family,
            font_size_in_lpxs,
            text: text.clone(),
            segment_lens,
            text_width_in_lpxs: segment_widths_in_lpxs.iter().sum(),
            segment_widths_in_lpxs,
        }
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn fit(&mut self, max_width_in_lpxs: f32) -> Option<Rc<ShapedText>> {
        let mut remaining_segment_count = self.segment_lens.len();
        let mut remaining_text_len = self.text.len();
        let mut remaining_text_width_in_lpxs = self.text_width_in_lpxs;
        while remaining_segment_count > 0 {
            let remaining_text = self.text.substr(..remaining_text_len);
            if let Some(shaped_text) = self.fit_step(
                max_width_in_lpxs,
                remaining_text,
                remaining_text_width_in_lpxs,
            ) {
                self.text = self.text.substr(remaining_text_len..);
                self.text_width_in_lpxs -= remaining_text_width_in_lpxs;
                self.segment_lens.drain(..remaining_segment_count);
                self.segment_widths_in_lpxs.drain(..remaining_segment_count);
                return Some(shaped_text);
            }
            remaining_segment_count -= 1;
            remaining_text_len -= self.segment_lens[remaining_segment_count];
            remaining_text_width_in_lpxs -= self.segment_widths_in_lpxs[remaining_segment_count];
        }
        None
    }

    fn fit_step(
        &mut self,
        max_width_in_lpxs: f32,
        text: Substr,
        text_width_in_lpxs: f32,
    ) -> Option<Rc<ShapedText>> {
        if 0.5 * text_width_in_lpxs > max_width_in_lpxs {
            return None;
        }
        let shaped_text = self.font_family.get_or_shape(text);
        let shaped_text_width_in_lpxs = shaped_text.width_in_ems() * self.font_size_in_lpxs;
        if shaped_text_width_in_lpxs > max_width_in_lpxs {
            return None;
        }
        Some(shaped_text)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LayoutParams {
    pub options: LayoutOptions,
    pub text: Rc<Text>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LayoutOptions {
    pub max_width_in_lpxs: NonNanF32,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            max_width_in_lpxs: NonNanF32::new(f32::INFINITY).unwrap(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LaidoutText {
    pub rows: Vec<LaidoutRow>,
}

impl LaidoutText {
    pub fn walk_rows(&self, point_in_lpxs: Point<f32>, f: impl FnMut(Point<f32>, &LaidoutRow)) {
        let mut point_in_lpxs = point_in_lpxs;
        let mut f = f;
        for (index, row) in self.rows.iter().enumerate() {
            if index > 0 {
                point_in_lpxs.y += row.ascender_in_lpxs;
            }
            f(point_in_lpxs, row);
            point_in_lpxs.y += row.line_gap_in_lpxs - row.descender_in_lpxs;
        }
    }

    pub fn push_row(&mut self, row: LaidoutRow) {
        self.rows.push(row);
    }
}

#[derive(Clone, Debug)]
pub struct LaidoutRow {
    pub ascender_in_lpxs: f32,
    pub descender_in_lpxs: f32,
    pub line_gap_in_lpxs: f32,
    pub glyphs: Vec<LaidoutGlyph>,
}

impl LaidoutRow {
    pub fn walk_glyphs(&self, point_in_lpxs: Point<f32>, f: impl FnMut(Point<f32>, &LaidoutGlyph)) {
        let mut point_in_lpxs = point_in_lpxs;
        let mut f = f;
        for glyph in &self.glyphs {
            f(point_in_lpxs, glyph);
            point_in_lpxs.x += glyph.advance_in_lpxs;
        }
    }

    pub fn push_glyph(&mut self, glyph: LaidoutGlyph) {
        self.ascender_in_lpxs = self.ascender_in_lpxs.max(glyph.ascender_in_lpxs());
        self.descender_in_lpxs = self.descender_in_lpxs.max(glyph.descender_in_lpxs());
        self.line_gap_in_lpxs = self.line_gap_in_lpxs.max(glyph.line_gap_in_lpxs());
        self.glyphs.push(glyph);
    }
}

impl Default for LaidoutRow {
    fn default() -> Self {
        Self {
            ascender_in_lpxs: 0.0,
            descender_in_lpxs: 0.0,
            line_gap_in_lpxs: 0.0,
            glyphs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LaidoutGlyph {
    pub font: Rc<Font>,
    pub font_size_in_lpxs: f32,
    pub id: GlyphId,
    pub advance_in_lpxs: f32,
    pub offset_in_lpxs: f32,
}

impl LaidoutGlyph {
    pub fn ascender_in_lpxs(&self) -> f32 {
        self.font.ascender_in_ems() * self.font_size_in_lpxs
    }

    pub fn descender_in_lpxs(&self) -> f32 {
        self.font.descender_in_ems() * self.font_size_in_lpxs
    }

    pub fn line_gap_in_lpxs(&self) -> f32 {
        self.font.line_gap_in_ems() * self.font_size_in_lpxs
    }

    pub fn rasterize(&self, dpx_per_em: f32) -> Option<RasterizedGlyph> {
        self.font.rasterize_glyph(self.id, dpx_per_em)
    }
}
