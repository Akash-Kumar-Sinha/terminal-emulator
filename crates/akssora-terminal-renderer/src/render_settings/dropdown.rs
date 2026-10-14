use akssora_terminal_config::Theme;
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders the open dropdown list for line numbers on top of other content.
#[allow(clippy::too_many_arguments)]
pub fn render_line_number_dropdown_list(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    current_mode: akssora_terminal_config::LineNumberMode,
    hovered_action: Option<SettingsAction>,
    workspace_x: f32,
    workspace_y: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let (lx, ly, lw, lh) =
        SettingsView::line_number_dropdown_rect(workspace_x, workspace_y, cell_h);
    let options = akssora_terminal_config::LineNumberMode::ALL;
    let list_h = SettingsView::dropdown_item_height(cell_h) * options.len() as f32;
    let list_y = ly + lh;

    // Solid opaque panel background with rounded corners (borderless)
    push_rounded_rect(
        verts,
        lx,
        list_y,
        lx + lw,
        list_y + list_h,
        6.0,
        circle_uv,
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );

    for (idx, &mode) in options.iter().enumerate() {
        let (ix, iy, iw, ih) =
            SettingsView::line_number_dropdown_item_rect(idx, workspace_x, workspace_y, cell_h);
        let is_selected = mode == current_mode;
        let is_item_hover = hovered_action == Some(SettingsAction::SelectLineNumberMode(mode));

        let item_bg = if is_item_hover {
            theme.session_hover_background
        } else if is_selected {
            theme.session_selected_background
        } else {
            theme.sidebar_background
        };

        push_rounded_rect(
            verts,
            ix + 2.0,
            iy + 1.0,
            ix + iw - 2.0,
            iy + ih - 1.0,
            4.0,
            circle_uv,
            solid_uv,
            item_bg,
            screen_w,
            screen_h,
        );

        let text_y = iy + (ih - cell_h) / 2.0;
        renderer.render_text(
            queue,
            fonts,
            verts,
            mode.label(),
            ix + 14.0,
            text_y,
            iw - 20.0,
            if is_selected {
                theme.text_primary
            } else {
                theme.text_secondary
            },
            is_selected,
            screen_w,
            screen_h,
        );
    }
}
