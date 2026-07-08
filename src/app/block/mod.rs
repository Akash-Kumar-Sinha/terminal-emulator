use crate::renderer::glyph::{ChromeRect, ChromeText};
use crate::terminal::Terminal;
use crate::theme;

pub struct Block {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32,
    pub bg_color: [f32; 4],
    pub border_top: bool,
    pub border_bottom: bool,
    pub border_thickness: f32,
    pub border_color: [f32; 4],
    pub text_color: [f32; 4],
    pub text: Option<String>,
    pub shell: Terminal,
}

impl Block {
    pub fn new(
        width: Option<u32>,
        height: Option<u32>,
        x: Option<f32>,
        y: Option<f32>,
        bg_color: Option<[f32; 4]>,
        border_top: Option<bool>,
        border_bottom: Option<bool>,
        border_thickness: Option<f32>,
        border_color: Option<[f32; 4]>,
        text_color: Option<[f32; 4]>,
        text: Option<String>,
        shell: Terminal,
    ) -> Self {
        Block {
            id: rand::random(),
            width: width.unwrap_or(100),
            height: height.unwrap_or(100),
            x: x.unwrap_or(0.0),
            y: y.unwrap_or(0.0),
            bg_color: bg_color.unwrap_or(theme::Theme::default().background),
            border_top: border_top.unwrap_or(false),
            border_bottom: border_bottom.unwrap_or(true),
            border_thickness: border_thickness.unwrap_or(1.0),
            border_color: border_color.unwrap_or(theme::Theme::default().border),
            text_color: text_color.unwrap_or(theme::Theme::default().foreground),
            text: text.or_else(|| Some(String::from("Default Text"))),
            shell,
        }
    }

    pub fn text_run(&self, x: f32, y: f32) -> Option<ChromeText> {
        self.text.as_ref().map(|t| ChromeText {
            x,
            y,
            text: t.clone(),
            color: self.text_color,
            bold: false,
        })
    }

    pub fn rect(&self, x: f32, y: f32) -> ChromeRect {
        ChromeRect {
            x,
            y,
            w: self.width as f32,
            h: self.height as f32,
            color: self.bg_color,
        }
    }
}
