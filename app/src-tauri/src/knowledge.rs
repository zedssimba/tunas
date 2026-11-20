//! Knowledge: the user's own files, read where they are (never copied into TUNAS) and indexed on this
//! machine so assistants can answer from them.
//!
//! - **Folders TUNAS can read** (`grants`): the user allows a folder or file once, with a clear consent
//!   step in the app. Knowledge only reads inside allowed places, and removing an allowance removes its index.
//! - **Knowledge sets** (`ksets`): a name plus allowed folders or files. Several assistants can share one.
//! - **Index**: text pieces with the page they came from, a meaning vector (Qwen3 Embedding 0.6B, run by
//!   a second, small engine process on the processor) and a keyword index (SQLite FTS5). Search mixes both,
//!   so exact reference numbers and names are found as well as meaning.
//! - **Light on the machine**: indexing runs one file at a time at low priority and waits while a chat
//!   answer is being written.

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub const EMBED_MODEL: &str = "qwen3-embed-0.6b";
/// Vectors are cut to 512 numbers (the model is trained for this) to halve the index size.
const DIM: usize = 512;
const CHUNK_CHARS: usize = 1200;
const OVERLAP_CHARS: usize = 200;
const MAX_FILES_PER_SET: usize = 20_000;
const EMBED_BATCH: usize = 16;
const QUERY_PREFIX: &str = "Instruct: Given a question, retrieve passages from the user's documents that answer it\nQuery: ";

// ---------------------------------------------------------------- log

/// A short record of what reading does (logs\\knowledge.log), so a slow or stuck reading explains itself.
pub fn klog(msg: &str) {
    use std::io::Write;
    let path = crate::paths::logs_dir().join("knowledge.log");
    // Keep the log small: start again when it passes 1 MB.
    if std::fs::metadata(&path).map(|m| m.len() > 1 << 20).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg);
    }
}

// ---------------------------------------------------------------- chat activity (indexing yields to chat)

pub static CHATS_ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// Held while a chat answer is being written; indexing waits until none are.
pub struct ChatGuard;
impl ChatGuard {
    pub fn new() -> Self {
        CHATS_ACTIVE.fetch_add(1, Ordering::SeqCst);
        ChatGuard
    }
}
impl Drop for ChatGuard {
    fn drop(&mut self) {
        CHATS_ACTIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------- types

#[derive(Serialize, Clone, Debug)]
pub struct Grant {
    pub path: String,
    pub is_dir: bool,
    pub exists: bool,
    pub created: String,
    /// Knowledge sets reading from here.
    pub used_by: Vec<String>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct SetInfo {
    pub id: String,
    pub name: String,
    pub sources: Vec<String>,
    pub files: i64,
    pub done: i64,
    pub pending: i64,
    pub failed: i64,
    pub chunks: i64,
    pub bytes: i64,
    pub paused: bool,
    /// ready | indexing | waiting_model | paused | empty
    pub state: String,
    pub current: String,
    pub assistants: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FailedFile {
    pub path: String,
    pub error: String,
}

/// Similarity by meaning above which a piece is about the question on its own. Measured with the
/// Qwen3 embedding model: greetings and unrelated requests scored 0.17-0.47, document questions 0.55-0.76.
const MEANING_STRONG: f32 = 0.56;
/// Similarity by meaning that is enough when content words also match.
const MEANING_WITH_WORDS: f32 = 0.50;
/// Similarity by meaning that is enough when a reference or version number also matches.
const MEANING_WITH_IDENTIFIER: f32 = 0.40;
/// Common words that say nothing about a document (English and Malay), plus greetings.
const STOPWORDS: &[&str] = &[
    "the", "and", "for", "you", "your", "are", "was", "what", "how", "who", "why", "when", "where", "which", "this", "that",
    "with", "from", "have", "has", "can", "could", "would", "should", "please", "about", "tell", "give", "any", "all", "our",
    "hello", "thanks", "thank", "good", "morning", "afternoon", "evening", "night", "okay", "yes", "hey", "hai", "helo",
    "salam", "assalamualaikum", "waalaikumsalam", "selamat", "pagi", "petang", "malam", "tengahari", "terima", "kasih", "apa",
    "khabar", "baik", "yang", "dan", "untuk", "dengan", "ini", "itu", "ada", "boleh", "saya", "kami", "kita", "anda", "awak",
    "dia", "mereka", "adalah", "dalam", "pada", "kepada", "dari", "daripada", "tidak", "tak", "bagaimana", "kenapa", "mengapa",
    "siapa", "bila", "mana", "berapa", "tolong", "sila", "nak", "mahu", "ke", "lah", "pun", "juga", "sahaja", "saja",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hit {
    pub n: usize,
    pub name: String,
    pub path: String,
    pub label: String,
    pub page: u32,
    pub set: String,
    pub text: String,
    pub score: f32,
}

struct EmbedServer {
    child: std::process::Child,
    port: u16,
    key: String,
    last_used: Instant,
}

#[derive(Clone)]
pub struct Knowledge {
    db: Arc<Mutex<Connection>>,
    embed: Arc<tokio::sync::Mutex<Option<EmbedServer>>>,
    wake: Arc<tokio::sync::Notify>,
    cache: Arc<Mutex<HashMap<String, Arc<Vec<(i64, Vec<f32>)>>>>>,
    current: Arc<Mutex<HashMap<String, String>>>,
}

// ---------------------------------------------------------------- database

fn open_db() -> Result<Connection> {
    let path = crate::paths::data_dir().join("knowledge.db");
    let c = Connection::open(path)?;
    c.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        CREATE TABLE IF NOT EXISTS grants (path TEXT PRIMARY KEY, created TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS ksets (id TEXT PRIMARY KEY, name TEXT NOT NULL, created TEXT NOT NULL,
            paused INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS kset_sources (set_id TEXT NOT NULL, path TEXT NOT NULL, PRIMARY KEY(set_id, path));
        CREATE TABLE IF NOT EXISTS assistant_sets (assistant_id TEXT NOT NULL, set_id TEXT NOT NULL,
            PRIMARY KEY(assistant_id, set_id));
        CREATE TABLE IF NOT EXISTS kfiles (id INTEGER PRIMARY KEY, set_id TEXT NOT NULL, path TEXT NOT NULL,
            size INTEGER NOT NULL DEFAULT 0, mtime INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL,
            error TEXT NOT NULL DEFAULT '', chunks INTEGER NOT NULL DEFAULT 0, UNIQUE(set_id, path));
        CREATE INDEX IF NOT EXISTS idx_kfiles_status ON kfiles(status, set_id);
        CREATE TABLE IF NOT EXISTS kchunks (id INTEGER PRIMARY KEY, file_id INTEGER NOT NULL, set_id TEXT NOT NULL,
            page INTEGER NOT NULL DEFAULT 0, label TEXT NOT NULL DEFAULT '', text TEXT NOT NULL, vec BLOB);
        CREATE INDEX IF NOT EXISTS idx_kchunks_set ON kchunks(set_id);
        CREATE INDEX IF NOT EXISTS idx_kchunks_file ON kchunks(file_id);
        CREATE VIRTUAL TABLE IF NOT EXISTS kfts USING fts5(text, tokenize='unicode61 remove_diacritics 2');
        CREATE TABLE IF NOT EXISTS kmeta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        "#,
    )?;
    // Index format 2 (0.3.1): Markdown is read section by section with its headings. Read old Markdown again.
    let v: Option<String> = c.query_row("SELECT value FROM kmeta WHERE key = 'format'", [], |r| r.get(0)).optional()?;
    if v.as_deref() != Some("2") {
        c.execute_batch(
            "DELETE FROM kfts WHERE rowid IN (SELECT c.id FROM kchunks c JOIN kfiles f ON f.id = c.file_id WHERE lower(f.path) LIKE '%.md');
             DELETE FROM kchunks WHERE file_id IN (SELECT id FROM kfiles WHERE lower(path) LIKE '%.md');
             UPDATE kfiles SET status = 'pending', chunks = 0, error = '' WHERE lower(path) LIKE '%.md';
             INSERT OR REPLACE INTO kmeta(key, value) VALUES ('format', '2');",
        )?;
    }
    Ok(c)
}

fn norm_path(p: &str) -> String {
    let s = p.trim().trim_end_matches(['/', '\\']).to_string();
    if s.is_empty() { p.to_string() } else { s }
}

/// True when `path` is the same as, or inside, `base` (case-insensitive on Windows).
fn is_within(path: &str, base: &str) -> bool {
    let (p, b) = if cfg!(windows) { (path.to_lowercase().replace('/', "\\"), base.to_lowercase().replace('/', "\\")) } else { (path.to_string(), base.to_string()) };
    let sep = if cfg!(windows) { '\\' } else { '/' };
    p == b || p.starts_with(&format!("{b}{sep}"))
}

fn file_name(p: &str) -> String {
    Path::new(p).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string())
}

impl Knowledge {
    pub fn open() -> Result<Self> {
        Ok(Self {
            db: Arc::new(Mutex::new(open_db()?)),
            embed: Arc::new(tokio::sync::Mutex::new(None)),
            wake: Arc::new(tokio::sync::Notify::new()),
            cache: Arc::new(Mutex::new(HashMap::new())),
            current: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn kick(&self) {
        self.wake.notify_one();
    }

    // ------------------------------------------------------------ folders TUNAS can read

    pub fn grant(&self, path: &str) -> Result<()> {
        let p = norm_path(path);
        if !Path::new(&p).exists() {
            return Err(anyhow!("{p} was not found"));
        }
        self.db.lock().execute("INSERT OR IGNORE INTO grants(path, created) VALUES (?1, ?2)", params![p, crate::db::now()])?;
        Ok(())
    }

    pub fn is_allowed(&self, path: &str) -> bool {
        let c = self.db.lock();
        let mut st = match c.prepare("SELECT path FROM grants") { Ok(s) => s, Err(_) => return false };
        let rows: Vec<String> = st.query_map([], |r| r.get(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        rows.iter().any(|g| is_within(path, g))
    }

    pub fn grants(&self) -> Result<Vec<Grant>> {
        let c = self.db.lock();
        let mut st = c.prepare("SELECT path, created FROM grants ORDER BY path")?;
        let rows: Vec<(String, String)> = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.flatten().collect();
        let mut st2 = c.prepare("SELECT s.path, k.name FROM kset_sources s JOIN ksets k ON k.id = s.set_id")?;
        let sources: Vec<(String, String)> = st2.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.flatten().collect();
        Ok(rows
            .into_iter()
            .map(|(path, created)| {
                let pb = Path::new(&path);
                let mut used: Vec<String> = sources.iter().filter(|(s, _)| is_within(s, &path)).map(|(_, n)| n.clone()).collect();
                used.sort();
                used.dedup();
                Grant { is_dir: pb.is_dir(), exists: pb.exists(), path, created, used_by: used }
            })
            .collect())
    }

    /// Stop reading a place: it leaves every knowledge set and its index is deleted. The files are untouched.
    pub fn revoke(&self, path: &str) -> Result<()> {
        let p = norm_path(path);
        let sources: Vec<(String, String)> = {
            let c = self.db.lock();
            let mut st = c.prepare("SELECT set_id, path FROM kset_sources")?;
            let v = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.flatten().collect();
            v
        };
        for (set_id, sp) in sources {
            if is_within(&sp, &p) {
                self.remove_source(&set_id, &sp)?;
            }
        }
        self.db.lock().execute("DELETE FROM grants WHERE path = ?1", [&p])?;
        Ok(())
    }

    // ------------------------------------------------------------ knowledge sets

    pub fn create_set(&self, name: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = if name.trim().is_empty() { "My knowledge".to_string() } else { name.trim().to_string() };
        self.db.lock().execute("INSERT INTO ksets(id, name, created) VALUES (?1, ?2, ?3)", params![id, name, crate::db::now()])?;
        Ok(id)
    }

    pub fn rename_set(&self, id: &str, name: &str) -> Result<()> {
        self.db.lock().execute("UPDATE ksets SET name = ?2 WHERE id = ?1", params![id, name.trim()])?;
        Ok(())
    }

    pub fn set_paused(&self, id: &str, paused: bool) -> Result<()> {
        self.db.lock().execute("UPDATE ksets SET paused = ?2 WHERE id = ?1", params![id, paused as i64])?;
        self.kick();
        Ok(())
    }

    pub fn delete_set(&self, id: &str) -> Result<()> {
        {
            let mut c = self.db.lock();
            let tx = c.transaction()?;
            tx.execute("DELETE FROM kfts WHERE rowid IN (SELECT id FROM kchunks WHERE set_id = ?1)", [id])?;
            tx.execute("DELETE FROM kchunks WHERE set_id = ?1", [id])?;
            tx.execute("DELETE FROM kfiles WHERE set_id = ?1", [id])?;
            tx.execute("DELETE FROM kset_sources WHERE set_id = ?1", [id])?;
            tx.execute("DELETE FROM assistant_sets WHERE set_id = ?1", [id])?;
            tx.execute("DELETE FROM ksets WHERE id = ?1", [id])?;
            tx.commit()?;
        }
        self.cache.lock().remove(id);
        Ok(())
    }

    pub fn add_source(&self, set_id: &str, path: &str) -> Result<()> {
        let p = norm_path(path);
        if !self.is_allowed(&p) {
            return Err(anyhow!("TUNAS has not been allowed to read {p}"));
        }
        self.db.lock().execute("INSERT OR IGNORE INTO kset_sources(set_id, path) VALUES (?1, ?2)", params![set_id, p])?;
        self.scan_set(set_id)?;
        self.kick();
        Ok(())
    }

    pub fn remove_source(&self, set_id: &str, path: &str) -> Result<()> {
        let p = norm_path(path);
        self.db.lock().execute("DELETE FROM kset_sources WHERE set_id = ?1 AND path = ?2", params![set_id, p])?;
        self.scan_set(set_id)?;
        Ok(())
    }

    /// Read again: files that changed are indexed again, new ones added, removed ones dropped.
    pub fn rescan(&self, set_id: &str, force: bool) -> Result<()> {
        if force {
            let mut c = self.db.lock();
            let tx = c.transaction()?;
            tx.execute("DELETE FROM kfts WHERE rowid IN (SELECT id FROM kchunks WHERE set_id = ?1)", [set_id])?;
            tx.execute("DELETE FROM kchunks WHERE set_id = ?1", [set_id])?;
            tx.execute("UPDATE kfiles SET status = 'pending', error = '', chunks = 0, mtime = 0 WHERE set_id = ?1", [set_id])?;
            tx.commit()?;
            self.cache.lock().remove(set_id);
        }
        self.scan_set(set_id)?;
        self.kick();
        Ok(())
    }

    pub fn rescan_all(&self) {
        let ids: Vec<String> = {
            let c = self.db.lock();
            let mut st = match c.prepare("SELECT id FROM ksets") { Ok(s) => s, Err(_) => return };
            let v = st.query_map([], |r| r.get(0)).map(|it| it.flatten().collect()).unwrap_or_default();
            v
        };
        for id in ids {
            let _ = self.scan_set(&id);
        }
        self.kick();
    }

    fn scan_set(&self, set_id: &str) -> Result<()> {
        let sources: Vec<String> = {
            let c = self.db.lock();
            let mut st = c.prepare("SELECT path FROM kset_sources WHERE set_id = ?1")?;
            let v = st.query_map([set_id], |r| r.get(0))?.flatten().collect();
            v
        };
        let mut found: HashMap<String, (i64, i64)> = HashMap::new();
        for s in &sources {
            if !self.is_allowed(s) {
                continue;
            }
            walk(Path::new(s), 0, &mut found);
            if found.len() >= MAX_FILES_PER_SET {
                break;
            }
        }
        let existing: Vec<(i64, String, i64, i64)> = {
            let c = self.db.lock();
            let mut st = c.prepare("SELECT id, path, size, mtime FROM kfiles WHERE set_id = ?1")?;
            let v = st.query_map([set_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.flatten().collect();
            v
        };
        let mut c = self.db.lock();
        let tx = c.transaction()?;
        let mut changed = false;
        for (id, path, size, mtime) in &existing {
            match found.remove(path) {
                None => {
                    tx.execute("DELETE FROM kfts WHERE rowid IN (SELECT id FROM kchunks WHERE file_id = ?1)", [id])?;
                    tx.execute("DELETE FROM kchunks WHERE file_id = ?1", [id])?;
                    tx.execute("DELETE FROM kfiles WHERE id = ?1", [id])?;
                    changed = true;
                }
                Some((s, m)) if s != *size || m != *mtime => {
                    tx.execute("DELETE FROM kfts WHERE rowid IN (SELECT id FROM kchunks WHERE file_id = ?1)", [id])?;
                    tx.execute("DELETE FROM kchunks WHERE file_id = ?1", [id])?;
                    tx.execute("UPDATE kfiles SET status = 'pending', error = '', chunks = 0, size = ?2, mtime = ?3 WHERE id = ?1", params![id, s, m])?;
                    changed = true;
                }
                _ => {}
            }
        }
        for (path, (size, mtime)) in found {
            tx.execute(
                "INSERT OR IGNORE INTO kfiles(set_id, path, size, mtime, status) VALUES (?1, ?2, ?3, ?4, 'pending')",
                params![set_id, path, size, mtime],
            )?;
        }
        tx.commit()?;
        if changed {
            self.cache.lock().remove(set_id);
        }
        Ok(())
    }

    /// `assistants`: assistant id → name, from the main database.
    pub fn sets(&self, assistants: &HashMap<String, String>) -> Result<Vec<SetInfo>> {
        let embed_ready = embed_model_path().is_some();
        let c = self.db.lock();
        let mut st = c.prepare("SELECT id, name, paused FROM ksets ORDER BY name")?;
        let base: Vec<(String, String, bool)> = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? != 0)))?.flatten().collect();
        let names = assistants;
        let mut out = vec![];
        for (id, name, paused) in base {
            let mut s = SetInfo { id: id.clone(), name, paused, ..Default::default() };
            let mut st = c.prepare("SELECT path FROM kset_sources WHERE set_id = ?1 ORDER BY path")?;
            s.sources = st.query_map([&id], |r| r.get(0))?.flatten().collect();
            let mut st = c.prepare("SELECT status, COUNT(*), COALESCE(SUM(chunks),0), COALESCE(SUM(size),0) FROM kfiles WHERE set_id = ?1 GROUP BY status")?;
            for row in st.query_map([&id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?)))?.flatten() {
                s.files += row.1;
                s.chunks += row.2;
                s.bytes += row.3;
                match row.0.as_str() {
                    "ok" => s.done += row.1,
                    "error" => s.failed += row.1,
                    _ => s.pending += row.1,
                }
            }
            let mut st = c.prepare("SELECT assistant_id FROM assistant_sets WHERE set_id = ?1")?;
            s.assistants = st.query_map([&id], |r| r.get::<_, String>(0))?.flatten().filter_map(|a| names.get(&a).cloned()).collect();
            s.current = self.current.lock().get(&id).cloned().unwrap_or_default();
            s.state = if s.files == 0 {
                "empty".into()
            } else if s.pending == 0 {
                "ready".into()
            } else if paused {
                "paused".into()
            } else if !embed_ready {
                "waiting_model".into()
            } else {
                "indexing".into()
            };
            out.push(s);
        }
        Ok(out)
    }

    /// Sets in `ids` that are not fully read yet, with how far they are: (name, done, total, waiting_for_model).
    pub fn not_ready(&self, ids: &[String]) -> Vec<(String, i64, i64, bool)> {
        let model = embed_model_path().is_some();
        let c = self.db.lock();
        let mut out = vec![];
        for id in ids {
            let r: Option<(String, i64, i64)> = c
                .query_row(
                    "SELECT k.name, (SELECT COUNT(*) FROM kfiles WHERE set_id = k.id AND status != 'pending'),
                            (SELECT COUNT(*) FROM kfiles WHERE set_id = k.id) FROM ksets k WHERE k.id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()
                .ok()
                .flatten();
            if let Some((name, done, total)) = r {
                if done < total {
                    out.push((name, done, total, !model));
                }
            }
        }
        out
    }

    pub fn failed_files(&self, set_id: &str) -> Result<Vec<FailedFile>> {
        let c = self.db.lock();
        let mut st = c.prepare("SELECT path, error FROM kfiles WHERE set_id = ?1 AND status = 'error' ORDER BY path LIMIT 500")?;
        let v = st.query_map([set_id], |r| Ok(FailedFile { path: r.get(0)?, error: r.get(1)? }))?.flatten().collect();
        Ok(v)
    }

    // ------------------------------------------------------------ assistants and chats

    pub fn assistant_sets(&self, assistant_id: &str) -> Vec<String> {
        let c = self.db.lock();
        let mut st = match c.prepare("SELECT a.set_id FROM assistant_sets a JOIN ksets k ON k.id = a.set_id WHERE a.assistant_id = ?1") {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let v = st.query_map([assistant_id], |r| r.get(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        v
    }

    pub fn set_assistant_sets(&self, assistant_id: &str, set_ids: &[String]) -> Result<()> {
        let mut c = self.db.lock();
        let tx = c.transaction()?;
        tx.execute("DELETE FROM assistant_sets WHERE assistant_id = ?1", [assistant_id])?;
        for s in set_ids {
            tx.execute("INSERT OR IGNORE INTO assistant_sets(assistant_id, set_id) VALUES (?1, ?2)", params![assistant_id, s])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_names(&self) -> HashMap<String, String> {
        let c = self.db.lock();
        let mut st = match c.prepare("SELECT id, name FROM ksets") { Ok(s) => s, Err(_) => return HashMap::new() };
        let v = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map(|it| it.flatten().collect()).unwrap_or_default();
        v
    }

    // ------------------------------------------------------------ search

    /// Find the pieces of the user's documents that best answer `query`, by meaning and by exact words.
    pub async fn search(&self, set_ids: &[String], query: &str, k: usize) -> Vec<Hit> {
        if set_ids.is_empty() || query.trim().is_empty() {
            return vec![];
        }
        let mut ranks: HashMap<i64, f32> = HashMap::new();
        let mut best_meaning: f32 = 0.0;
        let mut meaning_ok = false;
        let mut word_hits = 0usize;
        let mut ident_hits = 0usize;

        // By meaning.
        if let Ok(v) = self.embed_texts(&[format!("{QUERY_PREFIX}{}", query.chars().take(2000).collect::<String>())]).await {
            if let Some(q) = v.first() {
                let mut scored: Vec<(i64, f32)> = vec![];
                for sid in set_ids {
                    let vecs = self.vectors(sid);
                    scored.extend(vecs.iter().map(|(id, e)| (*id, dot(q, e))));
                }
                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                best_meaning = scored.first().map(|x| x.1).unwrap_or(0.0);
                meaning_ok = true;
                for (rank, (id, score)) in scored.into_iter().take(30).enumerate() {
                    if score < 0.2 {
                        break;
                    }
                    *ranks.entry(id).or_default() += 1.0 / (60.0 + rank as f32);
                }
            }
        }

        // By exact words (reference numbers, names).
        // Keep numbers like 0.2.5, 7/2026 and JPA(S)PKPA/7 together; keep any word with a digit even if short.
        let words: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '/' && c != '-' && c != '.')
            .map(|w| w.trim_matches(|c: char| c == '/' || c == '-' || c == '.'))
            .filter(|w| w.chars().count() >= 3 || (w.chars().any(|c| c.is_ascii_digit()) && !w.is_empty()))
            .filter(|w| !STOPWORDS.contains(&w.to_lowercase().as_str()))
            .map(|w| format!("\"{}\"", w.replace('"', "")))
            .take(24)
            .collect();
        if !words.is_empty() {
            let placeholders = set_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT f.rowid FROM kfts f JOIN kchunks c ON c.id = f.rowid WHERE kfts MATCH ? AND c.set_id IN ({placeholders}) ORDER BY bm25(kfts) LIMIT 30"
            );
            let mut args: Vec<String> = vec![words.join(" OR ")];
            args.extend(set_ids.iter().cloned());
            let ids: Vec<i64> = {
                let c = self.db.lock();
                let mut v: Vec<i64> = vec![];
                if let Ok(mut st) = c.prepare(&sql) {
                    if let Ok(rows) = st.query_map(rusqlite::params_from_iter(args.iter()), |r| r.get::<_, i64>(0)) {
                        v = rows.flatten().collect();
                    }
                }
                v
            };
            word_hits = ids.len();
            for (rank, id) in ids.into_iter().enumerate() {
                *ranks.entry(id).or_default() += 1.0 / (60.0 + rank as f32);
            }
            // Identifiers (version and reference numbers: anything with a digit) are a strong signal:
            // a piece that contains them counts double.
            let ident: Vec<String> = words.iter().filter(|w| w.chars().any(|c| c.is_ascii_digit())).cloned().collect();
            if !ident.is_empty() {
                let mut args: Vec<String> = vec![ident.join(" OR ")];
                args.extend(set_ids.iter().cloned());
                let ids: Vec<i64> = {
                    let c = self.db.lock();
                    let mut v: Vec<i64> = vec![];
                    if let Ok(mut st) = c.prepare(&sql) {
                        if let Ok(rows) = st.query_map(rusqlite::params_from_iter(args.iter()), |r| r.get::<_, i64>(0)) {
                            v = rows.flatten().collect();
                        }
                    }
                    v
                };
                ident_hits = ids.len();
                for (rank, id) in ids.into_iter().take(10).enumerate() {
                    *ranks.entry(id).or_default() += 2.0 / (60.0 + rank as f32);
                }
            }
        }

        // Only use the documents when the message is really about them. Greetings and small talk
        // ("hi", "salam", "terima kasih") score low by meaning and match no content words.
        // Without the reading model (not downloaded yet, or it could not start), content words decide.
        let relevant = if meaning_ok {
            best_meaning >= MEANING_STRONG || (ident_hits > 0 && best_meaning >= MEANING_WITH_IDENTIFIER) || (word_hits > 0 && best_meaning >= MEANING_WITH_WORDS)
        } else {
            word_hits > 0 || ident_hits > 0
        };
        klog(&format!(
            "search: meaning {}, word matches {word_hits}, identifier matches {ident_hits}: {} for \"{}\"",
            if meaning_ok { format!("{best_meaning:.3}") } else { "unavailable".into() },
            if relevant { "used" } else { "not relevant" },
            query.chars().take(80).collect::<String>().replace('\n', " ")
        ));
        if !relevant {
            return vec![];
        }
        let mut best: Vec<(i64, f32)> = ranks.into_iter().collect();
        best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        // "Latest / newest / terkini" questions: what is newest is usually at the start of a document
        // (release notes, change logs, minutes). Add the opening pieces of the best-matching file.
        let q = query.to_lowercase();
        let recency = ["latest", "newest", "most recent", "current version", "recent", "terkini", "terbaru", "paling baru", "baru-baru", "semasa"]
            .iter()
            .any(|w| q.contains(w));
        if recency {
            if let Some((top, score)) = best.first().cloned() {
                let opening: Vec<i64> = {
                    let c = self.db.lock();
                    let mut v = vec![];
                    if let Ok(mut st) = c.prepare("SELECT id FROM kchunks WHERE file_id = (SELECT file_id FROM kchunks WHERE id = ?1) ORDER BY id LIMIT 2") {
                        if let Ok(rows) = st.query_map([top], |r| r.get::<_, i64>(0)) {
                            v = rows.flatten().collect();
                        }
                    }
                    v
                };
                for (i, id) in opening.into_iter().enumerate() {
                    best.retain(|(x, _)| *x != id);
                    best.insert(i, (id, score + 0.001));
                }
            }
        }
        best.truncate(k);
        // Give the pieces to the model file by file (best file first), and within a file in the order they
        // appear in it, so order in the document ("newest first", numbered steps) is kept.
        let file_of: HashMap<i64, i64> = {
            let c = self.db.lock();
            best.iter()
                .filter_map(|(id, _)| c.query_row("SELECT file_id FROM kchunks WHERE id = ?1", [id], |r| r.get::<_, i64>(0)).ok().map(|f| (*id, f)))
                .collect()
        };
        let mut file_rank: HashMap<i64, usize> = HashMap::new();
        for (i, (id, _)) in best.iter().enumerate() {
            if let Some(f) = file_of.get(id) {
                file_rank.entry(*f).or_insert(i);
            }
        }
        best.sort_by_key(|(id, _)| (file_of.get(id).and_then(|f| file_rank.get(f)).copied().unwrap_or(usize::MAX), *id));
        let names = self.set_names();
        let c = self.db.lock();
        let mut out = vec![];
        for (id, score) in best {
            let row: Option<(String, String, u32, String, String)> = c
                .query_row(
                    "SELECT c.text, c.label, c.page, f.path, c.set_id FROM kchunks c JOIN kfiles f ON f.id = c.file_id WHERE c.id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )
                .optional()
                .ok()
                .flatten();
            if let Some((text, label, page, path, set_id)) = row {
                out.push(Hit {
                    n: out.len() + 1,
                    name: file_name(&path),
                    path,
                    label,
                    page,
                    set: names.get(&set_id).cloned().unwrap_or_default(),
                    text,
                    score,
                });
            }
        }
        out
    }

    /// The section headings of a file in order (for Markdown and slide decks), so the model sees the whole
    /// structure, e.g. every version in release notes. Empty when the file has fewer than two sections.
    pub fn outline(&self, path: &str) -> Vec<String> {
        let c = self.db.lock();
        let mut st = match c.prepare(
            "SELECT c.label FROM kchunks c JOIN kfiles f ON f.id = c.file_id WHERE f.path = ?1 AND c.label != '' ORDER BY c.id",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let labels: Vec<String> = st.query_map([path], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        let mut out: Vec<String> = vec![];
        for l in labels {
            // Only the top level of a "Section › Subsection" path.
            let top = l.split(" › ").next().unwrap_or(&l).to_string();
            if out.last() != Some(&top) && !out.contains(&top) {
                out.push(top);
            }
        }
        if out.len() < 2 || out.iter().all(|l| l.starts_with("p. ")) { vec![] } else { out.into_iter().take(40).collect() }
    }

    fn vectors(&self, set_id: &str) -> Arc<Vec<(i64, Vec<f32>)>> {
        if let Some(v) = self.cache.lock().get(set_id) {
            return v.clone();
        }
        let v: Vec<(i64, Vec<f32>)> = {
            let c = self.db.lock();
            let mut st = match c.prepare("SELECT id, vec FROM kchunks WHERE set_id = ?1 AND vec IS NOT NULL") {
                Ok(s) => s,
                Err(_) => return Arc::new(vec![]),
            };
            let v = st
                .query_map([set_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))
                .map(|it| it.flatten().map(|(id, b)| (id, from_bytes(&b))).collect())
                .unwrap_or_default();
            v
        };
        let v = Arc::new(v);
        self.cache.lock().insert(set_id.to_string(), v.clone());
        v
    }

    // ------------------------------------------------------------ embedding engine

    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let (port, key) = self.ensure_embedder().await?;
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("http://127.0.0.1:{port}/v1/embeddings"))
            .bearer_auth(&key)
            .json(&json!({ "input": texts }))
            .timeout(Duration::from_secs(300))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("the search engine answered {}", resp.status()));
        }
        let j: serde_json::Value = resp.json().await?;
        let data = j.get("data").and_then(|d| d.as_array()).ok_or_else(|| anyhow!("unexpected answer from the search engine"))?;
        let mut out = vec![];
        for d in data {
            let e: Vec<f32> = d
                .get("embedding")
                .and_then(|e| e.as_array())
                .map(|a| a.iter().take(DIM).map(|x| x.as_f64().unwrap_or(0.0) as f32).collect())
                .unwrap_or_default();
            out.push(normalize(e));
        }
        if let Some(s) = self.embed.lock().await.as_mut() {
            s.last_used = Instant::now();
        }
        Ok(out)
    }

    async fn ensure_embedder(&self) -> Result<(u16, String)> {
        let mut g = self.embed.lock().await;
        if let Some(s) = g.as_mut() {
            if matches!(s.child.try_wait(), Ok(None)) {
                return Ok((s.port, s.key.clone()));
            }
        }
        *g = None;
        let model = embed_model_path().ok_or_else(|| anyhow!("the search model is not downloaded yet"))?;
        // Graphics when the chat engine has run on graphics on this machine; it is many times faster.
        let gpu = crate::paths::data_dir().join("gpu-ok").exists() && crate::engine::server_exe("vulkan").exists();
        let mut kinds = vec![];
        if gpu {
            kinds.push("vulkan");
        }
        kinds.push("cpu");
        let mut last = anyhow!("the search engine could not start");
        for kind in kinds {
            match start_embedder(&model, kind).await {
                Ok(server) => {
                    let (port, key) = (server.port, server.key.clone());
                    klog(&format!("search engine started on {} (port {port})", if kind == "vulkan" { "graphics" } else { "processor" }));
                    *g = Some(server);
                    return Ok((port, key));
                }
                Err(e) => {
                    klog(&format!("search engine could not start on {kind}: {e}"));
                    last = e;
                }
            }
        }
        Err(last)
    }

    pub async fn stop_embedder(&self) {
        if let Some(mut s) = self.embed.lock().await.take() {
            let _ = s.child.kill();
            let _ = s.child.wait();
        }
    }

}

async fn start_embedder(model: &Path, kind: &str) -> Result<EmbedServer> {
    {
        let exe = crate::engine::server_exe(kind);
        if !exe.exists() {
            return Err(anyhow!("the engine is not installed"));
        }
        let port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        let key = uuid::Uuid::new_v4().simple().to_string();
        // Low priority (below), so all processor cores can be used without slowing the user down.
        let threads = sysinfo::System::new().physical_core_count().unwrap_or(4).clamp(1, 16);
        let log = crate::paths::logs_dir().join("engine-search.log");
        let _ = std::fs::remove_file(&log);
        let mut cmd = std::process::Command::new(&exe);
        cmd.current_dir(exe.parent().unwrap())
            .arg("-m").arg(model)
            .args(["--host", "127.0.0.1", "--port", &port.to_string(), "--api-key", &key])
            // One piece at a time (measured: several at once is no faster on the processor).
            .args(["--embedding", "-c", "8192", "-b", "8192", "-ub", "2048", "-np", "1"])
            .args(["-t", &threads.to_string(), "-ngl", if kind == "vulkan" { "99" } else { "0" }, "--no-webui"])
            .arg("--log-file").arg(&log)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS: never slow down what the user is doing.
            cmd.creation_flags(0x0800_0000 | 0x0000_4000);
        }
        let child = cmd.spawn().map_err(|e| anyhow!("could not start the search engine: {e}"))?;
        #[cfg(windows)]
        crate::engine::attach_to_job(&child);
        let mut server = EmbedServer { child, port, key: key.clone(), last_used: Instant::now() };
        let client = reqwest::Client::new();
        let t0 = Instant::now();
        loop {
            if let Ok(Some(code)) = server.child.try_wait() {
                return Err(anyhow!("the search engine stopped while loading (exit {code}); see logs\\engine-search.log"));
            }
            if let Ok(r) = client.get(format!("http://127.0.0.1:{port}/health")).timeout(Duration::from_secs(2)).send().await {
                if r.status().is_success() {
                    break;
                }
            }
            if t0.elapsed() > Duration::from_secs(if kind == "vulkan" { 180 } else { 120 }) {
                let _ = server.child.kill();
                return Err(anyhow!("the search engine took too long to load"));
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        Ok(server)
    }
}

impl Knowledge {
    // ------------------------------------------------------------ the background indexer

    pub fn start_indexer(&self, app: AppHandle) {
        let k = self.clone();
        tauri::async_runtime::spawn(async move {
            // Pick up changes made while TUNAS was closed.
            tokio::time::sleep(Duration::from_secs(5)).await;
            let k2 = k.clone();
            let _ = tokio::task::spawn_blocking(move || k2.rescan_all()).await;
            loop {
                let next = k.next_pending();
                let Some((file_id, set_id, path)) = next else {
                    // Nothing to do: free the search engine's memory after a few idle minutes.
                    let idle = k.embed.lock().await.as_ref().map(|s| s.last_used.elapsed() > Duration::from_secs(300)).unwrap_or(false);
                    if idle {
                        k.stop_embedder().await;
                    }
                    let _ = tokio::time::timeout(Duration::from_secs(60), k.wake.notified()).await;
                    continue;
                };
                if embed_model_path().is_none() {
                    let _ = app.emit("knowledge-changed", json!({ "set_id": set_id }));
                    let _ = tokio::time::timeout(Duration::from_secs(30), k.wake.notified()).await;
                    continue;
                }
                k.current.lock().insert(set_id.clone(), file_name(&path));
                let _ = app.emit("knowledge-changed", json!({ "set_id": set_id }));
                let res = k.index_file(Some(&app), file_id, &set_id, &path).await;
                if let Err(e) = &res {
                    klog(&format!("could not read {path}: {e}"));
                    let _ = k.db.lock().execute("UPDATE kfiles SET status = 'error', error = ?2 WHERE id = ?1", params![file_id, e.to_string()]);
                }
                k.current.lock().remove(&set_id);
                k.cache.lock().remove(&set_id);
                let _ = app.emit("knowledge-changed", json!({ "set_id": set_id }));
            }
        });
    }

    fn next_pending(&self) -> Option<(i64, String, String)> {
        let c = self.db.lock();
        c.query_row(
            "SELECT f.id, f.set_id, f.path FROM kfiles f JOIN ksets k ON k.id = f.set_id
             WHERE f.status = 'pending' AND k.paused = 0 ORDER BY f.id LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .ok()
        .flatten()
    }

    async fn wait_for_chat(&self) {
        while CHATS_ACTIVE.load(Ordering::SeqCst) > 0 {
            tokio::time::sleep(Duration::from_millis(700)).await;
        }
    }

    async fn index_file(&self, app: Option<&AppHandle>, file_id: i64, set_id: &str, path: &str) -> Result<()> {
        self.wait_for_chat().await;
        let p = PathBuf::from(path);
        if !self.is_allowed(path) {
            return Err(anyhow!("TUNAS is no longer allowed to read this place"));
        }
        let parts = tokio::task::spawn_blocking(move || crate::extract::extract_parts(&p)).await??;
        let name = file_name(path);
        let mut pieces: Vec<(u32, String, String)> = vec![];
        for part in &parts {
            for t in chunk_text(&part.text) {
                pieces.push((part.page, part.label.clone(), t));
            }
        }
        let t0 = Instant::now();
        klog(&format!("reading {path}: {} part(s), {} piece(s)", parts.len(), pieces.len()));
        let mut vecs: Vec<Vec<f32>> = Vec::with_capacity(pieces.len());
        let mut last_emit = Instant::now();
        for batch in pieces.chunks(EMBED_BATCH) {
            if CHATS_ACTIVE.load(Ordering::SeqCst) > 0 {
                klog("waiting: an answer is being written");
                self.wait_for_chat().await;
            }
            let inputs: Vec<String> = batch.iter().map(|(_, label, t)| format!("{name} {label}\n{t}")).collect();
            vecs.extend(self.embed_texts(&inputs).await?);
            // Show progress inside big files ("SOP.pdf · 120 of 800 pieces").
            if pieces.len() > EMBED_BATCH {
                self.current.lock().insert(set_id.to_string(), format!("{name} · {} of {} pieces", vecs.len(), pieces.len()));
                if let Some(a) = app {
                    if last_emit.elapsed() > Duration::from_secs(2) {
                        let _ = a.emit("knowledge-changed", json!({ "set_id": set_id }));
                        last_emit = Instant::now();
                    }
                }
            }
        }
        klog(&format!("done {path}: {} piece(s) in {:.1} s", pieces.len(), t0.elapsed().as_secs_f32()));
        let mut c = self.db.lock();
        let tx = c.transaction()?;
        tx.execute("DELETE FROM kfts WHERE rowid IN (SELECT id FROM kchunks WHERE file_id = ?1)", [file_id])?;
        tx.execute("DELETE FROM kchunks WHERE file_id = ?1", [file_id])?;
        for ((page, label, text), v) in pieces.iter().zip(vecs.iter()) {
            tx.execute(
                "INSERT INTO kchunks(file_id, set_id, page, label, text, vec) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![file_id, set_id, page, label, text, to_bytes(v)],
            )?;
            let id = tx.last_insert_rowid();
            tx.execute("INSERT INTO kfts(rowid, text) VALUES (?1, ?2)", params![id, format!("{name} {text}")])?;
        }
        tx.execute("UPDATE kfiles SET status = 'ok', error = '', chunks = ?2 WHERE id = ?1", params![file_id, pieces.len() as i64])?;
        tx.commit()?;
        Ok(())
    }
}

// ---------------------------------------------------------------- helpers

pub fn embed_model_path() -> Option<PathBuf> {
    let p = crate::models::model_path("Qwen3-Embedding-0.6B-Q8_0.gguf");
    if p.exists() { Some(p) } else { None }
}

fn walk(dir: &Path, depth: usize, out: &mut HashMap<String, (i64, i64)>) {
    if out.len() >= MAX_FILES_PER_SET || depth > 12 {
        return;
    }
    let add = |p: &Path, out: &mut HashMap<String, (i64, i64)>| {
        let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if name.starts_with('.') || name.starts_with("~$") {
            return;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !crate::extract::KNOWLEDGE_TYPES.contains(&ext.as_str()) {
            return;
        }
        if let Ok(m) = std::fs::metadata(p) {
            if m.len() == 0 || m.len() > 100 << 20 {
                return;
            }
            let mtime = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64).unwrap_or(0);
            out.insert(p.display().to_string(), (m.len() as i64, mtime));
        }
    };
    if dir.is_file() {
        add(dir, out);
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "$RECYCLE.BIN" {
            continue;
        }
        match e.file_type() {
            Ok(t) if t.is_dir() => walk(&p, depth + 1, out),
            Ok(t) if t.is_file() => add(&p, out),
            _ => {}
        }
        if out.len() >= MAX_FILES_PER_SET {
            return;
        }
    }
}

/// Cut text into pieces of about CHUNK_CHARS, on paragraph or sentence boundaries, with a little overlap.
pub fn chunk_text(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    let push = |cur: &mut String, out: &mut Vec<String>| {
        let t = cur.trim().to_string();
        if t.chars().count() >= 20 {
            out.push(t.clone());
        }
        let tail: String = {
            let chars: Vec<char> = t.chars().collect();
            let start = chars.len().saturating_sub(OVERLAP_CHARS);
            let s: String = chars[start..].iter().collect();
            // Start the overlap at a word boundary.
            match s.find(' ') { Some(i) if i < s.len() - 1 => s[i + 1..].to_string(), _ => s }
        };
        *cur = tail;
    };
    for para in text.split("\n\n").flat_map(|p| split_long(p, CHUNK_CHARS)) {
        if cur.chars().count() + para.chars().count() > CHUNK_CHARS && cur.chars().count() > OVERLAP_CHARS {
            push(&mut cur, &mut out);
        }
        if !cur.is_empty() {
            cur.push_str("\n\n");
        }
        cur.push_str(&para);
    }
    if cur.trim().chars().count() >= 20 && (out.is_empty() || !out.last().unwrap().ends_with(cur.trim())) {
        out.push(cur.trim().to_string());
    }
    out
}

fn split_long(p: &str, max: usize) -> Vec<String> {
    if p.chars().count() <= max {
        return vec![p.to_string()];
    }
    let mut out = vec![];
    let mut cur = String::new();
    for sentence in p.split_inclusive(['.', '!', '?', '\n']) {
        if cur.chars().count() + sentence.chars().count() > max && !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        if sentence.chars().count() > max {
            let chars: Vec<char> = sentence.chars().collect();
            for c in chars.chunks(max) {
                out.push(c.iter().collect());
            }
        } else {
            cur.push_str(sentence);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
    v
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn from_bytes(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

/// The text added to the system prompt when knowledge is used.
pub fn prompt_block(hits: &[Hit], outline: &[String]) -> String {
    let mut s = String::from(
        "\n\nThe user has given you excerpts from their own documents (Knowledge) below. Use them to answer when they are relevant, \
         and cite them with their number in square brackets, like [1] or [2][3], right after the sentence that uses them. \
         If the excerpts do not contain the answer, say clearly that the user's documents do not cover it; you may then add general \
         knowledge, saying that it is general knowledge. Never invent document content. \
         These documents are the user's own and more specific than any built-in reference: for releases, versions, dates, \
         figures and other details, answer from the documents. Each excerpt shows its file and the section, page, slide or \
         sheet it comes from; keep details from different sections (for example different versions) apart. \
         Excerpts from the same file are listed in the order they appear in that file.\n",
    );
    if !outline.is_empty() {
        if let Some(h) = hits.first() {
            s.push_str(&format!("\nThe sections of {} in the order they appear (the start of a document often holds the newest entries):\n", h.name));
            for l in outline {
                s.push_str(&format!("- {l}\n"));
            }
        }
    }
    for h in hits {
        let where_ = if h.label.is_empty() { String::new() } else { format!(", {}", h.label) };
        s.push_str(&format!("\n[{}] {}{}\n{}\n", h.n, h.name, where_, h.text));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunks_have_overlap_and_size() {
        let para = "Ayat ini adalah contoh. ".repeat(40);
        let text = format!("{para}\n\n{para}\n\n{para}");
        let c = chunk_text(&text);
        assert!(c.len() >= 2, "{}", c.len());
        assert!(c.iter().all(|x| x.chars().count() <= CHUNK_CHARS + OVERLAP_CHARS + 10));
    }
    /// Real files, real search model: `cargo test --lib knowledge::tests::e2e -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn e2e() {
        let k = Knowledge::open().unwrap();
        let dir = std::env::var("KDIR").unwrap_or("/tmp/kdocs".into()); k.grant(&dir).unwrap();
        let id = k.create_set("E2E test").unwrap();
        assert!(k.add_source(&id, "/tmp/nowhere").is_err(), "must refuse a place that was not allowed");
        k.add_source(&id, &dir).unwrap();
        let t0 = Instant::now();
        while let Some((fid, sid, path)) = k.next_pending() {
            if let Err(e) = k.index_file(None, fid, &sid, &path).await {
                eprintln!("FAILED {path}: {e}");
                k.db.lock().execute("UPDATE kfiles SET status='error', error=?2 WHERE id=?1", params![fid, e.to_string()]).unwrap();
            }
        }
        let info = k.sets(&HashMap::new()).unwrap().into_iter().find(|s| s.id == id).unwrap();
        eprintln!("indexed {} files, {} pieces, {} failed in {:?}", info.done, info.chunks, info.failed, t0.elapsed());
        for q in [
            "Berapa hari seminggu boleh bekerja dari rumah?",
            "JPA(S)PKPA/7/2026",
            "How fast must a critical ICT complaint be fixed?",
            "Berapa kos latihan AI dalam bajet 2027?",
            "Bila latihan TUNAS dan di mana?",
            "What did management decide about laptops?",
        ] {
            let hits = k.search(&[id.clone()], q, 3).await;
            eprintln!("Q: {q}");
            for h in &hits {
                eprintln!("   {:.4} {} {} :: {}", h.score, h.name, h.label, h.text.replace('\n', " ").chars().take(70).collect::<String>());
            }
        }
        k.delete_set(&id).unwrap();
        k.revoke(&dir).unwrap();
        k.stop_embedder().await;
    }

    /// `KSET=id KQ="question" cargo test --lib knowledge::tests::probe -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn probe() {
        let k = Knowledge::open().unwrap();
        let set = std::env::var("KSET").unwrap();
        let q = std::env::var("KQ").unwrap();
        for h in k.search(&[set], &q, 6).await {
            eprintln!("{:.4} {} | {} :: {}", h.score, h.name, h.label, h.text.replace('\n', " ").chars().take(60).collect::<String>());
        }
        k.stop_embedder().await;
    }

    #[test]
    fn within() {
        if !cfg!(windows) {
            assert!(is_within("/a/b/c.pdf", "/a/b"));
            assert!(!is_within("/a/bc/c.pdf", "/a/b"));
        }
    }
}
