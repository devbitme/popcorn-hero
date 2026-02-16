fn main() {
    // Load .env file from the workspace root so that option_env!() picks up
    // variables like TMDB_API_KEY at compile time.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let dotenv_path = workspace_root.join(".env");

    if dotenv_path.exists() {
        println!("cargo:rerun-if-changed={}", dotenv_path.display());

        if let Ok(contents) = std::fs::read_to_string(&dotenv_path) {
            for line in contents.lines() {
                let line = line.trim();
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    // Only set if not already set by the environment (env vars take precedence)
                    if std::env::var(key).is_err() {
                        println!("cargo:rustc-env={}={}", key, value);
                    }
                }
            }
        }
    }

    tauri_build::build()
}
