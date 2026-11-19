//! TUNAS — personal AI for your machine. S.A.L.T CI.
//! S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY,
//! distributed and commercialised by Pemangkin Teknologi.

mod brand;
mod db;
mod engine;
mod extract;
mod gateway;
mod hardware;
mod models;
mod paths;
mod secret;

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
    tokio::task::spawn_blocking(move || models::add_custom(&app, &db, &tr, &PathBuf::from(path), &name))
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
fn engine_stop(st: State<AppState>) {
    st.engine.stop();
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
async fn files_extract(paths: Vec<String>, max_chars: usize) -> Vec<Result<extract::Extracted, String>> {
    tokio::task::spawn_blocking(move || {
        paths
            .iter()
            .map(|p| extract::extract(&PathBuf::from(p), max_chars).map_err(e))
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

#[tauri::command]
async fn brand_check_logo(path: String) -> brand::LogoCheck {
    tokio::task::spawn_blocking(move || brand::check_logo(&PathBuf::from(path))).await.unwrap_or_default()
}

#[tauri::command]
async fn brand_save(app: AppHandle, layer: String, brand: brand::Brand, logo_path: Option<String>, remove_logo: bool) -> R<brand::BrandState> {
    let a2 = app.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        brand::save_layer(&layer, brand, logo_path.map(PathBuf::from), remove_logo)?;
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
        let p = brand::import_pack(&PathBuf::from(path))?;
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
    tokio::task::spawn_blocking(move || brand::export_pack(&layer, &PathBuf::from(path), list, policy, &issuer))
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
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
            if let Some(p) = pack_from_args(&argv) {
                let _ = app.emit("open-pack", p);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
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
            // Apply the brand to the window title and icon at start.
            std::thread::spawn(move || {
                let _ = brand::apply_to_system(&h);
            });
            Ok(())
        })
        .on_window_event(|w, ev| {
            if let tauri::WindowEvent::Destroyed = ev {
                w.app_handle().state::<AppState>().engine.stop();
            }
        })
        .invoke_handler(tauri::generate_handler![
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
        .run(tauri::generate_context!())
        .expect("error while running TUNAS");
}
