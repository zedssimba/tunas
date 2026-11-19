//! Model catalogue, downloads (with resume and checksum), offline import and custom models.

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CatalogModel {
    pub id: String,
    pub file: String,
    pub name: String,
    pub family: String,
    pub params: String,
    pub quant: String,
    pub tier: String,
    pub role: String,
    pub bundle: bool,
    pub min_ram_gb: f64,
    pub size: u64,
    pub sha256: String,
    pub license: String,
    #[serde(default)]
    pub license_note: String,
    pub description: String,
    pub urls: Vec<String>,
    pub source_page: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Catalog {
    pub version: u32,
    pub updated: String,
    pub models: Vec<CatalogModel>,
}

pub fn catalog() -> Catalog {
    let p = crate::paths::app_dir().join("resources").join("catalog.json");
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|_| include_str!("../resources/catalog.json").to_string());
    serde_json::from_str(&txt).unwrap_or(Catalog { version: 0, updated: String::new(), models: vec![] })
}

pub fn find(id: &str) -> Option<CatalogModel> {
    catalog().models.into_iter().find(|m| m.id == id)
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Progress {
    pub id: String,
    pub phase: String, // downloading | verifying | copying | done | error | cancelled
    pub received: u64,
    pub total: u64,
    pub bytes_per_sec: f64,
    pub message: String,
}

#[derive(Clone, Default)]
pub struct Transfers {
    cancelled: Arc<Mutex<HashSet<String>>>,
    active: Arc<Mutex<HashSet<String>>>,
}

impl Transfers {
    pub fn cancel(&self, id: &str) {
        self.cancelled.lock().insert(id.to_string());
    }
    fn is_cancelled(&self, id: &str) -> bool {
        self.cancelled.lock().contains(id)
    }
    pub fn active(&self) -> Vec<String> {
        self.active.lock().iter().cloned().collect()
    }
}

fn emit(app: &AppHandle, p: Progress) {
    let _ = app.emit("model-progress", p);
}

fn register(conn: &rusqlite::Connection, m: &CatalogModel) -> Result<()> {
    crate::db::add_model(
        conn,
        &crate::db::InstalledModel {
            id: m.id.clone(),
            file: m.file.clone(),
            name: m.name.clone(),
            source: "catalog".into(),
            size: m.size as i64,
            sha256: Some(m.sha256.clone()),
            installed_at: crate::db::now(),
        },
    )
}

/// Download a catalogue model, trying each mirror in turn. Resumes a partial download.
pub async fn download(app: AppHandle, db: Arc<Mutex<rusqlite::Connection>>, tr: Transfers, id: String) -> Result<()> {
    let m = find(&id).ok_or_else(|| anyhow!("unknown model {id}"))?;
    if !tr.active.lock().insert(id.clone()) {
        return Ok(()); // already running
    }
    tr.cancelled.lock().remove(&id);
    let result = download_inner(&app, &db, &tr, &m).await;
    tr.active.lock().remove(&id);
    match &result {
        Ok(()) => emit(&app, Progress { id: id.clone(), phase: "done".into(), received: m.size, total: m.size, ..Default::default() }),
        Err(e) if tr.is_cancelled(&id) => {
            let _ = e;
            emit(&app, Progress { id: id.clone(), phase: "cancelled".into(), total: m.size, ..Default::default() })
        }
        Err(e) => emit(&app, Progress { id: id.clone(), phase: "error".into(), total: m.size, message: e.to_string(), ..Default::default() }),
    }
    result
}

async fn download_inner(app: &AppHandle, db: &Arc<Mutex<rusqlite::Connection>>, tr: &Transfers, m: &CatalogModel) -> Result<()> {
    let dest = crate::paths::models_dir().join(&m.file);
    let part = dest.with_extension("gguf.part");
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("TUNAS/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut last_err = anyhow!("no download source answered");
    'mirrors: for url in &m.urls {
        let have = std::fs::metadata(&part).map(|x| x.len()).unwrap_or(0);
        let mut req = client.get(url);
        if have > 0 && have < m.size {
            req = req.header("Range", format!("bytes={have}-"));
        }
        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = anyhow!("{url}: {e}");
                continue;
            }
        };
        let status = resp.status();
        if !(status.is_success()) {
            last_err = anyhow!("{url}: HTTP {status}");
            continue;
        }
        let resuming = status.as_u16() == 206;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resuming)
            .truncate(!resuming)
            .open(&part)?;
        let mut received = if resuming { have } else { 0 };
        let mut stream = resp.bytes_stream();
        let t0 = Instant::now();
        let start_bytes = received;
        let mut last_emit = Instant::now();
        while let Some(chunk) = stream.next().await {
            if tr.is_cancelled(&m.id) {
                return Err(anyhow!("cancelled"));
            }
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    last_err = anyhow!("download interrupted: {e}");
                    continue 'mirrors;
                }
            };
            file.write_all(&chunk)?;
            received += chunk.len() as u64;
            if last_emit.elapsed().as_millis() > 250 {
                last_emit = Instant::now();
                let secs = t0.elapsed().as_secs_f64().max(0.001);
                emit(app, Progress {
                    id: m.id.clone(),
                    phase: "downloading".into(),
                    received,
                    total: m.size,
                    bytes_per_sec: (received - start_bytes) as f64 / secs,
                    message: url_host(url),
                });
            }
        }
        file.flush()?;
        drop(file);
        if received < m.size {
            last_err = anyhow!("download ended early");
            continue;
        }
        emit(app, Progress { id: m.id.clone(), phase: "verifying".into(), received, total: m.size, ..Default::default() });
        let p2 = part.clone();
        let hash = tokio::task::spawn_blocking(move || sha256_file(&p2)).await??;
        if !hash.eq_ignore_ascii_case(&m.sha256) {
            let _ = std::fs::remove_file(&part);
            last_err = anyhow!("the downloaded file failed its checksum and was removed; please try again");
            continue;
        }
        std::fs::rename(&part, &dest)?;
        register(&db.lock(), m)?;
        return Ok(());
    }
    Err(last_err)
}

fn url_host(u: &str) -> String {
    u.split("//").nth(1).and_then(|s| s.split('/').next()).unwrap_or("").to_string()
}

pub fn sha256_file(p: &Path) -> Result<String> {
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 4 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

fn copy_with_progress(app: &AppHandle, tr: &Transfers, id: &str, src: &Path, dst: &Path) -> Result<()> {
    let total = std::fs::metadata(src)?.len();
    let tmp = dst.with_extension("gguf.part");
    let mut r = std::fs::File::open(src)?;
    let mut w = std::fs::File::create(&tmp)?;
    let mut buf = vec![0u8; 8 << 20];
    let mut done = 0u64;
    let t0 = Instant::now();
    let mut last = Instant::now();
    loop {
        if tr.is_cancelled(id) {
            drop(w);
            let _ = std::fs::remove_file(&tmp);
            return Err(anyhow!("cancelled"));
        }
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        w.write_all(&buf[..n])?;
        done += n as u64;
        if last.elapsed().as_millis() > 250 {
            last = Instant::now();
            emit(app, Progress {
                id: id.into(),
                phase: "copying".into(),
                received: done,
                total,
                bytes_per_sec: done as f64 / t0.elapsed().as_secs_f64().max(0.001),
                message: String::new(),
            });
        }
    }
    w.flush()?;
    drop(w);
    std::fs::rename(&tmp, dst)?;
    Ok(())
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ImportReport {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

fn find_ggufs(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && depth > 0 {
                find_ggufs(&p, depth - 1, out);
            } else if p.extension().map(|x| x.eq_ignore_ascii_case("gguf")).unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

/// Offline source: copy catalogue models found in a folder (USB stick or image), checking each checksum.
pub fn import_folder(app: &AppHandle, db: &Arc<Mutex<rusqlite::Connection>>, tr: &Transfers, dir: &Path) -> Result<ImportReport> {
    let cat = catalog();
    let mut files = vec![];
    find_ggufs(dir, 3, &mut files);
    let mut rep = ImportReport::default();
    for f in files {
        let size = std::fs::metadata(&f).map(|m| m.len()).unwrap_or(0);
        let fname = f.file_name().unwrap_or_default().to_string_lossy().to_string();
        let Some(m) = cat.models.iter().find(|m| m.size > 0 && m.size == size)
        else {
            rep.skipped.push(format!("{fname} (not in the TUNAS catalogue; add it as a custom model instead)"));
            continue;
        };
        let dest = crate::paths::models_dir().join(&m.file);
        if dest.exists() && crate::db::get_model(&db.lock(), &m.id).is_some() {
            rep.skipped.push(format!("{} (already installed)", m.name));
            continue;
        }
        emit(app, Progress { id: m.id.clone(), phase: "verifying".into(), total: m.size, message: fname.clone(), ..Default::default() });
        match sha256_file(&f) {
            Ok(h) if h.eq_ignore_ascii_case(&m.sha256) => {}
            Ok(_) => {
                rep.failed.push(format!("{fname} (checksum does not match {}; the file may be damaged)", m.name));
                continue;
            }
            Err(e) => {
                rep.failed.push(format!("{fname} ({e})"));
                continue;
            }
        }
        match copy_with_progress(app, tr, &m.id, &f, &dest) {
            Ok(()) => {
                register(&db.lock(), m)?;
                emit(app, Progress { id: m.id.clone(), phase: "done".into(), received: m.size, total: m.size, ..Default::default() });
                rep.imported.push(m.name.clone());
            }
            Err(e) => rep.failed.push(format!("{} ({e})", m.name)),
        }
    }
    Ok(rep)
}

/// Advanced: add any GGUF file as a custom model (copied into the models folder).
pub fn add_custom(app: &AppHandle, db: &Arc<Mutex<rusqlite::Connection>>, tr: &Transfers, src: &Path, name: &str) -> Result<crate::db::InstalledModel> {
    let mut magic = [0u8; 4];
    std::fs::File::open(src)?.read_exact(&mut magic)?;
    if &magic != b"GGUF" {
        return Err(anyhow!("this is not a GGUF model file"));
    }
    let fname = src.file_name().unwrap_or_default().to_string_lossy().to_string();
    let id = format!("custom-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let dest = crate::paths::models_dir().join(format!("{id}-{fname}"));
    copy_with_progress(app, tr, &id, src, &dest)?;
    let m = crate::db::InstalledModel {
        id: id.clone(),
        file: dest.file_name().unwrap().to_string_lossy().to_string(),
        name: if name.trim().is_empty() { fname.trim_end_matches(".gguf").to_string() } else { name.trim().to_string() },
        source: "custom".into(),
        size: std::fs::metadata(&dest)?.len() as i64,
        sha256: None,
        installed_at: crate::db::now(),
    };
    crate::db::add_model(&db.lock(), &m)?;
    emit(app, Progress { id, phase: "done".into(), received: m.size as u64, total: m.size as u64, ..Default::default() });
    Ok(m)
}

pub fn model_path(file: &str) -> PathBuf {
    crate::paths::models_dir().join(file)
}

pub fn remove(db: &Arc<Mutex<rusqlite::Connection>>, id: &str) -> Result<()> {
    let c = db.lock();
    if let Some(m) = crate::db::get_model(&c, id) {
        let _ = std::fs::remove_file(model_path(&m.file));
    }
    crate::db::remove_model(&c, id)?;
    // Assistants that used it fall back to the default model.
    c.execute("UPDATE assistants SET model_id='' WHERE model_id=?1", [id])?;
    Ok(())
}

/// Copy installed catalogue models to a folder, for preparing a USB stick or an image.
pub fn export_to_folder(db: &Arc<Mutex<rusqlite::Connection>>, dir: &Path) -> Result<Vec<String>> {
    let models = crate::db::list_models(&db.lock())?;
    let mut done = vec![];
    for m in models.into_iter().filter(|m| m.source == "catalog") {
        let src = model_path(&m.file);
        let dst = dir.join(&m.file);
        if !dst.exists() {
            std::fs::copy(&src, &dst)?;
        }
        done.push(m.name);
    }
    Ok(done)
}
