//! A declarative tray menu.
mod id;

use crate::SmolStr;

pub use id::Id;

/// A tray menu.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Menu {
    /// The menu items.
    pub items: Vec<Item>,
}

impl Menu {
    /// Creates a new [`Menu`] with the given items.
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }
}

/// A tray menu item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    /// A standard menu item.
    Standard {
        /// The menu item identifier.
        id: Id,
        /// The label of the menu item.
        label: SmolStr,
        /// Whether the menu item is enabled.
        enabled: bool,
    },

    /// A checkable menu item.
    Check {
        /// The menu item identifier.
        id: Id,
        /// The label of the menu item.
        label: SmolStr,
        /// Whether the menu item is checked.
        checked: bool,
        /// Whether the menu item is enabled.
        enabled: bool,
    },

    /// A separator.
    Separator,

    /// A submenu containing another menu tree.
    Submenu {
        /// The label of the submenu.
        label: SmolStr,
        /// The submenu.
        menu: Menu,
        /// Whether the submenu is enabled.
        enabled: bool,
    },
}

impl Item {
    /// Create a [`Standard`] menu item.
    ///
    /// [`Standard`]: Item::Standard
    pub fn new(id: impl Into<Id>, label: impl Into<SmolStr>) -> Self {
        Self::Standard {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }

    /// Create a [`Check`] menu item.
    ///
    /// [`Check`]: Item::Check
    pub fn check(
        id: impl Into<Id>,
        label: impl Into<SmolStr>,
        checked: bool,
    ) -> Self {
        Self::Check {
            id: id.into(),
            label: label.into(),
            checked,
            enabled: true,
        }
    }

    /// Create a [`Submenu`] menu item.
    ///
    /// [`Submenu`]: Item::Submenu
    pub fn submenu(label: impl Into<SmolStr>, menu: Menu) -> Self {
        Self::Submenu {
            label: label.into(),
            menu,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Id;

    #[test]
    fn unique_generates_different_ids() {
        let a = Id::unique();
        let b = Id::unique();

        assert_ne!(a, b);
    }
}
