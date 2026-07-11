use tauri::{AppHandle, GlobalShortcutManager, Manager, Runtime};

use crate::operations::window_manager::{self, ApplyConfig};
use crate::setup::state::AppState;

pub fn rebuild_shortcuts<R: Runtime>(app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();
    let profiles = state.profiles.lock().unwrap().clone();
    drop(state);

    let mut mgr = app_handle.global_shortcut_manager();
    let _ = mgr.unregister_all();

    for profile in profiles {
        let shortcut = match &profile.shortcut {
            Some(s) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        let profile_clone = profile.clone();
        let result = mgr.register(&shortcut, move || {
            let _ = window_manager::apply_profile(
                &profile_clone,
                ApplyConfig::new().retry(true).monitor(true),
            );
        });

        if let Err(e) = result {
            eprintln!("Failed to register shortcut '{}': {}", shortcut, e);
        }
    }
}
