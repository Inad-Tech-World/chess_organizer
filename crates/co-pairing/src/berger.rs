//! Round robin with FIDE Berger tables, FIDE Handbook C.05 Annex 1,
//! including double round robin (colours reversed in the second cycle).
//!
//! Planned for Phase 2. Until then [`BergerEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct BergerEngine;

impl PairingEngine for BergerEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Berger
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("Round-robin (Berger) pairing", 2)
    }
}
