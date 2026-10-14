use akssora_terminal_config::{Config, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

#[allow(clippy::too_many_arguments)]
pub fn render_editor_tab(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    config: &Config,
    hovered_action: Option<SettingsAction>,
    line_number_dropdown_open: bool,
    workspace_x: f32,
    workspace_y: f32,
    content_title_x: f32,
    sec_y: f32,
    cell_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Editor Configuration",
        content_title_x,
        sec_y,
        300.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    let (tx, ty, tw, th) = SettingsView::tab_size_rect(workspace_x, workspace_y, cell_h);
    let bg_tab = if hovered_action == Some(SettingsAction::CycleTabSize) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        tx,
        ty,
        tx + tw,
        ty + th,
        4.0,
        circle_uv,
        solid_uv,
        bg_tab,
        screen_w,
        screen_h,
    );
    let tab_label = format!("Tab Size: {} spaces", config.tab_size);
    renderer.render_text(
        queue,
        fonts,
        verts,
        &tab_label,
        tx + 14.0,
        ty + (th - cell_h) / 2.0,
        tw - 20.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // INFO: Line Height setting button (configurable only from Settings)
    let (lh_x, lh_y, lh_w, lh_h) = SettingsView::line_height_rect(workspace_x, workspace_y, cell_h);
    let bg_lh = if hovered_action == Some(SettingsAction::CycleLineHeight) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        lh_x,
        lh_y,
        lh_x + lh_w,
        lh_y + lh_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_lh,
        screen_w,
        screen_h,
    );
    let lh_label = format!("Line Height: {:.2}", config.line_height);
    renderer.render_text(
        queue,
        fonts,
        verts,
        &lh_label,
        lh_x + 14.0,
        lh_y + (lh_h - cell_h) / 2.0,
        lh_w - 20.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Line Number Mode dropdown section
    let ln_sec_y = sec_y + cell_h * 2.0 + 20.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Line Numbers",
        content_title_x,
        ln_sec_y,
        300.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    let (lx, ly, lw, lh) =
        SettingsView::line_number_dropdown_rect(workspace_x, workspace_y, cell_h);
    let is_ln_hover = hovered_action == Some(SettingsAction::ToggleLineNumberDropdown);
    let bg_ln = if is_ln_hover || line_number_dropdown_open {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        lx,
        ly,
        lx + lw,
        ly + lh,
        4.0,
        circle_uv,
        solid_uv,
        bg_ln,
        screen_w,
        screen_h,
    );

    let mode_label = config.line_number_mode.label();
    renderer.render_text(
        queue,
        fonts,
        verts,
        mode_label,
        lx + 14.0,
        ly + (lh - cell_h) / 2.0,
        lw - 32.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Chevron caret (▾ collapsed / ▴ expanded)
    let caret = if line_number_dropdown_open {
        '\u{25B4}'
    } else {
        '\u{25BE}'
    };
    if let Some(info) = renderer.glyph(queue, fonts, caret, false, false) {
        let gx = lx + lw - info.width - 12.0;
        let gy = ly + (lh - info.height) / 2.0;
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

    // Vim & Editing Settings Toggles
    // Row 1: Vim Mode & Vim System Clipboard
    let (vx, vy, vw, vh) = SettingsView::vim_mode_rect(workspace_x, workspace_y, cell_h);
    let bg_vim = if hovered_action == Some(SettingsAction::ToggleVimMode) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        vx,
        vy,
        vx + vw,
        vy + vh,
        4.0,
        circle_uv,
        solid_uv,
        bg_vim,
        screen_w,
        screen_h,
    );
    let vim_str = format!("Vim Mode: {}", if config.vim_mode { "ON" } else { "OFF" });
    renderer.render_text(
        queue,
        fonts,
        verts,
        &vim_str,
        vx + 14.0,
        vy + (vh - cell_h) / 2.0,
        vw - 28.0,
        if config.vim_mode {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.vim_mode,
        screen_w,
        screen_h,
    );

    let (vc_x, vc_y, vc_w, vc_h) =
        SettingsView::vim_clipboard_rect(workspace_x, workspace_y, cell_h);
    let bg_vc = if hovered_action == Some(SettingsAction::ToggleVimClipboard) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        vc_x,
        vc_y,
        vc_x + vc_w,
        vc_y + vc_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_vc,
        screen_w,
        screen_h,
    );
    let vc_str = format!(
        "System Clipboard: {}",
        if config.vim_unnamed_system_clipboard {
            "ON"
        } else {
            "OFF"
        }
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        &vc_str,
        vc_x + 14.0,
        vc_y + (vc_h - cell_h) / 2.0,
        vc_w - 28.0,
        if config.vim_unnamed_system_clipboard {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.vim_unnamed_system_clipboard,
        screen_w,
        screen_h,
    );

    // Row 2: Vim Status Bar & Autocomplete Symbols
    let (vs_x, vs_y, vs_w, vs_h) =
        SettingsView::vim_status_bar_rect(workspace_x, workspace_y, cell_h);
    let bg_vs = if hovered_action == Some(SettingsAction::ToggleVimStatusBar) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        vs_x,
        vs_y,
        vs_x + vs_w,
        vs_y + vs_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_vs,
        screen_w,
        screen_h,
    );
    let vs_str = format!(
        "Vim Status Bar: {}",
        if config.vim_status_bar { "ON" } else { "OFF" }
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        &vs_str,
        vs_x + 14.0,
        vs_y + (vs_h - cell_h) / 2.0,
        vs_w - 28.0,
        if config.vim_status_bar {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.vim_status_bar,
        screen_w,
        screen_h,
    );

    let (ac_x, ac_y, ac_w, ac_h) =
        SettingsView::autocomplete_symbols_rect(workspace_x, workspace_y, cell_h);
    let bg_ac = if hovered_action == Some(SettingsAction::ToggleAutocompleteSymbols) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        ac_x,
        ac_y,
        ac_x + ac_w,
        ac_y + ac_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_ac,
        screen_w,
        screen_h,
    );
    let ac_str = format!(
        "Auto Brackets: {}",
        if config.autocomplete_symbols {
            "ON"
        } else {
            "OFF"
        }
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        &ac_str,
        ac_x + 14.0,
        ac_y + (ac_h - cell_h) / 2.0,
        ac_w - 28.0,
        if config.autocomplete_symbols {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.autocomplete_symbols,
        screen_w,
        screen_h,
    );

    // Row 3: Format on Save & Line Wrap
    let (fs_x, fs_y, fs_w, fs_h) =
        SettingsView::format_on_save_rect(workspace_x, workspace_y, cell_h);
    let bg_fs = if hovered_action == Some(SettingsAction::ToggleFormatOnSave) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        fs_x,
        fs_y,
        fs_x + fs_w,
        fs_y + fs_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_fs,
        screen_w,
        screen_h,
    );
    let fs_str = format!(
        "Format On Save: {}",
        if config.format_on_save { "ON" } else { "OFF" }
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        &fs_str,
        fs_x + 14.0,
        fs_y + (fs_h - cell_h) / 2.0,
        fs_w - 28.0,
        if config.format_on_save {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.format_on_save,
        screen_w,
        screen_h,
    );

    let (lw_x, lw_y, lw_w, lw_h) = SettingsView::line_wrap_rect(workspace_x, workspace_y, cell_h);
    let bg_lw = if hovered_action == Some(SettingsAction::ToggleLineWrap) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        lw_x,
        lw_y,
        lw_x + lw_w,
        lw_y + lw_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_lw,
        screen_w,
        screen_h,
    );
    let lw_str = format!("Line Wrap: {}", if config.line_wrap { "ON" } else { "OFF" });
    renderer.render_text(
        queue,
        fonts,
        verts,
        &lw_str,
        lw_x + 14.0,
        lw_y + (lw_h - cell_h) / 2.0,
        lw_w - 28.0,
        if config.line_wrap {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.line_wrap,
        screen_w,
        screen_h,
    );

    // Row 4: Auto Save Checkbox
    let (as_x, as_y, as_w, as_h) = SettingsView::auto_save_rect(workspace_x, workspace_y, cell_h);
    let bg_as = if hovered_action == Some(SettingsAction::ToggleAutoSave) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        as_x,
        as_y,
        as_x + as_w,
        as_y + as_h,
        4.0,
        circle_uv,
        solid_uv,
        bg_as,
        screen_w,
        screen_h,
    );
    let check_size = 14.0;
    let check_x = as_x + 14.0;
    let check_y = as_y + (as_h - check_size) / 2.0;
    push_rounded_rect(
        verts,
        check_x,
        check_y,
        check_x + check_size,
        check_y + check_size,
        3.0,
        circle_uv,
        solid_uv,
        if config.auto_save {
            theme.accent
        } else {
            theme.code_review_header_divider
        },
        screen_w,
        screen_h,
    );
    if config.auto_save {
        renderer.render_text(
            queue,
            fonts,
            verts,
            "\u{2713}",
            check_x + 2.0,
            check_y + (check_size - cell_h) / 2.0,
            check_size,
            theme.header_background,
            true,
            screen_w,
            screen_h,
        );
    }
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Auto Save Files",
        as_x + 36.0,
        as_y + (as_h - cell_h) / 2.0,
        as_w - 44.0,
        if config.auto_save {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.auto_save,
        screen_w,
        screen_h,
    );
}
