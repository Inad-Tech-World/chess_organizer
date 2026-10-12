//! FIDE Lim System, FIDE Handbook C.04.4.3.
//!
//! Score groups are paired from the top; inside a group the upper half (S1)
//! meets the lower half (S2) in order, S1[1] against S2[1] and so on, with
//! colour preferences deciding between otherwise equal alternatives. See
//! [`crate::bracket`] for how completeness, floats and colours are weighed.

use crate::bracket::{self, Style};
use crate::swiss_model::Tournament;
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{PairingSystem, Result};

pub struct LimEngine;

impl PairingEngine for LimEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Lim
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        let t = Tournament::from_input(input)?;
        bracket::pair(&t, input, Style::Lim)
    }
}
