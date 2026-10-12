//! CSV output (RFC 4180), one header row then the list rows.

use crate::Report;

pub fn render(report: &Report) -> String {
    let mut out = String::new();
    let header = report.columns.iter().map(|c| c.label.as_str());
    push_row(&mut out, header);
    for row in &report.rows {
        push_row(&mut out, row.iter().map(String::as_str));
    }
    out
}

fn push_row<'a>(out: &mut String, cells: impl Iterator<Item = &'a str>) {
    let line: Vec<String> = cells.map(escape).collect();
    out.push_str(&line.join(","));
    out.push_str("\r\n");
}

fn escape(cell: &str) -> String {
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}
