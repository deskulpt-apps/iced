//! Configure and interact with system tray.

use crate::core::Icon;
use crate::core::tray::{self, Event, Id, Settings};
use crate::futures::Subscription;
use crate::futures::futures::channel::oneshot;
use crate::futures::subscription;
use crate::task::{self, Task};

/// An operation to be performed on a tray.
#[derive(Debug)]
pub enum Action {
    /// Create a tray with some [`Settings`].
    Create(Id, Settings, oneshot::Sender<Id>),

    /// Remove a tray.
    Remove(Id),

    /// Change the tray [`Icon`].
    SetIcon(Id, Icon),

    /// Change the tray icon tooltip.
    SetTooltip(Id, Option<crate::core::SmolStr>),

    /// Change visibility of the tray icon.
    SetVisible(Id, bool),

    /// Replace the tray menu.
    SetMenu(Id, Option<tray::menu::Menu>),

    /// Request showing the tray menu.
    ShowMenu(Id),
}

/// Subscribes to all tray events of the running application.
pub fn events() -> Subscription<(Id, Event)> {
    #[derive(Hash)]
    struct TrayEvents;

    subscription::filter_map(TrayEvents, |event| {
        let subscription::Event::Tray { tray, event } = event else {
            return None;
        };
        Some((tray, event))
    })
}

/// Creates a new tray with the given [`Settings`].
///
/// Produces the [`Id`] of the tray on completion.
pub fn create(settings: Settings) -> (Id, Task<Id>) {
    let id = Id::unique();

    (
        id,
        task::oneshot(|channel| {
            crate::Action::Tray(Action::Create(id, settings, channel))
        }),
    )
}

/// Removes the tray with `id`.
pub fn remove<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Tray(Action::Remove(id)))
}

/// Sets the icon of the tray with `id`.
pub fn set_icon<T>(id: Id, icon: Icon) -> Task<T> {
    task::effect(crate::Action::Tray(Action::SetIcon(id, icon)))
}

/// Sets the tooltip of the tray icon with `id`.
pub fn set_tooltip<T>(
    id: Id,
    tooltip: Option<crate::core::SmolStr>,
) -> Task<T> {
    task::effect(crate::Action::Tray(Action::SetTooltip(id, tooltip)))
}

/// Sets the visibility of the tray icon with `id`.
pub fn set_visible<T>(id: Id, visible: bool) -> Task<T> {
    task::effect(crate::Action::Tray(Action::SetVisible(id, visible)))
}

/// Sets the menu of the tray with `id`.
pub fn set_menu<T>(id: Id, menu: Option<tray::menu::Menu>) -> Task<T> {
    task::effect(crate::Action::Tray(Action::SetMenu(id, menu)))
}

/// Requests showing the menu of the tray with `id`.
pub fn show_menu<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Tray(Action::ShowMenu(id)))
}
