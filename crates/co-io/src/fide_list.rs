//! FIDE rating list import (Phase 1).
//!
//! FIDE publishes monthly lists as zipped XML (`players_list_xml.zip`) and
//! fixed-width text. Both are streamed entry by entry so the full list
//! (well over a million players) never sits in memory.

use crate::Result;
use co_core::{Sex, Title, not_yet};
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FideListEntry {
    pub fide_id: u32,
    pub name: String,
    pub federation: String,
    pub sex: Option<Sex>,
    pub title: Option<Title>,
    pub rating_standard: Option<u16>,
    pub rating_rapid: Option<u16>,
    pub rating_blitz: Option<u16>,
    pub k_factor: Option<u8>,
    pub birth_year: Option<u16>,
    /// FIDE flag, e.g. `i` (inactive), `w` (woman), `wi`.
    pub flag: Option<String>,
}

/// Streams entries from FIDE's XML list, returning the number read.
pub fn read_xml<R: BufRead>(_reader: R, _on_entry: impl FnMut(FideListEntry)) -> Result<usize> {
    Ok(not_yet!("FIDE XML rating list import", 1)?)
}

/// Streams entries from FIDE's fixed-width TXT list.
pub fn read_txt<R: BufRead>(_reader: R, _on_entry: impl FnMut(FideListEntry)) -> Result<usize> {
    Ok(not_yet!("FIDE TXT rating list import", 1)?)
}
