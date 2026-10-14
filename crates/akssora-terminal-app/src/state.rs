use akssora_terminal_config::Theme;
use akssora_terminal_renderer::{FontManager, GlyphRenderer};
use akssora_terminal_ui::{Block, Header};

// INFO: Encapsulates WGPU device, command queue, swapchain surface, and pass submission.
pub struct WgpuState {
    surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    glyphs: GlyphRenderer,
}

impl WgpuState {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        glyphs: GlyphRenderer,
    ) -> Self {
        Self {
            surface,
            adapter,
            device,
            queue,
            config,
            glyphs,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn clear_glyph_cache(&mut self) {
        self.glyphs.clear_cache();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        fonts: &FontManager,
        ui_fonts: &FontManager,
        theme: &Theme,
        config: &akssora_terminal_config::Config,
        active_settings_tab: akssora_terminal_ui::SettingsTab,
        settings_search_query: &str,
        conflicts: &[akssora_terminal_config::ConflictWarning],
        header: &Header,
        blocks: &[&Block],
        sidebar: &akssora_terminal_ui::Sidebar,
        tab_manager: &akssora_terminal_ui::TabManager,
        tab_layout: akssora_terminal_config::TabLayoutPreference,
        split_trees: &std::collections::HashMap<u64, akssora_terminal_ui::PaneTree>,
        active_pane_id: Option<u64>,
        active_tooltip: Option<&akssora_terminal_ui::Tooltip>,
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
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("surface lost");
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.glyphs.prepare(
            &self.device,
            &self.queue,
            header,
            blocks,
            fonts,
            ui_fonts,
            theme,
            config,
            active_settings_tab,
            settings_search_query,
            conflicts,
            self.config.width as f32,
            self.config.height as f32,
            sidebar,
            tab_manager,
            tab_layout,
            split_trees,
            active_pane_id,
            active_tooltip,
            hovered_setting_action,
            layout_dropdown_open,
            line_number_dropdown_open,
            confirm_exit,
            drop_indicator,
            sessions,
            context_menu,
            explorer,
            editor_documents,
            active_editor,
            file_tabs,
            hovered_file_tab,
            hovered_file_close,
            hovered_file_toolbar,
            quick_search,
            editor_scrollbars,
            hovered_editor_gutter,
            unsaved_changes_prompt,
            code_review,
            hovered_code_review_action,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let (cr, cg, cb, ca) = theme.clear_color_linear();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("akssora main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: cr,
                            g: cg,
                            b: cb,
                            a: ca,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.glyphs.draw(&mut pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
