//! Migration from files *exported* by Swiss-Manager (Phase 4).
//!
//! Only the program's documented export formats are read. Nothing here is
//! derived from its binaries or private file layouts (see docs/legal.md).

use crate::Result;
use co_core::{Tournament, not_yet};

pub fn import(_bytes: &[u8]) -> Result<Tournament> {
    Ok(not_yet!("Swiss-Manager export import", 4)?)
}
