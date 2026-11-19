//! Where TUNAS keeps things on the notebook.
//!
//! - Local data (big, per user, not roaming): %LOCALAPPDATA%\TUNAS
//!   models\, tunas.db, logs\
//! - Personal brand (per user): %LOCALAPPDATA%\TUNAS\brand\personal
//! - Machine-wide (OEM / organisation image, written by the installer or IT):
//!   %ProgramData%\TUNAS\brand\{oem,org}, %ProgramData%\TUNAS\policy.json
//! - App resources (next to the exe): engine\cpu, engine\vulkan, catalog.json

use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let p = base.join("TUNAS");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn models_dir() -> PathBuf {
    let p = data_dir().join("models");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn logs_dir() -> PathBuf {
    let p = data_dir().join("logs");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn db_path() -> PathBuf {
    data_dir().join("tunas.db")
}

pub fn user_brand_root() -> PathBuf {
    let p = data_dir().join("brand");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn machine_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let pd = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
        PathBuf::from(pd).join("TUNAS")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/etc/tunas")
    }
}

/// Folder holding the exe (resources are installed next to it).
pub fn app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn engine_dir(kind: &str) -> PathBuf {
    // Each version has its own engine folder, so a stuck old engine process can never block an update.
    app_dir().join("engine").join(env!("CARGO_PKG_VERSION")).join(kind)
}
