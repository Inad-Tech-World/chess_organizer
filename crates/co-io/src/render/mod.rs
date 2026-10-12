//! Renderers for [`Report`](crate::Report)s.

pub mod csv;
pub mod html;
pub mod pdf;
pub mod xlsx;

use crate::{Report, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Html,
    Pdf,
    Csv,
    Xlsx,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Html => "html",
            Format::Pdf => "pdf",
            Format::Csv => "csv",
            Format::Xlsx => "xlsx",
        }
    }
}

pub fn render(report: &Report, format: Format) -> Result<Vec<u8>> {
    match format {
        Format::Html => html::render(report).map(String::into_bytes),
        Format::Pdf => pdf::render(report),
        Format::Csv => Ok(csv::render(report).into_bytes()),
        Format::Xlsx => xlsx::render(report),
    }
}
