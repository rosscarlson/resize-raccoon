use std::fs;
use tauri::api::path;
use tauri::{AppHandle, Runtime};

const BUNDLED: &[(&str, &str)] = &[
    ("en", include_str!("../../locales/en.json")),
    ("es", include_str!("../../locales/es.json")),
];

fn locales_dir<R: Runtime>(app: &AppHandle<R>) -> Result<std::path::PathBuf, String> {
    let app_data = path::app_data_dir(&app.config())
        .ok_or_else(|| "Cannot determine app data directory".to_string())?;
    Ok(app_data.join("locales"))
}

fn seed_if_needed<R: Runtime>(app: &AppHandle<R>) -> Result<std::path::PathBuf, String> {
    let dir = locales_dir(app)?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    // Always overwrite bundled locales so users get the latest translations after updates.
    // User-added language files are not in BUNDLED and are left untouched.
    for (lang, content) in BUNDLED {
        fs::write(dir.join(format!("{}.json", lang)), content)
            .map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

#[tauri::command]
pub fn locale_list<R: Runtime>(app: AppHandle<R>) -> Result<Vec<String>, String> {
    let dir = seed_if_needed(&app)?;
    let mut langs: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str() == Some("json") {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();
    langs.sort();
    Ok(langs)
}

#[tauri::command]
pub fn locale_load<R: Runtime>(app: AppHandle<R>, lang: String) -> Result<String, String> {
    let dir = seed_if_needed(&app)?;
    let path = dir.join(format!("{}.json", lang));
    if path.exists() {
        return fs::read_to_string(&path).map_err(|e| e.to_string());
    }
    // Fall back to bundled content
    for (bundled_lang, content) in BUNDLED {
        if *bundled_lang == lang.as_str() {
            return Ok(content.to_string());
        }
    }
    Err(format!("Locale '{}' not found", lang))
}
