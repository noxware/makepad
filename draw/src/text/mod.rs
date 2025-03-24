pub mod builtins;
pub mod color;
pub mod font;
pub mod font_atlas;
pub mod font_face;
pub mod font_family;
pub mod loader;
pub mod fonts;
pub mod geom;
pub mod glyph_outline;
pub mod glyph_raster_image;
pub mod rasterizer;
pub mod image;
pub mod intern;
pub mod layouter;
pub mod num;
pub mod sdfer;
pub mod selection;
pub mod shaper;
pub mod slice;
pub mod substr;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        use {
            super::{
                color::Color,
                layouter::{LayoutOptions, LayoutParams, Layouter, Settings, Span, Style},
            },
            std::{fs::File, io::BufWriter},
        };

        let mut layouter = Layouter::new(Settings::default());
        let text = "The quick brown fox jumps over the lazy dog繁😊😔";
        let text = layouter.get_or_layout(LayoutParams {
            text: text.into(),
            spans: [
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: 16.0,
                        color: Some(Color::RED),
                    },
                    len: 10,
                },
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: 16.0,
                        color: Some(Color::GREEN),
                    },
                    len: 10,
                },
                Span {
                    style: Style {
                        font_family_id: "Sans".into(),
                        font_size_in_lpxs: 16.0,
                        color: Some(Color::BLUE),
                    },
                    len: text.len() - 20,
                },
            ]
            .into(),
            options: LayoutOptions {
                max_width_in_lpxs: Some(256.0),
                ..LayoutOptions::default()
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
        let rasterizer = layouter.rasterizer().borrow();
        let size = rasterizer.grayscale_atlas_size();
        let mut encoder = png::Encoder::new(
            writer,
            size.width as u32,
            size.height as u32,
        );
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let pixels = rasterizer.grayscale_atlas_image().as_pixels();
        let data =
            unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len()) };
        writer.write_image_data(&data).unwrap();

        let file = File::create("/Users/ejpbruel/Desktop/color.png").unwrap();
        let writer = BufWriter::new(file);
        let size = rasterizer.color_atlas_size();
        let mut encoder = png::Encoder::new(
            writer,
            size.width as u32,
            size.height as u32,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let pixels = rasterizer.color_atlas_image().as_pixels();
        let data =
            unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * 4) };
        writer.write_image_data(&data).unwrap();
    }
}
