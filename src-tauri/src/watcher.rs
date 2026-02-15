use notify::event::ModifyKind;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::media::{self, MediaEntry, VIDEO_EXTENSIONS};

/// Payload emitted to the frontend when the library changes
#[derive(serde::Serialize, Clone, Debug)]
pub struct MediaChangeEvent {
    pub kind: String, // "full_scan", "incremental"
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
    pub total: usize,
}

/// Maximum number of retries when waiting for a file transfer to complete
const FILE_STABILITY_MAX_RETRIES: u32 = 5;
/// Delay between each file stability check
const FILE_STABILITY_CHECK_DELAY: Duration = Duration::from_millis(500);

/// State holding the active watchers
pub struct WatcherState {
    watchers: HashMap<String, RecommendedWatcher>,
    stop_flag: Arc<AtomicBool>,
    debounce_thread: Option<thread::JoinHandle<()>>,
    pending_changes: Arc<Mutex<Vec<PendingChange>>>,
}

#[derive(Debug, Clone)]
struct PendingChange {
    path: PathBuf,
    kind: ChangeKind,
}

#[derive(Debug, Clone)]
enum ChangeKind {
    Added,
    Removed,
    Modified,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            debounce_thread: None,
            pending_changes: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Check if a path has a media file extension
fn has_media_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Create a MediaEntry from a file path
fn entry_from_path(path: &Path) -> Option<MediaEntry> {
    if !path.is_file() {
        return None;
    }
    let ext = path.extension()?.to_str()?.to_lowercase();
    if !VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return None;
    }
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let file_path = path.to_string_lossy().replace('\\', "/");

    Some(MediaEntry {
        id: uuid::Uuid::new_v4().to_string(),
        path: file_path,
        filename,
        extension: ext,
        size_bytes: size,
    })
}

/// Wait for a file to finish being transferred/copied.
/// Returns true if the file is stable (transfer complete), false if it's still changing or disappeared.
fn wait_for_file_stable(path: &Path) -> bool {
    for attempt in 0..FILE_STABILITY_MAX_RETRIES {
        let size_before = match std::fs::metadata(path) {
            Ok(m) => m.len(),
            Err(_) => return false, // File disappeared
        };

        thread::sleep(FILE_STABILITY_CHECK_DELAY);

        let size_after = match std::fs::metadata(path) {
            Ok(m) => m.len(),
            Err(_) => return false, // File disappeared
        };

        if size_before == size_after && size_after > 0 {
            log::debug!(
                "[Watcher] File stable after {} check(s): {}",
                attempt + 1,
                path.display()
            );
            return true;
        }

        log::debug!(
            "[Watcher] File still changing ({} -> {} bytes), retry {}/{}: {}",
            size_before,
            size_after,
            attempt + 1,
            FILE_STABILITY_MAX_RETRIES,
            path.display()
        );
    }

    log::warn!(
        "[Watcher] File not stable after {} retries, skipping: {}",
        FILE_STABILITY_MAX_RETRIES,
        path.display()
    );
    false
}

/// Start watching all configured folders for a given user.
/// Performs an initial full scan and then watches for changes.
pub fn start_watching(app: &AppHandle, user_id: &str) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<WatcherState>>>();
    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Stop any existing watchers
    state.stop_flag.store(true, Ordering::SeqCst);
    state.watchers.clear();
    if let Some(handle) = state.debounce_thread.take() {
        let _ = handle.join();
    }

    // Reset stop flag for the new watcher
    let stop_flag = Arc::new(AtomicBool::new(false));
    state.stop_flag = stop_flag.clone();

    // Load config
    let config = media::get_media_config(app.clone(), user_id.to_string())?;
    let settings = media::get_settings(app.clone(), user_id.to_string())?;

    if config.folders.is_empty() {
        log::info!("[Watcher] No folders to watch for user {}", user_id);
        return Ok(());
    }

    // Initial full scan (only if enabled in settings)
    if settings.scan_on_startup {
        log::info!(
            "[Watcher] Performing initial scan of {} folder(s) for user {}",
            config.folders.len(),
            user_id
        );
        let scan_result = media::scan_media_folders(app.clone(), user_id.to_string())?;
        let _ = app.emit(
            "media-change",
            MediaChangeEvent {
                kind: "full_scan".to_string(),
                added: vec![],
                modified: vec![],
                removed: vec![],
                total: scan_result.total_found,
            },
        );
    } else {
        log::info!("[Watcher] Scan on startup disabled for user {}", user_id);
    }

    // If live scan is disabled, don't start the watcher
    if !settings.live_scan {
        log::info!("[Watcher] Live scan disabled for user {}", user_id);
        return Ok(());
    }

    // Set up pending changes buffer
    let pending = state.pending_changes.clone();

    // Start debounce processor thread
    let pending_for_thread = pending.clone();
    let app_for_thread = app.clone();
    let user_for_thread = user_id.to_string();
    let stop_for_thread = stop_flag.clone();

    let debounce_thread = thread::spawn(move || {
        log::info!("[Watcher] Debounce processor thread started");

        while !stop_for_thread.load(Ordering::SeqCst) {
            // Sleep for 2 seconds (debounce interval)
            thread::sleep(Duration::from_secs(2));

            if stop_for_thread.load(Ordering::SeqCst) {
                break;
            }

            // Drain pending changes
            let changes: Vec<PendingChange> = {
                let mut pending = match pending_for_thread.lock() {
                    Ok(p) => p,
                    Err(e) => e.into_inner(),
                };
                if pending.is_empty() {
                    continue;
                }
                pending.drain(..).collect()
            };

            if changes.is_empty() {
                continue;
            }

            log::info!(
                "[Watcher] Processing {} raw pending change(s)",
                changes.len()
            );

            // Deduplicate changes per path: keep the highest-priority kind.
            // Priority: Added > Removed > Modified (if a file was Created then
            // Modified by the OS, we only want "Added"; if Created then Removed
            // we only want "Removed").
            let changes: Vec<PendingChange> = {
                let mut map: std::collections::HashMap<PathBuf, ChangeKind> =
                    std::collections::HashMap::new();
                for c in changes {
                    map.entry(c.path.clone())
                        .and_modify(|existing| {
                            // Merge rules:
                            // Added + Modified  → Added  (OS writes after create)
                            // Added + Removed   → Removed
                            // Modified + Added  → Added
                            // Modified + Removed→ Removed
                            // Removed + Added   → Added  (replaced file)
                            match (&existing, &c.kind) {
                                (ChangeKind::Added, ChangeKind::Modified) => { /* keep Added */ }
                                (ChangeKind::Added, ChangeKind::Removed) => {
                                    *existing = ChangeKind::Removed;
                                }
                                (ChangeKind::Modified, ChangeKind::Added) => {
                                    *existing = ChangeKind::Added;
                                }
                                (ChangeKind::Modified, ChangeKind::Removed) => {
                                    *existing = ChangeKind::Removed;
                                }
                                (ChangeKind::Removed, ChangeKind::Added) => {
                                    *existing = ChangeKind::Added;
                                }
                                _ => { /* same kind or Removed+Modified → keep existing */ }
                            }
                        })
                        .or_insert(c.kind);
                }
                map.into_iter()
                    .map(|(path, kind)| PendingChange { path, kind })
                    .collect()
            };

            log::info!(
                "[Watcher] After dedup: {} unique change(s)",
                changes.len()
            );

            // Load current library
            let current_entries =
                match media::get_media_library(app_for_thread.clone(), user_for_thread.clone()) {
                    Ok(entries) => entries,
                    Err(e) => {
                        log::warn!("[Watcher] Failed to load library: {}", e);
                        continue;
                    }
                };

            let mut entries = current_entries;
            let mut changed = false;
            let mut added_files: Vec<String> = Vec::new();
            let mut modified_files: Vec<String> = Vec::new();
            let mut removed_files: Vec<String> = Vec::new();

            for change in &changes {
                let normalized_path = change.path.to_string_lossy().replace('\\', "/");
                let filename = change
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match change.kind {
                    ChangeKind::Added => {
                        // Wait for file transfer to complete before processing
                        if !wait_for_file_stable(&change.path) {
                            log::info!(
                                "[Watcher] Skipping unstable/disappeared file: {}",
                                normalized_path
                            );
                            continue;
                        }
                        if let Some(entry) = entry_from_path(&change.path) {
                            // Only count as "added" if it wasn't already in the library
                            let was_present = entries.iter().any(|e| e.path == normalized_path);
                            entries.retain(|e| e.path != normalized_path);
                            log::info!("[Watcher] Media file added: {}", entry.path);
                            entries.push(entry);
                            changed = true;
                            if !was_present {
                                added_files.push(filename);
                            }
                        }
                    }
                    ChangeKind::Modified => {
                        // Wait for file to stabilize (e.g. still being written)
                        if !wait_for_file_stable(&change.path) {
                            log::info!(
                                "[Watcher] Skipping unstable/disappeared file: {}",
                                normalized_path
                            );
                            continue;
                        }
                        if let Some(entry) = entry_from_path(&change.path) {
                            entries.retain(|e| e.path != normalized_path);
                            log::info!("[Watcher] Media file modified: {}", entry.path);
                            entries.push(entry);
                            changed = true;
                            modified_files.push(filename);
                        }
                    }
                    ChangeKind::Removed => {
                        let before = entries.len();
                        entries.retain(|e| e.path != normalized_path);
                        if entries.len() != before {
                            log::info!("[Watcher] Media file removed: {}", normalized_path);
                            changed = true;
                            removed_files.push(filename);
                        }
                    }
                }
            }

            if changed {
                // Sort and dedup
                entries.sort_by(|a, b| a.path.cmp(&b.path));
                entries.dedup_by(|a, b| a.path == b.path);

                // Save library
                let library_path = match media::get_library_path_public(
                    &app_for_thread,
                    &user_for_thread,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!("[Watcher] Failed to get library path: {}", e);
                        continue;
                    }
                };

                match serde_json::to_string_pretty(&entries) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&library_path, json) {
                            log::warn!("[Watcher] Failed to write library: {}", e);
                            continue;
                        }
                    }
                    Err(e) => {
                        log::warn!("[Watcher] Failed to serialize library: {}", e);
                        continue;
                    }
                }

                let total = entries.len();
                log::info!(
                    "[Watcher] Library updated: +{} ~{} -{}, {} total",
                    added_files.len(),
                    modified_files.len(),
                    removed_files.len(),
                    total
                );

                let _ = app_for_thread.emit(
                    "media-change",
                    MediaChangeEvent {
                        kind: "incremental".to_string(),
                        added: added_files.clone(),
                        modified: modified_files,
                        removed: removed_files,
                        total,
                    },
                );

                // Trigger metadata fetch for newly added files in background
                if !added_files.is_empty() {
                    let app_meta = app_for_thread.clone();
                    let user_meta = user_for_thread.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = crate::metadata::fetch_missing_metadata(&app_meta, &user_meta) {
                            log::warn!("[Watcher] Failed to fetch metadata for new files: {}", e);
                        }
                    });
                }
            }
        }

        log::info!("[Watcher] Debounce processor thread stopped");
    });

    state.debounce_thread = Some(debounce_thread);

    // Create file system watcher
    let pending_for_watcher = pending.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                let change_kind = match event.kind {
                    EventKind::Create(_) => Some(ChangeKind::Added),
                    EventKind::Modify(ModifyKind::Data(_))
                    | EventKind::Modify(ModifyKind::Any) => Some(ChangeKind::Modified),
                    EventKind::Remove(_) => Some(ChangeKind::Removed),
                    _ => None,
                };

                if let Some(kind) = change_kind {
                    for path in &event.paths {
                        if has_media_extension(path) {
                            let mut pending =
                                pending_for_watcher.lock().unwrap_or_else(|e| e.into_inner());
                            pending.push(PendingChange {
                                path: path.clone(),
                                kind: kind.clone(),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("[Watcher] Watch error: {}", e);
            }
        }
    })
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch each configured folder
    for folder in &config.folders {
        let path = Path::new(folder);
        if path.is_dir() {
            match watcher.watch(path, RecursiveMode::Recursive) {
                Ok(()) => {
                    log::info!("[Watcher] Watching folder: {}", folder);
                }
                Err(e) => {
                    log::warn!("[Watcher] Failed to watch folder {}: {}", folder, e);
                }
            }
        }
    }

    state.watchers.insert(user_id.to_string(), watcher);

    log::info!(
        "[Watcher] File watcher started for user {} ({} folder(s))",
        user_id,
        config.folders.len()
    );
    // Start the metadata retry background thread
    if let Err(e) = crate::metadata::start_metadata_retry(app, user_id) {
        log::warn!("[Watcher] Failed to start metadata retry thread: {}", e);
    }
    Ok(())
}

/// Stop all watchers for a user
pub fn stop_watching(app: &AppHandle, user_id: &str) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<WatcherState>>>();
    let mut state = state.lock().map_err(|e| e.to_string())?;

    if state.watchers.remove(user_id).is_some() {
        log::info!("[Watcher] Stopped watching for user {}", user_id);
    }

    if state.watchers.is_empty() {
        // Signal the debounce thread to stop
        state.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = state.debounce_thread.take() {
            let _ = handle.join();
            log::info!("[Watcher] Debounce processor stopped");
        }
    }

    // Stop the metadata retry thread
    if let Err(e) = crate::metadata::stop_metadata_retry(app) {
        log::warn!("[Watcher] Failed to stop metadata retry thread: {}", e);
    }

    Ok(())
}

/// Restart watcher (e.g. after folder config changes)
pub fn restart_watching(app: &AppHandle, user_id: &str) -> Result<(), String> {
    log::info!("[Watcher] Restarting watcher for user {}", user_id);
    stop_watching(app, user_id)?;
    start_watching(app, user_id)
}
