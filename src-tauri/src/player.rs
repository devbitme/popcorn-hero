use std::path::Path;
use std::process::Command;

#[tauri::command]
pub fn player_open_vlc(path: String) -> Result<(), String> {
    log::info!("[Player] Opening in VLC: {}", path);

    let file_path = Path::new(&path);

    // Verify the file actually exists
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    // Canonicalize path to get a clean absolute path
    let canonical = file_path.canonicalize().map_err(|e| format!("Invalid path: {}", e))?;
    // Strip the \\?\ prefix that Windows canonicalize adds — VLC can't handle it
    let canonical_str = canonical
        .to_string_lossy()
        .strip_prefix(r"\\?\")
        .unwrap_or(&canonical.to_string_lossy())
        .to_string();

    log::info!("[Player] Canonical path: {}", canonical_str);

    // Try common VLC paths on Windows
    let vlc_paths = [
        r"C:\Program Files\VideoLAN\VLC\vlc.exe",
        r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe",
        "vlc",
    ];

    for vlc in &vlc_paths {
        match Command::new(vlc)
            .arg(&canonical_str)
            .spawn()
        {
            Ok(_) => {
                log::info!("[Player] VLC launched successfully with '{}'", vlc);
                return Ok(());
            }
            Err(e) => {
                log::debug!("[Player] Failed with '{}': {}", vlc, e);
                continue;
            }
        }
    }

    Err("VLC is not installed or was not found. Please install VLC media player.".to_string())
}
