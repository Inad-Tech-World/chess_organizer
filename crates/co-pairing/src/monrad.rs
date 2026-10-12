//! Monrad / simple Swiss for club play: pair top-down within score groups,
//! avoiding repeats, without the FIDE quality criteria.
//!
//! Planned for Phase 4. Until then [`MonradEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct MonradEngine;

impl PairingEngine for MonradEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Monrad
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("Monrad (simple Swiss) pairing", 4)
    }
}
