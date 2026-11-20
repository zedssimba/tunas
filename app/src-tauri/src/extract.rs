//! Read text out of files dropped into a chat: PDF, Word, Excel, PowerPoint, text, Markdown, CSV.

use anyhow::{anyhow, Result};
use serde::Serialize;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Clone, Debug, Default)]
pub struct Extracted {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub chars: usize,
    pub text: String,
    pub truncated: bool,
    pub warning: String,
}

pub const SUPPORTED: &[&str] = &["pdf", "docx", "xlsx", "xlsm", "xls", "pptx", "txt", "md", "csv", "json", "log", "html", "htm"];

pub fn extract(path: &Path, max_chars: usize) -> Result<Extracted> {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let meta = std::fs::metadata(path)?;
    if meta.len() > 200 << 20 {
        return Err(anyhow!("{name} is too large (over 200 MB)"));
    }
    let (kind, text) = match ext.as_str() {
        "pdf" => ("PDF", pdf(path)?),
        "docx" => ("Word", office_xml(path, &["word/document.xml"], "w:p")?),
        "pptx" => ("PowerPoint", pptx(path)?),
        "xlsx" | "xlsm" | "xls" => ("Excel", excel(path)?),
        "txt" | "md" | "csv" | "json" | "log" => ("Text", read_text(path)?),
        "html" | "htm" => ("Web page", strip_tags(&read_text(path)?)),
        _ => return Err(anyhow!("{name}: this file type is not supported yet")),
    };
    let text = normalise(&text);
    let mut warning = String::new();
    if text.trim().is_empty() {
        warning = if ext == "pdf" {
            "No text was found. This PDF may be a scanned image; reading scanned PDFs is not supported yet.".into()
        } else {
            "No text was found in this file.".into()
        };
    }
    let total = text.chars().count();
    let truncated = total > max_chars;
    let text = if truncated { text.chars().take(max_chars).collect() } else { text };
    Ok(Extracted { name, path: path.display().to_string(), kind: kind.into(), chars: total, text, truncated, warning })
}

fn read_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(match String::from_utf8(bytes.clone()) {
        Ok(s) => s,
        Err(_) => bytes.iter().map(|&b| b as char).collect(), // Latin-1 fallback
    })
}

fn normalise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank = 0;
    for line in s.lines() {
        let l = line.trim_end();
        if l.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(l);
        out.push('\n');
    }
    out.trim().to_string()
}

fn pdf(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    // pdf-extract can panic on unusual files; keep the app alive.
    let r = std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(&bytes));
    match r {
        Ok(Ok(t)) => Ok(t),
        Ok(Err(e)) => Err(anyhow!("could not read this PDF: {e}")),
        Err(_) => Err(anyhow!("could not read this PDF")),
    }
}

fn zip_entry(z: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<String> {
    let mut e = z.by_name(name).ok()?;
    let mut s = String::new();
    e.read_to_string(&mut s).ok()?;
    Some(s)
}

/// Pull text out of Office XML, putting a line break after each paragraph element.
fn xml_text(xml: &str, para_tag: &str) -> String {
    use quick_xml::events::Event;
    let mut r = quick_xml::Reader::from_str(xml);
    let mut out = String::new();
    let para = para_tag.as_bytes();
    loop {
        match r.read_event() {
            Ok(Event::Text(t)) => {
                if let Ok(s) = t.unescape() {
                    out.push_str(&s);
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == para => out.push('\n'),
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                let n = e.name();
                let n = n.as_ref();
                if n == b"w:tab" || n == b"a:tab" {
                    out.push('\t');
                } else if n == b"w:br" || n == b"a:br" {
                    out.push('\n');
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"w:tc" => out.push_str(" | "),
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

fn office_xml(path: &Path, parts: &[&str], para: &str) -> Result<String> {
    let f = std::fs::File::open(path)?;
    let mut z = zip::ZipArchive::new(f).map_err(|_| anyhow!("this file is damaged or not a Word document"))?;
    let mut out = String::new();
    for p in parts {
        if let Some(x) = zip_entry(&mut z, p) {
            out.push_str(&xml_text(&x, para));
        }
    }
    Ok(out)
}

fn pptx(path: &Path) -> Result<String> {
    let f = std::fs::File::open(path)?;
    let mut z = zip::ZipArchive::new(f).map_err(|_| anyhow!("this file is damaged or not a PowerPoint file"))?;
    let mut slides: Vec<(usize, String)> = (0..z.len())
        .filter_map(|i| {
            let n = z.by_index(i).ok()?.name().to_string();
            let num = n.strip_prefix("ppt/slides/slide")?.strip_suffix(".xml")?.parse::<usize>().ok()?;
            Some((num, n))
        })
        .collect();
    slides.sort();
    let mut out = String::new();
    for (num, name) in slides {
        if let Some(x) = zip_entry(&mut z, &name) {
            out.push_str(&format!("\n--- Slide {num} ---\n"));
            out.push_str(&xml_text(&x, "a:p"));
        }
    }
    Ok(out)
}

fn excel(path: &Path) -> Result<String> {
    use calamine::{open_workbook_auto, Data, Reader};
    let mut wb = open_workbook_auto(path).map_err(|e| anyhow!("could not read this workbook: {e}"))?;
    let mut out = String::new();
    for name in wb.sheet_names().to_owned() {
        if let Ok(range) = wb.worksheet_range(&name) {
            out.push_str(&format!("\n--- Sheet: {name} ---\n"));
            for row in range.rows() {
                let cells: Vec<String> = row
                    .iter()
                    .map(|c| match c {
                        Data::Empty => String::new(),
                        Data::Float(f) => {
                            if f.fract() == 0.0 && f.abs() < 1e15 { format!("{}", *f as i64) } else { format!("{f}") }
                        }
                        other => other.to_string(),
                    })
                    .collect();
                if cells.iter().any(|c| !c.is_empty()) {
                    out.push_str(&cells.join(" | "));
                    out.push('\n');
                }
            }
        }
    }
    Ok(out)
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let lower = html.to_lowercase();
    let mut skip_until: Option<&str> = None;
    let mut i = 0;
    let chars: Vec<char> = html.chars().collect();
    let lchars: Vec<char> = lower.chars().collect();
    while i < chars.len() {
        if let Some(end) = skip_until {
            let rest: String = lchars[i..chars.len().min(i + end.len())].iter().collect();
            if rest == end {
                skip_until = None;
                i += end.len();
            } else {
                i += 1;
            }
            continue;
        }
        let c = chars[i];
        if c == '<' {
            let rest: String = lchars[i..chars.len().min(i + 7)].iter().collect();
            if rest.starts_with("<script") {
                skip_until = Some("</script>");
            } else if rest.starts_with("<style") {
                skip_until = Some("</style>");
            }
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag {
            out.push(c);
        }
        i += 1;
    }
    out.replace("&nbsp;", " ").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
}

/// A piece of a file with where it came from: page (PDF), slide (PowerPoint) or sheet (Excel).
#[derive(Clone, Debug)]
pub struct Part {
    /// Page or slide number, 0 when not paged.
    pub page: u32,
    /// "p. 3", "slide 4", "sheet Budget" or "".
    pub label: String,
    pub text: String,
}

/// Read a file as parts, so search results can point at the page. Used by Knowledge.
pub fn extract_parts(path: &Path) -> Result<Vec<Part>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let meta = std::fs::metadata(path)?;
    if meta.len() > 100 << 20 {
        return Err(anyhow!("too large (over 100 MB)"));
    }
    let split = |text: String, marker: &str, slide: bool| -> Vec<Part> {
        let mut parts = vec![];
        for chunk in text.split(marker).skip(1) {
            let (head, body) = chunk.split_once(" ---").unwrap_or(("", chunk));
            let head = head.trim().to_string();
            let page = head.parse::<u32>().unwrap_or(0);
            let label = if slide { format!("slide {head}") } else { format!("sheet {head}") };
            parts.push(Part { page, label, text: normalise(body) });
        }
        parts
    };
    let parts = match ext.as_str() {
        "pdf" => {
            let bytes = std::fs::read(path)?;
            let pages = match std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem_by_pages(&bytes)) {
                Ok(Ok(p)) => p,
                Ok(Err(e)) => return Err(anyhow!("could not read this PDF: {e}")),
                Err(_) => return Err(anyhow!("could not read this PDF")),
            };
            pages
                .into_iter()
                .enumerate()
                .map(|(i, t)| Part { page: i as u32 + 1, label: format!("p. {}", i + 1), text: normalise(&t) })
                .collect()
        }
        "pptx" => split(pptx(path)?, "--- Slide ", true),
        "xlsx" | "xlsm" | "xls" => split(excel(path)?, "--- Sheet: ", false),
        "docx" => vec![Part { page: 0, label: String::new(), text: normalise(&office_xml(path, &["word/document.xml"], "w:p")?) }],
        "md" => md_sections(&read_text(path)?),
        "txt" | "csv" | "log" => vec![Part { page: 0, label: String::new(), text: normalise(&read_text(path)?) }],
        "html" | "htm" => vec![Part { page: 0, label: String::new(), text: normalise(&strip_tags(&read_text(path)?)) }],
        _ => return Err(anyhow!("this file type is not supported")),
    };
    let parts: Vec<Part> = parts.into_iter().filter(|p| !p.text.trim().is_empty()).collect();
    if parts.is_empty() {
        return Err(anyhow!(if ext == "pdf" { "no text found (maybe a scanned PDF; scanned PDFs are not supported yet)" } else { "no text found" }));
    }
    Ok(parts)
}

/// File types Knowledge reads.
pub const KNOWLEDGE_TYPES: &[&str] = &["pdf", "docx", "xlsx", "xlsm", "xls", "pptx", "txt", "md", "csv", "html", "htm"];

/// Split Markdown at its headings (#, ##, ###), so each piece knows which section it belongs to,
/// for example "0.2.4 — 20 November 2026" in release notes.
fn md_sections(text: &str) -> Vec<Part> {
    let mut parts: Vec<Part> = vec![];
    let mut label = String::new();
    let mut trail: Vec<(usize, String)> = vec![];
    let mut cur = String::new();
    let flush = |label: &str, cur: &mut String, parts: &mut Vec<Part>| {
        let t = normalise(cur);
        if !t.trim().is_empty() {
            parts.push(Part { page: 0, label: label.to_string(), text: t });
        }
        cur.clear();
    };
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        let hashes = line.chars().take_while(|c| *c == '#').count();
        if !in_code && (1..=3).contains(&hashes) && line[hashes..].starts_with(' ') {
            flush(&label, &mut cur, &mut parts);
            let h = line[hashes..].trim().to_string();
            trail.retain(|(lv, _)| *lv < hashes);
            trail.push((hashes, h));
            // Section path without the document title (level 1), at most two levels.
            let path: Vec<&str> = trail.iter().filter(|(lv, _)| *lv > 1 || trail.len() == 1).map(|(_, t)| t.as_str()).collect();
            label = path.iter().rev().take(2).rev().cloned().collect::<Vec<_>>().join(" › ");
            label = label.chars().take(120).collect();
        }
        cur.push_str(line);
        cur.push('\n');
    }
    flush(&label, &mut cur, &mut parts);
    parts
}
