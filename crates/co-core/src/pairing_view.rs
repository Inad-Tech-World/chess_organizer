//! The engine-facing view of a tournament, used by every pairing system.
//!
//! It carries the same information as a TRF file, so pairing from a
//! `Tournament` and pairing from a TRF file give engines identical input.

use crate::ids::{PlayerId, RoundNo, TeamId};
use crate::model::{
    AccelerationMethod, ByeKind, Color, Pairing, PointSystem, TeamSettings, Tournament,
    TournamentKind,
};
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
    /// Round robins: play every opponent twice, colours reversed.
    pub double: bool,
    /// Team events: every team in starting-rank order; empty otherwise.
    pub teams: Vec<PairingTeam>,
    pub team: TeamSettings,
}

/// A team as the team pairing engines see it.
#[derive(Debug, Clone, PartialEq)]
pub struct PairingTeam {
    pub id: TeamId,
    /// 1-based position in the team list (the team starting rank).
    pub rank: u32,
    /// Average rating of the top `boards` members, 0 if unrated.
    pub rating: u16,
    pub match_points: Points,
    pub game_points: Points,
    /// Members in board order who can play this round.
    pub available: Vec<PlayerId>,
    /// One entry per previous round.
    pub history: Vec<TeamHistory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamHistory {
    pub round: RoundNo,
    /// `None` for a bye or when the team was not paired.
    pub opponent: Option<TeamId>,
    /// Whether the team had White on board 1 (the team's colour).
    pub color: Option<Color>,
    pub match_points: Points,
    pub game_points: Points,
    pub bye: bool,
}

/// Match and game points of both teams of a match, `None` while a board
/// result is missing. A bye scores as a win on every board.
pub fn team_match_points(
    m: &crate::model::TeamMatch,
    ts: &TeamSettings,
    ps: &PointSystem,
) -> ((Points, Points), (Points, Points), bool) {
    if m.away.is_none() {
        let gp = Points(ps.win.0 * i32::from(ts.boards));
        return ((ts.match_win, gp), (Points::ZERO, Points::ZERO), true);
    }
    let (mut hg, mut ag, mut complete) = (Points::ZERO, Points::ZERO, true);
    for g in &m.games {
        if let Pairing::Game { white, result, .. } = *g {
            match result {
                Some(r) => {
                    let (w, b) = (
                        scoring::side_points(r.white, ps),
                        scoring::side_points(r.black, ps),
                    );
                    if m.home_lineup.contains(&white) {
                        hg += w;
                        ag += b;
                    } else {
                        hg += b;
                        ag += w;
                    }
                }
                None => complete = false,
            }
        }
    }
    let (hm, am) = match hg.cmp(&ag) {
        std::cmp::Ordering::Greater => (ts.match_win, ts.match_loss),
        std::cmp::Ordering::Less => (ts.match_loss, ts.match_win),
        std::cmp::Ordering::Equal => (ts.match_draw, ts.match_draw),
    };
    ((hm, hg), (am, ag), complete)
}

fn team_view(t: &Tournament, round: RoundNo) -> Vec<PairingTeam> {
    let ts = &t.settings.team;
    let ps = &t.settings.points;
    t.teams
        .iter()
        .enumerate()
        .map(|(i, team)| {
            let mut history = Vec::new();
            for r in t.rounds.iter().take(usize::from(round) - 1) {
                let m = r
                    .matches
                    .iter()
                    .find(|m| m.home == team.id || m.away == Some(team.id));
                history.push(match m {
                    Some(m) => {
                        let (home, away, _) = team_match_points(m, ts, ps);
                        let is_home = m.home == team.id;
                        let (mp, gp) = if is_home { home } else { away };
                        TeamHistory {
                            round: r.number,
                            opponent: if is_home { m.away } else { Some(m.home) },
                            color: m
                                .away
                                .map(|_| if is_home { Color::White } else { Color::Black }),
                            match_points: mp,
                            game_points: gp,
                            bye: m.away.is_none(),
                        }
                    }
                    None => TeamHistory {
                        round: r.number,
                        opponent: None,
                        color: None,
                        match_points: Points::ZERO,
                        game_points: Points::ZERO,
                        bye: false,
                    },
                });
            }
            let available: Vec<PlayerId> = team
                .members
                .iter()
                .copied()
                .filter(|id| {
                    t.player(*id)
                        .is_some_and(|p| p.is_available(round) && !p.byes.contains_key(&round))
                })
                .collect();
            let ratings: Vec<u32> = team
                .members
                .iter()
                .filter_map(|id| t.player(*id)?.rating(t.settings.rating_type))
                .take(usize::from(ts.boards.max(1)))
                .map(u32::from)
                .collect();
            let rating = if ratings.is_empty() {
                0
            } else {
                (ratings.iter().sum::<u32>() / ratings.len() as u32) as u16
            };
            PairingTeam {
                id: team.id,
                rank: i as u32 + 1,
                rating,
                match_points: history.iter().map(|h| h.match_points).sum(),
                game_points: history.iter().map(|h| h.game_points).sum(),
                available,
                history,
            }
        })
        .collect()
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
    /// Acceleration points of every round, index 0 = round 1.
    pub accelerations: Vec<Points>,
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

/// Baku acceleration (C.04.5.1) for the player at 0-based `rank_index` of
/// `players`, one entry per accelerated round. As in bbpPairings: the first
/// half of the players (rounded up) get a win's worth of virtual points in
/// the first quarter of the rounds and a draw's worth in the rest of the
/// first half; nobody is accelerated after that.
pub fn baku_accelerations(
    rank_index: usize,
    players: usize,
    rounds: RoundNo,
    points: &PointSystem,
) -> Vec<Points> {
    if players == 0 || rounds == 0 || rank_index > (players - 1) / 2 {
        return Vec::new();
    }
    let last_accelerated = usize::from(rounds - 1) / 2;
    let last_full = last_accelerated / 2;
    (0..=last_accelerated)
        .map(|r| {
            if r <= last_full {
                points.win
            } else {
                points.draw
            }
        })
        .collect()
}

impl PairingInput {
    /// Builds the input for pairing the next unpaired round of `t`.
    pub fn for_next_round(t: &Tournament) -> PairingInput {
        let round = t.paired_rounds() + 1;
        let scores = scoring::scores_after(t, round - 1);
        let indexes: Vec<_> = t.rounds.iter().map(|r| r.index()).collect();
        let by_rank = t.players_by_rank();
        let n = by_rank.len();
        let baku = t.settings.acceleration == Some(AccelerationMethod::Baku)
            && by_rank.iter().all(|p| p.acceleration.is_empty());
        let players = by_rank
            .into_iter()
            .enumerate()
            .map(|(rank_index, p)| {
                let accelerations = if baku {
                    baku_accelerations(rank_index, n, t.settings.rounds, &t.settings.points)
                } else {
                    p.acceleration.clone()
                };
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
                    acceleration: accelerations
                        .get(usize::from(round) - 1)
                        .copied()
                        .unwrap_or_default(),
                    accelerations,
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
            double: matches!(
                t.kind,
                TournamentKind::RoundRobin { double: true }
                    | TournamentKind::TeamRoundRobin { double: true }
            ),
            teams: if t.kind.is_team() {
                team_view(t, round)
            } else {
                Vec::new()
            },
            team: t.settings.team,
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

    pub fn team(&self, id: TeamId) -> Option<&PairingTeam> {
        self.teams.iter().find(|t| t.id == id)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baku_follows_bbp_pairings() {
        let ps = PointSystem::default();
        // 9 rounds: rounds 1-3 a full point, 4-5 half a point.
        let a = baku_accelerations(0, 20, 9, &ps);
        assert_eq!(
            a,
            [
                Points::ONE,
                Points::ONE,
                Points::ONE,
                Points::HALF,
                Points::HALF
            ]
        );
        // Top half of 21 players (rounded up) = 11 players.
        assert!(!baku_accelerations(10, 21, 9, &ps).is_empty());
        assert!(baku_accelerations(11, 21, 9, &ps).is_empty());
        // 11 rounds: 1-3 full, 4-6 half.
        assert_eq!(baku_accelerations(0, 20, 11, &ps).len(), 6);
        assert_eq!(baku_accelerations(0, 20, 11, &ps)[3], Points::HALF);
    }
}
