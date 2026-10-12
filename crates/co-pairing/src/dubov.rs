//! FIDE Dubov System, FIDE Handbook C.04.4.1.
//!
//! Planned for Phase 4. Until then [`DubovEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct DubovEngine;

impl PairingEngine for DubovEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Dubov
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("FIDE Dubov pairing", 4)
    }
}
