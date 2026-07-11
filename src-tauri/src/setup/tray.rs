use tauri::{
    AppHandle, Builder, CustomMenuItem, Manager, Runtime, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu,
};
use uuid::Uuid;

use crate::operations::profile::Profile;
use crate::operations::window_manager::{self, ApplyConfig};
use crate::setup::state::AppState;

pub fn build_tray_menu(profiles: &[Profile]) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Show"))
        .add_native_item(SystemTrayMenuItem::Separator);

    for profile in profiles {
        let submenu = SystemTraySubmenu::new(
            &profile.name,
            SystemTrayMenu::new()
                .add_item(CustomMenuItem::new(format!("apply-{}", profile.uuid), "Apply")),
        );
        menu = menu.add_submenu(submenu);
    }

    menu.add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("exit", "Exit"))
}

pub fn rebuild_tray_menu<R: Runtime>(app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();
    let profiles = state.profiles.lock().unwrap().clone();
    let _ = app_handle.tray_handle().set_menu(build_tray_menu(&profiles));
}

pub fn setup_tray<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let system_tray = SystemTray::new().with_menu(build_tray_menu(&[]));

    builder
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                "exit" => {
                    app.exit(0);
                }
                other if other.starts_with("apply-") => {
                    let uuid_str = other.strip_prefix("apply-").unwrap();
                    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                        let state = app.state::<AppState>();
                        let profiles = state.profiles.lock().unwrap();
                        if let Some(profile) = profiles.iter().find(|p| p.uuid == uuid) {
                            let _ = window_manager::apply_profile(
                                profile,
                                ApplyConfig::new().retry(true).monitor(true),
                            );
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        })
}
