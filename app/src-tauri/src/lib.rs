//! TUNAS — personal AI for your machine. S.A.L.T CI.
//! S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY,
//! distributed and commercialised by Pemangkin Teknologi.

mod brand;
mod db;
mod documents;
mod engine;
mod extract;
mod gateway;
mod hardware;
mod knowledge;
mod models;
mod paths;
mod secret;
mod update;
mod userfiles;

use parking_lot::Mutex;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub engine: engine::Engine,
    pub transfers: models::Transfers,
    pub pending_pack: Mutex<Option<String>>,
    pub gateway: gateway::Gateway,
    pub knowledge: knowledge::Knowledge,
}

type R<T> = Result<T, String>;
fn e<E: std::fmt::Display>(x: E) -> String {
    x.to_string()
}

// ---------------- app ----------------

#[derive(Serialize)]
struct AppInfo {
    version: String,
    data_dir: String,
    models_dir: String,
    engines: Vec<String>,
    setup_done: bool,
    os: String,
}

#[tauri::command]
fn app_info(st: State<AppState>) -> AppInfo {
    let engines = ["cpu", "vulkan"]
        .iter()
        .filter(|k| engine::server_exe(k).exists())
        .map(|s| s.to_string())
        .collect();
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        data_dir: paths::data_dir().display().to_string(),
        models_dir: paths::models_dir().display().to_string(),
        engines,
        setup_done: db::get_setting(&st.db.lock(), "setup_done").is_some(),
        os: std::env::consts::OS.into(),
    }
}

#[tauri::command]
fn setting_get(st: State<AppState>, key: String) -> Option<String> {
    db::get_setting(&st.db.lock(), &key)
}

#[tauri::command]
fn setting_set(st: State<AppState>, key: String, value: String) -> R<()> {
    db::set_setting(&st.db.lock(), &key, &value).map_err(e)
}

#[tauri::command]
fn take_pending_pack(st: State<AppState>) -> Option<String> {
    st.pending_pack.lock().take()
}

// ---------------- hardware & models ----------------

#[tauri::command]
async fn hardware_check() -> hardware::Hardware {
    tokio::task::spawn_blocking(hardware::check).await.unwrap_or_default()
}

#[derive(Serialize)]
struct CatalogEntry {
    #[serde(flatten)]
    model: models::CatalogModel,
    installed: bool,
}

#[tauri::command]
fn catalog_list(st: State<AppState>) -> Vec<CatalogEntry> {
    let installed = db::list_models(&st.db.lock()).unwrap_or_default();
    models::catalog()
        .models
        .into_iter()
        .map(|m| {
            let inst = installed.iter().any(|i| i.id == m.id) && models::model_path(&m.file).exists();
            CatalogEntry { model: m, installed: inst }
        })
        .collect()
}

#[tauri::command]
fn models_installed(st: State<AppState>) -> Vec<db::InstalledModel> {
    db::list_models(&st.db.lock())
        .unwrap_or_default()
        .into_iter()
        .filter(|m| models::model_path(&m.file).exists())
        .collect()
}

#[tauri::command]
fn model_download(app: AppHandle, st: State<AppState>, id: String) {
    let db = st.db.clone();
    let tr = st.transfers.clone();
    tauri::async_runtime::spawn(async move {
        let _ = models::download(app, db, tr, id).await;
    });
}

#[tauri::command]
fn model_cancel(st: State<AppState>, id: String) {
    st.transfers.cancel(&id);
}

#[tauri::command]
fn transfers_active(st: State<AppState>) -> Vec<String> {
    st.transfers.active()
}

#[tauri::command]
async fn model_import_folder(app: AppHandle, st: State<'_, AppState>, path: String) -> R<models::ImportReport> {
    let db = st.db.clone();
    let tr = st.transfers.clone();
    tokio::task::spawn_blocking(move || models::import_folder(&app, &db, &tr, &PathBuf::from(path)))
        .await
        .map_err(e)?
        .map_err(e)
}

#[tauri::command]
async fn model_add_custom(app: AppHandle, st: State<'_, AppState>, path: String, name: String) -> R<db::InstalledModel> {
    let db = st.db.clone();
    let tr = st.transfers.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<db::InstalledModel> {
        let p = userfiles::local_in(&app, &path)?;
        models::add_custom(&app, &db, &tr, &p, &name)
    })
        .await
        .map_err(e)?
        .map_err(e)
}

#[tauri::command]
fn model_remove(st: State<AppState>, id: String) -> R<()> {
    if st.engine.status().model_id == id {
        st.engine.stop();
    }
    models::remove(&st.db, &id).map_err(e)
}

#[tauri::command]
async fn models_export(st: State<'_, AppState>, path: String) -> R<Vec<String>> {
    let db = st.db.clone();
    tokio::task::spawn_blocking(move || models::export_to_folder(&db, &PathBuf::from(path)))
        .await
        .map_err(e)?
        .map_err(e)
}

// ---------------- engine ----------------

fn resolve_model(st: &AppState, wanted: &str) -> Option<db::InstalledModel> {
    let c = st.db.lock();
    let installed: Vec<_> = db::list_models(&c)
        .unwrap_or_default()
        .into_iter()
        .filter(|m| models::model_path(&m.file).exists())
        .collect();
    if !wanted.is_empty() {
        if let Some(m) = installed.iter().find(|m| m.id == wanted) {
            return Some(m.clone());
        }
    }
    if let Some(def) = db::get_setting(&c, "default_model") {
        if let Some(m) = installed.iter().find(|m| m.id == def) {
            return Some(m.clone());
        }
    }
    // Prefer catalogue chat models in a sensible order, then anything installed.
    for id in ["qwen3.5-2b", "qwen3.5-4b", "granite-4.2-3b", "qwen3.5-0.8b"] {
        if let Some(m) = installed.iter().find(|m| m.id == id) {
            return Some(m.clone());
        }
    }
    installed.into_iter().find(|m| !m.id.contains("embed"))
}

#[tauri::command]
async fn engine_ensure(app: AppHandle, st: State<'_, AppState>, model_id: String) -> R<engine::EngineStatus> {
    let m = resolve_model(&st, &model_id).ok_or("No model is installed yet. Open Models to download or import one.")?;
    let prefer = db::get_setting(&st.db.lock(), "engine_pref").unwrap_or_else(|| "auto".into());
    st.engine.ensure(&app, &m.id, models::model_path(&m.file), &prefer).await.map_err(e)
}

#[tauri::command]
fn engine_status(st: State<AppState>) -> engine::EngineStatus {
    st.engine.status()
}

#[tauri::command]
fn engine_stop(app: AppHandle, st: State<AppState>) {
    st.engine.stop();
    let _ = app.emit("engine-status", st.engine.status());
}

#[tauri::command]
fn resolved_model(st: State<AppState>, model_id: String) -> Option<db::InstalledModel> {
    resolve_model(&st, &model_id)
}

// ---------------- assistants ----------------

#[tauri::command]
fn assistants_list(st: State<AppState>) -> Vec<db::Assistant> {
    db::list_assistants(&st.db.lock()).unwrap_or_default()
}

#[tauri::command]
fn assistant_save(st: State<AppState>, mut assistant: db::Assistant) -> R<db::Assistant> {
    let c = st.db.lock();
    let exists = db::get_assistant(&c, &assistant.id).is_some();
    if !exists && !brand::policy().allow_create_assistants {
        return Err("Your organisation does not allow new assistants on this machine.".into());
    }
    if assistant.name.trim().is_empty() {
        return Err("Give the assistant a name.".into());
    }
    let now = db::now();
    if assistant.id.is_empty() {
        assistant.id = uuid::Uuid::new_v4().to_string();
        assistant.created = now.clone();
        assistant.starter = false;
        assistant.sort = 100;
    }
    assistant.updated = now;
    db::save_assistant(&c, &assistant).map_err(e)?;
    Ok(assistant)
}

#[tauri::command]
fn assistant_delete(st: State<AppState>, id: String) -> R<()> {
    let c = st.db.lock();
    db::delete_assistant(&c, &id).map_err(e)
}

// ---------------- chats ----------------

#[tauri::command]
fn chats_list(st: State<AppState>, query: String) -> Vec<db::Chat> {
    db::list_chats(&st.db.lock(), &query).unwrap_or_default()
}

#[tauri::command]
fn chat_create(st: State<AppState>, assistant_id: String, title: String) -> R<db::Chat> {
    db::create_chat(&st.db.lock(), &assistant_id, &title).map_err(e)
}

#[tauri::command]
fn chat_rename(st: State<AppState>, id: String, title: String) -> R<()> {
    db::rename_chat(&st.db.lock(), &id, &title).map_err(e)
}

#[tauri::command]
fn chat_delete(st: State<AppState>, id: String) -> R<()> {
    db::delete_chat(&st.db.lock(), &id).map_err(e)
}

#[tauri::command]
fn messages_list(st: State<AppState>, chat_id: String) -> Vec<db::Message> {
    db::list_messages(&st.db.lock(), &chat_id).unwrap_or_default()
}

#[tauri::command]
fn message_add(st: State<AppState>, mut message: db::Message) -> R<db::Message> {
    if message.id.is_empty() {
        message.id = uuid::Uuid::new_v4().to_string();
    }
    message.created = db::now();
    db::add_message(&st.db.lock(), &message).map_err(e)?;
    Ok(message)
}

#[tauri::command]
fn messages_delete_from(st: State<AppState>, chat_id: String, message_id: String) -> R<()> {
    db::delete_messages_from(&st.db.lock(), &chat_id, &message_id).map_err(e)
}

// ---------------- files ----------------

#[tauri::command]
async fn files_extract(app: AppHandle, paths: Vec<String>, max_chars: usize) -> Vec<Result<extract::Extracted, String>> {
    tokio::task::spawn_blocking(move || {
        paths
            .iter()
            .map(|p| userfiles::local_in(&app, p).map_err(e).and_then(|lp| extract::extract(&lp, max_chars).map_err(e)))
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
fn files_supported() -> Vec<String> {
    extract::SUPPORTED.iter().map(|s| s.to_string()).collect()
}

// ---------------- brand ----------------

#[tauri::command]
fn brand_state() -> brand::BrandState {
    brand::state()
}

/// Android: documents picked for Knowledge are copied into TUNAS's own folder (Android lets an app read
/// only the files the person picks, not whole folders). Returns the folder, or adds to `into` when given.
#[tauri::command]
async fn knowledge_copy_in(app: AppHandle, paths: Vec<String>, into: Option<String>) -> R<String> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        let dir = match into {
            Some(d) => PathBuf::from(d),
            None => paths::data_dir().join("knowledge-files").join(uuid::Uuid::new_v4().simple().to_string()),
        };
        std::fs::create_dir_all(&dir)?;
        for p in &paths {
            let local = userfiles::local_in(&app, p)?;
            let name = local.file_name().map(|n| n.to_os_string()).unwrap_or_default();
            let dest = dir.join(&name);
            if std::fs::rename(&local, &dest).is_err() {
                std::fs::copy(&local, &dest)?;
            }
        }
        Ok(dir.display().to_string())
    })
    .await
    .map_err(e)?
    .map_err(e)
}

// ---------------- updates ----------------

/// Is there a newer TUNAS? (The organisation's policy can switch this off or point it elsewhere.)
#[tauri::command]
async fn update_check() -> update::UpdateInfo {
    let pol = brand::policy();
    if !pol.allow_update_check {
        return update::UpdateInfo { current: env!("CARGO_PKG_VERSION").into(), error: "off".into(), ..Default::default() };
    }
    let src = if pol.update_source.trim().is_empty() { update::DEFAULT_SOURCE.to_string() } else { pol.update_source.trim().to_string() };
    update::check(&src).await
}

#[tauri::command]
async fn update_download(app: AppHandle, file: update::UpdFile) -> R<String> {
    update::download(&app, &file).await.map(|p| p.display().to_string()).map_err(e)
}

#[tauri::command]
fn update_install(app: AppHandle, path: String) -> R<()> {
    update::install(&app, std::path::Path::new(&path)).map_err(e)
}

#[tauri::command]
async fn brand_check_logo(app: AppHandle, path: String) -> brand::LogoCheck {
    tokio::task::spawn_blocking(move || match userfiles::local_in(&app, &path) {
        Ok(p) => brand::check_logo(&p),
        Err(_) => brand::LogoCheck::default(),
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
async fn brand_save(app: AppHandle, layer: String, brand: brand::Brand, logo_path: Option<String>, remove_logo: bool) -> R<brand::BrandState> {
    let a2 = app.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let logo = match logo_path { Some(p) => Some(userfiles::local_in(&a2, &p)?), None => None };
        brand::save_layer(&layer, brand, logo, remove_logo)?;
        brand::apply_to_system(&a2)?;
        Ok(())
    })
    .await
    .map_err(e)?
    .map_err(e)?;
    let s = brand::state();
    let _ = app.emit("brand-changed", &s);
    Ok(s)
}

#[tauri::command]
async fn brand_clear(app: AppHandle, layer: String) -> R<brand::BrandState> {
    let a2 = app.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        brand::clear_layer(&layer)?;
        brand::apply_to_system(&a2)?;
        Ok(())
    })
    .await
    .map_err(e)?
    .map_err(e)?;
    let s = brand::state();
    let _ = app.emit("brand-changed", &s);
    Ok(s)
}

#[tauri::command]
async fn brand_import_pack(app: AppHandle, st: State<'_, AppState>, path: String) -> R<brand::PackReport> {
    let a2 = app.clone();
    let imported = tokio::task::spawn_blocking(move || -> anyhow::Result<brand::ImportedPack> {
        let p = brand::import_pack(&userfiles::local_in(&a2, &path)?)?;
        brand::apply_to_system(&a2)?;
        Ok(p)
    })
    .await
    .map_err(e)?
    .map_err(e)?;
    // Add the pack's assistants (replacing earlier copies from a pack with the same names).
    {
        let c = st.db.lock();
        let existing = db::list_assistants(&c).unwrap_or_default();
        for (i, pa) in imported.assistants.iter().enumerate() {
            if pa.name.trim().is_empty() {
                continue;
            }
            let id = existing
                .iter()
                .find(|a| a.id.starts_with("pack-") && a.name == pa.name)
                .map(|a| a.id.clone())
                .unwrap_or_else(|| format!("pack-{}", uuid::Uuid::new_v4()));
            let now = db::now();
            let _ = db::save_assistant(
                &c,
                &db::Assistant {
                    id,
                    name: pa.name.clone(),
                    icon: if pa.icon.is_empty() { "chat".into() } else { pa.icon.clone() },
                    color: pa.color.clone(),
                    persona: pa.persona.clone(),
                    model_id: String::new(),
                    starter: false,
                    sort: 50 + i as i64,
                    created: now.clone(),
                    updated: now,
                },
            );
        }
    }
    let _ = app.emit("brand-changed", brand::state());
    Ok(imported.report)
}

#[tauri::command]
async fn brand_export_pack(
    app: AppHandle,
    st: State<'_, AppState>,
    path: String,
    layer: String,
    assistant_ids: Vec<String>,
    policy: Option<brand::Policy>,
    issuer: String,
) -> R<()> {
    let list: Vec<brand::PackAssistant> = db::list_assistants(&st.db.lock())
        .unwrap_or_default()
        .into_iter()
        .filter(|a| assistant_ids.contains(&a.id))
        .map(|a| brand::PackAssistant { name: a.name, icon: a.icon, color: a.color, persona: a.persona })
        .collect();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let out = userfiles::local_out(&path, "tunasbrand");
        brand::export_pack(&layer, &out.path, list, policy, &issuer)?;
        userfiles::finish_out(&app, &out)
    })
        .await
        .map_err(e)?
        .map_err(e)
}

#[tauri::command]
async fn brand_apply(app: AppHandle) -> R<()> {
    tokio::task::spawn_blocking(move || brand::apply_to_system(&app)).await.map_err(e)?.map_err(e)
}


// ---------------- connections: providers, TERAS, local API ----------------

const DEFAULT_API_PORT: u16 = 11435;

#[derive(Serialize)]
struct GatewayInfo {
    port: u16,
    key: String,
}

#[tauri::command]
fn gateway_info(st: State<AppState>) -> GatewayInfo {
    GatewayInfo { port: *st.gateway.internal_port.lock(), key: st.gateway.internal_key.clone() }
}

#[derive(Serialize)]
struct RouteInfo {
    kind: String,
    model_id: String,
    label: String,
    provider: String,
    ctx: u32,
    error: String,
}

#[tauri::command]
fn route_info(app: AppHandle, model_id: String) -> RouteInfo {
    match gateway::resolve_target(&app, &model_id) {
        Ok(gateway::Target::Local(m)) => {
            let mut s = sysinfo::System::new();
            s.refresh_memory();
            let ctx = if s.total_memory() as f64 / 1024f64.powi(3) >= 15.0 { 16384 } else { 8192 };
            RouteInfo { kind: "local".into(), model_id: m.id, label: m.name, provider: String::new(), ctx, error: String::new() }
        }
        Ok(gateway::Target::Provider(p, m)) => RouteInfo { kind: if p.kind == "teras" { "teras".into() } else { "cloud".into() }, model_id: format!("prov:{}:{}", p.id, m), label: m, provider: p.name, ctx: 32768, error: String::new() },
        Err(e) => RouteInfo { kind: "none".into(), model_id: String::new(), label: String::new(), provider: String::new(), ctx: 8192, error: e },
    }
}

fn normalise_base(kind: &str, url: &str) -> String {
    let preset = match kind {
        "openai" => "https://api.openai.com/v1",
        "openrouter" => "https://openrouter.ai/api/v1",
        "anthropic" => "https://api.anthropic.com/v1",
        "gemini" => "https://generativelanguage.googleapis.com/v1beta/openai",
        _ => "",
    };
    let u = url.trim().trim_end_matches('/').to_string();
    if u.is_empty() {
        return preset.into();
    }
    let u = if u.starts_with("http://") || u.starts_with("https://") { u } else { format!("https://{u}") };
    if kind == "teras" && !u.ends_with("/v1") {
        return format!("{u}/v1");
    }
    u
}

async fn fetch_models(p: &db::Provider) -> Result<Vec<String>, String> {
    let key = secret::reveal(&p.key_enc).unwrap_or_default();
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().map_err(e)?;
    let mut req = client.get(format!("{}/models", p.base_url.trim_end_matches('/')));
    if !key.is_empty() {
        req = req.bearer_auth(&key);
    }
    if p.kind == "anthropic" {
        req = req.header("x-api-key", &key).header("anthropic-version", "2023-06-01");
    }
    let r = req.send().await.map_err(|x| format!("Could not reach {}: {x}", p.name))?;
    if r.status().as_u16() == 401 || r.status().as_u16() == 403 {
        return Err(format!("{} refused the key ({}). Check the key.", p.name, r.status()));
    }
    if !r.status().is_success() {
        return Err(format!("{} answered {} when listing models. You can still type model names.", p.name, r.status()));
    }
    let v: serde_json::Value = r.json().await.map_err(e)?;
    let mut ids: Vec<String> = v
        .get("data")
        .or_else(|| v.get("models"))
        .and_then(|d| d.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(|s| s.trim_start_matches("models/").to_string())).collect())
        .unwrap_or_default();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

#[tauri::command]
fn providers_list(st: State<AppState>) -> Vec<db::Provider> {
    db::list_providers(&st.db.lock()).unwrap_or_default()
}

#[derive(serde::Deserialize)]
struct ProviderInput {
    id: String,
    kind: String,
    name: String,
    base_url: String,
    enabled: bool,
    models: Vec<String>,
}

#[derive(Serialize)]
struct ProviderResult {
    provider: db::Provider,
    available: Vec<String>,
    error: String,
}

#[tauri::command]
async fn provider_save(st: State<'_, AppState>, provider: ProviderInput, key: Option<String>) -> R<ProviderResult> {
    if provider.kind != "teras" && !brand::policy().allow_cloud {
        return Err("Your organisation does not allow cloud AI providers on this machine.".into());
    }
    let existing = if provider.id.is_empty() { None } else { db::get_provider(&st.db.lock(), &provider.id) };
    let mut p = db::Provider {
        id: if provider.id.is_empty() { uuid::Uuid::new_v4().to_string()[..8].to_string() } else { provider.id.clone() },
        kind: provider.kind.clone(),
        name: if provider.name.trim().is_empty() { provider.kind.clone() } else { provider.name.trim().to_string() },
        base_url: normalise_base(&provider.kind, &provider.base_url),
        key_enc: existing.as_ref().map(|x| x.key_enc.clone()).unwrap_or_default(),
        has_key: false,
        enabled: provider.enabled,
        models: provider.models.clone(),
        created: existing.as_ref().map(|x| x.created.clone()).unwrap_or_else(db::now),
    };
    if p.base_url.is_empty() {
        return Err("Enter the address.".into());
    }
    if let Some(k) = key {
        let k = k.trim().to_string();
        p.key_enc = if k.is_empty() { String::new() } else { secret::protect(&k).map_err(e)? };
    }
    let (available, error) = match fetch_models(&p).await {
        Ok(list) => (list, String::new()),
        Err(x) => (vec![], x),
    };
    if p.models.is_empty() && !available.is_empty() && p.kind == "teras" {
        p.models = available.clone();
    }
    db::save_provider(&st.db.lock(), &p).map_err(e)?;
    p.has_key = !p.key_enc.is_empty();
    Ok(ProviderResult { provider: p, available, error })
}

#[tauri::command]
async fn provider_available(st: State<'_, AppState>, id: String) -> R<Vec<String>> {
    let p = db::get_provider(&st.db.lock(), &id).ok_or("not found")?;
    fetch_models(&p).await
}

#[tauri::command]
fn provider_delete(st: State<AppState>, id: String) -> R<()> {
    db::delete_provider(&st.db.lock(), &id).map_err(e)
}

#[derive(Serialize)]
struct ApiSettings {
    enabled: bool,
    port: u16,
    running: bool,
    allowed: bool,
}

#[tauri::command]
fn api_settings(app: AppHandle, st: State<AppState>) -> ApiSettings {
    let c = st.db.lock();
    ApiSettings {
        enabled: db::get_setting(&c, "api_enabled").as_deref() == Some("1"),
        port: db::get_setting(&c, "api_port").and_then(|p| p.parse().ok()).unwrap_or(DEFAULT_API_PORT),
        running: gateway::external_port(&app).is_some(),
        allowed: brand::policy().allow_local_api,
    }
}

#[tauri::command]
async fn api_set(app: AppHandle, st: State<'_, AppState>, enabled: bool, port: u16) -> R<()> {
    let port = if port < 1024 { DEFAULT_API_PORT } else { port };
    gateway::set_external(app.clone(), enabled, port).await.map_err(e)?;
    let c = st.db.lock();
    db::set_setting(&c, "api_enabled", if enabled { "1" } else { "0" }).map_err(e)?;
    db::set_setting(&c, "api_port", &port.to_string()).map_err(e)?;
    Ok(())
}

#[tauri::command]
fn app_keys_list(st: State<AppState>) -> Vec<db::AppKey> {
    db::list_app_keys(&st.db.lock()).unwrap_or_default()
}

#[derive(Serialize)]
struct NewKey {
    key: String,
    entry: db::AppKey,
}

#[tauri::command]
fn app_key_create(st: State<AppState>, name: String) -> R<NewKey> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Give the key a name, such as the app that will use it.".into());
    }
    let key = secret::new_key("tunas-");
    let entry = db::AppKey {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.into(),
        key_hash: secret::hash_key(&key),
        hint: format!("{}…{}", &key[..10], &key[key.len() - 4..]),
        created: db::now(),
        last_used: String::new(),
        uses: 0,
    };
    db::add_app_key(&st.db.lock(), &entry).map_err(e)?;
    Ok(NewKey { key, entry })
}

#[tauri::command]
fn app_key_delete(st: State<AppState>, id: String) -> R<()> {
    db::delete_app_key(&st.db.lock(), &id).map_err(e)
}

fn pack_from_args(args: &[String]) -> Option<String> {
    args.iter().skip(1).find(|a| a.to_lowercase().ends_with(".tunasbrand")).cloned()
}


// ---------------------------------------------------------------- knowledge and folder access

#[tauri::command]
fn folders_list(st: State<AppState>) -> R<Vec<knowledge::Grant>> {
    st.knowledge.grants().map_err(e)
}

#[tauri::command]
fn folder_allow(st: State<AppState>, path: String) -> R<()> {
    st.knowledge.grant(&path).map_err(e)
}

#[tauri::command]
async fn folder_revoke(st: State<'_, AppState>, path: String) -> R<()> {
    let k = st.knowledge.clone();
    tokio::task::spawn_blocking(move || k.revoke(&path)).await.map_err(e)?.map_err(e)
}

#[tauri::command]
fn folder_is_allowed(st: State<AppState>, path: String) -> bool {
    st.knowledge.is_allowed(&path)
}

#[derive(Serialize)]
struct KnowledgeOverview {
    model_ready: bool,
    sets: Vec<knowledge::SetInfo>,
}

#[tauri::command]
fn knowledge_list(st: State<AppState>) -> R<KnowledgeOverview> {
    let names: std::collections::HashMap<String, String> =
        db::list_assistants(&st.db.lock()).unwrap_or_default().into_iter().map(|a| (a.id, a.name)).collect();
    Ok(KnowledgeOverview { model_ready: knowledge::embed_model_path().is_some(), sets: st.knowledge.sets(&names).map_err(e)? })
}

#[tauri::command]
async fn knowledge_create(st: State<'_, AppState>, name: String, paths: Vec<String>) -> R<String> {
    let k = st.knowledge.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        let id = k.create_set(&name)?;
        for p in paths {
            k.add_source(&id, &p)?;
        }
        Ok(id)
    })
    .await
    .map_err(e)?
    .map_err(e)
}

#[tauri::command]
fn knowledge_rename(st: State<AppState>, id: String, name: String) -> R<()> {
    st.knowledge.rename_set(&id, &name).map_err(e)
}

#[tauri::command]
fn knowledge_delete(st: State<AppState>, id: String) -> R<()> {
    st.knowledge.delete_set(&id).map_err(e)
}

#[tauri::command]
fn knowledge_pause(st: State<AppState>, id: String, paused: bool) -> R<()> {
    st.knowledge.set_paused(&id, paused).map_err(e)
}

#[tauri::command]
async fn knowledge_add_source(st: State<'_, AppState>, id: String, path: String) -> R<()> {
    let k = st.knowledge.clone();
    tokio::task::spawn_blocking(move || k.add_source(&id, &path)).await.map_err(e)?.map_err(e)
}

#[tauri::command]
async fn knowledge_remove_source(st: State<'_, AppState>, id: String, path: String) -> R<()> {
    let k = st.knowledge.clone();
    tokio::task::spawn_blocking(move || k.remove_source(&id, &path)).await.map_err(e)?.map_err(e)
}

#[tauri::command]
async fn knowledge_reindex(st: State<'_, AppState>, id: String, force: bool) -> R<()> {
    let k = st.knowledge.clone();
    tokio::task::spawn_blocking(move || k.rescan(&id, force)).await.map_err(e)?.map_err(e)
}

#[tauri::command]
fn knowledge_failed(st: State<AppState>, id: String) -> R<Vec<knowledge::FailedFile>> {
    st.knowledge.failed_files(&id).map_err(e)
}

#[tauri::command]
fn knowledge_for_assistant(st: State<AppState>, assistant_id: String) -> Vec<String> {
    st.knowledge.assistant_sets(&assistant_id)
}

#[tauri::command]
fn knowledge_set_for_assistant(st: State<AppState>, assistant_id: String, set_ids: Vec<String>) -> R<()> {
    st.knowledge.set_assistant_sets(&assistant_id, &set_ids).map_err(e)
}

#[tauri::command]
fn knowledge_kick(st: State<AppState>) {
    st.knowledge.kick();
}

/// Open one of the user's files (a knowledge source) in its usual app. Only places TUNAS may read.
#[tauri::command]
fn open_file(st: State<AppState>, path: String) -> R<()> {
    if !st.knowledge.is_allowed(&path) {
        return Err("TUNAS is not allowed to open this file.".into());
    }
    tauri_plugin_opener::open_path(&path, None::<&str>).map_err(e)
}

// ---------------------------------------------------------------- documents

/// Write an answer as a Word, Excel or PDF file at `path` (chosen by the user in a save dialog).
#[tauri::command]
async fn doc_export(app: AppHandle, markdown: String, format: String, path: String, title: String, letterhead: bool) -> R<()> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let out = userfiles::local_out(&path, &format);
        documents::export(&markdown, &format, &out.path, &title, letterhead)?;
        userfiles::finish_out(&app, &out)
    })
        .await
        .map_err(e)?
        .map_err(e)
}

/// The full address of a file TERAS made (answers link to it as /api/files/...).
fn teras_file_url(st: &AppState, provider_id: &str, file_path: &str) -> Result<(reqwest::Url, String), String> {
    let p = db::get_provider(&st.db.lock(), provider_id).ok_or("This TERAS connection no longer exists.")?;
    if p.kind != "teras" {
        return Err("Not a TERAS connection.".into());
    }
    if !file_path.starts_with("/api/files/") {
        return Err("Not a TERAS file.".into());
    }
    let base = p.base_url.trim_end_matches('/').trim_end_matches("/v1").to_string() + "/";
    let url = reqwest::Url::parse(&base).and_then(|b| b.join(file_path.trim_start_matches('/'))).map_err(e)?;
    Ok((url, secret::reveal(&p.key_enc).unwrap_or_default()))
}

/// Download a file TERAS made into `save_to` (chosen by the user), with the TERAS key.
/// "auth" in the error means TERAS would not give the file to the key (open it in the browser instead).
#[tauri::command]
async fn teras_download(app: AppHandle, st: State<'_, AppState>, provider_id: String, file_path: String, save_to: String) -> R<()> {
    let (url, key) = teras_file_url(&st, &provider_id, &file_path)?;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().map_err(e)?;
    let mut req = client.get(url);
    if !key.is_empty() {
        req = req.bearer_auth(&key);
    }
    let resp = req.send().await.map_err(|x| format!("Could not reach TERAS: {x}"))?;
    let code = resp.status().as_u16();
    if code == 401 || code == 403 {
        return Err(format!("auth: TERAS did not allow this download with the TUNAS key ({code})."));
    }
    if !resp.status().is_success() {
        return Err(format!("TERAS answered {code} for this file."));
    }
    let bytes = resp.bytes().await.map_err(e)?;
    let ext = file_path.rsplit('.').next().unwrap_or("bin").to_string();
    let out = userfiles::local_out(&save_to, &ext);
    std::fs::write(&out.path, &bytes).map_err(e)?;
    userfiles::finish_out(&app, &out).map_err(e)
}

/// Open a file TERAS made in the browser (the person signs in to TERAS there if needed).
#[tauri::command]
fn teras_open(st: State<AppState>, provider_id: String, file_path: String) -> R<()> {
    let (url, _) = teras_file_url(&st, &provider_id, &file_path)?;
    tauri_plugin_opener::open_url(url.as_str(), None::<&str>).map_err(e)
}

/// Open a file TUNAS has just written or downloaded for the user.
#[tauri::command]
fn open_written(path: String) -> R<()> {
    let ok = ["docx", "xlsx", "pdf", "pptx", "csv", "md", "txt", "png", "jpg", "zip"].iter().any(|x| path.to_lowercase().ends_with(&format!(".{x}")));
    if !ok {
        return Err("Only documents made by TUNAS can be opened here.".into());
    }
    tauri_plugin_opener::open_path(&path, None::<&str>).map_err(e)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::open().expect("could not open the TUNAS database");
    let args: Vec<String> = std::env::args().collect();
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        engine: engine::Engine::default(),
        transfers: models::Transfers::default(),
        pending_pack: Mutex::new(pack_from_args(&args)),
        gateway: gateway::Gateway::default(),
        knowledge: knowledge::Knowledge::open().expect("could not open the TUNAS knowledge index"),
    };

    let builder = tauri::Builder::default();
    // One TUNAS at a time on desktops (a second start brings the first to the front). Android does this itself.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.unminimize();
            let _ = w.set_focus();
        }
        if let Some(p) = pack_from_args(&argv) {
            let _ = app.emit("open-pack", p);
        }
    }));
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
            // Files copied in from Android's file picker are only needed while they are read.
            let _ = std::fs::remove_dir_all(paths::data_dir().join("inbox"));
            let _ = std::fs::remove_dir_all(paths::data_dir().join("outbox"));
            #[cfg(unix)]
            engine::stop_stale_engines();
            app.state::<AppState>().knowledge.start_indexer(app.handle().clone());
            let g = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = gateway::start_internal(g.clone()).await;
                let (on, port) = {
                    let st = g.state::<AppState>();
                    let c = st.db.lock();
                    (db::get_setting(&c, "api_enabled").as_deref() == Some("1"),
                     db::get_setting(&c, "api_port").and_then(|p| p.parse().ok()).unwrap_or(DEFAULT_API_PORT))
                };
                if on {
                    let _ = gateway::set_external(g, true, port).await;
                }
            });
            let h = app.handle().clone();
            // Apply the brand to the window title and icon before the window appears, so the taskbar icon
            // does not change a few seconds after start.
            let _ = brand::apply_to_system(&h);
            #[cfg(desktop)]
            if let Some(w) = h.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            Ok(())
        })
        .on_window_event(|w, ev| {
            if let tauri::WindowEvent::Destroyed = ev {
                let st = w.app_handle().state::<AppState>();
                st.engine.stop();
                st.knowledge.shutdown();
            }
        })
        .invoke_handler(tauri::generate_handler![
            doc_export, open_written, knowledge_copy_in, update_check, update_download, update_install, teras_download, teras_open, folders_list, folder_allow, folder_revoke, folder_is_allowed, knowledge_list, knowledge_create, knowledge_rename, knowledge_delete, knowledge_pause, knowledge_add_source, knowledge_remove_source, knowledge_reindex, knowledge_failed, knowledge_for_assistant, knowledge_set_for_assistant, knowledge_kick, open_file,
            app_info,
            setting_get,
            setting_set,
            take_pending_pack,
            hardware_check,
            catalog_list,
            models_installed,
            model_download,
            model_cancel,
            transfers_active,
            model_import_folder,
            model_add_custom,
            model_remove,
            models_export,
            engine_ensure,
            engine_status,
            engine_stop,
            resolved_model,
            assistants_list,
            assistant_save,
            assistant_delete,
            chats_list,
            chat_create,
            chat_rename,
            chat_delete,
            messages_list,
            message_add,
            messages_delete_from,
            files_extract,
            files_supported,
            brand_state,
            brand_check_logo,
            brand_save,
            brand_clear,
            brand_import_pack,
            brand_export_pack,
            brand_apply,
            gateway_info,
            route_info,
            providers_list,
            provider_save,
            provider_available,
            provider_delete,
            api_settings,
            api_set,
            app_keys_list,
            app_key_create,
            app_key_delete,
        ])
        .build(tauri::generate_context!())
        .expect("error while running TUNAS")
        .run(|_app, _ev| {
            // macOS hands a double-clicked .tunasbrand file to the running app as an "opened" event.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = &_ev {
                for u in urls {
                    if let Ok(p) = u.to_file_path() {
                        let p = p.to_string_lossy().to_string();
                        if p.to_lowercase().ends_with(".tunasbrand") {
                            *_app.state::<AppState>().pending_pack.lock() = Some(p.clone());
                            let _ = _app.emit("open-pack", p);
                        }
                    }
                }
            }
        });
}
