//! "A new TUNAS is ready": a quiet check against S.A.L.T's download address (get.saltci.app).
//!
//! The check sends nothing about the person: only an HTTPS request for `latest.json`. That file names
//! the newest version and, per platform, its installer with its size and SHA-256. A download is kept
//! only if its SHA-256 matches. Then Windows runs the installer, a Mac opens the disk image, and
//! Android opens its own "Install this update?" screen.

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub const DEFAULT_SOURCE: &str = "https://get.saltci.app/tunas/latest.json";

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct UpdFile {
    pub name: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Deserialize, Debug, Default)]
struct Latest {
    version: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    files: std::collections::HashMap<String, UpdFile>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub available: bool,
    pub date: String,
    pub notes_url: String,
    pub file: Option<UpdFile>,
    pub error: String,
}

/// Which installer this copy of TUNAS needs.
pub fn platform_key() -> &'static str {
    if cfg!(target_os = "android") {
        "android"
    } else if cfg!(windows) {
        "windows"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "mac_apple_silicon"
    } else if cfg!(target_os = "macos") {
        "mac_intel"
    } else {
        "linux"
    }
}

fn parse_version(v: &str) -> Vec<u64> {
    v.trim().trim_start_matches('v').split(['.', '-', '+']).take(3).map(|p| p.parse().unwrap_or(0)).collect()
}

pub fn newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

fn resolve(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    reqwest::Url::parse(base).and_then(|b| b.join(url)).map(|u| u.to_string()).unwrap_or_else(|_| url.to_string())
}

pub async fn check(source: &str) -> UpdateInfo {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let mut info = UpdateInfo { current: current.clone(), ..Default::default() };
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent(format!("TUNAS/{current} ({})", platform_key()))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            info.error = e.to_string();
            return info;
        }
    };
    let latest: Latest = match client.get(source).send().await {
        Ok(r) if r.status().is_success() => match r.json().await {
            Ok(l) => l,
            Err(_) => {
                info.error = "The update information could not be read.".into();
                return info;
            }
        },
        Ok(r) => {
            info.error = format!("The update address answered {}.", r.status().as_u16());
            return info;
        }
        Err(_) => {
            info.error = "Could not reach the update address. Check the internet connection.".into();
            return info;
        }
    };
    info.latest = latest.version.clone();
    info.date = latest.date;
    info.notes_url = if latest.notes.is_empty() { String::new() } else { resolve(source, &latest.notes) };
    if let Some(f) = latest.files.get(platform_key()) {
        let mut f = f.clone();
        f.url = resolve(source, &f.url);
        info.available = newer(&latest.version, &current) && !f.sha256.is_empty();
        info.file = Some(f);
    }
    info
}

pub fn updates_dir() -> PathBuf {
    let p = crate::paths::data_dir().join("updates");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Download the installer, check its SHA-256, and return where it is.
pub async fn download(app: &AppHandle, f: &UpdFile) -> Result<PathBuf> {
    if !f.url.starts_with("https://") && !f.url.starts_with("http://") {
        return Err(anyhow!("Not a download address."));
    }
    let name: String = f.name.chars().filter(|c| c.is_ascii_alphanumeric() || "._-".contains(*c)).collect();
    if name.is_empty() {
        return Err(anyhow!("The update has no file name."));
    }
    let dir = updates_dir();
    // Only the update being fetched is kept.
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy() != name {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let dest = dir.join(&name);
    if dest.exists() && sha256_file(&dest).map(|h| h.eq_ignore_ascii_case(&f.sha256)).unwrap_or(false) {
        return Ok(dest);
    }
    let part = dir.join(format!("{name}.part"));
    let client = reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(20)).build()?;
    let resp = client.get(&f.url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("The download answered {}.", resp.status().as_u16()));
    }
    let total = resp.content_length().unwrap_or(f.size);
    let mut out = std::fs::File::create(&part)?;
    let mut hasher = Sha256::new();
    let mut done: u64 = 0;
    let mut last = std::time::Instant::now();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        out.write_all(&chunk)?;
        hasher.update(&chunk);
        done += chunk.len() as u64;
        if last.elapsed().as_millis() > 300 {
            last = std::time::Instant::now();
            let _ = app.emit("update-progress", serde_json::json!({ "done": done, "total": total }));
        }
    }
    out.flush()?;
    drop(out);
    let got = hex::encode(hasher.finalize());
    if !got.eq_ignore_ascii_case(&f.sha256) {
        let _ = std::fs::remove_file(&part);
        return Err(anyhow!("The downloaded file did not match its fingerprint, so it was not kept. Try again later."));
    }
    std::fs::rename(&part, &dest)?;
    let _ = app.emit("update-progress", serde_json::json!({ "done": total, "total": total }));
    Ok(dest)
}

fn sha256_file(p: &std::path::Path) -> Result<String> {
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha256::new();
    std::io::copy(&mut f, &mut h)?;
    Ok(hex::encode(h.finalize()))
}

/// Start the installer that was downloaded (Windows and macOS; Android installs from the page).
pub fn install(app: &AppHandle, path: &std::path::Path) -> Result<()> {
    if !path.starts_with(updates_dir()) || !path.exists() {
        return Err(anyhow!("Not a TUNAS update."));
    }
    #[cfg(windows)]
    {
        // The installer closes TUNAS itself, then installs over it.
        std::process::Command::new(path).spawn()?;
        let a = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(800));
            a.exit(0);
        });
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        tauri_plugin_opener::open_path(path.to_string_lossy().to_string(), None::<&str>).map_err(|e| anyhow!("{e}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        let _ = app;
        Err(anyhow!("Install it from the file that was downloaded."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions() {
        assert!(newer("0.6.1", "0.6.0"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(!newer("0.6.0", "0.6.0"));
        assert!(!newer("0.5.9", "0.6.0"));
        assert_eq!(resolve("https://get.saltci.app/tunas/latest.json", "0.6.0/x.exe"), "https://get.saltci.app/tunas/0.6.0/x.exe");
    }
}
