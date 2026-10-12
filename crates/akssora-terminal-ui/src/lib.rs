pub mod block;
pub mod button;
pub mod context_menu;
pub mod drop_zone;
pub mod header;
pub mod primitives;
pub mod scrollbar;
pub mod settings_ui;
pub mod sidebar;
pub mod sidebar_session;
pub mod split;
pub mod tab;
pub mod tooltip;

pub use context_menu::{ContextMenu, ContextMenuAction, ContextMenuItem};

pub use block::Block;
pub use button::ReusableButton;
pub use drop_zone::DropZone;
pub use header::{
    BUTTON_WIDTH, Header, HeaderButton, HeaderItem, SIDEBAR_TOGGLE_WIDTH, TAB_BAR_HEIGHT,
    TAB_HEIGHT, TAB_MAX_WIDTH, TAB_MIN_WIDTH,
};
pub use primitives::{ChromeRect, ChromeText};
pub use scrollbar::Scrollbar;
pub use settings_ui::{SettingsAction, SettingsTab, SettingsView};
pub use sidebar::{Sidebar, SidebarRow, SidebarRowKind, SidebarTarget};
pub use sidebar_session::{
    PROJECT_HEADER_HEIGHT, SESSION_ITEM_HEIGHT, SessionRowData, compute_project_rows,
    hit_test_project_rows,
};
pub use split::{PaneTree, SplitDirection};
pub use tab::{Tab, TabKind, TabManager};
pub use tooltip::Tooltip;
