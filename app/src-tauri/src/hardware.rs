//! "Check my notebook": reads RAM, processor and graphics, and recommends a model size and engine.

use serde::Serialize;
use sysinfo::{Disks, System};

#[derive(Serialize, Clone, Debug, Default)]
pub struct Gpu {
    pub name: String,
    pub vram_mb: u64,
    pub integrated: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Hardware {
    pub ram_gb: f64,
    pub ram_free_gb: f64,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub gpus: Vec<Gpu>,
    pub vulkan_devices: Vec<String>,
    pub disk_free_gb: f64,
    pub os: String,
    pub recommended_models: Vec<String>,
    pub recommended_engine: String,
    pub verdict: String,
    pub notes: Vec<String>,
}

#[cfg(windows)]
fn hidden(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn hidden(_cmd: &mut std::process::Command) {}

fn read_gpus() -> Vec<Gpu> {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM | ConvertTo-Json -Compress",
        ]);
        hidden(&mut cmd);
        if let Ok(out) = cmd.output() {
            let txt = String::from_utf8_lossy(&out.stdout).to_string();
            let v: serde_json::Value = serde_json::from_str(txt.trim()).unwrap_or(serde_json::Value::Null);
            let arr = match v {
                serde_json::Value::Array(a) => a,
                serde_json::Value::Object(_) => vec![v],
                _ => vec![],
            };
            return arr
                .into_iter()
                .filter_map(|g| {
                    let name = g.get("Name")?.as_str()?.to_string();
                    let vram = g.get("AdapterRAM").and_then(|x| x.as_u64()).unwrap_or(0) / (1024 * 1024);
                    let l = name.to_lowercase();
                    let integrated = l.contains("intel") && !l.contains("arc a")
                        || l.contains("radeon(tm) graphics")
                        || l.contains("radeon graphics")
                        || l.contains("vega")
                        || l.contains("basic render");
                    Some(Gpu { name, vram_mb: vram, integrated })
                })
                .collect();
        }
        vec![]
    }
    #[cfg(not(windows))]
    {
        vec![]
    }
}

/// Ask the Vulkan build of the engine which devices it can use.
pub fn vulkan_devices() -> Vec<String> {
    let exe = crate::engine::server_exe("vulkan");
    if !exe.exists() {
        return vec![];
    }
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--list-devices");
    cmd.current_dir(exe.parent().unwrap());
    hidden(&mut cmd);
    let out = match cmd.output() {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let txt = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    txt.lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("Vulkan"))
        .map(|l| l.to_string())
        .filter(|l| !l.to_lowercase().contains("llvmpipe") && !l.to_lowercase().contains("basic render"))
        .collect()
}

pub fn check() -> Hardware {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    let ram_gb = sys.total_memory() as f64 / 1024f64.powi(3);
    let ram_free_gb = sys.available_memory() as f64 / 1024f64.powi(3);
    let cpu_name = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default();
    let cpu_threads = sys.cpus().len();
    let cpu_cores = sys.physical_core_count().unwrap_or(cpu_threads.max(2) / 2).max(1);

    let data = crate::paths::data_dir();
    let disks = Disks::new_with_refreshed_list();
    let disk_free_gb = disks
        .list()
        .iter()
        .filter(|d| data.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
        .map(|d| d.available_space() as f64 / 1024f64.powi(3))
        .unwrap_or(0.0);

    let gpus = read_gpus();
    let vk = vulkan_devices();
    let mut notes = vec![];

    let dedicated = gpus.iter().any(|g| !g.integrated && g.vram_mb >= 3500);
    let recommended_engine = if !vk.is_empty() { "vulkan" } else { "cpu" }.to_string();
    if vk.is_empty() {
        notes.push("No usable graphics for AI was found, so TUNAS will use the processor. Small models are recommended.".into());
    } else if dedicated {
        notes.push("A graphics card was found. TUNAS will use it to speed up answers.".into());
    } else {
        notes.push("Built-in graphics were found. TUNAS will try them and fall back to the processor if they are slower.".into());
    }

    let (models, verdict): (Vec<&str>, &str) = if ram_gb < 5.5 {
        (vec!["qwen3.5-0.8b"], "This notebook has little memory. TUNAS will run, but only very small models.")
    } else if ram_gb < 11.0 {
        (
            vec!["qwen3.5-2b", "qwen3.5-4b"],
            "Good for everyday use with small models. The 2B model is the quick default; the 4B model writes better but is slower.",
        )
    } else if ram_gb < 24.0 || !dedicated {
        (vec!["qwen3.5-4b", "qwen3.5-2b", "granite-4.2-3b"], "Comfortable for 3–4B models.")
    } else {
        (vec!["qwen3.5-4b", "granite-4.2-3b", "qwen3.5-2b"], "Strong notebook. Small models will be fast; larger ones are possible.")
    };
    if disk_free_gb > 0.0 && disk_free_gb < 8.0 {
        notes.push(format!("Only {:.0} GB of free disk space. Each model needs 1–3 GB.", disk_free_gb));
    }
    if cpu_cores <= 2 {
        notes.push("The processor has few cores, so answers will be slow on the processor.".into());
    }

    Hardware {
        ram_gb: (ram_gb * 10.0).round() / 10.0,
        ram_free_gb: (ram_free_gb * 10.0).round() / 10.0,
        cpu_name,
        cpu_cores,
        cpu_threads,
        gpus,
        vulkan_devices: vk,
        disk_free_gb: (disk_free_gb * 10.0).round() / 10.0,
        os: System::long_os_version().unwrap_or_default(),
        recommended_models: models.into_iter().map(String::from).collect(),
        recommended_engine,
        verdict: verdict.into(),
        notes,
    }
}
