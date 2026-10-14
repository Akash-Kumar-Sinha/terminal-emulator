use akssora_editor::{Document, MarkdownViewMode};
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{EditorFileTab, EditorTabBar, EditorToolbarAction, Scrollbar};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

pub mod markdown;
pub mod source;
pub mod status_bar;
pub mod tabs;

pub use markdown::render_markdown_preview;
pub use source::render_source_editor_view;
pub use status_bar::render_editor_status_bar;
pub use tabs::render_editor_tabs;

pub const GUTTER_WIDTH: f32 = 48.0;
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

// INFO: Renders the integrated code editor with fixed file-tab bar, Markdown toolbar, line numbers, cursor, selection, Vim status bar, and pill scrollbars.
#[allow(clippy::too_many_arguments)]
pub fn render_editor_view(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    doc: &Document,
    file_tabs: &[EditorFileTab],
    hovered_tab: Option<usize>,
    hovered_close: Option<usize>,
    hovered_toolbar: Option<EditorToolbarAction>,
    editor_scrollbars: Option<(&Scrollbar, &Scrollbar)>,
    hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    ws_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if ws_w <= 0.0 || ws_h <= 0.0 {
        return;
    }

    // 1. Editor Canvas Background (full editor panel)
    push_quad(
        verts,
        ws_x,
        ws_y,
        ws_x + ws_w,
        ws_y + ws_h,
        solid_uv,
        solid_uv,
        theme.background,
        screen_w,
        screen_h,
    );

    let tab_bar_h = if !file_tabs.is_empty() {
        EditorTabBar::HEIGHT
    } else {
        0.0
    };

    // 2. Fixed top file-tab bar directly above file content area
    render_editor_tabs(
        renderer,
        queue,
        ui_fonts,
        theme,
        doc,
        file_tabs,
        hovered_tab,
        hovered_close,
        hovered_toolbar,
        ws_x,
        ws_y,
        ws_w,
        tab_bar_h,
        verts,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    let content_y = ws_y + tab_bar_h;
    let editor_content_h = (ws_h - tab_bar_h - STATUS_BAR_HEIGHT).max(0.0);

    // 3. Render based on Markdown view mode
    match doc.markdown_mode {
        MarkdownViewMode::Source => {
            render_source_editor_view(
                renderer,
                queue,
                fonts,
                ui_fonts,
                theme,
                doc,
                editor_scrollbars,
                hovered_gutter,
                ws_x,
                content_y,
                ws_w,
                editor_content_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }
        MarkdownViewMode::Preview => {
            render_markdown_preview(
                renderer,
                queue,
                fonts,
                ui_fonts,
                theme,
                doc,
                ws_x,
                content_y,
                ws_w,
                editor_content_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }
        MarkdownViewMode::Split => {
            let left_w = ((ws_w - 1.0) * 0.5).floor().max(50.0);
            let right_x = ws_x + left_w + 1.0;
            let right_w = (ws_w - left_w - 1.0).max(50.0);

            // Left pane: editable source editor
            render_source_editor_view(
                renderer,
                queue,
                fonts,
                ui_fonts,
                theme,
                doc,
                editor_scrollbars,
                hovered_gutter,
                ws_x,
                content_y,
                left_w,
                editor_content_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );

            // Vertical divider between editor and markdown preview
            push_quad(
                verts,
                ws_x + left_w,
                content_y,
                ws_x + left_w + 1.0,
                content_y + editor_content_h,
                solid_uv,
                solid_uv,
                theme.border_subtle,
                screen_w,
                screen_h,
            );

            // Right pane: live rendered Markdown preview
            render_markdown_preview(
                renderer,
                queue,
                fonts,
                ui_fonts,
                theme,
                doc,
                right_x,
                content_y,
                right_w,
                editor_content_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }
    }

    // 4. Bottom Status Bar
    render_editor_status_bar(
        renderer,
        queue,
        fonts,
        theme,
        doc,
        ws_x,
        ws_y,
        ws_w,
        ws_h,
        STATUS_BAR_HEIGHT,
        verts,
        solid_uv,
        screen_w,
        screen_h,
    );
}
