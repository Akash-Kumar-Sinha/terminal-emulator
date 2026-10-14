use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::vertex::GlyphVertex;

// INFO: Renders code line with basic keyword highlighting.
#[allow(clippy::too_many_arguments)]
pub fn render_highlighted_code_line(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    line: &str,
    x: f32,
    y: f32,
    max_w: f32,
    theme: &Theme,
    screen_w: f32,
    screen_h: f32,
) {
    let cell_w = fonts.cell_w;
    let trimmed = line.trim_start();
    let indent_w = (line.len() - trimmed.len()) as f32 * cell_w;

    // Comment line
    if trimmed.starts_with("//") || trimmed.starts_with('#') {
        renderer.render_text(
            queue,
            fonts,
            verts,
            line,
            x,
            y,
            max_w,
            theme.text_muted,
            false,
            screen_w,
            screen_h,
        );
        return;
    }

    // Tokenize words and highlight keywords
    let mut cur_x = x + indent_w;
    let words = trimmed.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_');

    let keywords = [
        "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "match", "if", "else",
        "return", "for", "in", "while", "const", "use", "mod", "type", "async", "await", "def",
        "class", "import", "from", "function", "var", "export",
    ];

    for word in words {
        let token = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
        let suffix = &word[token.len()..];
        let is_keyword = keywords.contains(&token);

        let col = if is_keyword {
            theme.accent
        } else {
            theme.foreground
        };

        if !token.is_empty() {
            let tw = token.len() as f32 * cell_w;
            renderer.render_text(
                queue,
                fonts,
                verts,
                token,
                cur_x,
                y,
                (x + max_w - cur_x).max(0.0),
                col,
                is_keyword,
                screen_w,
                screen_h,
            );
            cur_x += tw;
        }

        if !suffix.is_empty() {
            let sw = suffix.len() as f32 * cell_w;
            renderer.render_text(
                queue,
                fonts,
                verts,
                suffix,
                cur_x,
                y,
                (x + max_w - cur_x).max(0.0),
                theme.foreground,
                false,
                screen_w,
                screen_h,
            );
            cur_x += sw;
        }
    }
}
