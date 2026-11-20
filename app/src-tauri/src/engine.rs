//! The AI engine: a bundled llama.cpp server, one model loaded at a time.
//!
//! TUNAS starts `llama-server.exe` on 127.0.0.1 with a random port and key, so only TUNAS
//! can talk to it. On Windows the process is placed in a job object, so it closes when
//! TUNAS closes (even after a crash).

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use serde::Serialize;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub fn server_exe(kind: &str) -> PathBuf {
    let name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    crate::paths::engine_dir(kind).join(name)
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct EngineStatus {
    pub state: String, // stopped | starting | ready | error
    pub model_id: String,
    pub engine: String, // cpu | vulkan
    pub port: u16,
    pub key: String,
    pub ctx: u32,
    pub message: String,
    /// Why the engine is not on graphics, when graphics was tried and failed.
    pub note: String,
}

pub struct Running {
    child: Child,
    status: EngineStatus,
    prefer: String,
}

#[derive(Clone, Default)]
pub struct Engine {
    inner: Arc<Mutex<Option<Running>>>,
    status: Arc<Mutex<EngineStatus>>,
    start_lock: Arc<tokio::sync::Mutex<()>>,
    #[cfg(windows)]
    job: Arc<Mutex<Option<isize>>>,
}

impl Engine {
    pub fn status(&self) -> EngineStatus {
        self.status.lock().clone()
    }

    fn set_status(&self, app: &AppHandle, s: EngineStatus) {
        *self.status.lock() = s.clone();
        let _ = app.emit("engine-status", s);
    }

    pub fn stop(&self) {
        if let Some(mut r) = self.inner.lock().take() {
            let _ = r.child.kill();
            let _ = r.child.wait();
        }
        let mut s = self.status.lock();
        s.state = "stopped".into();
        s.model_id.clear();
    }

    /// Make sure `model_id` is loaded and the engine answers. Returns the ready status.
    pub async fn ensure(&self, app: &AppHandle, model_id: &str, model_path: PathBuf, prefer: &str) -> Result<EngineStatus> {
        let _guard = self.start_lock.lock().await;
        {
            let mut g = self.inner.lock();
            if let Some(r) = g.as_mut() {
                let alive = matches!(r.child.try_wait(), Ok(None));
                // Restart when the model or the "Run AI on" choice changed.
                if alive && r.status.model_id == model_id && r.status.state == "ready" && r.prefer == prefer {
                    return Ok(r.status.clone());
                }
            }
        }
        self.stop();

        // Graphics first when asked for, or on "auto" when a usable graphics card was found and graphics
        // has not already failed with this engine version on this machine.
        let vk_failed_key = format!("vk_failed:{}", env!("CARGO_PKG_VERSION"));
        let vk_failed = {
            use tauri::Manager;
            let st = app.state::<crate::AppState>();
            let c = st.db.lock();
            crate::db::get_setting(&c, &vk_failed_key).filter(|v| !v.is_empty())
        };
        let try_vk = match prefer {
            "vulkan" => true,
            "cpu" => false,
            _ => vk_failed.is_none() && crate::hardware::has_usable_gpu(),
        };
        let mut order: Vec<&str> = vec![];
        if try_vk {
            order.push("vulkan");
        }
        order.push("cpu");

        let mut last_err = anyhow!("no engine available");
        let mut note = String::new();
        if !try_vk && prefer == "auto" {
            if let Some(why) = &vk_failed {
                note = format!("Graphics did not start earlier on this machine ({why}). Choose Graphics in Settings to try again.");
            }
        }
        for kind in order {
            if !server_exe(kind).exists() {
                last_err = anyhow!("engine '{}' is not installed", kind);
                continue;
            }
            match self.start(app, model_id, &model_path, kind, prefer).await {
                Ok(mut s) => {
                    if kind == "vulkan" && vk_failed.is_some() {
                        use tauri::Manager;
                        let st = app.state::<crate::AppState>();
                        let _ = crate::db::set_setting(&st.db.lock(), &vk_failed_key, "");
                    }
                    if !note.is_empty() {
                        s.note = note.clone();
                        if let Some(r) = self.inner.lock().as_mut() { r.status.note = note.clone(); }
                        self.set_status(app, s.clone());
                    }
                    return Ok(s);
                }
                Err(e) => {
                    if kind == "vulkan" {
                        note = format!("Graphics did not start: {e}. Running on the processor instead.");
                        use tauri::Manager;
                        let st = app.state::<crate::AppState>();
                        let _ = crate::db::set_setting(&st.db.lock(), &vk_failed_key, &e.to_string());
                    }
                    last_err = e;
                    self.stop();
                }
            }
        }
        let st = EngineStatus { state: "error".into(), message: last_err.to_string(), model_id: model_id.into(), ..Default::default() };
        self.set_status(app, st);
        Err(last_err)
    }

    async fn start(&self, app: &AppHandle, model_id: &str, model_path: &PathBuf, kind: &str, prefer: &str) -> Result<EngineStatus> {
        let port = free_port()?;
        let key = random_key();
        let hw = sysinfo_ram_gb();
        let ctx: u32 = if hw >= 15.0 { 16384 } else { 8192 };
        let threads = sysinfo::System::new().physical_core_count().unwrap_or(4).clamp(1, 16);
        let exe = server_exe(kind);
        let log_path = crate::paths::logs_dir().join(format!("engine-{kind}.log"));
        let _ = std::fs::remove_file(&log_path);

        let mut cmd = Command::new(&exe);
        cmd.current_dir(exe.parent().unwrap())
            .arg("-m").arg(model_path)
            .args(["--host", "127.0.0.1", "--port", &port.to_string()])
            .args(["--api-key", &key])
            .args(["-c", &ctx.to_string()])
            .args(["-t", &threads.to_string()])
            .args(["-np", "1"])
            .args(["-ngl", if kind == "vulkan" { "999" } else { "0" }])
            .args(["--cache-ram", if hw >= 15.0 { "512" } else { "0" }])
            .args(["--jinja", "--no-webui"])
            .arg("--log-file").arg(&log_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS would slow answers; keep normal priority.
            cmd.creation_flags(0x0800_0000);
        }

        let status = EngineStatus {
            state: "starting".into(),
            model_id: model_id.into(),
            engine: kind.into(),
            port,
            key: key.clone(),
            ctx,
            message: format!("Loading the model ({})...", if kind == "vulkan" { "graphics" } else { "processor" }),
            note: String::new(),
        };
        self.set_status(app, status.clone());

        let child = cmd.spawn().map_err(|e| anyhow!("could not start the engine: {e}"))?;
        #[cfg(windows)]
        self.attach_job(&child);
        *self.inner.lock() = Some(Running { child, status: status.clone(), prefer: prefer.to_string() });

        // Wait for /health to answer 200.
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/health");
        let t0 = Instant::now();
        loop {
            {
                let mut g = self.inner.lock();
                if let Some(r) = g.as_mut() {
                    if let Ok(Some(code)) = r.child.try_wait() {
                        return Err(anyhow!("the engine stopped while loading (exit {code}); see logs\\engine-{kind}.log"));
                    }
                } else {
                    return Err(anyhow!("engine was stopped"));
                }
            }
            if let Ok(resp) = client.get(&url).timeout(Duration::from_secs(2)).send().await {
                if resp.status().is_success() {
                    break;
                }
            }
            // Graphics can need extra time the first time it prepares its programs.
            if t0.elapsed() > Duration::from_secs(if kind == "vulkan" { 300 } else { 180 }) {
                return Err(anyhow!("the engine took too long to load the model"));
            }
            tokio::time::sleep(Duration::from_millis(400)).await;
        }

        let mut ready = status.clone();
        ready.state = "ready".into();
        ready.message = "Ready".into();
        if let Some(r) = self.inner.lock().as_mut() {
            r.status = ready.clone();
        }
        self.set_status(app, ready.clone());
        Ok(ready)
    }

    #[cfg(windows)]
    fn attach_job(&self, child: &Child) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let mut job = self.job.lock();
            if job.is_none() {
                let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if !h.is_null() {
                    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                    SetInformationJobObject(
                        h,
                        JobObjectExtendedLimitInformation,
                        &info as *const _ as *const _,
                        std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    );
                    *job = Some(h as isize);
                }
            }
            if let Some(h) = *job {
                AssignProcessToJobObject(h as _, child.as_raw_handle() as _);
            }
        }
    }
}

/// Put another TUNAS helper process (the search engine) in a job that Windows closes with TUNAS.
#[cfg(windows)]
pub fn attach_to_job(child: &Child) {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::*;
    static JOB: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    unsafe {
        let h = *JOB.get_or_init(|| {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if h.is_null() {
                return 0;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(h, JobObjectExtendedLimitInformation, &info as *const _ as *const _, std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32);
            h as isize
        });
        if h != 0 {
            AssignProcessToJobObject(h as _, child.as_raw_handle() as _);
        }
    }
}

fn sysinfo_ram_gb() -> f64 {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    s.total_memory() as f64 / 1024f64.powi(3)
}

fn free_port() -> Result<u16> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(l.local_addr()?.port())
}

fn random_key() -> String {
    use rand::Rng;
    let bytes: [u8; 24] = rand::thread_rng().gen();
    hex::encode(bytes)
}
