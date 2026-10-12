//! The engine-facing view of a tournament, used by every pairing system.
//!
//! It carries the same information as a TRF file, so pairing from a
//! `Tournament` and pairing from a TRF file give engines identical input.

use crate::ids::{PlayerId, RoundNo};
use crate::model::{ByeKind, Color, Pairing, PointSystem, Tournament};
use crate::points::Points;
use crate::scoring::{self, RoundOutcome};

#[derive(Debug, Clone, PartialEq)]
pub struct PairingInput {
    /// The round being paired.
    pub round: RoundNo,
    pub total_rounds: RoundNo,
    pub points: PointSystem,
    pub initial_color: Color,
    /// Every player in the tournament, in starting-rank order.
    pub players: Vec<PairingPlayer>,
    /// Groups of players who must not meet.
    pub forbidden: Vec<Vec<PlayerId>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairingPlayer {
    pub id: PlayerId,
    pub starting_rank: u32,
    pub rating: u16,
    /// Score before the round being paired.
    pub score: Points,
    /// Acceleration points for the round being paired.
    pub acceleration: Points,
    /// One entry per previous round.
    pub history: Vec<HistoryEntry>,
    pub availability: Availability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Available,
    /// Requested bye or pause for this round.
    Bye(ByeKind),
    /// Withdrawn or not yet joined.
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryEntry {
    pub round: RoundNo,
    pub opponent: Option<PlayerId>,
    pub color: Option<Color>,
    pub outcome: RoundOutcome,
}

impl PairingPlayer {
    pub fn is_available(&self) -> bool {
        self.availability == Availability::Available
    }

    /// Opponents met over the board or by forfeit.
    pub fn opponents(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.history.iter().filter_map(|h| h.opponent)
    }

    /// Colours of games actually played, oldest first.
    pub fn played_colors(&self) -> impl Iterator<Item = Color> + '_ {
        self.history.iter().filter_map(|h| match h.outcome {
            RoundOutcome::Game { result: Some(r) } if r.is_played() => h.color,
            _ => None,
        })
    }

    pub fn had_pairing_bye(&self) -> bool {
        self.history
            .iter()
            .any(|h| matches!(h.outcome, RoundOutcome::Bye(ByeKind::PairingAllocated)))
    }

    /// Score used for bracket placement: real score plus acceleration.
    pub fn pairing_score(&self) -> Points {
        self.score + self.acceleration
    }
}

impl PairingInput {
    /// Builds the input for pairing the next unpaired round of `t`.
    pub fn for_next_round(t: &Tournament) -> PairingInput {
        let round = t.paired_rounds() + 1;
        let scores = scoring::scores_after(t, round - 1);
        let indexes: Vec<_> = t.rounds.iter().map(|r| r.index()).collect();
        let players = t
            .players_by_rank()
            .into_iter()
            .map(|p| {
                let history = t
                    .rounds
                    .iter()
                    .zip(&indexes)
                    .map(|(r, index)| {
                        let entry = index.get(&p.id).copied();
                        let side = entry.and_then(|e| e.side_of(p.id));
                        HistoryEntry {
                            round: r.number,
                            opponent: side.map(|(_, o)| o),
                            color: side.map(|(c, _)| c),
                            outcome: scoring::outcome_of(p, r.number, entry),
                        }
                    })
                    .collect();
                let availability = if let Some(kind) = p.byes.get(&round) {
                    Availability::Bye(*kind)
                } else if p.is_available(round) {
                    Availability::Available
                } else {
                    Availability::Unavailable
                };
                PairingPlayer {
                    id: p.id,
                    starting_rank: p.starting_rank,
                    rating: p.rating(t.settings.rating_type).unwrap_or(0),
                    score: scores.get(&p.id).copied().unwrap_or_default(),
                    acceleration: p
                        .acceleration
                        .get(usize::from(round) - 1)
                        .copied()
                        .unwrap_or_default(),
                    history,
                    availability,
                }
            })
            .collect();
        PairingInput {
            round,
            total_rounds: t.settings.rounds,
            points: t.settings.points,
            initial_color: t.settings.initial_color,
            players,
            forbidden: t
                .settings
                .forbidden_groups
                .iter()
                .map(|g| g.members.clone())
                .collect(),
        }
    }

    /// Input for pairing `round` again as if it had not been paired yet:
    /// later rounds are dropped and the round's requested byes and absences
    /// become bye requests. Used to check or reproduce existing pairings.
    pub fn for_existing_round(t: &Tournament, round: RoundNo) -> PairingInput {
        let mut before = t.clone();
        before.rounds.truncate(usize::from(round).saturating_sub(1));
        if let Some(r) = t.round(round) {
            for p in &r.pairings {
                if let Pairing::Bye { player, kind } = *p
                    && kind != ByeKind::PairingAllocated
                    && let Some(pl) = before.player_mut(player)
                {
                    pl.byes.insert(round, kind);
                }
            }
        }
        PairingInput::for_next_round(&before)
    }

    pub fn player(&self, id: PlayerId) -> Option<&PairingPlayer> {
        self.players.iter().find(|p| p.id == id)
    }

    pub fn is_forbidden(&self, a: PlayerId, b: PlayerId) -> bool {
        self.forbidden
            .iter()
            .any(|g| g.contains(&a) && g.contains(&b))
    }
}
