use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::api::path;
use tauri::{AppHandle, Runtime};

static ENABLED: AtomicBool = AtomicBool::new(false);
static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

// Keep resize-rabbit.log plus this many numbered backups (resize-rabbit.1.log ...
// resize-rabbit.N.log) — old sessions are rotated, never silently deleted, so a
// user can still dig up an earlier repro after a later session overwrote "the" log.
const MAX_ROTATED_LOGS: u32 = 10;

// Deliberately the same per-user app-data dir as user_settings.json/profiles.json
// (not next to the .exe) — Program Files is not writable without elevation, which
// silently broke logging entirely on a non-admin install.
fn logs_dir<R: Runtime>(app_handle: &AppHandle<R>) -> Option<PathBuf> {
    Some(path::app_data_dir(&app_handle.config())?.join("logs"))
}

fn rotate_existing_log(dir: &Path) {
    let current = dir.join("resize-rabbit.log");
    if !current.exists() {
        return;
    }

    for i in (1..MAX_ROTATED_LOGS).rev() {
        let from = dir.join(format!("resize-rabbit.{}.log", i));
        if from.exists() {
            let to = dir.join(format!("resize-rabbit.{}.log", i + 1));
            let _ = fs::rename(&from, &to);
        }
    }

    let _ = fs::rename(&current, dir.join("resize-rabbit.1.log"));
}

/// Enabling starts a fresh log file for this session, rotating any previous one
/// aside (resize-rabbit.log -> .1.log -> .2.log, ...) rather than deleting it, so
/// an earlier repro isn't lost just because logging got turned on again later.
/// Disabling just closes the handle; the last session's log is left on disk.
/// Re-calling with `enabled: true` while already enabled (e.g. from an unrelated
/// settings change elsewhere in the same settings object) is a deliberate no-op —
/// otherwise every unrelated settings_update call would rotate/fragment the
/// current session's log.
pub fn set_enabled<R: Runtime>(enabled: bool, app_handle: &AppHandle<R>) {
    let was_enabled = ENABLED.load(Ordering::SeqCst);
    ENABLED.store(enabled, Ordering::SeqCst);

    let mut guard = LOG_FILE.lock().unwrap();

    if !enabled {
        *guard = None;
        return;
    }

    if was_enabled && guard.is_some() {
        return;
    }

    let Some(dir) = logs_dir(app_handle) else {
        *guard = None;
        return;
    };

    if fs::create_dir_all(&dir).is_err() {
        *guard = None;
        return;
    }

    rotate_existing_log(&dir);

    let path = dir.join("resize-rabbit.log");
    *guard = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .ok();
}

pub fn log(message: &str) {
    if !ENABLED.load(Ordering::SeqCst) {
        return;
    }

    let mut guard = LOG_FILE.lock().unwrap();
    if let Some(file) = guard.as_mut() {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
        let _ = file.flush();
    }
}
