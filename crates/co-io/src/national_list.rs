//! National rating list import from CSV with a user-chosen column mapping
//! (Phase 1).

use crate::Result;
use co_core::not_yet;

/// Which CSV column (0-based) holds each field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColumnMapping {
    pub national_id: Option<usize>,
    pub name: Option<usize>,
    pub rating: Option<usize>,
    pub fide_id: Option<usize>,
    pub federation: Option<usize>,
    pub sex: Option<usize>,
    pub birth_year: Option<usize>,
    pub club: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NationalEntry {
    pub national_id: String,
    pub name: String,
    pub rating: Option<u16>,
    pub fide_id: Option<u32>,
    pub federation: Option<String>,
    pub club: Option<String>,
    pub birth_year: Option<u16>,
}

pub fn read_csv(_text: &str, _mapping: &ColumnMapping) -> Result<Vec<NationalEntry>> {
    Ok(not_yet!("National rating list import", 1)?)
}
