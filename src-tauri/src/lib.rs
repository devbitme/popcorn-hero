mod media;
mod metadata;
mod user;
mod watcher;

use std::sync::{Arc, Mutex};
use tauri_plugin_log::{Target, TargetKind};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    log::info!("Greeting user: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .clear_targets()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(today),
                    }),
                    Target::new(TargetKind::Webview),
                ])
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(watcher::WatcherState::new())))
        .manage(Arc::new(Mutex::new(metadata::MetadataRetryState::new())))
        .invoke_handler(tauri::generate_handler![
            greet,
            user::check_user_exists,
            user::create_user,
            user::verify_pin,
            media::get_media_config,
            media::save_media_config,
            media::add_media_folder,
            media::remove_media_folder,
            media::scan_media_folders,
            media::get_media_library,
            media::pick_folder,
            media::get_settings,
            media::update_settings,
            media::start_media_watcher,
            media::stop_media_watcher,
            media::fetch_all_metadata,
            media::get_media_metadata,
        ])
        .setup(|_app| {
            log::info!("[App] Popcorn Hero started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
