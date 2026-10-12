//! FIDE Dubov System, FIDE Handbook C.04.4.1.
//!
//! Score groups are paired from the top. Inside a group the White-seekers,
//! ordered by ascending average rating of opponents (ARO), meet the
//! Black-seekers ordered by descending rating, first against first. Round 1
//! pairs the top half against the bottom half. See [`crate::bracket`] for
//! how completeness, floats and colours are weighed.

use crate::bracket::{self, Style};
use crate::swiss_model::Tournament;
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result};

pub struct DubovEngine;

impl PairingEngine for DubovEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Dubov
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        let t = Tournament::from_input(input)?;
        bracket::pair(&t, input, Style::Dubov)
    }
}
