use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};

use crate::buffer::TextBuffer;
use crate::markdown::{MarkdownViewMode, ParsedMarkdown};
use crate::search::SearchModel;
use crate::vim::VimEngine;
use crate::wrap::WrapLayout;

use akssora_terminal_config::LineNumberMode;

pub mod cursor;
pub mod edit;
pub mod git;
pub mod markdown;
pub mod selection;
pub mod smart_pairs;
pub mod vim_ops;

pub use git::GutterActionKind;

// INFO: Editor document model encapsulating file metadata, buffer content, cursor, selection, Vim engine, and undo state.
#[derive(Clone, Debug)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub buffer: TextBuffer,
    pub cursor: (usize, usize), // (row, col)
    pub selection_anchor: Option<(usize, usize)>,
    pub dirty: bool,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub tab_size: usize,
    pub line_height: f32,
    pub line_numbers: bool,
    pub line_number_mode: LineNumberMode,
    pub line_wrap: bool,
    pub vim_enabled: bool,
    pub vim: VimEngine,
    pub search: SearchModel,
    pub autocomplete_symbols: bool,
    pub markdown_mode: MarkdownViewMode,
    pub preview_scroll_row: usize,
    pub preview_scroll_col: usize,
    pub inline_diff: bool,
    pub is_deleted: bool,
    pub git_original_content: Option<String>,
    pub(crate) cached_git_statuses: RefCell<Option<Vec<akssora_git::LineGitStatus>>>,
    pub(crate) cached_git_hash: Cell<u64>,
    pub(crate) cached_max_line_len: Cell<Option<usize>>,
    pub(crate) cached_wrap_layout: RefCell<Option<WrapLayout>>,
    pub(crate) cached_parsed_markdown: RefCell<Option<ParsedMarkdown>>,
    pub(crate) cached_markdown_hash: Cell<u64>,
    pub(crate) undo_stack: Vec<(TextBuffer, (usize, usize))>,
    pub(crate) redo_stack: Vec<(TextBuffer, (usize, usize))>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            path: None,
            buffer: TextBuffer::new(),
            cursor: (0, 0),
            selection_anchor: None,
            dirty: false,
            scroll_row: 0,
            scroll_col: 0,
            tab_size: 4,
            line_height: 1.2,
            line_numbers: true,
            line_number_mode: LineNumberMode::Absolute,
            line_wrap: true,
            vim_enabled: true,
            vim: VimEngine::new(),
            search: SearchModel::default(),
            autocomplete_symbols: true,
            markdown_mode: MarkdownViewMode::Source,
            preview_scroll_row: 0,
            preview_scroll_col: 0,
            inline_diff: false,
            is_deleted: false,
            git_original_content: None,
            cached_git_statuses: RefCell::new(None),
            cached_git_hash: Cell::new(0),
            cached_max_line_len: Cell::new(None),
            cached_wrap_layout: RefCell::new(None),
            cached_parsed_markdown: RefCell::new(None),
            cached_markdown_hash: Cell::new(0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(path: PathBuf, content: &str) -> Self {
        Self {
            path: Some(path),
            buffer: TextBuffer::from_text(content),
            cursor: (0, 0),
            selection_anchor: None,
            dirty: false,
            scroll_row: 0,
            scroll_col: 0,
            tab_size: 4,
            line_height: 1.2,
            line_numbers: true,
            line_number_mode: LineNumberMode::Absolute,
            line_wrap: true,
            vim_enabled: true,
            vim: VimEngine::new(),
            search: SearchModel::default(),
            autocomplete_symbols: true,
            markdown_mode: MarkdownViewMode::Source,
            preview_scroll_row: 0,
            preview_scroll_col: 0,
            inline_diff: false,
            is_deleted: false,
            git_original_content: None,
            cached_git_statuses: RefCell::new(None),
            cached_git_hash: Cell::new(0),
            cached_max_line_len: Cell::new(None),
            cached_wrap_layout: RefCell::new(None),
            cached_parsed_markdown: RefCell::new(None),
            cached_markdown_hash: Cell::new(0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn open_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let content = if p.exists() {
            fs::read_to_string(&p)?
        } else {
            String::new()
        };
        let mut doc = Self::open(p.clone(), &content);
        if let Some(repo_root) = akssora_git::GitInspector::repo_root(&p) {
            doc.git_original_content =
                akssora_git::GitInspector::file_original_content(&repo_root, &p);
        }
        Ok(doc)
    }

    pub fn title(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn display_title(&self) -> String {
        let t = self.title();
        if self.dirty { format!("● {t}") } else { t }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            let content = self.buffer.to_string_content();
            fs::write(path, content)?;
            self.dirty = false;
            self.vim.status_message = Some(format!("\"{}\" written", self.title()));
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No target file path set for document",
            ))
        }
    }

    pub fn save_to(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let p = path.as_ref().to_path_buf();
        self.path = Some(p);
        self.save()
    }

    // INFO: Positions the cursor and anchors selection based on click coordinates relative to editor content bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_content_click(
        &mut self,
        click_x: f32,
        click_y: f32,
        content_x: f32,
        content_y: f32,
        content_w: f32,
        cell_w: f32,
        cell_h: f32,
    ) {
        let gutter_w = if self.line_numbers && self.line_number_mode != LineNumberMode::Off {
            48.0
        } else {
            0.0
        };
        let effective_line_h = (cell_h * self.line_height).max(cell_h);
        let text_x = content_x + gutter_w + 8.0;
        let vis_row =
            self.scroll_row + ((click_y - content_y).max(0.0) / effective_line_h) as usize;
        let is_in_gutter = click_x < text_x;

        if self.line_wrap {
            let wrap_w = if let Some(layout) = self.cached_wrap_layout.borrow().as_ref() {
                layout.wrap_width
            } else {
                let max_text_w = (content_w - gutter_w - 16.0 - 8.0).max(0.0);
                (max_text_w / cell_w).floor().max(1.0) as usize
            };
            let (actual_row, actual_col) = if is_in_gutter {
                let (r, _) = self.visual_to_logical(vis_row, 0, wrap_w);
                (r, 0)
            } else {
                let vis_col = ((click_x - text_x).max(0.0) / cell_w).round() as usize;
                self.visual_to_logical(vis_row, vis_col, wrap_w)
            };
            self.cursor = (actual_row, actual_col);
        } else {
            let max_row = self.buffer.line_count().saturating_sub(1);
            let actual_row = vis_row.min(max_row);
            let line_len = self.buffer.line_len(actual_row);
            let actual_col = if is_in_gutter {
                0
            } else {
                let vis_col = ((click_x - text_x).max(0.0) / cell_w).round() as usize;
                (self.scroll_col + vis_col).min(line_len)
            };
            self.cursor = (actual_row, actual_col);
        }

        self.selection_anchor = Some(self.cursor);
    }
}
