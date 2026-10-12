//! Team Swiss: teams paired by match (or game) points per the FIDE team
//! pairing rules; line-ups are entered per round.
//!
//! Planned for Phase 3. Until then [`TeamSwissEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct TeamSwissEngine;

impl PairingEngine for TeamSwissEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::TeamSwiss
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("Team Swiss pairing", 3)
    }
}
