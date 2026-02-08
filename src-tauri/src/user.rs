use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

/// Info returned by check_user_exists
#[derive(Serialize, Deserialize, Clone)]
pub struct ExistingUser {
    pub id: String,
    pub username: String,
}

/// Derive a 256-bit key from a 4-digit PIN using SHA-256
fn derive_key(pin: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(b"popcorn-hero-salt-2026");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Encrypt data using AES-256-GCM
fn encrypt(data: &[u8], pin: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(pin);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data).map_err(|e| e.to_string())?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data using AES-256-GCM
fn decrypt(data: &[u8], pin: &str) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }

    let key = derive_key(pin);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
}

/// Get the users directory path (next to logs)
fn get_users_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_local_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(app_local_data.join("users"))
}

/// Get a specific user's directory by UUID
fn get_user_dir(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    Ok(get_users_dir(app)?.join(user_id))
}

#[tauri::command]
pub fn check_user_exists(app: AppHandle) -> Result<Option<ExistingUser>, String> {
    let users_dir = get_users_dir(&app)?;

    if !users_dir.exists() {
        return Ok(None);
    }

    // Look for the first user directory (UUID-named)
    let entries = fs::read_dir(&users_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            // Try to read the username from the unencrypted metadata file
            let meta_path = entry.path().join("meta.json");
            if meta_path.exists() {
                let meta_raw = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_raw) {
                    if let (Some(id), Some(username)) =
                        (meta["id"].as_str(), meta["username"].as_str())
                    {
                        return Ok(Some(ExistingUser {
                            id: id.to_string(),
                            username: username.to_string(),
                        }));
                    }
                }
            }
        }
    }

    Ok(None)
}

#[tauri::command]
pub fn create_user(app: AppHandle, username: String, pin: String) -> Result<UserProfile, String> {
    // Validate username: no spaces, no special chars except - and _
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Username can only contain letters, numbers, hyphens and underscores".to_string());
    }

    // Validate PIN: exactly 4 digits
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN must be exactly 4 digits".to_string());
    }

    let user_id = Uuid::new_v4().to_string();
    let user_dir = get_user_dir(&app, &user_id)?;

    if user_dir.exists() {
        return Err("User already exists".to_string());
    }

    // Check no user exists yet (single-user app)
    let users_dir = get_users_dir(&app)?;
    if users_dir.exists() {
        let entries = fs::read_dir(&users_dir).map_err(|e| e.to_string())?;
        if entries.filter_map(|e| e.ok()).any(|e| e.path().is_dir()) {
            return Err("User already exists".to_string());
        }
    }

    // Create directories
    fs::create_dir_all(&user_dir).map_err(|e| e.to_string())?;

    let profile = UserProfile {
        id: user_id.clone(),
        username: username.clone(),
        created_at: chrono::Local::now().to_rfc3339(),
    };

    // Serialize and encrypt the profile
    let profile_json = serde_json::to_string(&profile).map_err(|e| e.to_string())?;
    let encrypted = encrypt(profile_json.as_bytes(), &pin)?;

    // Write the encrypted profile
    let profile_path = user_dir.join("profile.enc");
    fs::write(&profile_path, &encrypted).map_err(|e| e.to_string())?;

    // Write unencrypted metadata (id + username) for discovery
    let meta = serde_json::json!({ "id": user_id, "username": username });
    let meta_path = user_dir.join("meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).map_err(|e| e.to_string())?;

    // Store a PIN hash for verification (not the PIN itself)
    let pin_hash = derive_key(&pin);
    let hash_path = user_dir.join("pin.hash");
    fs::write(&hash_path, &pin_hash).map_err(|e| e.to_string())?;

    log::info!("[User] Created user: {} ({})", username, user_id);

    Ok(profile)
}

#[tauri::command]
pub fn verify_pin(app: AppHandle, user_id: String, pin: String) -> Result<UserProfile, String> {
    let user_dir = get_user_dir(&app, &user_id)?;

    if !user_dir.exists() {
        return Err("User not found".to_string());
    }

    // Verify PIN hash
    let hash_path = user_dir.join("pin.hash");
    let stored_hash = fs::read(&hash_path).map_err(|e| e.to_string())?;
    let provided_hash = derive_key(&pin);

    if stored_hash != provided_hash {
        return Err("Invalid PIN".to_string());
    }

    // Decrypt and return profile
    let profile_path = user_dir.join("profile.enc");
    let encrypted = fs::read(&profile_path).map_err(|e| e.to_string())?;
    let decrypted = decrypt(&encrypted, &pin)?;
    let profile: UserProfile =
        serde_json::from_slice(&decrypted).map_err(|e| e.to_string())?;

    log::info!("[User] PIN verified for user: {}", user_id);

    Ok(profile)
}
