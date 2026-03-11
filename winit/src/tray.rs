use crate::core::tray;
use crate::runtime;

use rustc_hash::FxHashMap;

pub struct TrayManager {
    trays: FxHashMap<tray::Id, tray::Settings>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            trays: FxHashMap::default(),
        }
    }

    pub fn handle_action(&mut self, action: runtime::tray::Action) {
        use runtime::tray::Action;

        match action {
            Action::Create(id, settings, channel) => {
                let _ = self.trays.insert(id, settings);
                let _ = channel.send(id);
            }
            Action::Remove(id) => {
                let _ = self.trays.remove(&id);
            }
            Action::SetIcon(id, icon) => {
                if let Some(settings) = self.trays.get_mut(&id) {
                    settings.icon = Some(icon);
                }
            }
            Action::SetTooltip(id, tooltip) => {
                if let Some(settings) = self.trays.get_mut(&id) {
                    settings.tooltip = tooltip;
                }
            }
            Action::SetVisible(id, visible) => {
                if let Some(settings) = self.trays.get_mut(&id) {
                    settings.visible = visible;
                }
            }
            Action::SetMenu(id, menu) => {
                if let Some(settings) = self.trays.get_mut(&id) {
                    settings.menu = menu;
                }
            }
            Action::ShowMenu(_) => {
                todo!();
            }
        }
    }
}
