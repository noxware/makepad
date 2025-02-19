use {
    super::{
        color::Color,
        font::{Font, GlyphId, RasterizedGlyph},
        font_atlas::{ColorAtlas, GrayscaleAtlas},
        font_family::{FontFamily, FontFamilyId},
        font_loader::{self, FontDefinitions, FontLoader},
        geom::{Point, Size},
        non_nan::NonNanF32,
        sdfer,
        shape::{self, ShapedText},
        substr::Substr,
    },
    std::{
        cell::RefCell,
        collections::{HashMap, VecDeque},
        ops::{ControlFlow, Range},
        rc::Rc,
    },
};

#[derive(Debug)]
pub struct Layouter {
    font_loader: FontLoader,
    cache_size: usize,
    cached_params: VecDeque<LayoutParams>,
    cached_results: HashMap<LayoutParams, Rc<LaidoutText>>,
}

impl Layouter {
    pub fn new(definitions: FontDefinitions, settings: Settings) -> Self {
        Self {
            font_loader: FontLoader::new(definitions, settings.font_loader),
            cache_size: settings.cache_size,
            cached_params: VecDeque::with_capacity(settings.cache_size),
            cached_results: HashMap::with_capacity(settings.cache_size),
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
        if !self.cached_results.contains_key(&params) {
            if self.cached_params.len() == self.cache_size {
                let params = self.cached_params.pop_front().unwrap();
                self.cached_results.remove(&params);
            }
            let result = self.layout(params.clone());
            self.cached_params.push_back(params.clone());
            self.cached_results.insert(params.clone(), Rc::new(result));
        }
        self.cached_results.get(&params).unwrap().clone()
    }

    fn layout(&mut self, params: LayoutParams) -> LaidoutText {
        let mut rows = Vec::new();
        LayoutContext {
            loader: &mut self.font_loader,
            text: &params.text,
            options: params.options,
            start_index: 0,
            current_index: 0,
            current_x_in_lpxs: 0.0,
            glyphs: Vec::new(),
            out_rows: &mut rows,
        }
        .layout(&params.spans);
        LaidoutText { rows }
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
                shaper: shape::Settings { cache_size: 4096 },
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
struct LayoutContext<'a> {
    loader: &'a mut FontLoader,
    text: &'a Substr,
    options: LayoutOptions,
    start_index: usize,
    current_index: usize,
    current_x_in_lpxs: f32,
    glyphs: Vec<LaidoutGlyph>,
    out_rows: &'a mut Vec<LaidoutRow>,
}

impl<'a> LayoutContext<'a> {
    fn max_width_in_lpxs(&self) -> f32 {
        self.options.max_width_in_lpxs.into_inner()
    }

    fn remaining_width_on_current_row_in_lpxs(&self) -> f32 {
        self.max_width_in_lpxs() - self.current_x_in_lpxs
    }

    fn layout(&mut self, spans: &[Span]) {
        for span in spans {
            self.layout_span(span);
        }
        self.finish_row();
    }

    fn layout_span(&mut self, span: &Span) {
        let font_family = self
            .loader
            .get_or_load_font_family(&span.style.font_family_id)
            .clone();
        if self.options.max_width_in_lpxs.into_inner() == f32::INFINITY {
            self.append_text(
                span.style.font_size_in_lpxs.into_inner(),
                span.style.baseline,
                span.style.color,
                &font_family.get_or_shape(self.text.substr(span.range.clone())),
            );
        } else {
            self.wrap_by_word(
                &font_family,
                span.style.font_size_in_lpxs.into_inner(),
                span.style.baseline,
                span.style.color,
                span.range.clone(),
            );
        }
    }

    fn wrap_by_word(
        &mut self,
        font_family: &Rc<FontFamily>,
        font_size_in_lpxs: f32,
        baseline: Baseline,
        color: Color,
        byte_range: Range<usize>,
    ) {
        use unicode_segmentation::UnicodeSegmentation;

        let text = self.text.substr(byte_range.clone());
        let segment_lens = text.split_word_bounds().map(|word| word.len()).collect();
        let mut fitter = Fitter::new(font_family, font_size_in_lpxs, text, segment_lens);
        while !fitter.is_empty() {
            match fitter.fit(self.remaining_width_on_current_row_in_lpxs()) {
                Some(text) => {
                    self.append_text(font_size_in_lpxs, baseline, color, &text);
                }
                None => {
                    if self.glyphs.is_empty() {
                        self.wrap_by_grapheme(
                            font_family,
                            font_size_in_lpxs,
                            baseline,
                            color,
                            0..fitter.pop_front(),
                        );
                    } else {
                        self.finish_row()
                    }
                }
            }
        }
    }

    fn wrap_by_grapheme(
        &mut self,
        font_family: &Rc<FontFamily>,
        font_size_in_lpxs: f32,
        baseline: Baseline,
        color: Color,
        byte_range: Range<usize>,
    ) {
        use unicode_segmentation::UnicodeSegmentation;

        let text = self.text.substr(byte_range.clone());
        let segment_lens = text.split_word_bounds().map(|word| word.len()).collect();
        let mut fitter = Fitter::new(font_family, font_size_in_lpxs, text, segment_lens);
        while !fitter.is_empty() {
            match fitter.fit(self.remaining_width_on_current_row_in_lpxs()) {
                Some(text) => {
                    self.append_text(font_size_in_lpxs, baseline, color, &text);
                }
                None => {
                    if self.glyphs.is_empty() {
                        self.append_text(
                            font_size_in_lpxs,
                            baseline,
                            color,
                            &font_family.get_or_shape(self.text.substr(0..fitter.pop_front())),
                        );
                    } else {
                        self.finish_row();
                    }
                }
            }
        }
    }

    fn append_text(
        &mut self,
        font_size_in_lpxs: f32,
        baseline: Baseline,
        color: Color,
        text: &ShapedText,
    ) {
        for glyph in &text.glyphs {
            let glyph = LaidoutGlyph {
                font: glyph.font.clone(),
                font_size_in_lpxs,
                baseline,
                color,
                id: glyph.id,
                cluster: self.current_index + glyph.cluster,
                advance_in_lpxs: glyph.advance_in_ems * font_size_in_lpxs,
                offset_in_lpxs: glyph.offset_in_ems * font_size_in_lpxs,
            };
            self.current_x_in_lpxs += glyph.advance_in_lpxs;
            self.glyphs.push(glyph);
        }
        self.current_index += text.text.len();
    }

    fn finish_row(&mut self) {
        use std::mem;

        let glyphs = mem::take(&mut self.glyphs);
        let row = LaidoutRow {
            text: self.text.substr(self.start_index..self.current_index),
            width_in_lpxs: self.current_x_in_lpxs,
            max_width_in_lpxs: if self.max_width_in_lpxs() == f32::INFINITY {
                self.current_x_in_lpxs
            } else {
                self.max_width_in_lpxs()
            },
            align: self.options.align,
            ascender_in_lpxs: glyphs
                .iter()
                .map(|glyph| glyph.baseline_y_in_lpxs() + glyph.ascender_in_lpxs())
                .reduce(f32::max)
                .unwrap_or(0.0),
            descender_in_lpxs: glyphs
                .iter()
                .map(|glyph| glyph.baseline_y_in_lpxs() + glyph.descender_in_lpxs())
                .reduce(f32::min)
                .unwrap_or(0.0),
            line_gap_in_lpxs: glyphs
                .iter()
                .map(|glyph| glyph.line_gap_in_lpxs())
                .reduce(f32::max)
                .unwrap_or(0.0),
            glyphs,
        };
        self.start_index = self.current_index;
        self.current_x_in_lpxs = 0.0;
        self.out_rows.push(row);
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
        text: Substr,
        segment_lens: Vec<usize>,
    ) -> Self {
        let segment_widths_in_lpxs: Vec<_> = segment_lens
            .iter()
            .copied()
            .scan(0, |segment_start, segment_len| {
                let segment_end = *segment_start + segment_len;
                let segment = text.substr(*segment_start..segment_end);
                let segment_width_in_ems = font_family.get_or_shape(segment).width_in_ems;
                let segment_width_in_lpxs = segment_width_in_ems * font_size_in_lpxs;
                *segment_start = segment_end;
                Some(segment_width_in_lpxs)
            })
            .collect();
        Self {
            font_family,
            font_size_in_lpxs,
            text,
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
        let shaped_text_width_in_lpxs = shaped_text.width_in_ems * self.font_size_in_lpxs;
        if shaped_text_width_in_lpxs > max_width_in_lpxs {
            return None;
        }
        Some(shaped_text)
    }

    fn pop_front(&mut self) -> usize {
        let segment_len = self.segment_lens[0];
        self.text = self.text.substr(self.segment_lens[0]..);
        self.text_width_in_lpxs -= self.segment_widths_in_lpxs[0];
        self.segment_lens.remove(0);
        self.segment_widths_in_lpxs.remove(0);
        segment_len
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LayoutParams {
    pub text: Substr,
    pub spans: Rc<[Span]>,
    pub options: LayoutOptions,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub style: Style,
    pub range: Range<usize>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Style {
    pub font_family_id: FontFamilyId,
    pub font_size_in_lpxs: NonNanF32,
    pub baseline: Baseline,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Baseline {
    Alphabetic,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LayoutOptions {
    pub max_width_in_lpxs: NonNanF32,
    pub align: Align,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            max_width_in_lpxs: NonNanF32::new(f32::INFINITY).unwrap(),
            align: Align::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Default for Align {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Clone, Debug)]
pub struct LaidoutText {
    pub rows: Vec<LaidoutRow>,
}

impl LaidoutText {
    pub fn walk_rows<'a, B>(
        &'a self,
        f: impl FnMut(f32, &'a LaidoutRow) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let mut current_y_in_lpxs = 0.0;
        let mut f = f;
        for (row_index, row) in self.rows.iter().enumerate() {
            let line_gap_above_in_lpxs = if row_index == 0 {
                0.0
            } else {
                self.rows[row_index - 1].line_gap_in_lpxs
            };
            current_y_in_lpxs += line_gap_above_in_lpxs;
            current_y_in_lpxs += row.ascender_in_lpxs;
            f(current_y_in_lpxs, row)?;
            current_y_in_lpxs -= row.descender_in_lpxs;
        }
        ControlFlow::Continue(())
    }

    pub fn point_in_lpxs_to_cursor(&self, point_in_lpxs: Point<f32>) -> Cursor {
        let mut row_index = 0;
        let mut row_start = 0;
        self.walk_rows(|row_origin_y_in_lpxs, row| {
            let row_top_y_in_lpxs = row_origin_y_in_lpxs - row.ascender_in_lpxs;
            let row_bottom_y_in_lpxs = row_origin_y_in_lpxs - row.descender_in_lpxs;
            let line_gap_below_in_lpxs = if row_index == self.rows.len() - 1 {
                0.0
            } else {
                row.line_gap_in_lpxs
            };
            if row_index == 0 && point_in_lpxs.y < row_top_y_in_lpxs {
                return ControlFlow::Break(Cursor {
                    index: row_start,
                    affinity: Affinity::After,
                });
            }
            if point_in_lpxs.y < row_bottom_y_in_lpxs + line_gap_below_in_lpxs / 2.0 {
                let index = row.x_in_lpxs_to_index(point_in_lpxs.x);
                return ControlFlow::Break(Cursor {
                    index,
                    affinity: if index == row_start {
                        Affinity::After
                    } else {
                        Affinity::Before
                    }
                });
            }
            if row_index == self.rows.len() - 1 {
                return ControlFlow::Break(Cursor {
                    index: row_start + row.text.len(),
                    affinity: Affinity::Before,
                });
            }
            row_index += 1;
            row_start += row.text.len();
            ControlFlow::Continue(())
        }).break_value().unwrap()
    }

    pub fn cursor_to_point_in_lpxs(&self, cursor: Cursor) -> Point<f32> {
        let mut row_index = 0;
        let mut row_start = 0;
        self.walk_rows(|row_origin_y_in_lpxs, row| {
            if match cursor.affinity {
                Affinity::Before => cursor.index < row_start + row.text.len(),
                Affinity::After => cursor.index <= row_start + row.text.len(),
            } {
                return ControlFlow::Break(Point::new(
                    row.index_to_x_in_lpxs(cursor.index - row_start),
                    row_origin_y_in_lpxs
                ));
            }
            if row_index == self.rows.len() - 1 {
                return ControlFlow::Break(Point::new(
                    row.align_x_in_lpxs() + row.width_in_lpxs,
                    row_origin_y_in_lpxs
                ));
            }
            row_index += 1;
            row_start += row.text.len();
            ControlFlow::Continue(())
        }).break_value().unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub index: usize,
    pub affinity: Affinity,
}

#[derive(Clone, Copy, Debug)]
pub enum Affinity {
    Before,
    After,
}

#[derive(Clone, Debug)]
pub struct LaidoutRow {
    pub text: Substr,
    pub width_in_lpxs: f32,
    pub max_width_in_lpxs: f32,
    pub align: Align,
    pub ascender_in_lpxs: f32,
    pub descender_in_lpxs: f32,
    pub line_gap_in_lpxs: f32,
    pub glyphs: Vec<LaidoutGlyph>,
}

impl LaidoutRow {
    pub fn align_x_in_lpxs(&self) -> f32 {
        match self.align {
            Align::Left => 0.0,
            Align::Center => (self.max_width_in_lpxs - self.width_in_lpxs) / 2.0,
            Align::Right => self.max_width_in_lpxs - self.width_in_lpxs,
        }
    }

    pub fn walk_glyphs<'a, B>(
        &'a self,
        f: impl FnMut(f32, &'a LaidoutGlyph) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let mut current_x_in_lpxs = self.align_x_in_lpxs();
        let mut f = f;
        for glyph in &self.glyphs {
            f(current_x_in_lpxs, glyph)?;
            current_x_in_lpxs += glyph.advance_in_lpxs;
        }
        ControlFlow::Continue(())
    }

    pub fn x_in_lpxs_to_index(&self, x_in_lpxs: f32) -> usize {
        fn handle_glyph_group(
            text: &str,
            start: usize,
            start_x_in_lpxs: f32,
            end: usize,
            end_x_in_lpxs: f32,
            x_in_lpxs: f32,
        ) -> Option<usize> {
            use unicode_segmentation::UnicodeSegmentation;

            let width_in_lpxs = end_x_in_lpxs - start_x_in_lpxs;
            let grapheme_count = text[start..end].graphemes(true).count();
            let grapheme_width_in_lpxs = width_in_lpxs / grapheme_count as f32;
            let mut grapheme_start_x_in_lpxs = start_x_in_lpxs;
            for (grapheme_start, _) in text[start..end].grapheme_indices(true) {
                if x_in_lpxs < grapheme_start_x_in_lpxs + grapheme_width_in_lpxs / 2.0 {
                    return Some(start + grapheme_start);
                }
                grapheme_start_x_in_lpxs += grapheme_width_in_lpxs;
            }
            None
        }

        let mut glyph_group_start = 0;
        let mut glyph_group_start_x_in_lpxs = self.align_x_in_lpxs();
        match self.walk_glyphs(|glyph_origin_x_in_lpxs, glyph| {
            if glyph.cluster == glyph_group_start {
                return ControlFlow::Continue(());
            }
            let glyph_group_end = glyph.cluster;
            let glyph_group_end_x_in_lpxs = glyph_origin_x_in_lpxs;
            if let Some(index) = handle_glyph_group(
                &self.text,
                glyph_group_start,
                glyph_group_start_x_in_lpxs,
                glyph_group_end,
                glyph_group_end_x_in_lpxs,
                x_in_lpxs,
            ) {
                return ControlFlow::Break(index);
            }
            glyph_group_start = glyph_group_end;
            glyph_group_start_x_in_lpxs = glyph_group_end_x_in_lpxs;
            ControlFlow::Continue(())
        }) {
            ControlFlow::Continue(()) => {
                if let Some(index) = handle_glyph_group(
                    &self.text,
                    glyph_group_start,
                    glyph_group_start_x_in_lpxs,
                    self.text.len(),
                    self.align_x_in_lpxs() + self.width_in_lpxs,
                    x_in_lpxs,
                ) {
                    return index;
                }
                return self.text.len();
            }
            ControlFlow::Break(index) => index,
        }
    }

    pub fn index_to_x_in_lpxs(&self, index: usize) -> f32 {
        fn handle_glyph_group(
            text: &str,
            start: usize,
            start_x_in_lpxs: f32,
            end: usize,
            end_x_in_lpxs: f32,
            index: usize,
        ) -> Option<f32> {
            use unicode_segmentation::UnicodeSegmentation;

            let width_in_lpxs = end_x_in_lpxs - start_x_in_lpxs;
            let grapheme_count = text[start..end].graphemes(true).count();
            let grapheme_width_in_lpxs = width_in_lpxs / grapheme_count as f32;
            let mut grapheme_start_x_in_lpxs = start_x_in_lpxs;
            for (grapheme_start, _) in text[start..end].grapheme_indices(true) {
                if index == start + grapheme_start {
                    return Some(grapheme_start_x_in_lpxs);
                }
                grapheme_start_x_in_lpxs += grapheme_width_in_lpxs;
            }
            None
        }

        let mut glyph_group_start = 0;
        let mut glyph_group_start_x_in_lpxs = self.align_x_in_lpxs();
        match self.walk_glyphs(|glyph_origin_x_in_lpxs, glyph| {
            if glyph.cluster == glyph_group_start {
                return ControlFlow::Continue(());
            }
            let glyph_group_end = glyph.cluster;
            let glyph_group_end_x_in_lpxs = glyph_origin_x_in_lpxs;
            if let Some(x_in_lpxs) = handle_glyph_group(
                &self.text,
                glyph_group_start,
                glyph_group_start_x_in_lpxs,
                glyph_group_end,
                glyph_group_end_x_in_lpxs,
                index,
            ) {
                return ControlFlow::Break(x_in_lpxs);
            }
            glyph_group_start = glyph_group_end;
            glyph_group_start_x_in_lpxs = glyph_group_end_x_in_lpxs;
            ControlFlow::Continue(())
        }) {
            ControlFlow::Continue(()) => {
                if let Some(x_in_lpxs) = handle_glyph_group(
                    &self.text,
                    glyph_group_start,
                    glyph_group_start_x_in_lpxs,
                    self.text.len(),
                    self.align_x_in_lpxs() + self.width_in_lpxs,
                    index,
                ) {
                    return x_in_lpxs;
                }
                return self.width_in_lpxs;
            }
            ControlFlow::Break(index) => index,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LaidoutGlyph {
    pub font: Rc<Font>,
    pub font_size_in_lpxs: f32,
    pub baseline: Baseline,
    pub color: Color,
    pub id: GlyphId,
    pub cluster: usize,
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

    pub fn baseline_y_in_lpxs(&self) -> f32 {
        match self.baseline {
            Baseline::Alphabetic => 0.0,
            Baseline::Top => self.ascender_in_lpxs(),
            Baseline::Bottom => self.descender_in_lpxs(),
        }
    }

    pub fn rasterize(&self, dpx_per_em: f32) -> Option<RasterizedGlyph> {
        self.font.rasterize_glyph(self.id, dpx_per_em)
    }
}
