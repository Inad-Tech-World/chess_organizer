//! FIDE Lim System, FIDE Handbook C.04.4.3.
//!
//! Planned for Phase 4. Until then [`LimEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct LimEngine;

impl PairingEngine for LimEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Lim
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("FIDE Lim pairing", 4)
    }
}
