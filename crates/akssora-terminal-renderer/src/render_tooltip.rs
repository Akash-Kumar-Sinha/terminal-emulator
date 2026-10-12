use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderItem, Sidebar, TabManager, Tooltip};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::vertex::GlyphVertex;

// INFO: Renders context-sensitive hover tooltips with zinc container and blue border accents.
#[allow(clippy::too_many_arguments)]
pub fn render_tooltip(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    header: &Header,
    sidebar: &Sidebar,
    tab_manager: &TabManager,
    tooltip: &Tooltip,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_w = fonts.cell_w;
    let cell_h = fonts.cell_h;

    let (tx, ty, tw, th) = if tooltip.target_rect.2 > 0.0 && tooltip.target_rect.3 > 0.0 {
        tooltip.bounds(cell_w, cell_h, screen_w, screen_h)
    } else {
        match header.hovered_item {
            Some(HeaderItem::SidebarToggle) => {
                let (bx, by, bw, bh) = header.sidebar_toggle_rect();
                tooltip.bounds_for_target(bx, by, bw, bh, cell_w, cell_h, screen_w)
            }
            Some(HeaderItem::NewTab) => {
                let (bx, by, bw, bh) = header.new_tab_rect(tab_manager.len(), screen_w);
                tooltip.bounds_for_target(bx, by, bw, bh, cell_w, cell_h, screen_w)
            }
            Some(HeaderItem::Settings) => {
                let (bx, by, bw, bh) = header.settings_button_rect(screen_w);
                tooltip.bounds_for_target(bx, by, bw, bh, cell_w, cell_h, screen_w)
            }
            _ => {
                if sidebar.hovered_toggle {
                    let (bx, by, bw, bh) = sidebar.toggle_button_rect();
                    tooltip.bounds_for_target(bx, by, bw, bh, cell_w, cell_h, screen_w)
                } else if sidebar.hovered_new_tab {
                    let (bx, by, bw, bh) =
                        sidebar.new_tab_button_rect(tab_manager.len(), header.height + 8.0);
                    tooltip.bounds_for_target(bx, by, bw, bh, cell_w, cell_h, screen_w)
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                }
            }
        }
    };

    if tw <= 0.0 || th <= 0.0 {
        return;
    }

    // Borderless tooltip with subtle dark background and rounded corners
    crate::quad::push_rounded_rect(
        verts,
        tx,
        ty,
        tx + tw,
        ty + th,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.tooltip_background,
        screen_w,
        screen_h,
    );

    let label = tooltip.display_label();
    let text_x = tx + 8.0;
    let text_y = ty + (th - cell_h) / 2.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &label,
        text_x,
        text_y,
        tw,
        theme.tooltip_foreground,
        false,
        screen_w,
        screen_h,
    );
}
