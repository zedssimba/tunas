//! Where TUNAS keeps things on the machine.
//!
//! - Local data (big, per user, not roaming): %LOCALAPPDATA%\TUNAS
//!   models\, tunas.db, logs\
//! - Personal brand (per user): %LOCALAPPDATA%\TUNAS\brand\personal
//! - Machine-wide (OEM / organisation image, written by the installer or IT):
//!   %ProgramData%\TUNAS\brand\{oem,org}, %ProgramData%\TUNAS\policy.json
//!   (macOS: /Library/Application Support/TUNAS; data in ~/Library/Application Support/TUNAS)
//! - App resources (next to the exe): engine\cpu, engine\vulkan, catalog.json

use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    let base = android_files_dir();
    #[cfg(not(target_os = "android"))]
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
    // Android has no machine-wide folder an IT department can write to; a brand comes from a pack.
    #[cfg(target_os = "android")]
    {
        data_dir().join("machine")
    }
    #[cfg(windows)]
    {
        let pd = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
        PathBuf::from(pd).join("TUNAS")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/TUNAS")
    }
    #[cfg(all(not(windows), not(target_os = "macos"), not(target_os = "android")))]
    {
        PathBuf::from("/etc/tunas")
    }
}

/// Android: the app's private files folder (/data/data/<package>/files). The process name is the package.
#[cfg(target_os = "android")]
fn android_files_dir() -> PathBuf {
    let pkg = std::fs::read("/proc/self/cmdline")
        .ok()
        .map(|b| String::from_utf8_lossy(b.split(|c| *c == 0).next().unwrap_or_default()).to_string())
        .filter(|s| !s.is_empty() && !s.contains('/'))
        .unwrap_or_else(|| "ci.salt.tunas".into());
    let pkg = pkg.split(':').next().unwrap_or("ci.salt.tunas").to_string();
    PathBuf::from(format!("/data/data/{pkg}/files"))
}

/// Android: the folder holding the app's native libraries, where the engine is installed as
/// libllama_server.so (Android only lets an app run programs from there). Found from where TUNAS's own
/// library is loaded.
#[cfg(target_os = "android")]
pub fn android_lib_dir() -> PathBuf {
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            if let Some(i) = line.find('/') {
                let path = &line[i..];
                if path.ends_with("/libtunas_lib.so") {
                    return PathBuf::from(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
                }
            }
        }
    }
    PathBuf::from(".")
}

/// Folder holding the exe (resources are installed next to it).
pub fn app_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        return android_lib_dir();
    }
    #[allow(unreachable_code)]
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    // In a macOS app bundle the program is in Contents/MacOS and its files in Contents/Resources.
    #[cfg(target_os = "macos")]
    {
        if dir.file_name().map(|n| n == "MacOS").unwrap_or(false) {
            if let Some(res) = dir.parent().map(|c| c.join("Resources")) {
                if res.exists() {
                    return res;
                }
            }
        }
    }
    dir
}

pub fn engine_dir(kind: &str) -> PathBuf {
    // Each version has its own engine folder, so a stuck old engine process can never block an update.
    app_dir().join("engine").join(env!("CARGO_PKG_VERSION")).join(kind)
}
