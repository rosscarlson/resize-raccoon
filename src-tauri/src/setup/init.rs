use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use std::thread;

use crate::debug_log;
use crate::operations::{
    process, profile,
    user_settings::{self, UserSettings},
    window_state,
};
use crate::setup::ipc;
use crate::setup::shortcuts;
use crate::setup::state::AppState;
use crate::setup::tray;
use tauri::{Builder, Manager, Runtime};

pub fn setup<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.setup(move |app| {
        let watcher_flag = Arc::new(AtomicBool::new(false));
        let poll_rate_flag = Arc::new(AtomicU64::new(1000));
        let app_handle = app.handle();

        let is_first_run = user_settings::get_user_settings_path(&app_handle)
            .map(|p| !p.exists())
            .unwrap_or(false);

        let user_settings = user_settings::get_user_settings(&app_handle).unwrap_or_else(|e| {
            debug_log!("Error loading user settings: {}", e);
            UserSettings::default()
        });

        crate::logging::set_enabled(user_settings.logging_enabled, &app_handle);
        debug_log!(
            "Resize Rabbit v{} starting up (logging enabled: {})",
            env!("CARGO_PKG_VERSION"),
            user_settings.logging_enabled
        );

        // Proves which build is actually running regardless of the version
        // number — settles "did the installer actually update anything" without
        // guessing, since the version string alone can't distinguish between two
        // installs of the same version.
        if let Ok(exe_path) = std::env::current_exe() {
            let modified = std::fs::metadata(&exe_path)
                .and_then(|m| m.modified())
                .map(|t| {
                    let local: chrono::DateTime<chrono::Local> = t.into();
                    local.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "unknown".to_string());
            debug_log!("Running exe: {} (last modified: {})", exe_path.display(), modified);
        }

        if is_first_run {
            user_settings::apply_installer_launch_on_start_choice(&app_handle);
        }

        let profiles = Arc::new(Mutex::new(
            profile::load_profiles(&app_handle).unwrap_or_else(|e| {
                debug_log!("Error loading profiles: {}", e);
                Vec::new()
            }),
        ));
        // Store the initial state of the process watcher
        watcher_flag.store(user_settings.process_watcher_enabled, Ordering::SeqCst);
        poll_rate_flag.store(user_settings.poll_rate, Ordering::SeqCst);

        let app_state = AppState {
            profiles: profiles.clone(),
            process_watcher_enabled: watcher_flag.clone(),
            poll_rate: poll_rate_flag.clone(),
        };
        app.manage(app_state);

        // Populate tray menu with loaded profiles and current language
        tray::rebuild_tray_menu(&app_handle);

        // Register global shortcuts for profiles that have them
        shortcuts::rebuild_shortcuts(&app_handle);

        let profiles_clone = profiles.clone();

        thread::Builder::new().name("Process watcher".to_string()).spawn(move || {
            process::watcher(watcher_flag.clone(), poll_rate_flag.clone(), profiles_clone);
        }).unwrap();

        let profiles_clone = profiles.clone();
        let ipc_handle = app_handle.clone();

        thread::Builder::new().name("IPC Listener".to_string()).spawn(move || {
            ipc::listener(profiles_clone, ipc_handle);
        }).unwrap();

        let window = app.get_window("main").unwrap();

        // Restore the window to whichever monitor/position it was closed on last time
        window_state::restore_window_state(&window);

        // Check if we should minimize to sys tray
        if user_settings.start_minimized {
            window.hide().unwrap();
        }

        Ok(())
    })
}
