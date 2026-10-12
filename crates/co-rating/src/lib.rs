//! FIDE rating calculations (Handbook B.02) and title norm checks (B.01).
//!
//! Rating changes land in Phase 2 and norm checks in Phase 4. The API is
//! final so the rating-report screens can be built against it now.

use co_core::{Player, PlayerId, Points, RatingType, Result, Tournament, not_yet};
use serde::Serialize;

/// One rated game from the player's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatedGame {
    pub opponent_rating: u16,
    pub score: Points,
}

/// Rating change for a player with `rating` and development coefficient `k`.
pub fn rating_change(_rating: u16, _k: u8, _games: &[RatedGame]) -> Result<f64> {
    not_yet!("FIDE rating change calculation", 2)
}

/// Development coefficient K for `player` in `rating_type` (B.02 §8).
pub fn k_factor(_player: &Player, _rating_type: RatingType) -> Result<u8> {
    not_yet!("K-factor determination", 2)
}

/// Tournament performance rating from opponents' ratings and the score.
pub fn performance(_games: &[RatedGame]) -> Result<u16> {
    not_yet!("Performance rating", 2)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NormTitle {
    GM,
    IM,
    WGM,
    WIM,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormReport {
    pub player: PlayerId,
    pub title: NormTitle,
    pub achieved: bool,
    pub games: u16,
    pub performance: u16,
    pub average_opponent_rating: u16,
    /// Requirements that were not met, in plain language.
    pub shortfalls: Vec<String>,
}

/// Checks every title norm for `player` after the last completed round.
pub fn norm_check(_tournament: &Tournament, _player: PlayerId) -> Result<Vec<NormReport>> {
    not_yet!("Title norm check", 4)
}

/// Per-player rating changes for the whole tournament (rating report).
pub fn tournament_rating_changes(_tournament: &Tournament) -> Result<Vec<(PlayerId, f64)>> {
    not_yet!("Tournament rating report", 2)
}
