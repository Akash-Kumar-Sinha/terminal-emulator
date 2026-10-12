use akssora_terminal_config::{TabLayoutPreference, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders the "Tab Layout" label + collapsed dropdown trigger row only.
// The open dropdown list is rendered separately (last) so it always sits on top.
#[allow(clippy::too_many_arguments)]
pub fn render_tab_layout_trigger(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    current_layout: TabLayoutPreference,
    hovered_action: Option<SettingsAction>,
    layout_dropdown_open: bool,
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    title_x: f32,
    sec_y: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;

    // ── Section heading ───────────────────────────────────────────────────────
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Tab Layout",
        title_x,
        sec_y,
        300.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // ── Dropdown trigger row ──────────────────────────────────────────────────
    let (ddx, ddy, ddw, ddh) =
        SettingsView::layout_dropdown_rect(workspace_x, workspace_y, workspace_w, cell_h);

    let trigger_bg =
        if hovered_action == Some(SettingsAction::ToggleLayoutDropdown) || layout_dropdown_open {
            theme.session_hover_background
        } else {
            theme.session_background
        };

    crate::quad::push_rounded_rect(
        verts,
        ddx,
        ddy,
        ddx + ddw,
        ddy + ddh,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        trigger_bg,
        screen_w,
        screen_h,
    );

    let label_y = ddy + (ddh - cell_h) / 2.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        current_layout.label(),
        ddx + 12.0,
        label_y,
        ddw - 40.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Chevron caret (▾ collapsed / ▴ expanded)
    let caret = if layout_dropdown_open {
        '\u{25B4}'
    } else {
        '\u{25BE}'
    };
    if let Some(info) = renderer.glyph(queue, fonts, caret, false, false) {
        let gx = ddx + ddw - info.width - 12.0;
        let gy = ddy + (ddh - info.height) / 2.0;
        push_quad(
            verts,
            gx,
            gy,
            gx + info.width,
            gy + info.height,
            info.uv_min,
            info.uv_max,
            theme.text_muted,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Renders the open dropdown list panel on top of all other settings content.
// Call this LAST in the settings render pass so it covers underlying sections.
#[allow(clippy::too_many_arguments)]
pub fn render_tab_layout_dropdown_list(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    current_layout: TabLayoutPreference,
    hovered_action: Option<SettingsAction>,
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;

    let (ddx, ddy, ddw, ddh) =
        SettingsView::layout_dropdown_rect(workspace_x, workspace_y, workspace_w, cell_h);

    let options = [
        (
            TabLayoutPreference::Sidebar,
            "Sidebar",
            "Vertical collapsible sidebar",
        ),
        (
            TabLayoutPreference::TitleBar,
            "Title Bar",
            "Horizontal tab bar in header",
        ),
    ];

    let list_h = SettingsView::dropdown_item_height(cell_h) * options.len() as f32;
    let list_y = ddy + ddh;

    // Solid opaque panel background with rounded corners (borderless)
    crate::quad::push_rounded_rect(
        verts,
        ddx,
        list_y,
        ddx + ddw,
        list_y + list_h,
        6.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );

    for (i, (opt, label, desc)) in options.into_iter().enumerate() {
        let (ix, iy, iw, ih) = SettingsView::layout_dropdown_item_rect(
            i,
            workspace_x,
            workspace_y,
            workspace_w,
            cell_h,
        );

        let is_selected = current_layout == opt;
        let is_hovered = hovered_action == Some(SettingsAction::SelectLayout(opt));

        let item_bg = if is_selected {
            theme.session_selected_background
        } else if is_hovered {
            theme.session_hover_background
        } else {
            theme.sidebar_background
        };

        push_quad(
            verts,
            ix,
            iy,
            ix + iw,
            iy + ih,
            solid_uv,
            solid_uv,
            item_bg,
            screen_w,
            screen_h,
        );

        // Item label — vertically centred
        let text_y = iy + (ih - cell_h) / 2.0;
        renderer.render_text(
            queue,
            fonts,
            verts,
            label,
            ix + 14.0,
            text_y,
            iw - 28.0,
            if is_selected {
                theme.text_primary
            } else {
                theme.text_secondary
            },
            is_selected,
            screen_w,
            screen_h,
        );

        // Tiny description line — only if it fits
        let sub_y = text_y + cell_h + 2.0;
        if sub_y + cell_h * 0.8 < iy + ih {
            renderer.render_text(
                queue,
                fonts,
                verts,
                desc,
                ix + 14.0,
                sub_y,
                iw - 28.0,
                theme.text_muted,
                false,
                screen_w,
                screen_h,
            );
        }
    }
}
