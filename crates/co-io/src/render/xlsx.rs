//! Excel export: one worksheet with the heading, the info lines and the
//! table. Numeric cells are written as numbers so they can be sorted and
//! summed; points like `2½` become 2.5.

use crate::{Report, Result};
use rust_xlsxwriter::{Format, Workbook};

fn as_number(cell: &str) -> Option<f64> {
    let s = cell.trim();
    if s.is_empty() {
        return None;
    }
    let s = s.replace('½', ".5");
    let s = if s.starts_with(".5") {
        format!("0{s}")
    } else {
        s
    };
    s.parse::<f64>().ok()
}

pub fn render(report: &Report) -> Result<Vec<u8>> {
    let err = |e: rust_xlsxwriter::XlsxError| {
        crate::Error::Core(co_core::Error::Invalid(format!("Excel export failed: {e}")))
    };
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    let bold = Format::new().set_bold();
    let title = Format::new().set_bold().set_font_size(14);
    ws.set_name(
        report
            .title
            .chars()
            .filter(|c| !"[]:*?/\\".contains(*c))
            .take(31)
            .collect::<String>(),
    )
    .map_err(err)?;
    if report.rtl && !report.ltr_table {
        ws.set_right_to_left(true);
    }
    let mut row: u32 = 0;
    ws.write_string_with_format(row, 0, &report.heading, &title)
        .map_err(err)?;
    row += 1;
    ws.write_string_with_format(row, 0, &report.title, &bold)
        .map_err(err)?;
    row += 1;
    for (k, v) in &report.info {
        ws.write_string(row, 0, k).map_err(err)?;
        ws.write_string(row, 1, v).map_err(err)?;
        row += 1;
    }
    row += 1;
    for (c, column) in report.columns.iter().enumerate() {
        ws.write_string_with_format(row, c as u16, &column.label, &bold)
            .map_err(err)?;
    }
    row += 1;
    for r in &report.rows {
        for (c, cell) in r.iter().enumerate() {
            match as_number(cell) {
                Some(n) => ws.write_number(row, c as u16, n).map_err(err)?,
                None => ws.write_string(row, c as u16, cell).map_err(err)?,
            };
        }
        row += 1;
    }
    for note in &report.notes {
        row += 1;
        ws.write_string(row, 0, note).map_err(err)?;
    }
    ws.autofit();
    wb.save_to_buffer().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::as_number;

    #[test]
    fn numbers_and_halves() {
        assert_eq!(as_number("2½"), Some(2.5));
        assert_eq!(as_number("½"), Some(0.5));
        assert_eq!(as_number("2380"), Some(2380.0));
        assert_eq!(as_number("Okello"), None);
        assert_eq!(as_number("1-0"), None);
    }
}
