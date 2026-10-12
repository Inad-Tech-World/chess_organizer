//! FIDE (Dutch) System, FIDE Handbook C.04.3 (2025 edition).
//!
//! Approach (docs/adr/0002-dutch-engine.md): follow bbpPairings (Apache-2.0).
//! Brackets are processed in score order; for each, every compatible pair is
//! an edge whose weight encodes the C.04.3 quality criteria as prioritised
//! fields of a wide unsigned integer; a maximum-weight matching
//! ([`crate::matching`]) picks the pairing; colours follow the colour
//! allocation rules and the pairing-allocated bye follows the 2025 rules.
//!
//! Planned for Phase 1. Until then [`DutchEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct DutchEngine;

impl PairingEngine for DutchEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Dutch
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("FIDE Dutch pairing", 1)
    }
}
