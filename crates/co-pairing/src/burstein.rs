//! FIDE Burstein System, FIDE Handbook C.04.4.2.
//!
//! Planned for Phase 4. Until then [`BursteinEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct BursteinEngine;

impl PairingEngine for BursteinEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Burstein
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("FIDE Burstein pairing", 4)
    }
}
