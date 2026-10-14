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
    // INFO: Tooltips use a compact, smaller font size scaled proportionally with UI font size.
    let tooltip_size = (fonts.size() * (11.0 / 14.0)).max(9.0);
    let mut tooltip_fonts = fonts.clone();
    tooltip_fonts.set_size(tooltip_size);
    let cell_w = tooltip_fonts.cell_w;
    let cell_h = tooltip_fonts.cell_h;

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

    // INFO: Tooltip design: 4px rounded container with 1px subtle border
    crate::quad::push_rounded_rect(
        verts,
        tx,
        ty,
        tx + tw,
        ty + th,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.tooltip_border,
        screen_w,
        screen_h,
    );
    crate::quad::push_rounded_rect(
        verts,
        tx + 1.0,
        ty + 1.0,
        tx + tw - 1.0,
        ty + th - 1.0,
        3.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.tooltip_background,
        screen_w,
        screen_h,
    );

    let text_x = tx + 8.0;
    let text_y = ty + (th - cell_h) / 2.0;

    if let Some(sc) = &tooltip.shortcut
        && !sc.trim().is_empty()
    {
        // Dual-tone tooltip: main action label in primary color, shortcut in muted secondary color
        let label = format!("{} ", tooltip.text);
        let label_w = label.len() as f32 * cell_w;
        renderer.render_text(
            queue,
            &tooltip_fonts,
            verts,
            &label,
            text_x,
            text_y,
            label_w,
            theme.tooltip_foreground,
            false,
            screen_w,
            screen_h,
        );

        let sc_text = format!("({})", sc);
        let sc_w = sc_text.len() as f32 * cell_w;
        renderer.render_text(
            queue,
            &tooltip_fonts,
            verts,
            &sc_text,
            text_x + label_w,
            text_y,
            sc_w,
            theme.tooltip_shortcut_foreground,
            false,
            screen_w,
            screen_h,
        );
    } else {
        renderer.render_text(
            queue,
            &tooltip_fonts,
            verts,
            &tooltip.text,
            text_x,
            text_y,
            tw - 16.0,
            theme.tooltip_foreground,
            false,
            screen_w,
            screen_h,
        );
    }
}
