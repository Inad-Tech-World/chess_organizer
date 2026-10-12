//! Baku acceleration, FIDE Handbook C.04.5.1.
//!
//! Planned for Phase 2. Computes the virtual points added to each player's
//! score for bracket placement in the accelerated rounds. Manual acceleration
//! (TRF `XXA`, `Player::acceleration`) already flows through `PairingInput`.

use co_core::pairing_view::PairingInput;
use co_core::{PlayerId, Points, Result, not_yet};

/// Acceleration points for each player in `input.round`.
pub fn acceleration_points(_input: &PairingInput) -> Result<Vec<(PlayerId, Points)>> {
    not_yet!("Baku acceleration", 2)
}
