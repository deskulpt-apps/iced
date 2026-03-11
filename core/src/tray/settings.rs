//! Configure your system tray.

use crate::{Icon, SmolStr};

use super::menu::Menu;

/// The tray settings.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The tray icon.
    pub icon: Option<Icon>,

    /// The tray icon tooltip shown by the system.
    pub tooltip: Option<SmolStr>,

    /// Whether the tray icon should be visible or not.
    pub visible: bool,

    /// The tray menu.
    pub menu: Option<Menu>,

    /// Whether a left click should automatically open the tray menu.
    pub menu_on_left_click: bool,

    /// Whether a right click should automatically open the tray menu.
    pub menu_on_right_click: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            icon: None,
            tooltip: None,
            visible: true,
            menu: None,
            menu_on_left_click: false,
            menu_on_right_click: true,
        }
    }
}
