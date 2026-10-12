//! Team round robin: Berger tables over teams; board colours alternate
//! within each match.
//!
//! Planned for Phase 3. Until then [`TeamRoundRobinEngine::pair`] returns
//! `Error::NotYetImplemented`; manual pairing and the shared rule checks
//! already work for this system.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result, not_yet};

pub struct TeamRoundRobinEngine;

impl PairingEngine for TeamRoundRobinEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::TeamRoundRobin
    }

    fn pair(&self, _input: &PairingInput) -> Result<RoundPairing> {
        not_yet!("Team round-robin pairing", 3)
    }
}
