use std::sync::Arc;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::Vector2F;

pub const FONT_SIZE: f32 = 14.0;
pub const LINE_HEIGHT_RATIO: f32 = 1.2;

pub struct RasterGlyph {
    pub width: u32,
    pub height: u32,
    pub coverage: Vec<u8>,
    pub bearing_left: f32,
    pub bearing_top: f32,
}

struct Faces {
    regular: Font,
    bold: Font,
    italic: Font,
    bold_italic: Font,
}

impl Faces {
    fn pick(&self, bold: bool, italic: bool) -> &Font {
        match (bold, italic) {
            (false, false) => &self.regular,
            (true, false) => &self.bold,
            (false, true) => &self.italic,
            (true, true) => &self.bold_italic,
        }
    }
}

pub struct FontManager {
    faces: Faces,
    fallbacks: Vec<Font>,
    size: f32,
    line_height_ratio: f32,

    pub cell_w: f32,
    pub cell_h: f32,
    pub ascent_px: f32,
}

fn load_face(bytes: &'static [u8]) -> Option<Font> {
    Font::from_bytes(Arc::new(bytes.to_vec()), 0).ok()
}

impl FontManager {
    pub fn new() -> Self {
        let regular = load_face(include_bytes!(
            "assets/Roboto_Mono/static/RobotoMono-Regular.ttf"
        ))
        .or_else(system_mono)
        .expect("no monospace font available (bundled or system)");

        let bold = load_face(include_bytes!(
            "assets/Roboto_Mono/static/RobotoMono-Bold.ttf"
        ))
        .unwrap_or_else(|| regular.clone());
        let italic = load_face(include_bytes!(
            "assets/Roboto_Mono/static/RobotoMono-Italic.ttf"
        ))
        .unwrap_or_else(|| regular.clone());
        let bold_italic = load_face(include_bytes!(
            "assets/Roboto_Mono/static/RobotoMono-BoldItalic.ttf"
        ))
        .unwrap_or_else(|| bold.clone());

        let faces = Faces {
            regular,
            bold,
            italic,
            bold_italic,
        };
        let fallbacks = load_system_fallbacks();

        let mut fm = Self {
            faces,
            fallbacks,
            size: FONT_SIZE,
            line_height_ratio: LINE_HEIGHT_RATIO,
            cell_w: FONT_SIZE * 0.6,
            cell_h: FONT_SIZE * LINE_HEIGHT_RATIO,
            ascent_px: FONT_SIZE,
        };
        fm.recompute_metrics();
        fm
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.max(4.0);
        self.recompute_metrics();
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    fn recompute_metrics(&mut self) {
        let px = self.size;
        let m = self.faces.regular.metrics();
        let upem = m.units_per_em as f32;

        let natural = (m.ascent - m.descent + m.line_gap) / upem * px;
        let cell_h = natural * self.line_height_ratio;
        let leading = (cell_h - natural) * 0.5;
        let ascent_px = leading + m.ascent / upem * px;

        let cell_w = self
            .faces
            .regular
            .glyph_for_char('M')
            .and_then(|gid| self.faces.regular.advance(gid).ok())
            .map(|adv| adv.x() / upem * px)
            .unwrap_or(px * 0.6);

        self.cell_w = cell_w;
        self.cell_h = cell_h;
        self.ascent_px = ascent_px;
    }

    pub fn rasterize(&self, ch: char, bold: bool, italic: bool) -> Option<RasterGlyph> {
        let styled = self.faces.pick(bold, italic);
        let (font, glyph_id) = if let Some(gid) = styled.glyph_for_char(ch) {
            (styled, gid)
        } else {
            let mut found = None;
            for f in &self.fallbacks {
                if let Some(gid) = f.glyph_for_char(ch) {
                    found = Some((f, gid));
                    break;
                }
            }
            found?
        };

        let px = self.size;
        let bounds = font
            .raster_bounds(
                glyph_id,
                px,
                Transform2F::default(),
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )
            .ok()?;

        let width = bounds.width();
        let height = bounds.height();
        if width <= 0 || height <= 0 {
            return None;
        }

        let mut canvas = Canvas::new(bounds.size(), Format::A8);
        let origin = Vector2F::new(-bounds.origin_x() as f32, -bounds.origin_y() as f32);
        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            px,
            Transform2F::from_translation(origin),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        )
        .ok()?;

        let w = width as usize;
        let h = height as usize;
        let mut coverage = Vec::with_capacity(w * h);
        for row in 0..h {
            let start = row * canvas.stride;
            coverage.extend_from_slice(&canvas.pixels[start..start + w]);
        }

        Some(RasterGlyph {
            width: width as u32,
            height: height as u32,
            coverage,
            bearing_left: bounds.origin_x() as f32,
            bearing_top: bounds.origin_y() as f32,
        })
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

fn system_mono() -> Option<Font> {
    SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .ok()
        .and_then(|h| h.load().ok())
}

fn load_system_fallbacks() -> Vec<Font> {
    let src = SystemSource::new();
    [FamilyName::Monospace, FamilyName::SansSerif]
        .iter()
        .filter_map(|fam| {
            src.select_best_match(&[fam.clone()], &Properties::new())
                .ok()
        })
        .filter_map(|h| h.load().ok())
        .collect()
}
