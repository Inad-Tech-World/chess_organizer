//! The per-player history model shared by the FIDE Swiss systems: one record
//! per round (opponent, colour, score, whether the game was played, whether
//! the player took part in the pairing), with colour preferences and floats
//! derived as in FIDE C.04.1 / C.04.3. Semantics follow bbpPairings'
//! `tournament` module (Apache-2.0) so the engines agree with it exactly.

use crate::RoundPairing;
use co_core::pairing_view::{Availability, PairingInput};
use co_core::scoring::RoundOutcome;
use co_core::{ByeKind, Color, Error, PlayerId, Result, SideResult};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum C {
    White,
    Black,
    None,
}

impl C {
    pub(crate) fn invert(self) -> C {
        match self {
            C::White => C::Black,
            C::Black => C::White,
            C::None => C::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Ms {
    Loss,
    Draw,
    Win,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Match {
    /// Opponent index, or the player's own index when there was none.
    pub opponent: usize,
    pub color: C,
    pub score: Ms,
    pub played: bool,
    /// Paired, or given the pairing-allocated bye.
    pub participated: bool,
}

impl Match {
    fn none(own: usize, score: Ms) -> Self {
        Self {
            opponent: own,
            color: C::None,
            score,
            played: false,
            participated: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Player {
    pub id: usize,
    pub player_id: PlayerId,
    pub matches: Vec<Match>,
    /// Per round, in tenths.
    pub accelerations: Vec<i64>,
    pub color_imbalance: usize,
    pub rank_index: usize,
    pub played_games: usize,
    pub score: i64,
    pub color_preference: C,
    pub repeated_color: C,
    pub strong_color_preference: bool,
    pub valid: bool,
}

impl Player {
    pub(crate) fn absolute_color_imbalance(&self) -> bool {
        self.color_imbalance > 1
    }

    pub(crate) fn absolute_color_preference(&self) -> bool {
        self.absolute_color_imbalance() || self.repeated_color != C::None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Tournament {
    pub players: Vec<Player>,
    pub played_rounds: usize,
    pub expected_rounds: usize,
    pub points_for_win: i64,
    pub points_for_draw: i64,
    pub points_for_loss: i64,
    pub points_for_zero_point_bye: i64,
    pub points_for_forfeit_loss: i64,
    pub points_for_pab: i64,
    pub initial_color: C,
    pub forbidden: Vec<HashSet<usize>>,
}

impl Tournament {
    pub(crate) fn get_points(&self, player: &Player, m: &Match) -> i64 {
        match m.score {
            Ms::Loss => {
                if m.participated {
                    if m.played {
                        self.points_for_loss
                    } else {
                        self.points_for_forfeit_loss
                    }
                } else {
                    self.points_for_zero_point_bye
                }
            }
            Ms::Win => {
                if m.opponent == player.id && m.participated {
                    self.points_for_pab
                } else {
                    self.points_for_win
                }
            }
            Ms::Draw => self.points_for_draw,
        }
    }

    /// Score with acceleration, `rounds_back` rounds before the current one.
    pub(crate) fn score_acc(&self, p: &Player, rounds_back: usize) -> i64 {
        let mut score = p.score;
        let mut round = self.played_rounds;
        for _ in 0..rounds_back {
            round -= 1;
            score -= self.get_points(p, &p.matches[round]);
        }
        score + p.accelerations.get(round).copied().unwrap_or(0)
    }

    /// `player0` ranks below `player1` (lower score, or equal score and a
    /// higher rank index).
    pub(crate) fn accelerated_lower(&self, p0: &Player, p1: &Player) -> bool {
        (self.score_acc(p0, 0), p1.rank_index) < (self.score_acc(p1, 0), p0.rank_index)
    }

    pub(crate) fn unaccelerated_lower(p0: &Player, p1: &Player) -> bool {
        (p0.score, p1.rank_index) < (p1.score, p0.rank_index)
    }

    pub(crate) fn eligible_for_bye(&self, p: &Player) -> bool {
        p.matches.iter().all(|m| {
            !(!m.played
                && (self.get_points(p, m) >= self.points_for_win
                    || (m.participated && m.opponent == p.id)))
        })
    }

    /// Builds the model for pairing `input.round`. Players with a requested
    /// bye, a pause or who are unavailable get a record for the current round
    /// and are therefore not paired, as in TRF.
    pub(crate) fn from_input(input: &PairingInput) -> Result<Self> {
        let index: HashMap<PlayerId, usize> = input
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i))
            .collect();
        let played_rounds = usize::from(input.round) - 1;
        let ps = &input.points;
        let mut t = Tournament {
            players: Vec::with_capacity(input.players.len()),
            played_rounds,
            expected_rounds: usize::from(input.total_rounds),
            points_for_win: i64::from(ps.win.0),
            points_for_draw: i64::from(ps.draw.0),
            points_for_loss: i64::from(ps.loss.0),
            points_for_zero_point_bye: i64::from(ps.zero_bye.0),
            points_for_forfeit_loss: i64::from(ps.forfeit_loss.0),
            points_for_pab: i64::from(ps.pairing_bye.0),
            initial_color: match input.initial_color {
                Color::White => C::White,
                Color::Black => C::Black,
            },
            forbidden: vec![HashSet::new(); input.players.len()],
        };

        for (i, pp) in input.players.iter().enumerate() {
            let mut matches = Vec::with_capacity(played_rounds + 1);
            for h in &pp.history {
                let m = match h.outcome {
                    RoundOutcome::Game { result: Some(r) } => {
                        let opponent =
                            h.opponent
                                .and_then(|o| index.get(&o).copied())
                                .ok_or_else(|| {
                                    Error::Invalid(format!("unknown opponent in round {}", h.round))
                                })?;
                        Match {
                            opponent,
                            color: match h.color {
                                Some(Color::White) => C::White,
                                Some(Color::Black) => C::Black,
                                None => C::None,
                            },
                            score: side_score(r),
                            played: r.is_played(),
                            participated: true,
                        }
                    }
                    RoundOutcome::Game { result: None } => {
                        return Err(Error::Invalid(format!(
                            "round {} still has games without a result",
                            h.round
                        )));
                    }
                    RoundOutcome::Bye(ByeKind::PairingAllocated) => Match {
                        opponent: i,
                        color: C::None,
                        score: Ms::Win,
                        played: false,
                        participated: true,
                    },
                    RoundOutcome::Bye(k) | RoundOutcome::LateEntry(k) => {
                        Match::none(i, bye_score(k))
                    }
                    RoundOutcome::Absent => Match::none(i, Ms::Loss),
                };
                matches.push(m);
            }
            match pp.availability {
                Availability::Available => {}
                Availability::Bye(k) => matches.push(Match::none(i, bye_score(k))),
                Availability::Unavailable => matches.push(Match::none(i, Ms::Loss)),
            }
            t.players.push(Player {
                id: i,
                player_id: pp.id,
                matches,
                accelerations: pp.accelerations.iter().map(|a| i64::from(a.0)).collect(),
                color_imbalance: 0,
                rank_index: i,
                played_games: 0,
                score: 0,
                color_preference: C::None,
                repeated_color: C::None,
                strong_color_preference: false,
                valid: true,
            });
        }
        // The engine's score is recomputed from the records, as bbpPairings does.
        for i in 0..t.players.len() {
            let p = &t.players[i];
            let score: i64 = p
                .matches
                .iter()
                .take(played_rounds)
                .map(|m| t.get_points(p, m))
                .sum();
            t.players[i].score = score;
        }
        for group in &input.forbidden {
            let members: Vec<usize> = group
                .iter()
                .filter_map(|id| index.get(id).copied())
                .collect();
            for &a in &members {
                t.forbidden[a].extend(members.iter().copied());
            }
        }
        t.update_ranks();
        t.compute_player_data();
        Ok(t)
    }

    fn update_ranks(&mut self) {
        let mut rank = 0;
        for p in &mut self.players {
            p.valid =
                p.matches.len() <= self.played_rounds || p.matches.iter().any(|m| m.participated);
            if p.valid {
                p.rank_index = rank;
                rank += 1;
            }
        }
    }

    fn compute_player_data(&mut self) {
        for p in &mut self.players {
            if !p.valid {
                continue;
            }
            let (mut white, mut black, mut consecutive, mut played) =
                (0usize, 0usize, 0usize, 0usize);
            for m in &p.matches {
                if m.played {
                    played += 1;
                    if m.color == C::White {
                        white += 1;
                    } else {
                        black += 1;
                    }
                    if consecutive == 0 || m.color != p.repeated_color {
                        consecutive = 1;
                    } else {
                        consecutive += 1;
                    }
                    p.repeated_color = m.color;
                }
            }
            p.played_games = played;
            let lower = if white > black { C::Black } else { C::White };
            p.color_imbalance = if lower == C::Black {
                white - black
            } else {
                black - white
            };
            p.color_preference = if p.color_imbalance > 1 {
                lower
            } else if consecutive > 1 {
                p.repeated_color.invert()
            } else if p.color_imbalance > 0 {
                lower
            } else if consecutive > 0 {
                p.repeated_color.invert()
            } else {
                C::None
            };
            if consecutive <= 1 {
                p.repeated_color = C::None;
            }
            p.strong_color_preference = !p.absolute_color_preference() && p.color_imbalance > 0;
        }
    }

    /// Converts `(white, black)` index pairs (bye: `white == black`) into a
    /// round, numbering boards in the published order.
    pub(crate) fn to_round(
        &self,
        input: &PairingInput,
        mut pairs: Vec<(usize, usize)>,
    ) -> RoundPairing {
        sort_results(&mut pairs, self);
        let mut games = Vec::new();
        let mut pab = None;
        for (w, b) in pairs {
            if w == b {
                pab = Some(self.players[w].player_id);
            } else {
                games.push((self.players[w].player_id, self.players[b].player_id));
            }
        }
        RoundPairing {
            pairings: crate::finalize_round(input, &games, pab),
            matches: Vec::new(),
        }
    }
}

fn side_score(r: SideResult) -> Ms {
    match r {
        SideResult::Win | SideResult::UnratedWin | SideResult::ForfeitWin => Ms::Win,
        SideResult::Draw | SideResult::UnratedDraw => Ms::Draw,
        SideResult::Loss | SideResult::UnratedLoss | SideResult::ForfeitLoss => Ms::Loss,
    }
}

fn bye_score(k: ByeKind) -> Ms {
    match k {
        ByeKind::PairingAllocated | ByeKind::Full => Ms::Win,
        ByeKind::Half => Ms::Draw,
        ByeKind::Zero => Ms::Loss,
    }
}

/// FIDE board order: games before the bye; higher scores first; then the
/// higher player's rank.
pub(crate) fn sort_results(pairs: &mut [(usize, usize)], t: &Tournament) {
    let key = |&(w, b): &(usize, usize)| {
        let (pw, pb) = (&t.players[w], &t.players[b]);
        let (higher, lower) = if Tournament::unaccelerated_lower(pw, pb) {
            (pb, pw)
        } else {
            (pw, pb)
        };
        (
            w == b,
            std::cmp::Reverse(higher.score),
            std::cmp::Reverse(lower.score),
            higher.rank_index,
        )
    };
    pairs.sort_by_key(key);
}

pub(crate) fn color_preferences_compatible(a: C, b: C) -> bool {
    a != b || a == C::None || b == C::None
}

/// Colours of the two players in the most recent round in which they differed.
pub(crate) fn find_first_color_difference(p0: &Player, p1: &Player) -> (C, C) {
    let mut i0 = p0.matches.iter().rev().filter(|m| m.played).peekable();
    let mut i1 = p1.matches.iter().rev().filter(|m| m.played).peekable();
    loop {
        match (i0.peek(), i1.peek()) {
            (Some(a), Some(b)) if a.color == b.color => {
                i0.next();
                i1.next();
            }
            (a, b) => {
                return (
                    a.map_or(C::None, |m| m.color),
                    b.map_or(C::None, |m| m.color),
                );
            }
        }
    }
}

/// Colour for `player` against `opponent` from preferences alone, or
/// `C::None` if they do not decide it.
pub(crate) fn choose_player_neutral_color(player: &Player, opponent: &Player) -> C {
    if color_preferences_compatible(player.color_preference, opponent.color_preference) {
        if player.color_preference != C::None {
            player.color_preference
        } else if opponent.color_preference != C::None {
            opponent.color_preference.invert()
        } else {
            C::None
        }
    } else if player.absolute_color_preference()
        && (player.color_imbalance > opponent.color_imbalance
            || !opponent.absolute_color_preference())
    {
        player.color_preference
    } else if opponent.absolute_color_preference()
        && (opponent.color_imbalance > player.color_imbalance
            || !player.absolute_color_preference())
    {
        opponent.color_preference.invert()
    } else if player.strong_color_preference && !opponent.strong_color_preference {
        player.color_preference
    } else if opponent.strong_color_preference && !player.strong_color_preference {
        opponent.color_preference.invert()
    } else {
        let (pc, oc) = find_first_color_difference(player, opponent);
        if pc != C::None && oc != C::None {
            oc
        } else {
            C::None
        }
    }
}
