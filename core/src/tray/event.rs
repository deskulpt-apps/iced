use crate::mouse;

use super::menu;

/// A tray event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The tray icon was clicked.
    IconClicked {
        /// The mouse button that was used.
        button: mouse::Button,
        /// The mouse click kind.
        kind: mouse::click::Kind,
    },

    /// A tray menu item was selected.
    MenuItemSelected(menu::Id),
}
