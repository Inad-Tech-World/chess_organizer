//! FIDE rating calculations (Handbook B.02) and title norm checks (B.01).
//!
//! The FIDE Rating Server computes official ratings; these figures let the
//! arbiter check a rating report before sending it, and show players their
//! expected change.

pub mod norms;
pub mod tables;

use co_core::{
    Pairing, Player, PlayerId, Points, RatingType, Result, SideResult, Tournament, scoring,
};
use serde::Serialize;

pub use norms::{NormReport, NormTitle, norm_check};

/// One rated game from the player's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatedGame {
    pub opponent_rating: u16,
    pub score: Points,
}

/// Rating change for a player rated `rating` with development coefficient
/// `k` (B.02 8.3): Σ K·(score − expected), with the 400-point rule and the
/// rule that K × games may not exceed 700.
pub fn rating_change(rating: u16, k: u8, games: &[RatedGame]) -> Result<f64> {
    if games.is_empty() {
        return Ok(0.0);
    }
    let n = games.len() as u32;
    let mut k = u32::from(k);
    if k * n > 700 {
        k = 700 / n;
    }
    let delta: f64 = games
        .iter()
        .map(|g| {
            g.score.as_f64()
                - tables::expected_capped(i32::from(rating) - i32::from(g.opponent_rating))
        })
        .sum();
    Ok(f64::from(k) * delta)
}

/// Development coefficient K (B.02 8.3.3). An explicit K from the rating list
/// wins; otherwise it is derived from rating and age.
pub fn k_factor(
    player: &Player,
    rating_type: RatingType,
    tournament_year: Option<i32>,
) -> Result<u8> {
    if let Some(k) = player.k_factor {
        return Ok(k);
    }
    if rating_type != RatingType::Standard {
        return Ok(20);
    }
    let rating = player.ratings.fide(rating_type).unwrap_or(0);
    if rating >= 2400 {
        return Ok(10);
    }
    let junior = match (player.birth_year(), tournament_year) {
        (Some(born), Some(year)) => year - i32::from(born) < 18,
        _ => false,
    };
    Ok(if junior && rating < 2300 { 40 } else { 20 })
}

/// Tournament performance rating: average opponent rating plus `dp`.
pub fn performance(games: &[RatedGame]) -> Result<u16> {
    if games.is_empty() {
        return Ok(0);
    }
    let n = games.len() as f64;
    let ra: f64 = games
        .iter()
        .map(|g| f64::from(g.opponent_rating))
        .sum::<f64>()
        / n;
    let p = games.iter().map(|g| g.score.as_f64()).sum::<f64>() / n;
    Ok((ra.round() as i32 + tables::dp(p)).max(0) as u16)
}

/// One row of the rating report.
#[derive(Debug, Clone, Serialize)]
pub struct RatingChangeRow {
    pub player: PlayerId,
    pub rating: Option<u16>,
    pub k: Option<u8>,
    /// Rated games played (forfeits and games against unrated players excluded).
    pub games: u32,
    pub score: f64,
    pub expected: f64,
    pub change: f64,
    pub average_opponent_rating: Option<u16>,
    pub performance: Option<u16>,
    /// For unrated players with enough games: an estimate of the initial rating.
    pub initial_rating_estimate: Option<u16>,
}

fn score_of(r: SideResult) -> Option<Points> {
    match r {
        SideResult::Win => Some(Points::ONE),
        SideResult::Draw => Some(Points::HALF),
        SideResult::Loss => Some(Points::ZERO),
        _ => None, // forfeits and unrated games are not rated
    }
}

/// Rated games of `player` (OTB, rated result) in the tournament's rating type.
pub fn rated_games(t: &Tournament, player: PlayerId) -> Vec<(PlayerId, Option<u16>, Points)> {
    let rt = t.settings.rating_type;
    let mut out = Vec::new();
    for round in &t.rounds {
        if let Some(Pairing::Game {
            white,
            black,
            result: Some(res),
            ..
        }) = round.entry_for(player)
        {
            let (side, opp) = if *white == player {
                (res.white, *black)
            } else {
                (res.black, *white)
            };
            if let Some(score) = score_of(side) {
                let opp_rating = t.player(opp).and_then(|p| p.ratings.fide(rt));
                out.push((opp, opp_rating, score));
            }
        }
    }
    out
}

/// Rating changes for every player (the arbiter's rating report preview).
pub fn tournament_rating_changes(t: &Tournament) -> Result<Vec<RatingChangeRow>> {
    let rt = t.settings.rating_type;
    let year = t
        .info
        .end_date
        .or(t.info.start_date)
        .map(|d| d.format("%Y").to_string().parse().unwrap_or(0));
    let mut rows = Vec::new();
    for p in t.players_by_rank() {
        let own = p.ratings.fide(rt);
        let games = rated_games(t, p.id);
        let vs_rated: Vec<RatedGame> = games
            .iter()
            .filter_map(|(_, r, s)| {
                r.map(|opponent_rating| RatedGame {
                    opponent_rating,
                    score: *s,
                })
            })
            .collect();
        let score: f64 = vs_rated.iter().map(|g| g.score.as_f64()).sum();
        let aro = (!vs_rated.is_empty()).then(|| {
            (vs_rated
                .iter()
                .map(|g| f64::from(g.opponent_rating))
                .sum::<f64>()
                / vs_rated.len() as f64)
                .round() as u16
        });
        let perf = (!vs_rated.is_empty())
            .then(|| performance(&vs_rated))
            .transpose()?;
        let (k, expected, change) = match own {
            Some(r) => {
                let k = k_factor(p, rt, year)?;
                let expected: f64 = vs_rated
                    .iter()
                    .map(|g| tables::expected_capped(i32::from(r) - i32::from(g.opponent_rating)))
                    .sum();
                (Some(k), expected, rating_change(r, k, &vs_rated)?)
            }
            None => (None, 0.0, 0.0),
        };
        // Unrated: initial rating from at least five games against rated
        // opponents, with two hypothetical draws against 1800-rated opponents.
        let initial = (own.is_none() && vs_rated.len() >= 5).then(|| {
            let n = vs_rated.len() as f64 + 2.0;
            let ra = (vs_rated
                .iter()
                .map(|g| f64::from(g.opponent_rating))
                .sum::<f64>()
                + 3600.0)
                / n;
            let p = (score + 1.0) / n;
            (ra.round() as i32 + tables::dp(p)).max(0) as u16
        });
        rows.push(RatingChangeRow {
            player: p.id,
            rating: own,
            k,
            games: vs_rated.len() as u32,
            score,
            expected,
            change,
            average_opponent_rating: aro,
            performance: perf,
            initial_rating_estimate: initial,
        });
    }
    let _ = scoring::scores_after;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(r: u16, s: i32) -> RatedGame {
        RatedGame {
            opponent_rating: r,
            score: Points(s),
        }
    }

    #[test]
    fn change_follows_fide_tables() {
        // 2000 vs 2000, win: K × (1 − 0.50).
        assert!((rating_change(2000, 20, &[g(2000, 10)]).unwrap() - 10.0).abs() < 1e-9);
        // 400-point rule: 2600 vs 2000 counts as a 400 difference (0.92).
        assert!((rating_change(2600, 10, &[g(2000, 10)]).unwrap() - 0.8).abs() < 1e-9);
        // K × n ≤ 700: 40 × 20 games → K = 35.
        let many = vec![g(2000, 10); 20];
        assert!((rating_change(2000, 40, &many).unwrap() - 35.0 * 20.0 * 0.5).abs() < 1e-9);
    }

    #[test]
    fn performance_uses_dp() {
        assert_eq!(
            performance(&[g(2000, 10), g(2100, 5)]).unwrap(),
            2050 + tables::dp(0.75) as u16
        );
    }
}
