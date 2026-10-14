pub mod block;
pub mod button;
pub mod code_review;
pub mod context_menu;
pub mod drop_zone;
pub mod editor_tabs;
pub mod explorer_ui;
pub mod header;
pub mod primitives;
pub mod quick_search;
pub mod scrollbar;
pub mod settings;
pub mod sidebar;
pub mod sidebar_session;
pub mod split;
pub mod tab;
pub mod tooltip;

pub use code_review::{
    CodeReviewAction, CodeReviewGutterHover, CodeReviewLayout, CodeReviewModel,
    CodeReviewSidebarMode, CodeReviewTreeItem, DiffViewMode, LineActionPopupGeometry,
    ReviewComment,
};

pub use context_menu::{ContextMenu, ContextMenuAction, ContextMenuItem};
pub use editor_tabs::{
    EditorFileTab, EditorTabBar, EditorTabTarget, EditorToolbarAction, file_type_icon,
};
pub use explorer_ui::{
    ExplorerContextAction, ExplorerContextMenu, ExplorerTarget, ExplorerToolbarAction, ExplorerUi,
    InlineInputMode, InlineInputState,
};
pub use quick_search::{QuickSearchResult, QuickSearchUi, fuzzy_match};

pub use block::Block;
pub use button::ReusableButton;
pub use drop_zone::DropZone;
pub use header::{
    BUTTON_WIDTH, EXPLORER_TOGGLE_WIDTH, Header, HeaderButton, HeaderItem, SIDEBAR_TOGGLE_WIDTH,
    TAB_BAR_HEIGHT, TAB_HEIGHT, TAB_MAX_WIDTH, TAB_MIN_WIDTH,
};
pub use primitives::{ChromeRect, ChromeText};
pub use scrollbar::{Scrollbar, ScrollbarOrientation};
pub use settings::{SettingsAction, SettingsTab, SettingsView};
pub use sidebar::{Sidebar, SidebarRow, SidebarRowKind, SidebarTarget};
pub use sidebar_session::{
    PROJECT_HEADER_HEIGHT, SESSION_ITEM_HEIGHT, SessionRowData, compute_project_rows,
    hit_test_project_rows,
};
pub use split::{PaneTree, SplitDirection};
pub use tab::{Tab, TabKind, TabManager};
pub use tooltip::Tooltip;
