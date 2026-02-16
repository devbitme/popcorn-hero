use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

/// Supported media file extensions
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts", "vob", "3gp",
    "ogv", "divx", "asf", "m2ts", "mts", "rmvb",
];

/// Configuration: list of directories to scan
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MediaConfig {
    pub folders: Vec<String>,
}

/// A single media entry found during scanning
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MediaEntry {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
}

/// Result of a scan operation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanResult {
    pub total_found: usize,
    pub media_entries: Vec<MediaEntry>,
}

/// Get a specific user's directory by UUID
fn get_user_dir(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    let app_local_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?;
    let user_dir = app_local_data.join("users").join(user_id);
    if !user_dir.exists() {
        return Err(format!("User directory not found: {}", user_id));
    }
    Ok(user_dir)
}

/// Get path to the settings file (in the user's directory)
fn get_settings_path(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    Ok(get_user_dir(app, user_id)?.join("settings.json"))
}

/// Get path to the media library flat file (in the user's directory)
fn get_library_path(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    Ok(get_user_dir(app, user_id)?.join("library.json"))
}

/// Public accessor for watcher module
pub fn get_library_path_public(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    get_library_path(app, user_id)
}

/// Public accessor for metadata module
pub fn get_user_dir_public(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    get_user_dir(app, user_id)
}

/// A metadata provider configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetadataProviderConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub api_key: String,
}

fn default_metadata_providers() -> Vec<MetadataProviderConfig> {
    vec![
        MetadataProviderConfig {
            id: "tmdb".to_string(),
            name: "The Movie Database (TMDB)".to_string(),
            enabled: true,
            api_key: String::new(),
        },
        MetadataProviderConfig {
            id: "omdb".to_string(),
            name: "Open Movie Database (OMDb)".to_string(),
            enabled: false,
            api_key: String::new(),
        },
    ]
}

/// The full settings file structure (will contain more settings in the future)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    #[serde(default)]
    pub media: MediaConfig,
    #[serde(default = "default_true")]
    pub scan_on_startup: bool,
    #[serde(default = "default_true")]
    pub live_scan: bool,
    #[serde(default = "default_metadata_providers")]
    pub metadata_providers: Vec<MetadataProviderConfig>,
    /// How many months metadata should be cached before being refreshed (1-6)
    #[serde(default = "default_cache_months")]
    pub metadata_cache_months: u32,
}

fn default_true() -> bool {
    true
}

fn default_cache_months() -> u32 {
    1
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            media: MediaConfig::default(),
            scan_on_startup: true,
            live_scan: true,
            metadata_providers: default_metadata_providers(),
            metadata_cache_months: 1,
        }
    }
}

/// Load settings from the user's settings.json
fn load_settings(app: &AppHandle, user_id: &str) -> Result<Settings, String> {
    let path = get_settings_path(app, user_id)?;

    if !path.exists() {
        log::info!("[Media] No settings file found for user {}, returning defaults", user_id);
        return Ok(Settings::default());
    }

    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings: Settings = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(settings)
}

/// Save settings to the user's settings.json
fn save_settings(app: &AppHandle, user_id: &str, settings: &Settings) -> Result<(), String> {
    let path = get_settings_path(app, user_id)?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Load the media configuration from the user's settings
#[tauri::command]
pub fn get_media_config(app: AppHandle, user_id: String) -> Result<MediaConfig, String> {
    let settings = load_settings(&app, &user_id)?;

    log::info!(
        "[Media] Loaded config with {} folder(s) for user {}",
        settings.media.folders.len(),
        user_id
    );
    Ok(settings.media)
}

/// Save the media configuration to the user's settings
#[tauri::command]
pub fn save_media_config(app: AppHandle, user_id: String, config: MediaConfig) -> Result<(), String> {
    let mut settings = load_settings(&app, &user_id)?;
    settings.media = config;
    save_settings(&app, &user_id, &settings)?;

    log::info!(
        "[Media] Saved config with {} folder(s) for user {}: {:?}",
        settings.media.folders.len(),
        user_id,
        settings.media.folders
    );
    Ok(())
}

/// Add a folder to the media configuration
#[tauri::command]
pub fn add_media_folder(app: AppHandle, user_id: String, folder: String) -> Result<MediaConfig, String> {
    let mut config = get_media_config(app.clone(), user_id.clone())?;

    // Normalize path separators
    let normalized = folder.replace('\\', "/");

    if config.folders.contains(&normalized) {
        log::warn!("[Media] Folder already in config: {}", normalized);
        return Err("This folder is already in the list.".to_string());
    }

    // Verify folder exists
    if !std::path::Path::new(&folder).is_dir() {
        log::warn!("[Media] Folder does not exist: {}", folder);
        return Err("The specified folder does not exist.".to_string());
    }

    config.folders.push(normalized.clone());
    save_media_config(app.clone(), user_id.clone(), config.clone())?;

    // Restart watcher to include the new folder
    if let Err(e) = crate::watcher::restart_watching(&app, &user_id) {
        log::warn!("[Media] Failed to restart watcher after adding folder: {}", e);
    }

    log::info!("[Media] Added folder: {}", normalized);
    Ok(config)
}

/// Remove a folder from the media configuration
#[tauri::command]
pub fn remove_media_folder(app: AppHandle, user_id: String, folder: String) -> Result<MediaConfig, String> {
    let mut config = get_media_config(app.clone(), user_id.clone())?;

    let normalized = folder.replace('\\', "/");
    let initial_len = config.folders.len();
    config.folders.retain(|f| f != &normalized);

    if config.folders.len() == initial_len {
        log::warn!("[Media] Folder not found in config: {}", normalized);
        return Err("Folder not found in configuration.".to_string());
    }

    save_media_config(app.clone(), user_id.clone(), config.clone())?;

    // Restart watcher without the removed folder
    if let Err(e) = crate::watcher::restart_watching(&app, &user_id) {
        log::warn!("[Media] Failed to restart watcher after removing folder: {}", e);
    }

    log::info!("[Media] Removed folder: {}", normalized);
    Ok(config)
}

/// Recursively scan a directory for media files
fn scan_directory(dir: &std::path::Path, entries: &mut Vec<MediaEntry>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            log::warn!("[Media] Cannot read directory {:?}: {}", dir, e);
            return;
        }
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory(&path, entries);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let file_path = path.to_string_lossy().replace('\\', "/");

                    entries.push(MediaEntry {
                        id: Uuid::new_v4().to_string(),
                        path: file_path,
                        filename,
                        extension: ext_lower,
                        size_bytes: size,
                    });
                }
            }
        }
    }
}

/// Scan all configured folders and update the media library flat file
#[tauri::command]
pub fn scan_media_folders(app: AppHandle, user_id: String) -> Result<ScanResult, String> {
    let config = get_media_config(app.clone(), user_id.clone())?;

    if config.folders.is_empty() {
        log::info!("[Media] No folders configured, nothing to scan");
        return Ok(ScanResult {
            total_found: 0,
            media_entries: vec![],
        });
    }

    log::info!(
        "[Media] Starting scan of {} folder(s)",
        config.folders.len()
    );

    let mut all_entries: Vec<MediaEntry> = Vec::new();

    for folder in &config.folders {
        let path = std::path::Path::new(folder);
        if path.is_dir() {
            log::info!("[Media] Scanning folder: {}", folder);
            scan_directory(path, &mut all_entries);
        } else {
            log::warn!("[Media] Skipping non-existent folder: {}", folder);
        }
    }

    // Deduplicate by path
    all_entries.sort_by(|a, b| a.path.cmp(&b.path));
    all_entries.dedup_by(|a, b| a.path == b.path);

    // Preserve existing UUIDs for known paths
    let library_path = get_library_path(&app, &user_id)?;
    if library_path.exists() {
        if let Ok(raw) = fs::read_to_string(&library_path) {
            if let Ok(existing) = serde_json::from_str::<Vec<MediaEntry>>(&raw) {
                let existing_map: std::collections::HashMap<String, String> = existing
                    .into_iter()
                    .filter(|e| !e.id.is_empty())
                    .map(|e| (e.path.clone(), e.id))
                    .collect();
                for entry in &mut all_entries {
                    if let Some(existing_id) = existing_map.get(&entry.path) {
                        entry.id = existing_id.clone();
                    }
                }
            }
        }
    }

    // Collect new entries (those that got a fresh UUID) for metadata fetching

    let result = ScanResult {
        total_found: all_entries.len(),
        media_entries: all_entries,
    };

    // Write the media library flat file
    let json = serde_json::to_string_pretty(&result.media_entries).map_err(|e| e.to_string())?;
    fs::write(&library_path, json).map_err(|e| e.to_string())?;

    // Trigger metadata fetch for entries missing metadata
    {
        let app_clone = app.clone();
        let user_id_clone = user_id.clone();
        std::thread::spawn(move || {
            if let Err(e) = crate::metadata::fetch_missing_metadata(&app_clone, &user_id_clone) {
                log::warn!("[Media] Failed to fetch metadata after scan: {}", e);
            }
        });
    }

    log::info!(
        "[Media] Scan complete: {} media file(s) found, library saved to {:?}",
        result.total_found,
        library_path
    );

    Ok(result)
}

/// Get the current media library from the flat file (without re-scanning)
#[tauri::command]
pub fn get_media_library(app: AppHandle, user_id: String) -> Result<Vec<MediaEntry>, String> {
    let library_path = get_library_path(&app, &user_id)?;

    if !library_path.exists() {
        log::info!("[Media] No media library file found, returning empty list");
        return Ok(vec![]);
    }

    let raw = fs::read_to_string(&library_path).map_err(|e| e.to_string())?;
    let entries: Vec<MediaEntry> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    log::info!(
        "[Media] Loaded {} media entries from library file",
        entries.len()
    );
    Ok(entries)
}

/// Pick a folder using native dialog (returns the selected path)
#[tauri::command]
pub fn pick_folder() -> Result<Option<String>, String> {
    // We'll use rfd (Rust File Dialog) for native folder picking
    let dialog = rfd::FileDialog::new()
        .set_title("Select a media folder");

    match dialog.pick_folder() {
        Some(path) => {
            let path_str = path.to_string_lossy().replace('\\', "/");
            log::info!("[Media] Folder picked: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            log::info!("[Media] Folder picker cancelled");
            Ok(None)
        }
    }
}

/// Get the full settings for a user
#[tauri::command]
pub fn get_settings(app: AppHandle, user_id: String) -> Result<Settings, String> {
    load_settings(&app, &user_id)
}

/// Update settings for a user
#[tauri::command]
pub fn update_settings(app: AppHandle, user_id: String, settings: Settings) -> Result<(), String> {
    save_settings(&app, &user_id, &settings)?;
    log::info!("[Media] Settings updated for user {}: scan_on_startup={}, live_scan={}", user_id, settings.scan_on_startup, settings.live_scan);
    Ok(())
}

/// Manually trigger metadata fetch for all entries missing metadata
#[tauri::command]
pub fn fetch_all_metadata(app: AppHandle, user_id: String) -> Result<String, String> {
    crate::metadata::fetch_missing_metadata(&app, &user_id)
}

/// Get metadata for a specific media entry
#[tauri::command]
pub fn get_media_metadata(app: AppHandle, user_id: String, media_id: String) -> Result<serde_json::Value, String> {
    crate::metadata::get_metadata(&app, &user_id, &media_id)
}

/// Start the file watcher for a user (called after login)
#[tauri::command]
pub fn start_media_watcher(app: AppHandle, user_id: String) -> Result<(), String> {
    crate::watcher::start_watching(&app, &user_id)
}

/// Stop the file watcher for a user (called on logout)
#[tauri::command]
pub fn stop_media_watcher(app: AppHandle, user_id: String) -> Result<(), String> {
    crate::watcher::stop_watching(&app, &user_id)
}
