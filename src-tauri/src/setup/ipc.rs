use miow::pipe::NamedPipeBuilder;
use regex::Regex;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};

use crate::debug_log;
use crate::operations::profile::Profile;
use crate::operations::window_manager::{self, ApplyConfig};

pub fn listener<R: Runtime>(profiles: Arc<Mutex<Vec<Profile>>>, app_handle: AppHandle<R>) {
    let pipe_name = r"\\.\pipe\resize-rabbit";
    let command_regex = Regex::new(r#"("[^"]+"|\S+)"#).unwrap();

    loop {
        let mut server = NamedPipeBuilder::new(pipe_name)
            .inbound(true)
            .outbound(true)
            .first(true)
            .create()
            .expect("Unable to create named pipe");

        debug_log!("Named pipe server waiting for connection...");

        server.connect().expect("Failed to wait for client");

        let mut buffer = [0; 256];
        let bytes_read = server.read(&mut buffer).expect("Failed to read from pipe");
        let command_string = String::from_utf8_lossy(&buffer[..bytes_read]);

        let tokens: Vec<String> = command_regex
            .captures_iter(&command_string)
            .map(|cap| cap[1].to_string().trim_matches('"').to_string())
            .collect();

        if tokens.is_empty() {
            continue;
        }

        let command = &tokens[0];
        let args = &tokens[1..];

        debug_log!("Received command: {}", command);
        debug_log!("Received arguments: {:?}", args);

        if command == "show" {
            if let Some(window) = app_handle.get_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        } else if command == "apply-profile" {
            let profiles_guard = profiles.lock().unwrap();
            let current_profiles = profiles_guard.clone();
            drop(profiles_guard);

            let profile_name = &args[0].to_lowercase();

            if let Some(profile) = current_profiles
                .iter()
                .find(|p| p.name.to_lowercase() == *profile_name)
            {
                let _ = window_manager::apply_profile(
                    profile,
                    ApplyConfig::new().retry(true).monitor(true),
                );
            } else {
                debug_log!("Profile not found: {}", profile_name)
            }
        }
    }
}
