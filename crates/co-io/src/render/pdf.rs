//! Direct PDF export (Phase 1): a table layout over the same `Report`, with
//! embedded Noto fonts so names in any script print correctly, including
//! right-to-left text. Until then, printing the HTML preview to PDF through
//! the operating system's print dialog gives the same content.

use crate::{Report, Result};
use co_core::not_yet;

pub fn render(_report: &Report) -> Result<Vec<u8>> {
    Ok(not_yet!("PDF export", 1)?)
}
