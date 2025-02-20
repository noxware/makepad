pub mod color;
pub mod font;
pub mod font_atlas;
pub mod font_data;
pub mod font_face;
pub mod font_family;
pub mod font_loader;
pub mod fonts;
pub mod geom;
pub mod glyph_outline;
pub mod glyph_raster_image;
pub mod image;
pub mod layout;
pub mod non_nan;
pub mod num;
pub mod sdfer;
pub mod shape;
pub mod slice;
pub mod substr;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        use {
            super::{
                color::Color,
                font_loader::FontDefinitions,
                layout::{Align, Baseline, LayoutOptions, LayoutParams, Layouter, Settings, Span, Style},
                non_nan::NonNanF32,
            },
            std::{fs::File, io::BufWriter},
        };

        let mut layouter = Layouter::new(FontDefinitions::default(), Settings::default());
        let text = "The quick brown fox jumps over the lazy dog繁😊😔";
        let text = layouter.get_or_layout(LayoutParams {
            text: text.into(),
            spans: [
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: NonNanF32::new(16.0).unwrap(),
                        baseline: Baseline::Alphabetic,
                        color: Color::RED,
                    },
                    range: 0..10,
                },
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: NonNanF32::new(16.0).unwrap(),
                        baseline: Baseline::Alphabetic,
                        color: Color::GREEN,
                    },
                    range: 10..20,
                },
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: NonNanF32::new(16.0).unwrap(),
                        baseline: Baseline::Alphabetic,
                        color: Color::BLUE,
                    },
                    range: 20..text.len(),
                },
            ]
            .into(),
            options: LayoutOptions {
                max_width_in_lpxs: NonNanF32::new(256.0).unwrap(),
                align: Align::Left,
            },
        });
        for row in &text.rows {
            for glyph in &row.glyphs {
                glyph.font.rasterize_glyph(glyph.id, 64.0);
                println!("{:?}", glyph.cluster);
            }
        }

        let file = File::create("/Users/ejpbruel/Desktop/grayscale.png").unwrap();
        let writer = BufWriter::new(file);
        let atlas = layouter.grayscale_atlas().borrow();
        let mut encoder = png::Encoder::new(
            writer,
            atlas.size().width as u32,
            atlas.size().height as u32,
        );
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let pixels = atlas.image().as_pixels();
        let data =
            unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len()) };
        writer.write_image_data(&data).unwrap();

        let file = File::create("/Users/ejpbruel/Desktop/color.png").unwrap();
        let writer = BufWriter::new(file);
        let atlas = layouter.color_atlas().borrow();
        let mut encoder = png::Encoder::new(
            writer,
            atlas.size().width as u32,
            atlas.size().height as u32,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let pixels = atlas.image().as_pixels();
        let data =
            unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * 4) };
        writer.write_image_data(&data).unwrap();
    }
}
