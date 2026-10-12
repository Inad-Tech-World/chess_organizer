//! Baku acceleration, FIDE Handbook C.04.5.1.
//!
//! The virtual points are computed where the engine input is built
//! ([`co_core::pairing_view::baku_accelerations`]), so every Swiss engine
//! sees them as ordinary acceleration; this module only exposes them.

use co_core::pairing_view::PairingInput;
use co_core::{PlayerId, Points, Result};

pub use co_core::pairing_view::baku_accelerations;

/// Acceleration points for each player in `input.round`.
pub fn acceleration_points(input: &PairingInput) -> Result<Vec<(PlayerId, Points)>> {
    Ok(input
        .players
        .iter()
        .map(|p| (p.id, p.acceleration))
        .collect())
}
