use std::collections::HashMap;

use akssora_terminal_config::{TabLayoutPreference, Theme};
use akssora_terminal_ui::{Block, Header, PaneTree, Sidebar, TabManager, Tooltip};

use crate::font::FontManager;
use crate::glyph_atlas::GlyphAtlas;
pub use crate::glyph_atlas::GlyphInfo;
use crate::glyph_pipeline::create_glyph_pipeline;
use crate::quad::push_quad;
use crate::render_block::{render_drop_indicator, render_terminal_blocks};
use crate::render_confirm_exit::render_confirm_exit_dialog;
use crate::render_header::render_header_bar;
use crate::render_settings::render_settings_panel;
use crate::render_sidebar::render_sidebar_panel;
use crate::render_tooltip::render_tooltip;
use crate::vertex::GlyphVertex;

// INFO: GPU text and 2D quad rendering pipeline coordinating atlas caching and frame draw passes.
pub struct GlyphRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    vertex_count: u32,
    atlas: GlyphAtlas,
}

impl GlyphRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let atlas = GlyphAtlas::new(device, queue);
        let pipeline = create_glyph_pipeline(device, format, &atlas.bind_group_layout);

        let vertex_capacity = 6 * 4096;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph vertices"),
            size: (vertex_capacity * std::mem::size_of::<GlyphVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
            vertex_count: 0,
            atlas,
        }
    }

    #[inline]
    pub fn solid_uv(&self) -> [f32; 2] {
        self.atlas.solid_uv
    }

    #[inline]
    pub fn circle_uv(&self) -> [[f32; 2]; 2] {
        self.atlas.circle_uv
    }

    pub fn glyph(
        &mut self,
        queue: &wgpu::Queue,
        fonts: &FontManager,
        ch: char,
        bold: bool,
        italic: bool,
    ) -> Option<GlyphInfo> {
        self.atlas.glyph(queue, fonts, ch, bold, italic)
    }

    pub fn clear_cache(&mut self) {
        self.atlas.clear_cache();
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &mut self,
        queue: &wgpu::Queue,
        fonts: &FontManager,
        verts: &mut Vec<GlyphVertex>,
        text: &str,
        mut start_x: f32,
        start_y: f32,
        max_w: f32,
        col: [f32; 4],
        bold: bool,
        screen_w: f32,
        screen_h: f32,
    ) {
        let end_x = start_x + max_w;
        for ch in text.chars() {
            if start_x + fonts.cell_w > end_x {
                break;
            }
            if ch != ' '
                && let Some(info) = self.glyph(queue, fonts, ch, bold, false)
            {
                let gx = start_x + info.bearing_left;
                let gy = start_y + fonts.ascent_px + info.bearing_top;
                push_quad(
                    verts,
                    gx,
                    gy,
                    gx + info.width,
                    gy + info.height,
                    info.uv_min,
                    info.uv_max,
                    col,
                    screen_w,
                    screen_h,
                );
            }
            start_x += fonts.cell_w;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        header: &Header,
        blocks: &[&Block],
        fonts: &FontManager,
        ui_fonts: &FontManager,
        theme: &Theme,
        config: &akssora_terminal_config::Config,
        active_settings_tab: akssora_terminal_ui::SettingsTab,
        settings_search_query: &str,
        conflicts: &[akssora_terminal_config::ConflictWarning],
        screen_w: f32,
        screen_h: f32,
        sidebar: &Sidebar,
        tab_manager: &TabManager,
        tab_layout: TabLayoutPreference,
        split_trees: &HashMap<u64, PaneTree>,
        active_pane_id: Option<u64>,
        active_tooltip: Option<&Tooltip>,
        hovered_setting_action: Option<akssora_terminal_ui::SettingsAction>,
        layout_dropdown_open: bool,
        line_number_dropdown_open: bool,
        confirm_exit: bool,
        drop_indicator: Option<(f32, f32, f32, f32)>,
        sessions: Option<&[akssora_terminal_ui::SessionRowData]>,
        context_menu: Option<&akssora_terminal_ui::ContextMenu>,
        explorer: Option<&akssora_terminal_ui::ExplorerUi>,
        editor_documents: &std::collections::HashMap<std::path::PathBuf, akssora_editor::Document>,
        active_editor: Option<&akssora_editor::Document>,
        file_tabs: &[akssora_terminal_ui::EditorFileTab],
        hovered_file_tab: Option<usize>,
        hovered_file_close: Option<usize>,
        hovered_file_toolbar: Option<akssora_terminal_ui::EditorToolbarAction>,
        quick_search: Option<&akssora_terminal_ui::QuickSearchUi>,
        editor_scrollbars: Option<(
            &akssora_terminal_ui::Scrollbar,
            &akssora_terminal_ui::Scrollbar,
        )>,
        hovered_editor_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
        unsaved_changes_prompt: Option<&str>,
        code_review: Option<&akssora_terminal_ui::CodeReviewModel>,
        hovered_code_review_action: Option<&akssora_terminal_ui::CodeReviewAction>,
    ) {
        let mut verts: Vec<GlyphVertex> = Vec::new();
        let show_sidebar = tab_layout.show_sidebar();
        let sb_w = sidebar.effective_width(show_sidebar);
        let exp_w = explorer.map(|e| e.effective_width()).unwrap_or(0.0);
        let top_h = header.total_height(tab_layout.includes_title_bar());
        let solid_uv = self.atlas.solid_uv;
        let circle_uv = self.atlas.circle_uv;

        let ws_x = sb_w + exp_w;
        let ws_y = top_h;
        let ws_w = (screen_w - ws_x).max(0.0);
        let ws_h = (screen_h - top_h).max(0.0);

        let is_settings = tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Settings))
            .unwrap_or(false);

        let is_code_review = tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);

        if is_settings {
            render_settings_panel(
                self,
                queue,
                ui_fonts,
                fonts.size(),
                theme,
                config,
                active_settings_tab,
                settings_search_query,
                conflicts,
                tab_layout,
                hovered_setting_action,
                layout_dropdown_open,
                line_number_dropdown_open,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        } else if is_code_review && let Some(cr) = code_review {
            crate::render_code_review_panel(
                self,
                queue,
                fonts,
                ui_fonts,
                theme,
                cr,
                editor_documents,
                active_editor,
                hovered_code_review_action,
                hovered_editor_gutter,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        } else if let Some(doc) = active_editor {
            crate::render_editor_view(
                self,
                queue,
                fonts,
                ui_fonts,
                theme,
                doc,
                file_tabs,
                hovered_file_tab,
                hovered_file_close,
                hovered_file_toolbar,
                editor_scrollbars,
                hovered_editor_gutter,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        } else {
            // 1. Terminal buffer blocks and pill scrollbar (strictly using terminal font sizing)
            render_terminal_blocks(
                self,
                queue,
                fonts,
                theme,
                blocks,
                active_pane_id,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );

            // 1.1 Render drop indicator highlight when repositioning panes via drag-and-drop
            if let Some(drop_rect) = drop_indicator {
                render_drop_indicator(&mut verts, drop_rect, theme, solid_uv, screen_w, screen_h);
            }
        }

        // 2. Sidebar panel when sidebar layout is active and expanded (using UI font)
        if show_sidebar && sidebar.is_expanded {
            render_sidebar_panel(
                self,
                queue,
                ui_fonts,
                theme,
                sidebar,
                tab_manager,
                split_trees,
                active_pane_id,
                sessions,
                top_h,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        // INFO: File Explorer panel when expanded
        if let Some(exp) = explorer
            && exp.is_expanded
            && exp.width > 0.0
        {
            crate::render_file_explorer_panel(
                self, queue, ui_fonts, theme, exp, sb_w, top_h, &mut verts, solid_uv, circle_uv,
                screen_w, screen_h,
            );
        }

        // 3. Application header and (if TitleBar layout) terminal tab bar (untouchable by font scaling)
        render_header_bar(
            self,
            queue,
            ui_fonts,
            theme,
            header,
            tab_manager,
            tab_layout,
            split_trees,
            code_review,
            &mut verts,
            solid_uv,
            circle_uv,
            screen_w,
            screen_h,
        );

        // 3.1 Terminal floating context menu overlay
        if let Some(cm) = context_menu {
            crate::render_context_menu::render_context_menu(
                self, queue, ui_fonts, theme, cm, &mut verts, circle_uv, solid_uv, screen_w,
                screen_h,
            );
        }

        // INFO: 3.2 Command-palette quick file search overlay
        if let Some(qs) = quick_search {
            crate::render_quick_search::render_quick_search_overlay(
                self,
                queue,
                ui_fonts,
                theme,
                qs,
                &mut verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
                header.height,
            );
        }

        // 3.3 Top-level Dropdown Overlays (highest Z-index above all panels, sidebar, explorer, and header)
        if header.branch_dropdown_open && header.git_branches.len() > 1 {
            crate::render_code_review_button::render_branch_dropdown_list(
                self,
                queue,
                ui_fonts,
                &mut verts,
                theme,
                header,
                show_sidebar,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        if is_settings {
            if layout_dropdown_open
                && active_settings_tab == akssora_terminal_ui::SettingsTab::General
            {
                crate::render_settings_layout::render_tab_layout_dropdown_list(
                    self,
                    queue,
                    ui_fonts,
                    theme,
                    tab_layout,
                    hovered_setting_action,
                    ws_x,
                    ws_y,
                    ws_w,
                    &mut verts,
                    solid_uv,
                    screen_w,
                    screen_h,
                );
            }
            if line_number_dropdown_open
                && active_settings_tab == akssora_terminal_ui::SettingsTab::Editor
            {
                crate::render_settings::render_line_number_dropdown_list(
                    self,
                    queue,
                    ui_fonts,
                    theme,
                    config.line_number_mode,
                    hovered_setting_action,
                    ws_x,
                    ws_y,
                    &mut verts,
                    solid_uv,
                    circle_uv,
                    screen_w,
                    screen_h,
                );
            }
        }

        // 4. Floating tooltip overlay
        if let Some(tt) = active_tooltip {
            render_tooltip(
                self,
                queue,
                ui_fonts,
                theme,
                header,
                sidebar,
                tab_manager,
                tt,
                &mut verts,
                solid_uv,
                screen_w,
                screen_h,
            );
        }

        // 5. Exit confirmation prompt modal dialog
        if confirm_exit {
            render_confirm_exit_dialog(
                self, queue, ui_fonts, theme, screen_w, screen_h, &mut verts, solid_uv,
            );
        }

        // 6. Unsaved changes confirmation prompt modal dialog
        if let Some(file_title) = unsaved_changes_prompt {
            crate::render_unsaved_changes_dialog(
                self, queue, ui_fonts, theme, screen_w, screen_h, file_title, &mut verts, solid_uv,
            );
        }

        self.vertex_count = verts.len() as u32;
        if verts.is_empty() {
            return;
        }

        if verts.len() > self.vertex_capacity {
            let new_cap = verts.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("glyph vertices"),
                size: (new_cap * std::mem::size_of::<GlyphVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_capacity = new_cap;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.atlas.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
