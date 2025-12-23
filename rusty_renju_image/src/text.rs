// LLM-generated

use crate::TextBitmap;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;

pub(crate) fn raster_text(font: &Font, text: &str, size: f32) -> TextBitmap {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0,
        y: 0.0,
        ..LayoutSettings::default()
    });
    layout.append(&[font.clone()], &TextStyle::new(text, size, 0));

    let glyphs = layout.glyphs();
    if glyphs.is_empty() {
        return TextBitmap { width: 0, height: 0, baseline: 0.0, data: Vec::new() };
    }

    let (min_x, min_y, max_x, max_y) = glyphs.iter().fold(
        (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        |(min_x, min_y, max_x, max_y), g| {
            (
                min_x.min(g.x),
                min_y.min(g.y),
                max_x.max(g.x + g.width as f32),
                max_y.max(g.y + g.height as f32),
            )
        },
    );

    let width = (max_x - min_x).ceil().max(0.0) as u32;
    let height = (max_y - min_y).ceil().max(0.0) as u32;
    let mut data = vec![0u8; (width * height) as usize];

    let baseline = layout
        .lines()
        .and_then(|lines| lines.first().map(|line| line.baseline_y - min_y))
        .unwrap_or(0.0);

    for glyph in glyphs {
        let (metrics, bitmap) = font.rasterize_indexed(glyph.key.glyph_index, glyph.key.px);
        if metrics.width == 0 || metrics.height == 0 {
            continue;
        }
        let base_x = (glyph.x - min_x).round() as i32;
        let base_y = (glyph.y - min_y).round() as i32;

        for (y, row) in bitmap.chunks(metrics.width).enumerate() {
            let py = base_y + y as i32;
            if py < 0 || py as u32 >= height {
                continue;
            }

            for (x, &alpha) in row.iter().enumerate() {
                let px = base_x + x as i32;
                if px < 0 || px as u32 >= width {
                    continue;
                }

                let idx = (py as u32 * width + px as u32) as usize;
                if alpha > data[idx] {
                    data[idx] = alpha;
                }
            }
        }
    }

    TextBitmap { width, height, baseline, data }
}
