use std::collections::HashMap;
use std::sync::Arc;

use akssora_terminal_config::{Config, FontConfig, Keybindings, TabLayoutPreference, Theme};
use akssora_terminal_renderer::FontManager;
use akssora_terminal_ui::{Block, Header, PaneTree, SettingsAction, Sidebar, TabManager, Tooltip};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use crate::state::WgpuState;

mod editor;
mod explorer;
mod input;
mod keyboard;
mod mouse;
mod mouse_click;
mod mouse_drag;
mod mouse_hover;
mod mouse_selection;
mod mouse_wheel;
pub mod review;
mod session_ops;
pub(crate) mod session_persist;
pub(crate) mod session_tracker;
mod split;
mod split_state;
mod tabs;
pub mod tooltips;
mod window;

pub use split_state::{PaneDragState, SplitManager};

#[derive(Debug, Clone)]
pub enum UnsavedCloseAction {
    CloseSingle(usize),
    CloseOthers(std::path::PathBuf),
    CloseAll,
}

// INFO: Multi-tab window handler coordinating event dispatch, terminal state updates, and rendering loop.
pub struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: Option<WgpuState>,
    pub(crate) fonts: Option<FontManager>,
    pub(crate) ui_fonts: Option<FontManager>,
    pub(crate) theme: Theme,
    pub config: Config,
    pub(crate) modifiers: ModifiersState,
    pub(crate) blocks: HashMap<u64, Block>,
    pub tab_manager: TabManager,
    pub split_trees: HashMap<u64, PaneTree>,
    pub split_manager: SplitManager,
    pub sidebar: Sidebar,
    pub header: Header,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) active_tooltip: Option<Tooltip>,
    pub(crate) command_buffers: HashMap<u64, String>,
    pub(crate) last_executed_commands: HashMap<u64, String>,
    pub(crate) pane_custom_names: HashMap<u64, String>,
    pub(crate) hovered_setting_action: Option<SettingsAction>,
    pub(crate) settings_layout_dropdown_open: bool,
    pub(crate) settings_line_number_dropdown_open: bool,
    pub(crate) active_settings_tab: akssora_terminal_ui::SettingsTab,
    pub(crate) settings_search_query: String,
    pub prompt_confirm_exit: bool,
    pub session_file: Option<std::path::PathBuf>,
    pub context_menu: Option<akssora_terminal_ui::ContextMenu>,
    pub explorer: akssora_terminal_ui::ExplorerUi,
    pub quick_search: akssora_terminal_ui::QuickSearchUi,
    pub editors: HashMap<u64, akssora_editor::Document>,
    pub editor_open_files: Vec<std::path::PathBuf>,
    pub editor_active_file: Option<std::path::PathBuf>,
    pub editor_documents: HashMap<std::path::PathBuf, akssora_editor::Document>,
    pub editor_preview_file: Option<std::path::PathBuf>,
    pub editor_pinned_files: std::collections::HashSet<std::path::PathBuf>,
    pub editor_hovered_tab: Option<usize>,
    pub editor_hovered_close: Option<usize>,
    pub editor_hovered_toolbar: Option<akssora_terminal_ui::EditorToolbarAction>,
    pub editor_hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    pub editor_dragging_tab: Option<usize>,
    pub editor_v_scrollbar: akssora_terminal_ui::Scrollbar,
    pub editor_h_scrollbar: akssora_terminal_ui::Scrollbar,
    pub unsaved_prompt_target: Option<UnsavedCloseAction>,
    pub pending_chord: Option<(String, std::time::Instant)>,
    pub explorer_has_custom_root: bool,
    pub code_review: akssora_terminal_ui::CodeReviewModel,
    pub hovered_code_review_action: Option<akssora_terminal_ui::CodeReviewAction>,
    pub diff_worker: crate::app::review::DiffBackgroundWorker,
    pub(crate) is_selecting_text: bool,
    pub(crate) selection_start_pane: Option<u64>,
    pub(crate) selection_start_pt: Option<akssora_terminal_screen::TerminalPoint>,
    pub(crate) mouse_press_pos: Option<(f32, f32)>,
    pub(crate) mouse_press_pane: Option<u64>,
    pub(crate) last_layout_time: Option<std::time::Instant>,
    pub(crate) last_render_time: Option<std::time::Instant>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        let mut header = Header::new("Akssora");
        let mut sidebar = Sidebar::new();
        let initial_width = config
            .sidebar_width
            .clamp(Sidebar::MIN_WIDTH, Sidebar::MAX_WIDTH);
        sidebar.saved_width = initial_width;
        sidebar.is_expanded = config.tab_layout.show_sidebar();
        sidebar.width = if sidebar.is_expanded {
            initial_width
        } else {
            0.0
        };
        let default_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let repo_root_opt = akssora_git::GitInspector::repo_root(&default_cwd);
        header.is_git_repo = repo_root_opt.is_some();
        if let Some(ref root) = repo_root_opt {
            header.git_branch = akssora_git::GitInspector::current_branch(root);
            header.git_branches = akssora_git::GitInspector::list_branches(root);
        }
        let mut explorer = akssora_terminal_ui::ExplorerUi::new(&default_cwd);
        explorer.git_status = akssora_git::GitInspector::status(&default_cwd);
        header.is_explorer_expanded = explorer.is_expanded;
        let code_review = akssora_terminal_ui::CodeReviewModel::new(&default_cwd);
        header.git_additions = code_review.diff.total_additions;
        header.git_deletions = code_review.diff.total_deletions;
        Self {
            window: None,
            state: None,
            fonts: None,
            ui_fonts: None,
            theme: config.theme.clone(),
            config,
            modifiers: ModifiersState::default(),
            blocks: HashMap::new(),
            tab_manager: TabManager::new(),
            split_trees: HashMap::new(),
            split_manager: SplitManager::new(),
            sidebar,
            header,
            cursor_pos: (0.0, 0.0),
            active_tooltip: None,
            command_buffers: HashMap::new(),
            last_executed_commands: HashMap::new(),
            pane_custom_names: HashMap::new(),
            hovered_setting_action: None,
            settings_layout_dropdown_open: false,
            settings_line_number_dropdown_open: false,
            active_settings_tab: akssora_terminal_ui::SettingsTab::General,
            settings_search_query: String::new(),
            prompt_confirm_exit: false,
            session_file: None,
            context_menu: None,
            explorer,
            quick_search: akssora_terminal_ui::QuickSearchUi::new(),
            editors: HashMap::new(),
            editor_open_files: Vec::new(),
            editor_active_file: None,
            editor_documents: HashMap::new(),
            editor_preview_file: None,
            editor_pinned_files: std::collections::HashSet::new(),
            editor_hovered_tab: None,
            editor_hovered_close: None,
            editor_hovered_toolbar: None,
            editor_hovered_gutter: None,
            editor_dragging_tab: None,
            editor_v_scrollbar: akssora_terminal_ui::Scrollbar::new(),
            editor_h_scrollbar: akssora_terminal_ui::Scrollbar::new_horizontal(),
            unsaved_prompt_target: None,
            pending_chord: None,
            explorer_has_custom_root: false,
            code_review,
            hovered_code_review_action: None,
            diff_worker: crate::app::review::DiffBackgroundWorker::new(),
            is_selecting_text: false,
            selection_start_pane: None,
            selection_start_pt: None,
            mouse_press_pos: None,
            mouse_press_pane: None,
            last_layout_time: None,
            last_render_time: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tab_layout(&self) -> TabLayoutPreference {
        self.config.tab_layout
    }

    pub fn set_tab_layout(&mut self, layout: TabLayoutPreference) {
        self.config.tab_layout = layout;
        let _ = self.config.save();
        self.sidebar.is_expanded = layout.show_sidebar();
        self.sidebar.width = if sidebar_width_needed(&self.sidebar) {
            if self.sidebar.saved_width >= Sidebar::MIN_WIDTH {
                self.sidebar.saved_width
            } else {
                Sidebar::DEFAULT_WIDTH
            }
        } else {
            0.0
        };
        self.header.is_sidebar_expanded = self.sidebar.is_expanded;
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn cycle_tab_layout(&mut self) {
        let next = self.config.tab_layout.next();
        self.set_tab_layout(next);
    }

    pub fn set_font_size(&mut self, size: f32) {
        let new_size = size.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
        self.config.font.size = new_size;
        let _ = self.config.save();
        if let Some(fonts) = self.fonts.as_mut() {
            fonts.set_size(new_size);
        }
        // INFO: ui_fonts drives the header, titlebar, and sidebar — keep it independently controlled and untouched by terminal zoom.
        if let Some(state) = self.state.as_mut() {
            state.clear_glyph_cache();
        }
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn increase_font_size(&mut self) {
        let curr = self.config.font.size;
        self.set_font_size(curr + FontConfig::STEP);
    }

    pub fn decrease_font_size(&mut self) {
        let curr = self.config.font.size;
        self.set_font_size(curr - FontConfig::STEP);
    }

    pub fn reset_font_size(&mut self) {
        self.set_font_size(FontConfig::DEFAULT_SIZE);
    }

    pub fn set_ui_font_size(&mut self, size: f32) {
        let new_size = size.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
        self.config.ui_font_size = new_size;
        let _ = self.config.save();
        if let Some(ui_fonts) = self.ui_fonts.as_mut() {
            ui_fonts.set_size(new_size);
        }
        if let Some(state) = self.state.as_mut() {
            state.clear_glyph_cache();
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn increase_ui_font_size(&mut self) {
        let curr = self.current_ui_font_size();
        self.set_ui_font_size(curr + FontConfig::STEP);
    }

    pub fn decrease_ui_font_size(&mut self) {
        let curr = self.current_ui_font_size();
        self.set_ui_font_size(curr - FontConfig::STEP);
    }

    pub fn reset_ui_font_size(&mut self) {
        self.set_ui_font_size(14.0);
    }

    pub fn set_test_fonts(&mut self, fonts: FontManager, ui_fonts: FontManager) {
        self.config.font.size = fonts.size();
        self.config.ui_font_size = ui_fonts.size();
        self.fonts = Some(fonts);
        self.ui_fonts = Some(ui_fonts);
    }

    pub fn terminal_font_size(&self) -> f32 {
        self.fonts
            .as_ref()
            .map(|f| f.size())
            .unwrap_or(self.config.font.size)
    }

    pub fn current_ui_font_size(&self) -> f32 {
        self.ui_fonts
            .as_ref()
            .map(|f| f.size())
            .unwrap_or(self.config.ui_font_size)
    }

    pub fn ui_cell_h(&self) -> f32 {
        self.ui_fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0)
    }

    pub fn toggle_sidebar(&mut self) {
        if self.config.tab_layout.includes_sidebar() {
            self.sidebar.toggle();
            self.header.is_sidebar_expanded = self.sidebar.is_expanded;
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    pub fn active_pane_id(&self) -> Option<u64> {
        let active_tab_id = self.tab_manager.active_id()?;
        if let Some(focused_pane) = self.split_manager.active_pane_id
            && let Some(tree) = self.split_trees.get(&active_tab_id)
            && tree.contains_leaf(focused_pane)
        {
            return Some(focused_pane);
        }
        Some(active_tab_id)
    }

    pub fn keybindings(&self) -> &Keybindings {
        &self.config.keybindings
    }

    pub fn set_test_cursor_pos(&mut self, x: f32, y: f32) {
        self.cursor_pos = (x, y);
    }

    pub fn set_test_modifiers(&mut self, mods: ModifiersState) {
        self.modifiers = mods;
    }

    pub fn handle_test_mouse_press(&mut self) {
        let (mx, my) = self.cursor_pos;
        self.focus_pane_at(mx, my);

        if let Some((pane_id, _, _, _, _)) = self.pane_at(mx, my) {
            self.mouse_press_pos = Some((mx, my));
            self.mouse_press_pane = Some(pane_id);
            self.selection_start_pane = Some(pane_id);

            let (cell_w, cell_h) = self
                .fonts
                .as_ref()
                .map(|f| (f.cell_w, f.cell_h))
                .unwrap_or((9.0, 18.0));

            if let Some(block) = self.blocks.get(&pane_id)
                && let Some((row, col)) =
                    crate::app::mouse_selection::pixel_to_grid_cell(block, mx, my, cell_w, cell_h)
            {
                self.selection_start_pt =
                    Some(akssora_terminal_screen::TerminalPoint::new(row, col));
            }

            let is_pane_drag_gesture = self.modifiers.control_key()
                || self
                    .blocks
                    .get(&pane_id)
                    .map(|b| my <= b.y + 24.0)
                    .unwrap_or(false);

            if is_pane_drag_gesture
                && let Some(active_tab_id) = self.tab_manager.active_id()
                && let Some(tree) = self.split_trees.get(&active_tab_id)
                && tree.all_leaf_ids().len() > 1
            {
                self.split_manager.pane_drag = Some(crate::app::split_state::PaneDragState {
                    source_pane_id: pane_id,
                    tab_id: active_tab_id,
                    start_x: mx,
                    start_y: my,
                    current_x: mx,
                    current_y: my,
                    hover_target: None,
                });
            }
        }
    }

    pub fn handle_test_mouse_release(&mut self) {
        self.split_manager.drag_state = None;
        if let Some(pane_drag) = self.split_manager.pane_drag.take() {
            let drag_dist = (pane_drag.current_x - pane_drag.start_x)
                .hypot(pane_drag.current_y - pane_drag.start_y);
            if drag_dist >= 10.0
                && let Some((target_id, zone, _)) = pane_drag.hover_target
            {
                self.reposition_pane(pane_drag.source_pane_id, target_id, zone);
            }
        }
        self.is_selecting_text = false;
        self.mouse_press_pos = None;
        self.mouse_press_pane = None;
        self.selection_start_pane = None;
        self.selection_start_pt = None;
    }
}

fn sidebar_width_needed(sidebar: &Sidebar) -> bool {
    sidebar.is_expanded
}
