//! Team Swiss.
//!
//! Teams are paired with the Dutch criteria applied to teams, which is the
//! core of the FIDE team Swiss rules: no team meets another twice, at most
//! one pairing-allocated bye per team, brackets by the primary score (match
//! or game points), and colour (White on board 1) alternation and balance.
//! The team's colour is the colour of its first board. Teams are ranked by
//! primary score, then by their position in the team list.
//!
//! For game-point events, each round's game points enter as the
//! difference to match points through the acceleration channel, so the
//! brackets and float history use game points throughout.

use crate::dutch::DutchEngine;
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::{Availability, HistoryEntry, PairingInput, PairingPlayer, PairingTeam};
use co_core::scoring::RoundOutcome;
use co_core::{
    ByeKind, Error, Pairing, PairingSystem, PlayerId, PointSystem, Points, Result, SideResult,
    TeamId, TeamMatch, TeamScore,
};

pub struct TeamSwissEngine;

/// A match with default line-ups: the first available members in board order.
pub(crate) fn build_match(
    input: &PairingInput,
    number: u16,
    home: &PairingTeam,
    away: Option<&PairingTeam>,
) -> TeamMatch {
    let boards = usize::from(input.team.boards.max(1));
    let mut m = TeamMatch {
        number,
        home: home.id,
        away: away.map(|a| a.id),
        home_lineup: home.available.iter().copied().take(boards).collect(),
        away_lineup: away.map_or_else(Vec::new, |a| {
            a.available.iter().copied().take(boards).collect()
        }),
        games: Vec::new(),
    };
    if away.is_some() {
        m.rebuild_games();
    }
    m
}

/// The team view as an individual-pairing input: one "player" per team,
/// with team ids as player ids.
fn as_individual(input: &PairingInput) -> PairingInput {
    let ts = &input.team;
    let points = PointSystem {
        win: ts.match_win,
        draw: ts.match_draw,
        loss: ts.match_loss,
        forfeit_win: ts.match_win,
        forfeit_loss: ts.match_loss,
        pairing_bye: ts.match_win,
        half_bye: ts.match_draw,
        full_bye: ts.match_win,
        zero_bye: ts.match_loss,
    };
    let game_points = ts.primary_score == TeamScore::GamePoints;
    let players = input
        .teams
        .iter()
        .map(|team| {
            let history: Vec<HistoryEntry> = team
                .history
                .iter()
                .map(|h| HistoryEntry {
                    round: h.round,
                    opponent: h.opponent.map(|o| PlayerId(o.0)),
                    color: h.color,
                    outcome: if h.bye {
                        RoundOutcome::Bye(ByeKind::PairingAllocated)
                    } else if h.opponent.is_some() {
                        let side = if h.match_points == ts.match_win {
                            SideResult::Win
                        } else if h.match_points == ts.match_loss {
                            SideResult::Loss
                        } else {
                            SideResult::Draw
                        };
                        RoundOutcome::Game { result: Some(side) }
                    } else {
                        RoundOutcome::Absent
                    },
                })
                .collect();
            // Before round j+1: game points minus match points so far.
            let accelerations = if game_points {
                let mut acc = Vec::with_capacity(history.len() + 1);
                let (mut gp, mut mp) = (Points::ZERO, Points::ZERO);
                acc.push(Points::ZERO);
                for h in &team.history {
                    gp += h.game_points;
                    mp += h.match_points;
                    acc.push(Points(gp.0 - mp.0));
                }
                acc
            } else {
                Vec::new()
            };
            PairingPlayer {
                id: PlayerId(team.id.0),
                starting_rank: team.rank,
                rating: team.rating,
                score: if game_points {
                    team.game_points
                } else {
                    team.match_points
                },
                acceleration: accelerations.last().copied().unwrap_or_default(),
                accelerations,
                history,
                availability: if team.available.is_empty() {
                    Availability::Unavailable
                } else {
                    Availability::Available
                },
            }
        })
        .collect();
    PairingInput {
        round: input.round,
        total_rounds: input.total_rounds,
        points,
        initial_color: input.initial_color,
        players,
        forbidden: Vec::new(),
        double: false,
        teams: Vec::new(),
        team: input.team,
    }
}

impl PairingEngine for TeamSwissEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::TeamSwiss
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        if input.teams.len() < 2 {
            return Err(Error::Invalid(
                "a team Swiss needs at least two teams".into(),
            ));
        }
        let individual = as_individual(input);
        let paired = DutchEngine.pair(&individual)?;
        let team = |id: PlayerId| {
            input
                .team(TeamId(id.0))
                .ok_or_else(|| Error::Invalid(format!("unknown team {id}")))
        };
        let mut matches = Vec::new();
        let mut byes = Vec::new();
        for p in &paired.pairings {
            match *p {
                Pairing::Game { white, black, .. } => {
                    let n = matches.len() as u16 + 1;
                    matches.push(build_match(input, n, team(white)?, Some(team(black)?)));
                }
                Pairing::Bye {
                    player,
                    kind: ByeKind::PairingAllocated,
                } => byes.push(team(player)?),
                Pairing::Bye { .. } => {}
            }
        }
        for b in byes {
            let n = matches.len() as u16 + 1;
            matches.push(build_match(input, n, b, None));
        }
        Ok(RoundPairing {
            pairings: Vec::new(),
            matches,
        })
    }

    fn check(&self, input: &PairingInput, _proposed: &[Pairing]) -> Vec<crate::Violation> {
        let _ = input;
        Vec::new()
    }
}
