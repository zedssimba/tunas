//! Co-branding: OEM / reseller, organisation and personal layers, branding packs, icons and shortcuts.
//!
//! Layers (most specific shown first): personal > org > oem. "Powered by S.A.L.T CI" is always shown
//! by the interface and is not part of any layer, so it cannot be removed.

use anyhow::{anyhow, Result};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Brand {
    pub name: String,
    pub assistant_name: String,
    pub welcome: String,
    pub color1: String,
    pub color2: String,
    pub logo_file: String,
    pub letterhead_org: String,
    pub letterhead_address: String,
    pub issuer: String,
    pub updated: String,
    /// '' (transparent, default) | 'light' | 'dark'
    pub logo_bg: String,
    /// 'square' (rounded) | 'circle'
    pub logo_shape: String,
    /// Also use the logo as the app icon (taskbar, desktop, Start menu). Off by default: the TUNAS icon stays.
    pub logo_as_icon: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Policy {
    pub allow_change_org_brand: bool,
    pub allow_personal_brand: bool,
    pub allow_create_assistants: bool,
    pub allow_cloud: bool,
    pub allow_knowledge_with_cloud: bool,
    pub allow_local_api: bool,
    pub source: String,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            allow_change_org_brand: true,
            allow_personal_brand: true,
            allow_create_assistants: true,
            allow_cloud: true,
            allow_knowledge_with_cloud: true,
            allow_local_api: true,
            source: "default".into(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct LayerView {
    pub layer: String,
    pub brand: Option<Brand>,
    pub logo_data_url: String,
    pub machine: bool,
    pub editable: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct BrandState {
    pub layers: Vec<LayerView>,
    pub effective_name: String,
    pub effective_assistant_name: String,
    pub effective_welcome: String,
    pub color1: String,
    pub color2: String,
    pub logo_data_url: String,
    pub logo_bg: String,
    pub logo_shape: String,
    pub letterhead_org: String,
    pub policy: Policy,
}

pub const DEFAULT_C1: &str = "#2F74D8";
pub const DEFAULT_C2: &str = "#6D4FE0";

fn machine_layer_dir(layer: &str) -> PathBuf {
    crate::paths::machine_dir().join("brand").join(layer)
}
fn user_layer_dir(layer: &str) -> PathBuf {
    crate::paths::user_brand_root().join(layer)
}

/// Where a layer lives. OEM is machine-only. Org is machine-wide if IT installed it there,
/// otherwise per user. Personal is per user.
fn layer_dir(layer: &str) -> (PathBuf, bool) {
    match layer {
        "oem" => (machine_layer_dir("oem"), true),
        "org" => {
            let m = machine_layer_dir("org");
            if m.join("brand.json").exists() {
                (m, true)
            } else {
                (user_layer_dir("org"), false)
            }
        }
        _ => (user_layer_dir("personal"), false),
    }
}

fn read_brand(dir: &Path) -> Option<Brand> {
    let txt = std::fs::read_to_string(dir.join("brand.json")).ok()?;
    let b: Brand = serde_json::from_str(&txt).ok()?;
    if b.name.trim().is_empty() && b.logo_file.is_empty() {
        return None;
    }
    Some(b)
}

fn data_url(path: &Path) -> String {
    let Ok(bytes) = std::fs::read(path) else { return String::new() };
    let mime = match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "image/png",
    };
    format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn policy() -> Policy {
    // Machine policy (set by IT, normal users cannot edit) wins over a policy from a pack.
    let machine = crate::paths::machine_dir().join("policy.json");
    if let Ok(t) = std::fs::read_to_string(&machine) {
        if let Ok(mut p) = serde_json::from_str::<Policy>(&t) {
            p.source = "machine".into();
            return p;
        }
    }
    let user = crate::paths::user_brand_root().join("org").join("policy.json");
    if let Ok(t) = std::fs::read_to_string(&user) {
        if let Ok(mut p) = serde_json::from_str::<Policy>(&t) {
            p.source = "pack".into();
            return p;
        }
    }
    Policy::default()
}

pub fn state() -> BrandState {
    let pol = policy();
    let mut layers = vec![];
    for layer in ["personal", "org", "oem"] {
        let (dir, machine) = layer_dir(layer);
        let brand = read_brand(&dir);
        let logo = brand
            .as_ref()
            .filter(|b| !b.logo_file.is_empty())
            .map(|b| data_url(&dir.join(&b.logo_file)))
            .unwrap_or_default();
        let editable = match layer {
            "oem" => false,
            "org" => !machine && pol.allow_change_org_brand,
            _ => pol.allow_personal_brand,
        };
        layers.push(LayerView { layer: layer.into(), brand, logo_data_url: logo, machine, editable });
    }
    let pick = |f: &dyn Fn(&Brand) -> String| -> String {
        layers
            .iter()
            .filter(|l| l.layer != "personal" || pol.allow_personal_brand)
            .filter_map(|l| l.brand.as_ref())
            .map(|b| f(b))
            .find(|s| !s.trim().is_empty())
            .unwrap_or_default()
    };
    let logo_layer = layers
        .iter()
        .filter(|l| l.layer != "personal" || pol.allow_personal_brand)
        .find(|l| !l.logo_data_url.is_empty());
    let logo = logo_layer.map(|l| l.logo_data_url.clone()).unwrap_or_default();
    let logo_bg = logo_layer.and_then(|l| l.brand.as_ref()).map(|b| b.logo_bg.clone()).unwrap_or_default();
    let logo_shape = logo_layer.and_then(|l| l.brand.as_ref()).map(|b| b.logo_shape.clone()).unwrap_or_default();
    let name = pick(&|b| b.name.clone());
    let c1 = pick(&|b| b.color1.clone());
    let c2 = pick(&|b| b.color2.clone());
    BrandState {
        effective_name: if name.is_empty() { "TUNAS".into() } else { name },
        effective_assistant_name: pick(&|b| b.assistant_name.clone()),
        effective_welcome: pick(&|b| b.welcome.clone()),
        color1: if c1.is_empty() { DEFAULT_C1.into() } else { c1 },
        color2: if c2.is_empty() { DEFAULT_C2.into() } else { c2 },
        logo_data_url: logo,
        logo_bg,
        logo_shape,
        letterhead_org: pick(&|b| b.letterhead_org.clone()),
        layers,
        policy: pol,
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct LogoCheck {
    pub ok: bool,
    pub width: u32,
    pub height: u32,
    pub warnings: Vec<String>,
    pub preview_data_url: String,
}

/// Load any logo (PNG, JPG or SVG) as an RGBA image.
/// What goes at the top of a document: organisation name, address and logo (as PNG), from the brand layers.
pub struct Letterhead {
    pub org: String,
    pub address: String,
    pub logo_png: Option<Vec<u8>>,
}

pub fn letterhead() -> Letterhead {
    let st = state();
    let usable = |l: &&LayerView| l.layer != "personal" || st.policy.allow_personal_brand;
    let pick = |f: &dyn Fn(&Brand) -> String| -> String {
        st.layers.iter().filter(usable).filter_map(|l| l.brand.as_ref()).map(f).find(|s| !s.trim().is_empty()).unwrap_or_default()
    };
    let org = {
        let o = pick(&|b| b.letterhead_org.clone());
        if o.is_empty() { pick(&|b| b.name.clone()) } else { o }
    };
    let address = pick(&|b| b.letterhead_address.clone());
    let logo_png = st
        .layers
        .iter()
        .filter(usable)
        .find_map(|l| {
            let b = l.brand.as_ref()?;
            if b.logo_file.is_empty() {
                return None;
            }
            Some(layer_dir(&l.layer).0.join(&b.logo_file))
        })
        .and_then(|p| load_logo(&p).ok())
        .and_then(|img| {
            let mut buf = std::io::Cursor::new(vec![]);
            image::DynamicImage::ImageRgba8(img).write_to(&mut buf, image::ImageFormat::Png).ok()?;
            Some(buf.into_inner())
        });
    Letterhead { org, address, logo_png }
}

fn load_logo(path: &Path) -> Result<image::RgbaImage> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "svg" {
        let data = std::fs::read(path)?;
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&data, &opt).map_err(|e| anyhow!("this SVG could not be read: {e}"))?;
        let size = tree.size();
        let scale = 512.0 / size.width().max(size.height()).max(1.0);
        let w = (size.width() * scale).ceil().max(1.0) as u32;
        let h = (size.height() * scale).ceil().max(1.0) as u32;
        let mut pix = resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| anyhow!("SVG too large"))?;
        resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pix.as_mut());
        // tiny-skia stores premultiplied RGBA; convert to straight alpha.
        let mut img = image::RgbaImage::new(w, h);
        for (i, px) in pix.pixels().iter().enumerate() {
            let c = px.demultiply();
            img.put_pixel(i as u32 % w, i as u32 / w, image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]));
        }
        Ok(img)
    } else {
        Ok(image::open(path).map_err(|e| anyhow!("this image could not be read: {e}"))?.to_rgba8())
    }
}

/// Trim transparent edges and fit into a square with a little padding.
fn square_icon(img: &image::RgbaImage, size: u32) -> image::RgbaImage {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 16 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    let cropped = if x1 >= x0 && y1 >= y0 {
        image::imageops::crop_imm(img, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image()
    } else {
        img.clone()
    };
    let (cw, ch) = cropped.dimensions();
    let inner = (size as f64 * 0.9) as u32;
    let scale = inner as f64 / cw.max(ch) as f64;
    let nw = ((cw as f64 * scale).round() as u32).max(1);
    let nh = ((ch as f64 * scale).round() as u32).max(1);
    let resized = image::imageops::resize(&cropped, nw, nh, image::imageops::FilterType::Lanczos3);
    let mut canvas = image::RgbaImage::new(size, size);
    image::imageops::overlay(&mut canvas, &resized, ((size - nw) / 2) as i64, ((size - nh) / 2) as i64);
    canvas
}

fn png_data_url(img: &image::RgbaImage) -> String {
    let mut buf = std::io::Cursor::new(vec![]);
    let _ = image::DynamicImage::ImageRgba8(img.clone()).write_to(&mut buf, image::ImageFormat::Png);
    format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(buf.into_inner()))
}

/// Put the icon on a light or dark plate (rounded square or circle) so it stays visible on any background.
fn on_plate(icon: &image::RgbaImage, size: u32, bg: &str, shape: &str) -> image::RgbaImage {
    use resvg::tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};
    let (r, g, b) = if bg == "dark" { (0x0B, 0x10, 0x22) } else { (0xFF, 0xFF, 0xFF) };
    let mut pm = Pixmap::new(size, size).unwrap();
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint.anti_alias = true;
    let s = size as f32;
    let path = if shape == "circle" {
        PathBuilder::from_circle(s / 2.0, s / 2.0, s / 2.0)
    } else {
        let rad = s * 0.22;
        let mut pb = PathBuilder::new();
        pb.move_to(rad, 0.0);
        pb.line_to(s - rad, 0.0);
        pb.quad_to(s, 0.0, s, rad);
        pb.line_to(s, s - rad);
        pb.quad_to(s, s, s - rad, s);
        pb.line_to(rad, s);
        pb.quad_to(0.0, s, 0.0, s - rad);
        pb.line_to(0.0, rad);
        pb.quad_to(0.0, 0.0, rad, 0.0);
        pb.close();
        pb.finish()
    };
    if let Some(p) = path {
        pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
    }
    let _ = Rect::from_xywh(0.0, 0.0, s, s);
    let mut out = image::RgbaImage::new(size, size);
    for (i, px) in pm.pixels().iter().enumerate() {
        let c = px.demultiply();
        out.put_pixel(i as u32 % size, i as u32 / size, image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]));
    }
    let inner = ((size as f64) * if shape == "circle" { 0.66 } else { 0.74 }).round() as u32;
    let small = image::imageops::resize(icon, inner.max(1), inner.max(1), image::imageops::FilterType::Lanczos3);
    image::imageops::overlay(&mut out, &small, ((size - inner) / 2) as i64, ((size - inner) / 2) as i64);
    out
}

pub fn check_logo(path: &Path) -> LogoCheck {
    match load_logo(path) {
        Err(e) => LogoCheck { ok: false, warnings: vec![e.to_string()], ..Default::default() },
        Ok(img) => {
            let (w, h) = img.dimensions();
            let mut warnings = vec![];
            let is_svg = path.extension().map(|e| e.eq_ignore_ascii_case("svg")).unwrap_or(false);
            if !is_svg && w.max(h) < 128 {
                warnings.push(format!("The logo is small ({w}×{h}). It may look blurry. 256×256 or larger, or an SVG, works best."));
            }
            let ratio = w.max(h) as f64 / w.min(h).max(1) as f64;
            if ratio > 2.5 {
                warnings.push("The logo is very wide. It will be hard to read as a small icon; a square version of the logo works better for the icon.".into());
            }
            let preview = square_icon(&img, 160);
            LogoCheck { ok: true, width: w, height: h, warnings, preview_data_url: png_data_url(&preview) }
        }
    }
}

/// Save a layer. `logo_path` is a new logo picked by the user, or None to keep the current one.
pub fn save_layer(layer: &str, mut b: Brand, logo_path: Option<PathBuf>, remove_logo: bool) -> Result<()> {
    if layer == "oem" {
        return Err(anyhow!("the OEM brand is set by the installer and cannot be changed here"));
    }
    let st = state();
    let lv = st.layers.iter().find(|l| l.layer == layer).ok_or_else(|| anyhow!("unknown layer"))?;
    if !lv.editable {
        return Err(anyhow!("your organisation does not allow this brand to be changed"));
    }
    let (dir, _) = layer_dir(layer);
    std::fs::create_dir_all(&dir)?;
    let old = read_brand(&dir);
    b.logo_file = old.map(|o| o.logo_file).unwrap_or_default();
    if remove_logo {
        if !b.logo_file.is_empty() {
            let _ = std::fs::remove_file(dir.join(&b.logo_file));
        }
        b.logo_file.clear();
    }
    if let Some(src) = logo_path {
        let check = check_logo(&src);
        if !check.ok {
            return Err(anyhow!(check.warnings.join(" ")));
        }
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
        if !b.logo_file.is_empty() {
            let _ = std::fs::remove_file(dir.join(&b.logo_file));
        }
        let fname = format!("logo.{ext}");
        std::fs::copy(&src, dir.join(&fname))?;
        b.logo_file = fname;
    }
    b.updated = crate::db::now();
    std::fs::write(dir.join("brand.json"), serde_json::to_string_pretty(&b)?)?;
    Ok(())
}

pub fn clear_layer(layer: &str) -> Result<()> {
    if layer == "oem" {
        return Err(anyhow!("the OEM brand is set by the installer"));
    }
    let st = state();
    let lv = st.layers.iter().find(|l| l.layer == layer).ok_or_else(|| anyhow!("unknown layer"))?;
    if !lv.editable {
        return Err(anyhow!("your organisation does not allow this brand to be changed"));
    }
    let (dir, machine) = layer_dir(layer);
    if !machine && dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

// ---------- branding packs (.tunasbrand) ----------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PackAssistant {
    pub name: String,
    pub icon: String,
    pub color: String,
    pub persona: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PackReport {
    pub brand_name: String,
    pub assistants: usize,
    pub policy: bool,
    pub issuer: String,
}

/// Export a layer as a .tunasbrand pack (zip): brand.json, logo, optional assistants.json and policy.json.
pub fn export_pack(layer: &str, dest: &Path, assistants: Vec<PackAssistant>, policy: Option<Policy>, issuer: &str) -> Result<()> {
    let (dir, _) = layer_dir(layer);
    let mut b = read_brand(&dir).ok_or_else(|| anyhow!("there is no brand to export on this layer yet"))?;
    if !issuer.trim().is_empty() {
        b.issuer = issuer.trim().into();
    }
    let f = std::fs::File::create(dest)?;
    let mut z = zip::ZipWriter::new(f);
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    z.start_file("brand.json", opt)?;
    z.write_all(serde_json::to_string_pretty(&b)?.as_bytes())?;
    if !b.logo_file.is_empty() {
        z.start_file(b.logo_file.clone(), opt)?;
        z.write_all(&std::fs::read(dir.join(&b.logo_file))?)?;
    }
    if !assistants.is_empty() {
        z.start_file("assistants.json", opt)?;
        z.write_all(serde_json::to_string_pretty(&assistants)?.as_bytes())?;
    }
    if let Some(p) = policy {
        z.start_file("policy.json", opt)?;
        z.write_all(serde_json::to_string_pretty(&p)?.as_bytes())?;
    }
    z.start_file("pack.json", opt)?;
    z.write_all(
        serde_json::json!({"format": "tunasbrand", "version": 1, "made_with": concat!("TUNAS ", env!("CARGO_PKG_VERSION")), "created": crate::db::now()})
            .to_string()
            .as_bytes(),
    )?;
    z.finish()?;
    Ok(())
}

pub struct ImportedPack {
    pub report: PackReport,
    pub assistants: Vec<PackAssistant>,
}

/// Import a pack into the organisation layer (per user). Returns the assistants it carries.
pub fn import_pack(src: &Path) -> Result<ImportedPack> {
    let st = state();
    let org = st.layers.iter().find(|l| l.layer == "org").unwrap();
    if org.machine {
        return Err(anyhow!("this machine's organisation brand is managed by your IT department"));
    }
    if !org.editable {
        return Err(anyhow!("your organisation does not allow its brand to be changed"));
    }
    let f = std::fs::File::open(src)?;
    let mut z = zip::ZipArchive::new(f).map_err(|_| anyhow!("this is not a TUNAS branding pack"))?;
    let read = |z: &mut zip::ZipArchive<std::fs::File>, name: &str| -> Option<Vec<u8>> {
        let mut e = z.by_name(name).ok()?;
        if e.size() > 20 << 20 {
            return None;
        }
        let mut v = vec![];
        e.read_to_end(&mut v).ok()?;
        Some(v)
    };
    let bj = read(&mut z, "brand.json").ok_or_else(|| anyhow!("this pack has no brand.json"))?;
    let mut b: Brand = serde_json::from_slice(&bj).map_err(|_| anyhow!("the pack's brand.json is not valid"))?;
    let dir = user_layer_dir("org");
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    if !b.logo_file.is_empty() {
        // Only allow a plain file name for the logo.
        let fname = Path::new(&b.logo_file).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = Path::new(&fname).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ["png", "jpg", "jpeg", "svg"].contains(&ext.as_str()) {
            if let Some(bytes) = read(&mut z, &b.logo_file) {
                let out = format!("logo.{ext}");
                std::fs::write(dir.join(&out), bytes)?;
                b.logo_file = out;
            } else {
                b.logo_file.clear();
            }
        } else {
            b.logo_file.clear();
        }
    }
    b.updated = crate::db::now();
    std::fs::write(dir.join("brand.json"), serde_json::to_string_pretty(&b)?)?;
    let mut has_policy = false;
    if let Some(p) = read(&mut z, "policy.json") {
        if let Ok(pol) = serde_json::from_slice::<Policy>(&p) {
            std::fs::write(dir.join("policy.json"), serde_json::to_string_pretty(&pol)?)?;
            has_policy = true;
        }
    }
    let assistants: Vec<PackAssistant> = read(&mut z, "assistants.json")
        .and_then(|a| serde_json::from_slice(&a).ok())
        .unwrap_or_default();
    Ok(ImportedPack {
        report: PackReport { brand_name: b.name.clone(), assistants: assistants.len(), policy: has_policy, issuer: b.issuer.clone() },
        assistants,
    })
}

// ---------- icon and shortcuts ----------

/// Build icon.ico from the effective logo (or the TUNAS mark), set it on the window and
/// rename/re-icon the desktop and Start menu shortcuts.
pub fn apply_to_system(app: &tauri::AppHandle) -> Result<()> {
    use tauri::Manager;
    let st = state();
    let ico_path = crate::paths::user_brand_root().join("icon.ico");
    let mut window_png: Option<image::RgbaImage> = None;

    let logo_path = st
        .layers
        .iter()
        .filter(|l| l.layer != "personal" || st.policy.allow_personal_brand)
        .find_map(|l| {
            let b = l.brand.as_ref()?;
            if b.logo_file.is_empty() || !b.logo_as_icon {
                return None;
            }
            Some((layer_dir(&l.layer).0.join(&b.logo_file), b.logo_bg.clone(), b.logo_shape.clone()))
        });

    if let Some((lp, bg, shape)) = logo_path {
        let img = load_logo(&lp)?;
        let mut frames = vec![];
        for s in [16u32, 24, 32, 48, 64, 128, 256] {
            let sq = square_icon(&img, s);
            let sq = if bg == "light" || bg == "dark" { on_plate(&square_icon(&img, 256), s, &bg, &shape) } else { sq };
            frames.push(image::codecs::ico::IcoFrame::as_png(sq.as_raw(), s, s, image::ExtendedColorType::Rgba8)?);
            if s == 256 {
                window_png = Some(sq);
            }
        }
        let f = std::fs::File::create(&ico_path)?;
        image::codecs::ico::IcoEncoder::new(f).encode_images(&frames)?;
    } else {
        let _ = std::fs::remove_file(&ico_path);
    }

    #[cfg(desktop)]
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title(&st.effective_name);
        match &window_png {
            Some(img) => {
                let _ = w.set_icon(tauri::image::Image::new_owned(img.as_raw().clone(), img.width(), img.height()));
            }
            None => {
                if let Some(icon) = app.default_window_icon() {
                    let _ = w.set_icon(icon.clone());
                }
            }
        }
    }

    // Shortcuts are slow to update (PowerShell); do it in the background so the window is not held up.
    #[cfg(windows)]
    {
        let a = app.clone();
        let name = st.effective_name.clone();
        std::thread::spawn(move || update_shortcuts(&a, &name, if ico_path.exists() { Some(ico_path) } else { None }));
    }
    Ok(())
}

#[cfg(windows)]
fn update_shortcuts(app: &tauri::AppHandle, name: &str, ico: Option<PathBuf>) {
    use tauri::Manager;
    let safe: String = name.chars().filter(|c| !"\\/:*?\"<>|".contains(*c)).collect::<String>().trim().to_string();
    let safe = if safe.is_empty() { "TUNAS".to_string() } else { safe };
    let exe = std::env::current_exe().unwrap_or_default();
    let state = app.state::<crate::AppState>();
    let prev = crate::db::get_setting(&state.db.lock(), "shortcut_name").unwrap_or_else(|| "TUNAS".into());
    let icon = ico.map(|p| format!("{},0", p.display())).unwrap_or_else(|| format!("{},0", exe.display()));
    // Only touch shortcuts we created (same name as before) and only if they exist; never add a desktop
    // shortcut the user deleted.
    let script = format!(
        r#"$ErrorActionPreference='SilentlyContinue'
$ws = New-Object -ComObject WScript.Shell
$dirs = @([Environment]::GetFolderPath('Desktop'), (Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'))
foreach ($d in $dirs) {{
 foreach ($n in @('{prev}', 'TUNAS') | Select-Object -Unique) {{
  $old = Join-Path $d ($n + '.lnk')
  if (Test-Path $old) {{
    $t = $ws.CreateShortcut($old).TargetPath
    if ($t -ieq '{exe}') {{
      if ($n -ne '{new}') {{ Remove-Item $old -Force }}
      $s = $ws.CreateShortcut((Join-Path $d '{new}.lnk'))
      $s.TargetPath = '{exe}'
      $s.WorkingDirectory = '{wd}'
      $s.IconLocation = '{icon}'
      $s.Description = '{new} - Powered by S.A.L.T CI'
      $s.Save()
    }}
  }}
 }}
}}
New-Item -Path 'HKCU:\Software\SALT CI\TUNAS' -Force | Out-Null
Set-ItemProperty -Path 'HKCU:\Software\SALT CI\TUNAS' -Name 'ShortcutName' -Value '{new}'"#,
        prev = prev.replace('\'', "''"),
        new = safe.replace('\'', "''"),
        exe = exe.display().to_string().replace('\'', "''"),
        wd = crate::paths::app_dir().display().to_string().replace('\'', "''"),
        icon = icon.replace('\'', "''"),
    );
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    if cmd.status().map(|s| s.success()).unwrap_or(false) {
        let _ = crate::db::set_setting(&state.db.lock(), "shortcut_name", &safe);
    }
}
