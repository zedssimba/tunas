//! Local SQLite store: settings, models, assistants, chats and messages.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub fn open() -> Result<Connection> {
    let conn = Connection::open(crate::paths::db_path())?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS models (
            id TEXT PRIMARY KEY, file TEXT NOT NULL, name TEXT NOT NULL,
            source TEXT NOT NULL, size INTEGER NOT NULL DEFAULT 0, sha256 TEXT,
            installed_at TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS assistants (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '', persona TEXT NOT NULL DEFAULT '',
            model_id TEXT NOT NULL DEFAULT '', starter INTEGER NOT NULL DEFAULT 0,
            sort INTEGER NOT NULL DEFAULT 100, created TEXT NOT NULL, updated TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY, assistant_id TEXT NOT NULL, title TEXT NOT NULL,
            created TEXT NOT NULL, updated TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY, chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
            role TEXT NOT NULL, content TEXT NOT NULL, files TEXT NOT NULL DEFAULT '[]',
            stats TEXT NOT NULL DEFAULT '', created TEXT NOT NULL);
        CREATE INDEX IF NOT EXISTS idx_messages_chat ON messages(chat_id, created);
        CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT NOT NULL, base_url TEXT NOT NULL,
            key_enc TEXT NOT NULL DEFAULT '', enabled INTEGER NOT NULL DEFAULT 1,
            models TEXT NOT NULL DEFAULT '[]', created TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS app_keys (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, key_hash TEXT NOT NULL, hint TEXT NOT NULL,
            created TEXT NOT NULL, last_used TEXT NOT NULL DEFAULT '', uses INTEGER NOT NULL DEFAULT 0);
        "#,
    )?;
    seed_starters(&conn)?;
    Ok(conn)
}

pub fn now() -> String {
    chrono::Local::now().to_rfc3339()
}

pub fn get_setting(c: &Connection, key: &str) -> Option<String> {
    c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get(0))
        .optional()
        .ok()
        .flatten()
}

pub fn set_setting(c: &Connection, key: &str, value: &str) -> Result<()> {
    c.execute(
        "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------- models ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledModel {
    pub id: String,
    pub file: String,
    pub name: String,
    pub source: String,
    pub size: i64,
    pub sha256: Option<String>,
    pub installed_at: String,
}

pub fn list_models(c: &Connection) -> Result<Vec<InstalledModel>> {
    let mut st = c.prepare("SELECT id,file,name,source,size,sha256,installed_at FROM models ORDER BY name")?;
    let rows = st.query_map([], |r| {
        Ok(InstalledModel {
            id: r.get(0)?,
            file: r.get(1)?,
            name: r.get(2)?,
            source: r.get(3)?,
            size: r.get(4)?,
            sha256: r.get(5)?,
            installed_at: r.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_model(c: &Connection, id: &str) -> Option<InstalledModel> {
    list_models(c).ok()?.into_iter().find(|m| m.id == id)
}

pub fn add_model(c: &Connection, m: &InstalledModel) -> Result<()> {
    c.execute(
        "INSERT OR REPLACE INTO models(id,file,name,source,size,sha256,installed_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![m.id, m.file, m.name, m.source, m.size, m.sha256, m.installed_at],
    )?;
    Ok(())
}

pub fn remove_model(c: &Connection, id: &str) -> Result<()> {
    c.execute("DELETE FROM models WHERE id=?1", [id])?;
    Ok(())
}

// ---------- assistants ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Assistant {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub persona: String,
    pub model_id: String,
    pub starter: bool,
    pub sort: i64,
    pub created: String,
    pub updated: String,
}

pub fn list_assistants(c: &Connection) -> Result<Vec<Assistant>> {
    let mut st = c.prepare(
        "SELECT id,name,icon,color,persona,model_id,starter,sort,created,updated FROM assistants ORDER BY sort, name",
    )?;
    let rows = st.query_map([], |r| {
        Ok(Assistant {
            id: r.get(0)?,
            name: r.get(1)?,
            icon: r.get(2)?,
            color: r.get(3)?,
            persona: r.get(4)?,
            model_id: r.get(5)?,
            starter: r.get::<_, i64>(6)? != 0,
            sort: r.get(7)?,
            created: r.get(8)?,
            updated: r.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_assistant(c: &Connection, id: &str) -> Option<Assistant> {
    list_assistants(c).ok()?.into_iter().find(|a| a.id == id)
}

pub fn save_assistant(c: &Connection, a: &Assistant) -> Result<()> {
    c.execute(
        "INSERT INTO assistants(id,name,icon,color,persona,model_id,starter,sort,created,updated)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, icon=excluded.icon, color=excluded.color,
           persona=excluded.persona, model_id=excluded.model_id, sort=excluded.sort, updated=excluded.updated",
        params![a.id, a.name, a.icon, a.color, a.persona, a.model_id, a.starter as i64, a.sort, a.created, a.updated],
    )?;
    Ok(())
}

pub fn delete_assistant(c: &Connection, id: &str) -> Result<()> {
    c.execute("DELETE FROM assistants WHERE id=?1", [id])?;
    Ok(())
}

struct Starter {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    color: &'static str,
    persona: &'static str,
}

const STARTERS: &[Starter] = &[
    Starter {
        id: "starter-general",
        name: "General Assistant",
        icon: "chat",
        color: "#2F74D8",
        persona: "You are a helpful, friendly general assistant. Give clear, practical answers. Use short paragraphs and lists when they help.",
    },
    Starter {
        id: "starter-surat-rasmi",
        name: "Surat Rasmi",
        icon: "letter",
        color: "#6D4FE0",
        persona: "Anda membantu menulis surat rasmi dalam Bahasa Melayu mengikut format surat rasmi kerajaan Malaysia: alamat pengirim, rujukan dan tarikh, alamat penerima, salam hormat (Tuan/Puan), tajuk huruf besar dan tebal, perenggan bernombor, penutup (\"Sekian, terima kasih.\" dan cogan kata jika diminta), dan ruang tandatangan. Tanya butiran yang tiada (nama, jawatan, alamat, tarikh) sebelum menulis versi akhir. Gunakan bahasa yang sopan dan tepat.",
    },
    Starter {
        id: "starter-official-letter",
        name: "Official Letter",
        icon: "letter",
        color: "#5B5BD6",
        persona: "You help write formal official letters in English: sender address, reference and date, recipient address, salutation, a bold subject line, numbered paragraphs, a courteous close and a signature block. Ask for any missing details (names, positions, addresses, dates) before writing the final version.",
    },
    Starter {
        id: "starter-minit-ringkas",
        name: "Minit Ringkas",
        icon: "minutes",
        color: "#0F8F9C",
        persona: "Anda menyediakan minit mesyuarat ringkas dalam Bahasa Melayu daripada nota atau transkrip yang diberi: tajuk mesyuarat, tarikh, masa, tempat, kehadiran, perkara yang dibincangkan (bernombor), keputusan, dan tindakan (siapa, apa, bila) dalam jadual. Jangan reka maklumat yang tiada dalam nota; tandakan sebagai [perlu disahkan].",
    },
    Starter {
        id: "starter-meeting-minutes",
        name: "Meeting Minutes",
        icon: "minutes",
        color: "#1F8FB8",
        persona: "You prepare concise meeting minutes in English from notes or a transcript: title, date, time, venue, attendance, numbered discussion items, decisions, and an action table (who, what, by when). Never invent details that are not in the notes; mark gaps as [to be confirmed].",
    },
    Starter {
        id: "starter-email",
        name: "Email Writer",
        icon: "mail",
        color: "#D99A00",
        persona: "You draft clear, polite emails. Ask who it is for and what outcome is wanted if that is not clear. Give a subject line and the email body, matching the tone the user asks for (formal, friendly, firm).",
    },
    Starter {
        id: "starter-summariser",
        name: "Summariser",
        icon: "summary",
        color: "#F05A28",
        persona: "You summarise text and documents. Start with a one-line gist, then the key points as a short list, then any actions, dates or numbers that matter. Keep the summary faithful: no new facts.",
    },
];

fn seed_starters(c: &Connection) -> Result<()> {
    if get_setting(c, "starters_seeded").is_some() {
        return Ok(());
    }
    let now = now();
    for (i, s) in STARTERS.iter().enumerate() {
        let a = Assistant {
            id: s.id.into(),
            name: s.name.into(),
            icon: s.icon.into(),
            color: s.color.into(),
            persona: s.persona.into(),
            model_id: String::new(),
            starter: true,
            sort: i as i64,
            created: now.clone(),
            updated: now.clone(),
        };
        save_assistant(c, &a)?;
    }
    set_setting(c, "starters_seeded", "1")?;
    Ok(())
}

// ---------- chats ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub id: String,
    pub assistant_id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub files: serde_json::Value,
    pub stats: String,
    pub created: String,
}

pub fn list_chats(c: &Connection, query: &str) -> Result<Vec<Chat>> {
    let q = format!("%{}%", query.trim());
    let mut st = c.prepare(
        "SELECT DISTINCT c.id,c.assistant_id,c.title,c.created,c.updated FROM chats c
         LEFT JOIN messages m ON m.chat_id=c.id
         WHERE ?1='%%' OR c.title LIKE ?1 OR m.content LIKE ?1
         ORDER BY c.updated DESC LIMIT 500",
    )?;
    let rows = st.query_map([q], |r| {
        Ok(Chat { id: r.get(0)?, assistant_id: r.get(1)?, title: r.get(2)?, created: r.get(3)?, updated: r.get(4)? })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_chat(c: &Connection, assistant_id: &str, title: &str) -> Result<Chat> {
    let now = now();
    let chat = Chat {
        id: uuid::Uuid::new_v4().to_string(),
        assistant_id: assistant_id.into(),
        title: title.into(),
        created: now.clone(),
        updated: now,
    };
    c.execute(
        "INSERT INTO chats(id,assistant_id,title,created,updated) VALUES(?1,?2,?3,?4,?5)",
        params![chat.id, chat.assistant_id, chat.title, chat.created, chat.updated],
    )?;
    Ok(chat)
}

pub fn rename_chat(c: &Connection, id: &str, title: &str) -> Result<()> {
    c.execute("UPDATE chats SET title=?2 WHERE id=?1", params![id, title])?;
    Ok(())
}

pub fn delete_chat(c: &Connection, id: &str) -> Result<()> {
    c.execute("DELETE FROM messages WHERE chat_id=?1", [id])?;
    c.execute("DELETE FROM chats WHERE id=?1", [id])?;
    Ok(())
}

pub fn list_messages(c: &Connection, chat_id: &str) -> Result<Vec<Message>> {
    let mut st = c.prepare(
        "SELECT id,chat_id,role,content,files,stats,created FROM messages WHERE chat_id=?1 ORDER BY created, rowid",
    )?;
    let rows = st.query_map([chat_id], |r| {
        let files: String = r.get(4)?;
        Ok(Message {
            id: r.get(0)?,
            chat_id: r.get(1)?,
            role: r.get(2)?,
            content: r.get(3)?,
            files: serde_json::from_str(&files).unwrap_or(serde_json::json!([])),
            stats: r.get(5)?,
            created: r.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn add_message(c: &Connection, m: &Message) -> Result<()> {
    c.execute(
        "INSERT INTO messages(id,chat_id,role,content,files,stats,created) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![m.id, m.chat_id, m.role, m.content, m.files.to_string(), m.stats, m.created],
    )?;
    c.execute("UPDATE chats SET updated=?2 WHERE id=?1", params![m.chat_id, m.created])?;
    Ok(())
}

pub fn delete_messages_from(c: &Connection, chat_id: &str, message_id: &str) -> Result<()> {
    // Remove a message and everything after it (used for "regenerate" and "edit and resend").
    let created: Option<String> = c
        .query_row("SELECT created FROM messages WHERE id=?1", [message_id], |r| r.get(0))
        .optional()?;
    if let Some(cr) = created {
        c.execute("DELETE FROM messages WHERE chat_id=?1 AND created>=?2", params![chat_id, cr])?;
    }
    Ok(())
}

// ---------- providers ----------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Provider {
    pub id: String,
    pub kind: String, // teras | openai | openrouter | anthropic | gemini | custom
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub key_enc: String,
    #[serde(default)]
    pub has_key: bool,
    pub enabled: bool,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub created: String,
}

pub fn list_providers(c: &Connection) -> Result<Vec<Provider>> {
    let mut st = c.prepare("SELECT id,kind,name,base_url,key_enc,enabled,models,created FROM providers ORDER BY created")?;
    let rows = st.query_map([], |r| {
        let key_enc: String = r.get(4)?;
        let models: String = r.get(6)?;
        Ok(Provider {
            id: r.get(0)?,
            kind: r.get(1)?,
            name: r.get(2)?,
            base_url: r.get(3)?,
            has_key: !key_enc.is_empty(),
            key_enc,
            enabled: r.get::<_, i64>(5)? != 0,
            models: serde_json::from_str(&models).unwrap_or_default(),
            created: r.get(7)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_provider(c: &Connection, id: &str) -> Option<Provider> {
    list_providers(c).ok()?.into_iter().find(|p| p.id == id)
}

pub fn save_provider(c: &Connection, p: &Provider) -> Result<()> {
    c.execute(
        "INSERT INTO providers(id,kind,name,base_url,key_enc,enabled,models,created) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name, base_url=excluded.base_url,
           key_enc=excluded.key_enc, enabled=excluded.enabled, models=excluded.models",
        params![p.id, p.kind, p.name, p.base_url, p.key_enc, p.enabled as i64, serde_json::to_string(&p.models)?, p.created],
    )?;
    Ok(())
}

pub fn delete_provider(c: &Connection, id: &str) -> Result<()> {
    c.execute("DELETE FROM providers WHERE id=?1", [id])?;
    c.execute("UPDATE assistants SET model_id='' WHERE model_id LIKE ?1", [format!("prov:{id}:%")])?;
    Ok(())
}

// ---------- app keys (local API) ----------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppKey {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub hint: String,
    pub created: String,
    pub last_used: String,
    pub uses: i64,
}

pub fn list_app_keys(c: &Connection) -> Result<Vec<AppKey>> {
    let mut st = c.prepare("SELECT id,name,key_hash,hint,created,last_used,uses FROM app_keys ORDER BY created")?;
    let rows = st.query_map([], |r| {
        Ok(AppKey { id: r.get(0)?, name: r.get(1)?, key_hash: r.get(2)?, hint: r.get(3)?, created: r.get(4)?, last_used: r.get(5)?, uses: r.get(6)? })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn add_app_key(c: &Connection, k: &AppKey) -> Result<()> {
    c.execute(
        "INSERT INTO app_keys(id,name,key_hash,hint,created,last_used,uses) VALUES(?1,?2,?3,?4,?5,'',0)",
        params![k.id, k.name, k.key_hash, k.hint, k.created],
    )?;
    Ok(())
}

pub fn delete_app_key(c: &Connection, id: &str) -> Result<()> {
    c.execute("DELETE FROM app_keys WHERE id=?1", [id])?;
    Ok(())
}

pub fn touch_app_key(c: &Connection, hash: &str) -> Option<String> {
    let name: Option<String> = c.query_row("SELECT name FROM app_keys WHERE key_hash=?1", [hash], |r| r.get(0)).optional().ok().flatten();
    if name.is_some() {
        let _ = c.execute("UPDATE app_keys SET uses=uses+1, last_used=?2 WHERE key_hash=?1", params![hash, now()]);
    }
    name
}
