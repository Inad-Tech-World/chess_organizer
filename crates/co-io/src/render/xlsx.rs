//! Excel export (Phase 2).

use crate::{Report, Result};
use co_core::not_yet;

pub fn render(_report: &Report) -> Result<Vec<u8>> {
    Ok(not_yet!("Excel export", 2)?)
}
