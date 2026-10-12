//! Direct PDF export: a paged table layout over the same `Report` the HTML
//! renderer uses. Noto Sans (and Noto Sans Arabic for Arabic text) is
//! embedded, so names in any of the supported scripts print correctly.
//! Right-to-left lists mirror the column order, except tables marked
//! left-to-right (pairings).

use crate::Result;
use crate::report::{Align, Report};
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::Fill;
use krilla::text::{Font, TextDirection};
use std::sync::OnceLock;

const REGULAR: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Bold.ttf");
const ARABIC: &[u8] = include_bytes!("../../../../assets/fonts/NotoSansArabic-Regular.ttf");

const MARGIN: f32 = 36.0;
const ROW_H: f32 = 13.0;
const BODY_SIZE: f32 = 8.5;

struct Fonts {
    regular: Font,
    bold: Font,
    arabic: Font,
}

fn fonts() -> &'static Fonts {
    static FONTS: OnceLock<Fonts> = OnceLock::new();
    FONTS.get_or_init(|| Fonts {
        regular: Font::new(REGULAR.to_vec().into(), 0).expect("bundled font"),
        bold: Font::new(BOLD.to_vec().into(), 0).expect("bundled font"),
        arabic: Font::new(ARABIC.to_vec().into(), 0).expect("bundled font"),
    })
}

/// Advance widths per font, for layout without shaping.
struct Metrics {
    faces: [ttf_parser::Face<'static>; 3],
}

fn metrics() -> &'static Metrics {
    static M: OnceLock<Metrics> = OnceLock::new();
    M.get_or_init(|| Metrics {
        faces: [
            ttf_parser::Face::parse(REGULAR, 0).expect("bundled font"),
            ttf_parser::Face::parse(BOLD, 0).expect("bundled font"),
            ttf_parser::Face::parse(ARABIC, 0).expect("bundled font"),
        ],
    })
}

fn is_arabic(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32, 0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
    })
}

#[derive(Clone, Copy)]
enum Style {
    Regular,
    Bold,
}

fn pick(text: &str, style: Style) -> (Font, usize) {
    let f = fonts();
    if is_arabic(text) {
        (f.arabic.clone(), 2)
    } else {
        match style {
            Style::Regular => (f.regular.clone(), 0),
            Style::Bold => (f.bold.clone(), 1),
        }
    }
}

fn width(text: &str, face: usize, size: f32) -> f32 {
    let face = &metrics().faces[face];
    let upem = f32::from(face.units_per_em());
    let units: f32 = text
        .chars()
        .map(|c| {
            face.glyph_index(c)
                .and_then(|g| face.glyph_hor_advance(g))
                .map_or(f32::from(face.units_per_em()) * 0.5, f32::from)
        })
        .sum();
    units * size / upem
}

/// Shortens `text` with an ellipsis so it fits in `max` points.
fn fit(text: &str, face: usize, size: f32, max: f32) -> String {
    if width(text, face, size) <= max {
        return text.to_string();
    }
    let mut s: String = text.to_string();
    while !s.is_empty() && width(&format!("{s}…"), face, size) > max {
        s.pop();
    }
    format!("{s}…")
}

fn fill(gray: u8) -> Fill {
    Fill {
        paint: rgb::Color::new(gray, gray, gray).into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    }
}

struct Writer<'a> {
    surface: krilla::surface::Surface<'a>,
}

impl Writer<'_> {
    fn text(&mut self, x: f32, baseline: f32, text: &str, size: f32, style: Style) {
        if text.is_empty() {
            return;
        }
        let (font, _) = pick(text, style);
        self.surface.set_fill(Some(fill(17)));
        self.surface.draw_text(
            Point::from_xy(x, baseline),
            font,
            size,
            text,
            false,
            TextDirection::Auto,
        );
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, gray: u8) {
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            self.surface.set_fill(Some(fill(gray)));
            self.surface.draw_path(&path);
        }
    }

    /// Text in a cell, aligned and truncated.
    #[allow(clippy::too_many_arguments)]
    fn cell(
        &mut self,
        x: f32,
        w: f32,
        baseline: f32,
        text: &str,
        align: Align,
        size: f32,
        style: Style,
    ) {
        let (_, face) = pick(text, style);
        let pad = 3.0;
        let t = fit(text, face, size, (w - 2.0 * pad).max(4.0));
        let tw = width(&t, face, size);
        let tx = match align {
            Align::Left => x + pad,
            Align::Right => x + w - pad - tw,
            Align::Center => x + (w - tw) / 2.0,
        };
        self.text(tx, baseline, &t, size, style);
    }
}

pub fn render(report: &Report) -> Result<Vec<u8>> {
    let total_hint: u32 = report.columns.iter().map(|c| u32::from(c.width)).sum();
    let landscape = total_hint > 80 || report.columns.len() > 11;
    let (pw, ph) = if landscape {
        (842.0, 595.0)
    } else {
        (595.0, 842.0)
    };
    let usable = pw - 2.0 * MARGIN;
    let scale = usable / total_hint.max(1) as f32;
    let mut widths: Vec<f32> = report
        .columns
        .iter()
        .map(|c| f32::from(c.width) * scale)
        .collect();
    let mut order: Vec<usize> = (0..report.columns.len()).collect();
    if report.rtl && !report.ltr_table {
        order.reverse();
        widths.reverse();
    }
    let mirror_align = |a: Align| {
        if report.rtl && !report.ltr_table {
            match a {
                Align::Left => Align::Right,
                Align::Right => Align::Left,
                Align::Center => Align::Center,
            }
        } else {
            a
        }
    };

    // Rows per page: the first page also carries the heading block.
    let header_block = 18.0 + 15.0 + if report.info.is_empty() { 0.0 } else { 14.0 } + 8.0;
    let footer_h = 20.0;
    let rows_first =
        (((ph - 2.0 * MARGIN - header_block - footer_h - ROW_H) / ROW_H).floor() as usize).max(1);
    let rows_other = (((ph - 2.0 * MARGIN - footer_h - ROW_H) / ROW_H).floor() as usize).max(1);
    let mut pages: Vec<std::ops::Range<usize>> = Vec::new();
    let mut start = 0;
    let n = report.rows.len();
    loop {
        let cap = if pages.is_empty() {
            rows_first
        } else {
            rows_other
        };
        let end = (start + cap).min(n);
        pages.push(start..end);
        start = end;
        if start >= n {
            break;
        }
    }
    let page_count = pages.len();
    let notes_on_last = !report.notes.is_empty();

    let mut document = Document::new();
    for (pi, range) in pages.into_iter().enumerate() {
        let mut page = document.start_page_with(
            PageSettings::from_wh(pw, ph)
                .ok_or_else(|| co_core::Error::Invalid("page size".into()))?,
        );
        {
            let mut w = Writer {
                surface: page.surface(),
            };
            let mut y = MARGIN;
            if pi == 0 {
                let heading_x = |text: &str, size: f32, style| {
                    if report.rtl {
                        let (_, face) = pick(text, style);
                        pw - MARGIN - width(text, face, size)
                    } else {
                        MARGIN
                    }
                };
                y += 14.0;
                w.text(
                    heading_x(&report.heading, 14.0, Style::Bold),
                    y,
                    &report.heading,
                    14.0,
                    Style::Bold,
                );
                y += 16.0;
                w.text(
                    heading_x(&report.title, 11.0, Style::Bold),
                    y,
                    &report.title,
                    11.0,
                    Style::Bold,
                );
                if !report.info.is_empty() {
                    y += 13.0;
                    let line = report
                        .info
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("   ");
                    w.text(
                        heading_x(&line, 8.0, Style::Regular),
                        y,
                        &line,
                        8.0,
                        Style::Regular,
                    );
                }
                y += 10.0;
            }

            // Header row.
            w.rect(MARGIN, y, usable, ROW_H, 225);
            let mut x = MARGIN;
            for (k, &ci) in order.iter().enumerate() {
                let c = &report.columns[ci];
                w.cell(
                    x,
                    widths[k],
                    y + ROW_H - 3.5,
                    &c.label,
                    mirror_align(c.align),
                    BODY_SIZE,
                    Style::Bold,
                );
                x += widths[k];
            }
            y += ROW_H;

            for (ri, row) in report.rows[range].iter().enumerate() {
                if ri % 2 == 1 {
                    w.rect(MARGIN, y, usable, ROW_H, 244);
                }
                let mut x = MARGIN;
                for (k, &ci) in order.iter().enumerate() {
                    let text = row.get(ci).map(String::as_str).unwrap_or("");
                    let align = mirror_align(report.columns[ci].align);
                    w.cell(
                        x,
                        widths[k],
                        y + ROW_H - 3.5,
                        text,
                        align,
                        BODY_SIZE,
                        Style::Regular,
                    );
                    x += widths[k];
                }
                y += ROW_H;
            }

            if pi + 1 == page_count && notes_on_last {
                y += 6.0;
                for note in &report.notes {
                    y += 10.0;
                    if y > ph - MARGIN - footer_h {
                        break;
                    }
                    w.text(MARGIN, y, note, 7.5, Style::Regular);
                }
            }

            let footer = format!("{}   ·   {}/{}", report.footer, pi + 1, page_count);
            w.text(MARGIN, ph - MARGIN + 6.0, &footer, 7.5, Style::Regular);
            let Writer { surface } = w;
            surface.finish();
        }
        page.finish();
    }
    document
        .finish()
        .map_err(|e| co_core::Error::Invalid(format!("PDF export failed: {e:?}")).into())
}
