//! Documents: turn an answer (Markdown) into a Word, Excel or PDF file, with the brand's letterhead.
//! Everything is made inside TUNAS, offline; no office suite is needed.

use anyhow::{anyhow, Result};
use std::path::Path;

// ---------------------------------------------------------------- a small document model

#[derive(Clone, Debug, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Debug)]
pub enum Block {
    Heading(u8, Vec<Run>),
    Para(Vec<Run>),
    List { ordered: bool, items: Vec<Vec<Run>> },
    Table { rows: Vec<Vec<String>> },
    Code(String),
    Quote(Vec<Run>),
    Rule,
}

/// Remove the [1], [2][3] source markers; they point to TUNAS's source list, not to anything in the file.
fn strip_citations(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '[' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == ']' && j - i <= 4 {
                // also drop one space before the marker
                if out.ends_with(' ') {
                    out.pop();
                }
                i = j + 1;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

pub fn parse(md: &str) -> Vec<Block> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
    let md = strip_citations(md);
    let mut blocks = vec![];
    let mut runs: Vec<Run> = vec![];
    let (mut bold, mut italic) = (0, 0);
    let mut list: Option<(bool, Vec<Vec<Run>>)> = None;
    let mut list_depth = 0;
    let mut table: Option<Vec<Vec<String>>> = None;
    let mut cell = String::new();
    let mut in_code = false;
    let mut code = String::new();
    let mut quote = false;

    let push_text = |runs: &mut Vec<Run>, t: &str, bold: i32, italic: i32| {
        runs.push(Run { text: t.to_string(), bold: bold > 0, italic: italic > 0 });
    };

    for ev in Parser::new_ext(&md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH) {
        match ev {
            Event::Start(Tag::Strong) => bold += 1,
            Event::End(TagEnd::Strong) => bold -= 1,
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::End(TagEnd::Emphasis) => italic -= 1,
            Event::Start(Tag::List(first)) => {
                list_depth += 1;
                if list_depth == 1 {
                    list = Some((first.is_some(), vec![]));
                }
            }
            Event::End(TagEnd::List(_)) => {
                list_depth -= 1;
                if list_depth == 0 {
                    if let Some((ordered, items)) = list.take() {
                        blocks.push(Block::List { ordered, items });
                    }
                }
            }
            Event::Start(Tag::Item) => {
                if list_depth > 1 && !runs.is_empty() {
                    // a nested item: finish the parent line first
                    if let Some((_, items)) = list.as_mut() {
                        items.push(std::mem::take(&mut runs));
                    }
                }
                if list_depth > 1 {
                    runs.push(Run { text: "– ".into(), ..Default::default() });
                }
            }
            Event::End(TagEnd::Item) => {
                if let Some((_, items)) = list.as_mut() {
                    if !runs.is_empty() {
                        items.push(std::mem::take(&mut runs));
                    }
                }
            }
            Event::Start(Tag::Table(_)) => table = Some(vec![]),
            Event::End(TagEnd::Table) => {
                if let Some(rows) = table.take() {
                    blocks.push(Block::Table { rows });
                }
            }
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                if let Some(t) = table.as_mut() {
                    t.push(vec![]);
                }
            }
            Event::Start(Tag::TableCell) => cell.clear(),
            Event::End(TagEnd::TableCell) => {
                if let Some(t) = table.as_mut() {
                    if let Some(r) = t.last_mut() {
                        r.push(cell.trim().to_string());
                    }
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code = true;
                code.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                blocks.push(Block::Code(code.trim_end().to_string()));
            }
            Event::Start(Tag::BlockQuote(_)) => quote = true,
            Event::End(TagEnd::BlockQuote(_)) => quote = false,
            Event::End(TagEnd::Heading(level)) => {
                let lv = level as u8;
                blocks.push(Block::Heading(lv, std::mem::take(&mut runs)));
            }
            Event::End(TagEnd::Paragraph) => {
                if list.is_some() {
                    continue; // list items collect their own runs
                }
                let r = std::mem::take(&mut runs);
                if !r.is_empty() {
                    blocks.push(if quote { Block::Quote(r) } else { Block::Para(r) });
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if in_code {
                    code.push_str(&t);
                } else if table.is_some() {
                    cell.push_str(&t);
                } else {
                    push_text(&mut runs, &t, bold, italic);
                }
            }
            Event::SoftBreak => {
                if table.is_none() {
                    push_text(&mut runs, " ", bold, italic);
                }
            }
            Event::HardBreak => push_text(&mut runs, "\n", bold, italic),
            Event::Rule => blocks.push(Block::Rule),
            _ => {}
        }
    }
    if !runs.is_empty() {
        blocks.push(Block::Para(runs));
    }
    blocks
}

fn plain(runs: &[Run]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

pub fn has_table(md: &str) -> bool {
    parse(md).iter().any(|b| matches!(b, Block::Table { .. }))
}

// ---------------------------------------------------------------- Word

fn write_docx(blocks: &[Block], lh: Option<&crate::brand::Letterhead>, path: &Path) -> Result<()> {
    use docx_rs::*;
    let font = RunFonts::new().ascii("Arial").hi_ansi("Arial").east_asia("Arial").cs("Arial");
    let mk_run = |r: &crate::documents::Run, size: usize| {
        let mut run = docx_rs::Run::new().add_text(&r.text).size(size).fonts(font.clone());
        if r.bold {
            run = run.bold();
        }
        if r.italic {
            run = run.italic();
        }
        run
    };
    let mut doc = Docx::new().default_fonts(font.clone()).default_size(22);

    if let Some(lh) = lh {
        if let Some(png) = &lh.logo_png {
            if let Ok(img) = image::load_from_memory(png) {
                // About 2 cm tall, keeping the shape.
                let h_emu: u32 = 720_000;
                let w_emu = (h_emu as f64 * img.width() as f64 / img.height().max(1) as f64) as u32;
                let pic = Pic::new(png).size(w_emu, h_emu);
                doc = doc.add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_image(pic)));
            }
        }
        if !lh.org.is_empty() {
            doc = doc.add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_text(&lh.org).bold().size(28).fonts(font.clone())));
        }
        for line in lh.address.lines().filter(|l| !l.trim().is_empty()) {
            doc = doc.add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_text(line.trim()).size(18).color("555555").fonts(font.clone())));
        }
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(docx_rs::Run::new().add_text("").size(8))
                .line_spacing(LineSpacing::new().after(200)),
        );
    }

    for b in blocks {
        match b {
            Block::Heading(lv, runs) => {
                let size = match lv { 1 => 32, 2 => 28, 3 => 24, _ => 22 };
                let mut p = Paragraph::new().line_spacing(LineSpacing::new().before(200).after(80));
                for r in runs {
                    p = p.add_run(mk_run(&crate::documents::Run { bold: true, ..r.clone() }, size));
                }
                doc = doc.add_paragraph(p);
            }
            Block::Para(runs) | Block::Quote(runs) => {
                let mut p = Paragraph::new().line_spacing(LineSpacing::new().after(120));
                if matches!(b, Block::Quote(_)) {
                    p = p.indent(Some(567), None, None, None);
                }
                for r in runs {
                    p = p.add_run(mk_run(r, 22));
                }
                doc = doc.add_paragraph(p);
            }
            Block::List { ordered, items } => {
                for (i, item) in items.iter().enumerate() {
                    let marker = if *ordered { format!("{}.\t", i + 1) } else { "•\t".to_string() };
                    let mut p = Paragraph::new()
                        .indent(Some(567), Some(SpecialIndentType::Hanging(340)), None, None)
                        .line_spacing(LineSpacing::new().after(60))
                        .add_run(docx_rs::Run::new().add_text(&marker).size(22).fonts(font.clone()));
                    for r in item {
                        p = p.add_run(mk_run(r, 22));
                    }
                    doc = doc.add_paragraph(p);
                }
            }
            Block::Table { rows } => {
                let cols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
                let trs: Vec<TableRow> = rows
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let cells: Vec<TableCell> = (0..cols)
                            .map(|c| {
                                let t = r.get(c).cloned().unwrap_or_default();
                                let mut run = docx_rs::Run::new().add_text(&t).size(20).fonts(font.clone());
                                if i == 0 {
                                    run = run.bold();
                                }
                                let mut cell = TableCell::new().add_paragraph(Paragraph::new().add_run(run));
                                if i == 0 {
                                    cell = cell.shading(Shading::new().fill("E8ECF4"));
                                }
                                cell
                            })
                            .collect();
                        TableRow::new(cells)
                    })
                    .collect();
                doc = doc.add_table(Table::new(trs));
                doc = doc.add_paragraph(Paragraph::new());
            }
            Block::Code(t) => {
                for line in t.lines() {
                    doc = doc.add_paragraph(
                        Paragraph::new().add_run(docx_rs::Run::new().add_text(line).size(19).fonts(RunFonts::new().ascii("Consolas").hi_ansi("Consolas"))),
                    );
                }
            }
            Block::Rule => {
                doc = doc.add_paragraph(Paragraph::new().add_run(docx_rs::Run::new().add_text("")));
            }
        }
    }
    let f = std::fs::File::create(path)?;
    doc.build().pack(f).map_err(|e| anyhow!("could not write the Word file: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------- Excel

fn write_xlsx(blocks: &[Block], lh: Option<&crate::brand::Letterhead>, title: &str, path: &Path) -> Result<()> {
    use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
    let mut wb = Workbook::new();
    let head = Format::new().set_bold().set_background_color(Color::RGB(0xE8ECF4)).set_border(FormatBorder::Thin);
    let cellf = Format::new().set_border(FormatBorder::Thin);
    let numf = Format::new().set_border(FormatBorder::Thin).set_num_format("#,##0.##");
    let titlef = Format::new().set_bold().set_font_size(14);
    let small = Format::new().set_font_color(Color::RGB(0x555555)).set_font_size(9);

    let tables: Vec<&Vec<Vec<String>>> = blocks.iter().filter_map(|b| if let Block::Table { rows } = b { Some(rows) } else { None }).collect();
    let mut sheet_no = 0;
    let write_top = |ws: &mut rust_xlsxwriter::Worksheet| -> Result<u32> {
        let mut row = 0u32;
        if let Some(lh) = lh {
            if !lh.org.is_empty() {
                ws.write_with_format(row, 0, &lh.org, &titlef)?;
                row += 1;
            }
            for line in lh.address.lines().filter(|l| !l.trim().is_empty()) {
                ws.write_with_format(row, 0, line.trim(), &small)?;
                row += 1;
            }
            if row > 0 {
                row += 1;
            }
        }
        Ok(row)
    };
    if tables.is_empty() {
        let ws = wb.add_worksheet();
        ws.set_name(sheet_name(title, 0))?;
        let mut row = write_top(ws)?;
        for b in blocks {
            let t = match b {
                Block::Heading(_, r) | Block::Para(r) | Block::Quote(r) => plain(r),
                Block::List { items, .. } => items.iter().map(|i| format!("• {}", plain(i))).collect::<Vec<_>>().join("\n"),
                Block::Code(t) => t.clone(),
                _ => continue,
            };
            for line in t.lines() {
                ws.write(row, 0, line)?;
                row += 1;
            }
        }
        ws.set_column_width(0, 100)?;
    }
    for rows in tables {
        let ws = wb.add_worksheet();
        ws.set_name(sheet_name(title, sheet_no))?;
        sheet_no += 1;
        let top = write_top(ws)?;
        let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut widths = vec![8usize; cols];
        for (i, r) in rows.iter().enumerate() {
            for (c, v) in r.iter().enumerate() {
                widths[c] = widths[c].max(v.chars().count().min(60) + 2);
                let row = top + i as u32;
                if i == 0 {
                    ws.write_with_format(row, c as u16, v, &head)?;
                } else if let Some(n) = as_number(v) {
                    ws.write_number_with_format(row, c as u16, n, &numf)?;
                } else {
                    ws.write_with_format(row, c as u16, v, &cellf)?;
                }
            }
        }
        for (c, w) in widths.iter().enumerate() {
            ws.set_column_width(c as u16, *w as f64)?;
        }
        if !rows.is_empty() {
            ws.set_freeze_panes(top + 1, 0)?;
        }
    }
    wb.save(path).map_err(|e| anyhow!("could not write the Excel file: {e}"))?;
    Ok(())
}

fn sheet_name(title: &str, n: usize) -> String {
    let base: String = title.chars().filter(|c| !"[]:*?/\\".contains(*c)).take(25).collect::<String>().trim().to_string();
    let base = if base.is_empty() { "Sheet".to_string() } else { base };
    if n == 0 { base } else { format!("{base} {}", n + 1) }
}

/// "1,234.50", "RM 1,200", "15%" → numbers Excel can add up. Text with letters stays text.
fn as_number(v: &str) -> Option<f64> {
    let t = v.trim();
    let t = t.strip_prefix("RM").or_else(|| t.strip_prefix("rm")).unwrap_or(t).trim();
    if t.is_empty() || t.ends_with('%') || t.chars().any(|c| c.is_alphabetic()) {
        return None;
    }
    // Keep codes such as 0123 or 2026-11-20 as text.
    if (t.starts_with('0') && t.len() > 1 && !t.starts_with("0.")) || t.matches('-').count() > 1 || t.contains('/') {
        return None;
    }
    t.replace(',', "").parse::<f64>().ok()
}

// ---------------------------------------------------------------- PDF

static FONT_REGULAR: &[u8] = include_bytes!("../resources/fonts/LiberationSans-Regular.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../resources/fonts/LiberationSans-Bold.ttf");
static FONT_ITALIC: &[u8] = include_bytes!("../resources/fonts/LiberationSans-Italic.ttf");
static FONT_BOLD_ITALIC: &[u8] = include_bytes!("../resources/fonts/LiberationSans-BoldItalic.ttf");

fn write_pdf(blocks: &[Block], lh: Option<&crate::brand::Letterhead>, title: &str, path: &Path) -> Result<()> {
    use genpdf::{elements, fonts, style, Alignment, Element};
    let fd = |b: &'static [u8]| fonts::FontData::new(b.to_vec(), None).map_err(|e| anyhow!("font: {e}"));
    let family = fonts::FontFamily { regular: fd(FONT_REGULAR)?, bold: fd(FONT_BOLD)?, italic: fd(FONT_ITALIC)?, bold_italic: fd(FONT_BOLD_ITALIC)? };
    let mut doc = genpdf::Document::new(family);
    doc.set_title(title);
    doc.set_font_size(11);
    doc.set_line_spacing(1.25);
    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(18);
    doc.set_page_decorator(deco);

    let styled = |runs: &[Run], size: u8| -> elements::Paragraph {
        let mut p = elements::Paragraph::default();
        for r in runs {
            let mut s = style::Style::new().with_font_size(size);
            if r.bold {
                s = s.bold();
            }
            if r.italic {
                s = s.italic();
            }
            p.push_styled(r.text.replace('\n', " "), s);
        }
        p
    };

    let mut tmp_logo: Option<std::path::PathBuf> = None;
    if let Some(lh) = lh {
        if let Some(png) = &lh.logo_png {
            // genpdf reads images itself; give it a flat (white-backed) PNG, about 2 cm tall.
            if let Ok(img) = image::load_from_memory(png) {
                let rgba = img.to_rgba8();
                let mut flat = image::RgbImage::new(rgba.width(), rgba.height());
                for (x, y, p) in rgba.enumerate_pixels() {
                    let a = p[3] as f32 / 255.0;
                    let mix = |c: u8| (c as f32 * a + 255.0 * (1.0 - a)) as u8;
                    flat.put_pixel(x, y, image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])]));
                }
                let tp = std::env::temp_dir().join(format!("tunas-logo-{}.png", uuid::Uuid::new_v4()));
                if flat.save(&tp).is_ok() {
                    if let Ok(mut im) = elements::Image::from_path(&tp) {
                        let h_px = flat.height().max(1) as f64;
                        // scale so the logo is ~20 mm tall at the given dpi: mm = px / dpi * 25.4
                        let dpi = h_px * 25.4 / 20.0;
                        im.set_dpi(dpi);
                        doc.push(im);
                    }
                    tmp_logo = Some(tp);
                }
            }
        }
        if !lh.org.is_empty() {
            doc.push(elements::Paragraph::default().styled_string(lh.org.clone(), style::Style::new().bold().with_font_size(14)));
        }
        for line in lh.address.lines().filter(|l| !l.trim().is_empty()) {
            doc.push(elements::Paragraph::default().styled_string(line.trim().to_string(), style::Style::new().with_font_size(9).with_color(style::Color::Rgb(85, 85, 85))));
        }
        doc.push(elements::Break::new(1.0));
    }

    for b in blocks {
        match b {
            Block::Heading(lv, runs) => {
                let size = match lv { 1 => 16, 2 => 14, 3 => 12, _ => 11 };
                let bold: Vec<Run> = runs.iter().map(|r| Run { bold: true, ..r.clone() }).collect();
                doc.push(elements::Break::new(0.4));
                doc.push(styled(&bold, size));
            }
            Block::Para(runs) => {
                doc.push(styled(runs, 11).padded(genpdf::Margins::trbl(0, 0, 2, 0)));
            }
            Block::Quote(runs) => {
                doc.push(styled(runs, 11).padded(genpdf::Margins::trbl(0, 0, 2, 8)));
            }
            Block::List { ordered, items } => {
                if *ordered {
                    let mut l = elements::OrderedList::new();
                    for it in items {
                        l.push(styled(it, 11));
                    }
                    doc.push(l.padded(genpdf::Margins::trbl(0, 0, 2, 0)));
                } else {
                    let mut l = elements::UnorderedList::new();
                    for it in items {
                        l.push(styled(it, 11));
                    }
                    doc.push(l.padded(genpdf::Margins::trbl(0, 0, 2, 0)));
                }
            }
            Block::Table { rows } => {
                let cols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
                let mut t = elements::TableLayout::new(vec![1; cols]);
                t.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
                for (i, r) in rows.iter().enumerate() {
                    let mut row = t.row();
                    for c in 0..cols {
                        let v = r.get(c).cloned().unwrap_or_default();
                        let mut s = style::Style::new().with_font_size(9);
                        if i == 0 {
                            s = s.bold();
                        }
                        row.push_element(elements::Paragraph::default().styled_string(v, s).padded(1));
                    }
                    row.push().map_err(|e| anyhow!("table: {e}"))?;
                }
                doc.push(t);
                doc.push(elements::Break::new(0.6));
            }
            Block::Code(t) => {
                for line in t.lines() {
                    doc.push(elements::Paragraph::default().styled_string(line.to_string(), style::Style::new().with_font_size(9)));
                }
            }
            Block::Rule => doc.push(elements::Break::new(1.0)),
        }
    }
    let _ = Alignment::Left;
    let r = doc.render_to_file(path).map_err(|e| anyhow!("could not write the PDF: {e}"));
    if let Some(tp) = tmp_logo {
        let _ = std::fs::remove_file(tp);
    }
    r
}

// ---------------------------------------------------------------- entry point

/// Write `markdown` to `path` as Word (docx), Excel (xlsx) or PDF.
pub fn export(markdown: &str, format: &str, path: &Path, title: &str, letterhead: bool) -> Result<()> {
    let blocks = parse(markdown);
    if blocks.is_empty() {
        return Err(anyhow!("there is nothing to put in the document"));
    }
    let lh = if letterhead { Some(crate::brand::letterhead()) } else { None };
    let lh = lh.as_ref().filter(|l| !l.org.is_empty() || l.logo_png.is_some() || !l.address.is_empty());
    match format {
        "docx" => write_docx(&blocks, lh, path),
        "xlsx" => write_xlsx(&blocks, lh, title, path),
        "pdf" => write_pdf(&blocks, lh, title, path),
        other => Err(anyhow!("unknown format {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = "# Surat Rasmi\n\nTuan,\n\n**Perkara:** Mesyuarat *bulanan* [1]\n\n1. Satu\n2. Dua\n\n- a\n- b\n\n| Item | Kos (RM) |\n|---|---|\n| Kertas | 1,200.50 |\n| Toner | 300 |\n\nSekian.";
    #[test]
    fn parses_blocks() {
        let b = parse(SAMPLE);
        assert!(matches!(b[0], Block::Heading(1, _)));
        assert!(b.iter().any(|x| matches!(x, Block::Table { rows } if rows.len() == 3)));
        assert!(b.iter().any(|x| matches!(x, Block::List { ordered: true, items } if items.len() == 2)));
        let all: String = b.iter().filter_map(|x| if let Block::Para(r) = x { Some(plain(r)) } else { None }).collect();
        assert!(!all.contains("[1]"));
    }
    #[test]
    fn numbers() {
        assert_eq!(as_number("1,200.50"), Some(1200.5));
        assert_eq!(as_number("RM 300"), Some(300.0));
        assert_eq!(as_number("0123"), None);
        assert_eq!(as_number("2026-11-20"), None);
    }
    #[test]
    fn writes_all_formats() {
        let dir = std::env::temp_dir();
        for f in ["docx", "xlsx", "pdf"] {
            let p = dir.join(format!("tunas-test.{f}"));
            let blocks = parse(SAMPLE);
            let lh = crate::brand::Letterhead { org: "Jabatan Contoh".into(), address: "Jalan 1\n43000 Kajang".into(), logo_png: std::fs::read("/tmp/exeico/app_6_256x256x32.png").ok() };
            match f {
                "docx" => write_docx(&blocks, Some(&lh), &p).unwrap(),
                "xlsx" => write_xlsx(&blocks, Some(&lh), "Kos", &p).unwrap(),
                _ => write_pdf(&blocks, Some(&lh), "Kos", &p).unwrap(),
            }
            assert!(std::fs::metadata(&p).unwrap().len() > 1000, "{f}");
        }
    }
}
