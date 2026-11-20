//! The TUNAS gateway: one OpenAI-compatible endpoint in front of the local engine and the connected providers.
//!
//! - Internal listener (always on, random port, internal key): used by the TUNAS window itself.
//! - App listener (off by default, fixed port, app keys): "Let other apps use TUNAS", e.g. NEXUS desktop.
//!   Both listen on 127.0.0.1 only.
//!
//! Routing of `model`:
//!   assistant slug or id  -> the assistant's persona + its base model
//!   local model id        -> the local engine (llama.cpp)
//!   "<provider>/<model>"  -> a connected provider (TERAS or a cloud provider)
//! The built-in S.A.L.T reference is added when a question is about S.A.L.T CI.

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub const SALT_REFERENCE: &str = include_str!("../resources/salt_reference.md");

#[derive(Clone)]
pub struct Gateway {
    pub internal_port: Arc<Mutex<u16>>,
    pub internal_key: String,
    external: Arc<Mutex<Option<(u16, tokio::sync::oneshot::Sender<()>)>>>,
}

impl Default for Gateway {
    fn default() -> Self {
        Gateway {
            internal_port: Arc::new(Mutex::new(0)),
            internal_key: crate::secret::new_key("tunas-int-"),
            external: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone)]
struct Ctx {
    app: AppHandle,
    external: bool,
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() || c == '.' {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

const TRIGGERS: &[&str] = &[
    "s.a.l.t", "salt ci", "salt-ci", "saltci", "salt.ci", "salt technology", "salt tech", "teras", "tunas", "pemangkin",
    "warkah", "pentas", "kembara", "mom studio", "capt zeds", "central intelligence",
];

const FOOD: &[&str] = &[
    "garam", "gula", "lada", "masak", "resipi", "recipe", "sugar", "pepper", "cook", "bake", "taste", "rasa", "sea salt",
    "salt water", "air garam", "teaspoon", "tsp", "sudu",
];

/// TERAS events seen by TUNAS's chat (tools and sources), in logs\\teras.log, for checking what TERAS sends.
fn teras_log(line: &str) {
    use std::io::Write;
    let path = crate::paths::logs_dir().join("teras.log");
    if std::fs::metadata(&path).map(|m| m.len() > 2_000_000).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line);
    }
}

/// The person's "About me" from Settings, as a short block for the model (empty fields left out).
fn about_me_block(app: &AppHandle) -> Option<String> {
    use tauri::Manager;
    let st = app.state::<crate::AppState>();
    let raw = crate::db::get_setting(&st.db.lock(), "about_me")?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    if v.get("on").and_then(|x| x.as_bool()) == Some(false) {
        return None;
    }
    let f = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|x| x.trim().chars().take(240).collect::<String>()).filter(|x| !x.is_empty());
    let mut lines = vec![];
    for (k, label) in [("call", "Call them"), ("position", "Position"), ("organisation", "Organisation"), ("work", "Works on"), ("languages", "Languages"), ("style", "Likes answers")] {
        if let Some(x) = f(k) {
            lines.push(format!("- {label}: {x}"));
        }
    }
    if lines.is_empty() {
        return None;
    }
    Some(format!(
        "\n\nAbout the user (from their settings; use it to fit your answers, address them as they asked, and sign documents for them with it; do not repeat it back unless it helps):\n{}",
        lines.join("\n")
    ))
}

/// Sampling settings that TERAS and cloud models may refuse.
const SAMPLING: &[&str] = &["temperature", "top_p", "top_k", "min_p", "presence_penalty", "frequency_penalty", "repeat_penalty"];

/// A short message that asks something ("and the deadline?", "bila?"), as opposed to "hi" or "terima kasih".
fn looks_like_question(text: &str) -> bool {
    let t = text.to_lowercase();
    if t.contains('?') {
        return true;
    }
    const Q: &[&str] = &["what", "when", "where", "which", "who", "why", "how", "and the", "what about", "apa", "bila", "mana", "siapa", "kenapa", "mengapa", "bagaimana", "macam mana", "berapa", "adakah", "bolehkah", "yang mana"];
    t.split(|c: char| !c.is_alphanumeric()).any(|w| Q.contains(&w)) || Q.iter().any(|q| q.contains(' ') && t.contains(q))
}

pub fn wants_reference(text: &str) -> bool {
    let t = text.to_lowercase();
    if TRIGGERS.iter().any(|k| t.contains(k)) || (t.contains("nexus") && (t.contains("salt") || t.contains("app"))) {
        return true;
    }
    // "salt" on its own as a word (e.g. "salt tu apa?"), but not in cooking talk.
    let has_word = t
        .split(|c: char| !c.is_alphanumeric() && c != '.')
        .any(|w| w.trim_matches('.') == "salt" || w == "s.a.l.t");
    has_word && !FOOD.iter().any(|f| t.contains(f))
}

/// The base instruction plus the assistant's persona (same text the chat has always used).
pub fn system_prompt(app: &AppHandle, a: Option<&crate::db::Assistant>, last_user: &str, about_me: bool) -> String {
    let st = crate::brand::state();
    let brand = if st.effective_name != "TUNAS" { st.effective_name.clone() } else { String::new() };
    let name = match a {
        Some(a) if a.id == "starter-general" && !st.effective_assistant_name.is_empty() => st.effective_assistant_name.clone(),
        Some(a) => a.name.clone(),
        None => "TUNAS".into(),
    };
    let now = chrono::Local::now();
    let today = now.format("%A, %-d %B %Y").to_string();
    // Small models get weekdays wrong ("25 November 2026 (Khamis)"): give them the next two weeks.
    let days: Vec<String> = (1..=14)
        .map(|d| (now + chrono::Duration::days(d)).format("%a %-d %b").to_string())
        .collect();
    let mut s = format!(
        "You are {name}{}.\nYou are part of TUNAS, powered by S.A.L.T CI.\n\
         Always reply in the language of the user's latest message: English if they write in English, Malay (Bahasa Melayu) if they write in Malay, and natural mixed Malay-English if they mix both. If they ask for a specific language, use it.\n\
         Be clear, accurate and well organised. Use Markdown for headings, lists and tables when it helps. If you are not sure, say so instead of guessing.\n\
         Today is {today}. The next days are: {}. Use these weekdays; never work one out yourself.\n\
         Who you are (your name, the app, the brand and S.A.L.T CI) is about you, the assistant, not about the user. When you write a letter, memo, minutes, email or any document for the user, it is from the user and their organisation: never put your own name, TUNAS, S.A.L.T CI, Pemangkin Teknologi or the app's brand in it as the sender, signature, reference, letterhead or content, unless the user asks for them.",
        if !brand.is_empty() && brand != name { format!(", the AI assistant of {brand}") } else { String::new() },
        days.join(", ")
    );
    // The sender for documents: the letterhead from Branding, else placeholders.
    // Only an organisation set as the letterhead counts: the brand's name may be the AI's own name.
    let org = st.letterhead_org.clone();
    let addr = crate::brand::letterhead().address;
    if !org.trim().is_empty() {
        s.push_str(&format!(
            "\n\nThe user's organisation for letters and documents: {}{}. Use it as the sender where a sender is needed.",
            org.trim(),
            if addr.trim().is_empty() { String::new() } else { format!(", {}", addr.trim().replace('\n', ", ")) }
        ));
    } else {
        s.push_str("\n\nWhen a letter or document needs a sender, an address, a reference number or a signature you do not know, write a placeholder in square brackets, such as [Nama Jabatan], [Alamat], [No. Rujukan] or [Nama Pegawai], instead of inventing one.");
    }
    if about_me {
        if let Some(me) = about_me_block(app) {
            s.push_str(&me);
        }
    }
    if let Some(a) = a {
        if !a.persona.trim().is_empty() {
            s.push_str("\n\nYour role:\n");
            s.push_str(a.persona.trim());
        }
    }
    if wants_reference(last_user) {
        s.push_str("\n\nFact (always true): S.A.L.T CI (S.A.L.T Central Intelligence; S.A.L.T = Scalable, Advanced AI, Local, Trustworthy) is a Malaysian sovereign AI platform, owned by S.A.L.T. CI TECHNOLOGY and distributed by Pemangkin Teknologi; TUNAS is its free personal AI app. Never invent other meanings for S.A.L.T.");
        s.push_str("\n\nWhen the user asks about S.A.L.T CI, TERAS, TUNAS, S.A.L.T. CI TECHNOLOGY, Pemangkin Teknologi or the S.A.L.T CI apps, answer only from this reference:\n\n");
        s.push_str(SALT_REFERENCE);
    }
    let _ = app;
    s
}

pub enum Target {
    Local(crate::db::InstalledModel),
    Provider(crate::db::Provider, String),
}

/// Resolve a model string (assistant base model or API `model`) to where it runs.
pub fn resolve_target(app: &AppHandle, model: &str) -> Result<Target, String> {
    let state = app.state::<crate::AppState>();
    if let Some(rest) = model.strip_prefix("prov:") {
        let (pid, m) = rest.split_once(':').ok_or("bad provider model")?;
        let p = crate::db::get_provider(&state.db.lock(), pid).ok_or("That provider is no longer connected. Choose another base model.")?;
        return provider_allowed(app, p, m.to_string());
    }
    if let Some((ps, m)) = model.split_once('/') {
        let providers = crate::db::list_providers(&state.db.lock()).unwrap_or_default();
        if let Some(p) = providers.into_iter().find(|p| slug(&p.name) == ps || p.id == ps) {
            return provider_allowed(app, p, m.to_string());
        }
    }
    crate::resolve_model(&state, model).map(Target::Local).ok_or_else(|| "No model is installed yet. Open Models to download or import one.".into())
}

fn provider_allowed(app: &AppHandle, p: crate::db::Provider, m: String) -> Result<Target, String> {
    if !p.enabled {
        return Err(format!("{} is switched off in Connections.", p.name));
    }
    if p.kind != "teras" {
        let pol = crate::brand::policy();
        let state = app.state::<crate::AppState>();
        let on = crate::db::get_setting(&state.db.lock(), "cloud_enabled").as_deref() == Some("1");
        if !pol.allow_cloud {
            return Err("Your organisation does not allow cloud AI providers on this machine.".into());
        }
        if !on {
            return Err("Cloud providers are switched off. Turn them on in Connections to use this model.".into());
        }
    }
    Ok(Target::Provider(p, m))
}

fn err(code: StatusCode, msg: &str) -> Response {
    (code, Json(json!({"error": {"message": msg, "type": "tunas_error", "code": code.as_u16()}}))).into_response()
}

fn auth(ctx: &Ctx, headers: &HeaderMap) -> Result<Option<String>, Response> {
    let key = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim()
        .to_string();
    let gw = ctx.app.state::<crate::AppState>().gateway.clone();
    if !ctx.external {
        return if key == gw.internal_key { Ok(None) } else { Err(err(StatusCode::UNAUTHORIZED, "invalid key")) };
    }
    let state = ctx.app.state::<crate::AppState>();
    let found = { let c = state.db.lock(); crate::db::touch_app_key(&c, &crate::secret::hash_key(&key)) };
    match found {
        Some(name) => Ok(Some(name)),
        None => Err(err(StatusCode::UNAUTHORIZED, "Invalid TUNAS key. Make a key in TUNAS > Connections > Apps that use TUNAS.")),
    }
}

async fn models(State(ctx): State<Ctx>, headers: HeaderMap) -> Response {
    if let Err(r) = auth(&ctx, &headers) {
        return r;
    }
    let state = ctx.app.state::<crate::AppState>();
    let mut data = vec![];
    let created = chrono::Utc::now().timestamp();
    {
        let c = state.db.lock();
        for a in crate::db::list_assistants(&c).unwrap_or_default() {
            data.push(json!({"id": slug(&a.name), "object": "model", "created": created, "owned_by": "tunas-assistant", "name": a.name}));
        }
        for m in crate::db::list_models(&c).unwrap_or_default() {
            if m.id.contains("embed") || !crate::models::model_path(&m.file).exists() {
                continue;
            }
            data.push(json!({"id": m.id, "object": "model", "created": created, "owned_by": "tunas-local", "name": m.name}));
        }
        let cloud_on = crate::db::get_setting(&c, "cloud_enabled").as_deref() == Some("1") && crate::brand::policy().allow_cloud;
        for p in crate::db::list_providers(&c).unwrap_or_default() {
            if !p.enabled || (p.kind != "teras" && !cloud_on) {
                continue;
            }
            for m in &p.models {
                data.push(json!({"id": format!("{}/{}", slug(&p.name), m), "object": "model", "created": created, "owned_by": p.kind, "name": format!("{} · {}", p.name, m)}));
            }
        }
    }
    Json(json!({"object": "list", "data": data})).into_response()
}

async fn chat(State(ctx): State<Ctx>, headers: HeaderMap, Json(mut body): Json<Value>) -> Response {
    if let Err(r) = auth(&ctx, &headers) {
        return r;
    }
    let app = ctx.app.clone();
    let state = app.state::<crate::AppState>();
    let requested = body.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let internal_assistant = if ctx.external { None } else { body.get("tunas_assistant").and_then(|v| v.as_str()).map(String::from) };
    let raw = body.get("tunas_raw").and_then(|v| v.as_bool()).unwrap_or(false) && !ctx.external;

    // Which assistant (if any) and which base model.
    let assistants = crate::db::list_assistants(&state.db.lock()).unwrap_or_default();
    let assistant = match &internal_assistant {
        Some(id) => assistants.iter().find(|a| &a.id == id).cloned(),
        None => assistants.iter().find(|a| slug(&a.name) == requested || a.id == requested).cloned(),
    };
    let base = match &assistant {
        Some(a) if internal_assistant.is_none() || requested.is_empty() => a.model_id.clone(),
        Some(_) => requested.clone(),
        None => requested.clone(),
    };
    let target = match resolve_target(&app, &base) {
        Ok(t) => t,
        Err(e) => return err(StatusCode::BAD_REQUEST, &e),
    };

    let mut hits: Vec<crate::knowledge::Hit> = vec![];
    // Messages: add the system prompt (persona, language rule, built-in reference) unless raw.
    let mut messages: Vec<Value> = body.get("messages").and_then(|m| m.as_array()).cloned().unwrap_or_default();
    if !raw {
        // The latest three user messages, so follow-up questions ("siapa pemiliknya?") keep the reference.
        let last_user = messages
            .iter()
            .rev()
            .filter(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            .take(3)
            .filter_map(|m| m.get("content"))
            .map(|c| if let Some(s) = c.as_str() { s.chars().take(2000).collect::<String>() } else { c.to_string() })
            .collect::<Vec<_>>()
            .join("\n");
        // About me goes to local and cloud models; TERAS has the person's own personalisation.
        let to_teras = matches!(&target, Target::Provider(p, _) if p.kind == "teras");
        let mut sys = system_prompt(&app, assistant.as_ref(), &last_user, !to_teras);
        // Knowledge: the assistant's sets plus any picked for this chat.
        let mut sets: Vec<String> = assistant.as_ref().map(|a| state.knowledge.assistant_sets(&a.id)).unwrap_or_default();
        if !ctx.external {
            if let Some(extra) = body.get("tunas_knowledge").and_then(|v| v.as_array()) {
                for s in extra.iter().filter_map(|v| v.as_str()) {
                    if !sets.iter().any(|x| x == s) {
                        sets.push(s.to_string());
                    }
                }
            }
        }
        if !sets.is_empty() {
            // Short follow-ups ("and the deadline?") are searched together with the question before.
            let users: Vec<String> = messages
                .iter()
                .rev()
                .filter(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
                .take(2)
                .filter_map(|m| m.get("content").and_then(|c| c.as_str()).map(|s| s.chars().take(1500).collect::<String>()))
                .collect();
            // The latest message on its own first. A short follow-up question ("and the deadline?",
            // "bila tarikhnya?") that finds nothing alone is searched again with the question before it.
            // Greetings and thanks are never joined to an earlier question.
            let last = users.first().cloned().unwrap_or_default();
            hits = state.knowledge.search(&sets, &last, 6).await;
            if hits.is_empty() {
                if let Some(prev) = users.get(1) {
                    if last.chars().count() < 60 && looks_like_question(&last) {
                        hits = state.knowledge.search(&sets, &format!("{prev}\n{last}"), 6).await;
                    }
                }
            }
            let pending = state.knowledge.not_ready(&sets);
            for (name, done, total, no_model) in &pending {
                if *no_model {
                    sys.push_str(&format!("\n\nNote: the knowledge set \"{name}\" has not been read yet because TUNAS's search model is still downloading. If the user asks about those documents, tell them the documents are not ready yet and to try again when the Knowledge page shows them as ready."));
                } else {
                    sys.push_str(&format!("\n\nNote: the knowledge set \"{name}\" is still being read ({done} of {total} files). If the answer is not in the excerpts, tell the user that the documents are still being read and to try again shortly."));
                }
            }
            // A TERAS model may cite its own knowledge as [1], [2]...; the user's local excerpts
            // start at [51] so the two never share a number.
            if matches!(&target, Target::Provider(p, _) if p.kind == "teras") {
                for (i, h) in hits.iter_mut().enumerate() {
                    h.n = 51 + i;
                }
            }
            if !hits.is_empty() {
                let outline = state.knowledge.outline(&hits[0].path);
                sys.push_str(&crate::knowledge::prompt_block(&hits, &outline));
            } else {
                sys.push_str("\n\nNo passages from the user's documents match this message. Answer it normally, without mentioning the documents and without citation numbers. Only if the user asks about their documents, say that you could not find it in them.");
            }
        }
        // Keep an app's own system message after ours.
        messages.insert(0, json!({"role": "system", "content": sys}));
    }
    if let Some(o) = body.as_object_mut() {
        o.remove("tunas_assistant");
        o.remove("tunas_raw");
        o.remove("tunas_knowledge");
        o.insert("messages".into(), Value::Array(messages));
    }

    let stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let (url, key) = match &target {
        Target::Local(m) => {
            let prefer = crate::db::get_setting(&state.db.lock(), "engine_pref").unwrap_or_else(|| "auto".into());
            let st = match state.engine.ensure(&app, &m.id, crate::models::model_path(&m.file), &prefer).await {
                Ok(s) => s,
                Err(e) => return err(StatusCode::SERVICE_UNAVAILABLE, &e.to_string()),
            };
            // Thinking: the request may say (TUNAS chat does); otherwise use the model's setting.
            let mode = crate::db::get_setting(&state.db.lock(), &format!("think:{}", m.id)).unwrap_or_else(|| "off".into());
            if let Some(o) = body.as_object_mut() {
                o.insert("model".into(), json!(m.id));
                let think = o
                    .get("chat_template_kwargs")
                    .and_then(|k| k.get("enable_thinking"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(mode != "off");
                o.insert("chat_template_kwargs".into(), json!({"enable_thinking": think}));
                if think {
                    // Small models can think in circles until nothing is left for the answer. A thinking
                    // budget makes the engine close the thinking and answer; the sampling Qwen recommends for
                    // thinking (top_k 20, presence penalty 1.5) stops the loops. Tested: without these the
                    // model looped for 5,900 tokens with no answer; with them it answered in about a minute.
                    let len = crate::db::get_setting(&state.db.lock(), &format!("think_len:{}", m.id)).unwrap_or_default();
                    let budget: u64 = match len.as_str() { "short" => 512, "long" => 2048, _ => 1024 };
                    if !o.contains_key("thinking_budget_tokens") && !o.contains_key("reasoning_budget_tokens") {
                        o.insert("thinking_budget_tokens".into(), json!(budget));
                        o.insert("reasoning_budget_message".into(), json!("\n\nOkay, I have thought enough. Time to answer now.\n"));
                    }
                    for (k, v) in [("top_k", json!(20)), ("top_p", json!(0.95)), ("presence_penalty", json!(1.5))] {
                        if !o.contains_key(k) { o.insert(k.into(), v); }
                    }
                    let asked = o.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    o.insert("max_tokens".into(), json!(asked.max(budget + 4096)));
                }
            }
            (format!("http://127.0.0.1:{}/v1/chat/completions", st.port), st.key)
        }
        Target::Provider(p, m) => {
            if let Some(o) = body.as_object_mut() {
                o.insert("model".into(), json!(m));
                o.remove("chat_template_kwargs");
                o.remove("timings_per_token");
                o.remove("cache_prompt");
                // TUNAS's own sampling settings are tuned for the small local models. TERAS and cloud
                // models choose their own, and some newer models refuse them outright ("`temperature`
                // is deprecated for this model"), so TUNAS's chat leaves them to the provider.
                // Settings sent by other apps through the API are passed on as they are.
                if !ctx.external {
                    for k in SAMPLING { o.remove(*k); }
                    // TUNAS's chat asks TERAS for its tool and source events, to show what TERAS is doing.
                    if p.kind == "teras" {
                        o.insert("teras".into(), json!({"events": true}));
                    }
                }
            }
            let key = crate::secret::reveal(&p.key_enc).unwrap_or_default();
            (format!("{}/chat/completions", p.base_url.trim_end_matches('/')), key)
        }
    };

    let client = reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(15)).build().unwrap();
    let mut req = client.post(&url).json(&body);
    if !key.is_empty() {
        req = req.bearer_auth(&key);
    }
    if let Target::Provider(p, _) = &target {
        if p.kind == "openrouter" {
            req = req.header("HTTP-Referer", "https://www.salt.ci").header("X-Title", "TUNAS");
        }
    }
    let mut resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            let who = match &target { Target::Provider(p, _) => p.name.clone(), _ => "the local engine".into() };
            return err(StatusCode::BAD_GATEWAY, &format!("Could not reach {who}: {e}"));
        }
    };
    // A provider that refuses a sampling setting (temperature, top_p…) gets the request once more
    // without them, so an app's settings never stop the answer altogether.
    if let Target::Provider(p, _) = &target {
        if resp.status().as_u16() == 400 && body.as_object().map(|o| SAMPLING.iter().any(|k| o.contains_key(*k))).unwrap_or(false) {
            let text = resp.text().await.unwrap_or_default();
            let low = text.to_lowercase();
            let about_sampling = SAMPLING.iter().any(|k| low.contains(k)) || low.contains("deprecated") || low.contains("not supported") || low.contains("unsupported");
            if about_sampling {
                if let Some(o) = body.as_object_mut() {
                    for k in SAMPLING { o.remove(*k); }
                }
                let mut again = client.post(&url).json(&body);
                if !key.is_empty() { again = again.bearer_auth(&key); }
                if p.kind == "openrouter" {
                    again = again.header("HTTP-Referer", "https://www.salt.ci").header("X-Title", "TUNAS");
                }
                resp = match again.send().await {
                    Ok(r) => r,
                    Err(e) => return err(StatusCode::BAD_GATEWAY, &format!("Could not reach {}: {e}", p.name)),
                };
            } else {
                let code = StatusCode::BAD_REQUEST;
                return Response::builder().status(code).header("content-type", "application/json").body(Body::from(text))
                    .unwrap_or_else(|_| err(code, "response error"));
            }
        }
    }
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let ctype = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or(if stream { "text/event-stream" } else { "application/json" }).to_string();
    // Indexing waits while this answer is being written (the guard lives as long as the stream).
    let guard = crate::knowledge::ChatGuard::new();
    let log_teras = !ctx.external && matches!(&target, Target::Provider(p, _) if p.kind == "teras");
    let upstream = resp.bytes_stream().map(move |x| {
        let _ = &guard;
        if log_teras {
            if let Ok(b) = &x {
                let t = String::from_utf8_lossy(b);
                for line in t.lines().filter(|l| l.contains("\"teras\"")) {
                    teras_log(&line.chars().take(1500).collect::<String>());
                }
            }
        }
        x
    });
    // TUNAS's own chat gets the sources first, as one extra event before the answer.
    let first: Vec<Result<bytes::Bytes, reqwest::Error>> = if stream && !ctx.external && !hits.is_empty() && status.is_success() {
        let src: Vec<Value> = hits.iter().map(|h| json!({"n": h.n, "name": h.name, "path": h.path, "label": h.label, "page": h.page, "set": h.set})).collect();
        vec![Ok(bytes::Bytes::from(format!("data: {}\n\n", json!({"tunas_sources": src}))))]
    } else {
        vec![]
    };
    let body_stream = futures_util::stream::iter(first).chain(upstream);
    Response::builder()
        .status(status)
        .header("content-type", ctype)
        .header("cache-control", "no-cache")
        .body(Body::from_stream(body_stream))
        .unwrap_or_else(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "response error"))
}

fn router(app: AppHandle, external: bool) -> Router {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    Router::new()
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .route("/health", get(|| async { "ok" }))
        .layer(cors)
        .with_state(Ctx { app, external })
}

/// Start the internal listener (called once at start-up).
pub async fn start_internal(app: AppHandle) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    *app.state::<crate::AppState>().gateway.internal_port.lock() = port;
    let r = router(app.clone(), false);
    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, r).await;
    });
    Ok(())
}

/// Start or stop the app listener on `port` (127.0.0.1 only).
pub async fn set_external(app: AppHandle, enabled: bool, port: u16) -> anyhow::Result<()> {
    let gw = app.state::<crate::AppState>().gateway.clone();
    if let Some((_, stop)) = gw.external.lock().take() {
        let _ = stop.send(());
    }
    if !enabled {
        return Ok(());
    }
    if !crate::brand::policy().allow_local_api {
        anyhow::bail!("your organisation does not allow other apps to use TUNAS on this machine");
    }
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| anyhow::anyhow!("port {port} is in use by another program ({e}); choose another port"))?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let r = router(app.clone(), true);
    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, r).with_graceful_shutdown(async move {
            let _ = rx.await;
        }).await;
    });
    *gw.external.lock() = Some((port, tx));
    Ok(())
}

pub fn external_port(app: &AppHandle) -> Option<u16> {
    app.state::<crate::AppState>().gateway.external.lock().as_ref().map(|(p, _)| *p)
}

#[cfg(test)]
mod tests {
    use super::wants_reference;
    #[test]
    fn triggers() {
        for q in ["salt tu apa ?", "SALT tu apa", "Apa itu S.A.L.T?", "siapa pemilik salt ci", "Tell me about TERAS", "what is salt?", "pemangkin teknologi"] {
            assert!(wants_reference(q), "should trigger: {q}");
        }
        for q in ["add a pinch of salt and pepper", "berapa sudu garam dan salt", "how do I write a letter", "salty food is bad"] {
            assert!(!wants_reference(q), "should not trigger: {q}");
        }
    }
}
