//! Files the user picks or saves through the system's file dialogs.
//!
//! On Windows, macOS and Linux these are ordinary paths. On Android the dialogs return `content://`
//! addresses instead: a picked file is copied in (with a name that keeps its type, so it is read the
//! right way), and a saved file is written to a temporary file first and then copied out.

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::PathBuf;
use tauri::AppHandle;

pub fn is_content(p: &str) -> bool {
    p.starts_with("content://")
}

/// A file the user picked, as a path TUNAS can read.
pub fn local_in(app: &AppHandle, p: &str) -> Result<PathBuf> {
    if !is_content(p) {
        return Ok(PathBuf::from(p.strip_prefix("file://").unwrap_or(p)));
    }
    use tauri_plugin_fs::FsExt;
    let fp: tauri_plugin_fs::FilePath = p.parse().map_err(|_| anyhow!("Could not open the file."))?;
    let mut src = app.fs().open(fp, tauri_plugin_fs::OpenOptions::new()).map_err(|e| anyhow!("Could not open the file: {e}"))?;
    // Copied in a stream (a model file can be several GB), then named from its address or its first bytes.
    let dir = crate::paths::data_dir().join("inbox").join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&dir)?;
    let tmp = dir.join("incoming");
    {
        let mut dst = std::fs::File::create(&tmp)?;
        std::io::copy(&mut src, &mut dst)?;
    }
    let mut head = vec![0u8; 64 * 1024];
    let n = {
        use std::io::Read;
        let mut f = std::fs::File::open(&tmp)?;
        f.read(&mut head)?
    };
    head.truncate(n);
    let out = dir.join(display_name(p, &head));
    std::fs::rename(&tmp, &out)?;
    Ok(out)
}

/// Where to write a file the user chose to save: the path itself, or a temporary file for `content://`.
pub struct Out {
    pub path: PathBuf,
    uri: Option<String>,
}

pub fn local_out(p: &str, ext: &str) -> Out {
    if !is_content(p) {
        return Out { path: PathBuf::from(p.strip_prefix("file://").unwrap_or(p)), uri: None };
    }
    let dir = crate::paths::data_dir().join("outbox");
    let _ = std::fs::create_dir_all(&dir);
    let ext = if ext.is_empty() { "bin".to_string() } else { ext.trim_start_matches('.').to_string() };
    Out { path: dir.join(format!("{}.{ext}", uuid::Uuid::new_v4().simple())), uri: Some(p.to_string()) }
}

/// Copy a temporary file out to the `content://` address the user chose (nothing to do for a path).
pub fn finish_out(app: &AppHandle, out: &Out) -> Result<()> {
    let Some(uri) = &out.uri else { return Ok(()) };
    use tauri_plugin_fs::FsExt;
    let bytes = std::fs::read(&out.path)?;
    let fp: tauri_plugin_fs::FilePath = uri.parse().map_err(|_| anyhow!("Could not save the file."))?;
    let mut o = tauri_plugin_fs::OpenOptions::new();
    o.read(false).write(true).truncate(true).create(true);
    let mut f = app.fs().open(fp, o).map_err(|e| anyhow!("Could not save the file: {e}"))?;
    f.write_all(&bytes)?;
    f.flush()?;
    let _ = std::fs::remove_file(&out.path);
    Ok(())
}

/// A readable name for a picked file: from the address when it carries one ("primary:Download/Report.pdf"),
/// otherwise from what the file holds.
fn display_name(uri: &str, bytes: &[u8]) -> String {
    let last = uri.rsplit('/').next().unwrap_or("");
    let decoded = percent_decode(last);
    let tail = decoded.rsplit(['/', ':']).next().unwrap_or("").trim().to_string();
    let has_ext = tail.rsplit_once('.').map(|(a, e)| !a.is_empty() && (1..=10).contains(&e.len()) && e.chars().all(|c| c.is_ascii_alphanumeric())).unwrap_or(false);
    let safe: String = tail.chars().filter(|c| !"\\/:*?\"<>|".contains(*c)).collect();
    if has_ext && !safe.is_empty() {
        return safe;
    }
    let stem = if safe.is_empty() { "file".to_string() } else { safe };
    format!("{stem}.{}", sniff_ext(bytes))
}

fn sniff_ext(b: &[u8]) -> &'static str {
    let starts = |m: &[u8]| b.len() >= m.len() && &b[..m.len()] == m;
    if starts(b"%PDF") {
        return "pdf";
    }
    if starts(b"GGUF") {
        return "gguf";
    }
    if starts(b"\x89PNG") {
        return "png";
    }
    if starts(&[0xFF, 0xD8, 0xFF]) {
        return "jpg";
    }
    if starts(b"PK") {
        let head = &b[..b.len().min(64 * 1024)];
        let has = |s: &[u8]| head.windows(s.len()).any(|w| w == s);
        if has(b"brand.json") {
            return "tunasbrand";
        }
        if has(b"word/") {
            return "docx";
        }
        if has(b"xl/") {
            return "xlsx";
        }
        if has(b"ppt/") {
            return "pptx";
        }
        return "zip";
    }
    let text = String::from_utf8_lossy(&b[..b.len().min(4096)]).to_lowercase();
    if text.contains("<svg") {
        return "svg";
    }
    if text.contains("<html") || text.contains("<!doctype html") {
        return "html";
    }
    "txt"
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_from_content_addresses() {
        assert_eq!(display_name("content://com.android.externalstorage.documents/document/primary%3ADownload%2FLaporan%20Q3.pdf", b"%PDF-1.7"), "Laporan Q3.pdf");
        assert_eq!(display_name("content://com.android.providers.downloads.documents/document/msf%3A1234", b"%PDF-1.7"), "1234.pdf");
        assert_eq!(display_name("content://x/document/msf%3A55", b"PK\x03\x04....word/document.xml"), "55.docx");
    }
}
