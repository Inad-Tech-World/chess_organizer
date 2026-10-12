//! Bracket-by-bracket Swiss pairing shared by the Dubov and Lim systems.
//!
//! Score groups are paired from the top. For each bracket (its own players
//! plus anyone floated down to it) a maximum-weight matching over the bracket
//! and every lower player decides which pairs are made inside the bracket.
//! The weight of an edge packs, in strict priority order:
//!
//! 1. compatibility (absolute criteria; the whole round must stay complete),
//! 2. the number of pairs inside the bracket,
//! 3. which players float (the bracket's lowest-placed players float),
//! 4. colour preferences that can both be met,
//! 5. the system's own pairing order inside the bracket
//!    (Lim: S1 against S2 in order; Dubov: White-seekers by ascending ARO
//!    against Black-seekers by descending rating).
//!
//! Absolute criteria are those of C.04.1: no rematch, no forbidden pair, no
//! two players with the same absolute colour preference, at most one
//! pairing-allocated bye per player.

// Index loops mirror the reference algorithm's vertex indices.
#![allow(clippy::needless_range_loop)]

use crate::RoundPairing;
use crate::matching::{Computer, Wide, max_weight_matching};
use crate::swiss_model::{
    C, Player, Tournament, choose_player_neutral_color, color_preferences_compatible,
};
use co_core::pairing_view::PairingInput;
use co_core::{Error, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Style {
    Dubov,
    Lim,
}

const COMPLETE: u128 = 1 << 90;
const IN_BRACKET: u128 = 1 << 70;
const FLOAT_SELECTION: u128 = 1 << 45;
const COLOUR: u128 = 1 << 30;

fn wide(template: &Wide, v: u128) -> Wide {
    let mut w = template.zero_like();
    let limbs = w.limbs_mut();
    limbs[0] = v as u64;
    limbs[1] = (v >> 64) as u64;
    w
}

/// The pairing context: who may meet whom.
pub(crate) struct Field<'a> {
    pub t: &'a Tournament,
    pub ratings: Vec<u16>,
    forbidden: Vec<HashSet<usize>>,
}

impl<'a> Field<'a> {
    pub(crate) fn new(t: &'a Tournament, input: &PairingInput) -> Self {
        let mut forbidden = t.forbidden.clone();
        for p in &t.players {
            for m in p.matches.iter().take(t.played_rounds) {
                if m.played {
                    forbidden[p.id].insert(m.opponent);
                }
            }
        }
        Self {
            t,
            ratings: input.players.iter().map(|p| p.rating).collect(),
            forbidden,
        }
    }

    pub(crate) fn player(&self, i: usize) -> &Player {
        &self.t.players[i]
    }

    /// Players to pair this round.
    pub(crate) fn available(&self) -> Vec<usize> {
        self.t
            .players
            .iter()
            .filter(|p| p.valid && p.matches.len() <= self.t.played_rounds)
            .map(|p| p.id)
            .collect()
    }

    pub(crate) fn compatible(&self, a: usize, b: usize) -> bool {
        let (pa, pb) = (self.player(a), self.player(b));
        a != b
            && !self.forbidden[a].contains(&b)
            && !(pa.absolute_color_preference()
                && pb.absolute_color_preference()
                && pa.color_preference == pb.color_preference)
    }

    pub(crate) fn score(&self, i: usize) -> i64 {
        self.t.score_acc(self.player(i), 0)
    }

    /// Average rating of the opponents actually played (0 if none).
    pub(crate) fn aro(&self, i: usize) -> u32 {
        let p = self.player(i);
        let rs: Vec<u32> = p
            .matches
            .iter()
            .take(self.t.played_rounds)
            .filter(|m| m.played)
            .map(|m| u32::from(self.ratings[m.opponent]))
            .filter(|&r| r > 0)
            .collect();
        if rs.is_empty() {
            0
        } else {
            rs.iter().sum::<u32>() / rs.len() as u32
        }
    }

    /// Whether `players` can all be paired with each other.
    pub(crate) fn perfect(&self, players: &[usize]) -> bool {
        if players.len() % 2 == 1 {
            return false;
        }
        let mut edges = Vec::new();
        for (i, &a) in players.iter().enumerate() {
            for (j, &b) in players.iter().enumerate().skip(i + 1) {
                if self.compatible(a, b) {
                    edges.push((i, j, 1));
                }
            }
        }
        max_weight_matching(players.len(), &edges)
            .iter()
            .all(Option::is_some)
    }

    /// The pairing-allocated bye: the lowest player in `order` who has not
    /// had one and whose absence leaves the rest pairable.
    pub(crate) fn choose_bye(&self, order: &[usize]) -> Result<Option<usize>> {
        if order.len().is_multiple_of(2) {
            return Ok(None);
        }
        for (k, &cand) in order.iter().enumerate().rev() {
            if !self.t.eligible_for_bye(self.player(cand)) {
                continue;
            }
            let rest: Vec<usize> = order
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != k)
                .map(|(_, &p)| p)
                .collect();
            if self.perfect(&rest) {
                return Ok(Some(cand));
            }
        }
        Err(Error::Invalid(
            "no player can receive the pairing-allocated bye while the others stay pairable".into(),
        ))
    }

    /// `(white, black)` for a pair: preferences first, then the higher-placed
    /// player's rank parity against the initial colour.
    pub(crate) fn colours(&self, a: usize, b: usize) -> (usize, usize) {
        let (pa, pb) = (self.player(a), self.player(b));
        let c = match choose_player_neutral_color(pa, pb) {
            C::None => {
                let higher = if pa.rank_index < pb.rank_index {
                    pa
                } else {
                    pb
                };
                let higher_colour = if higher.rank_index & 1 == 1 {
                    self.t.initial_color.invert()
                } else {
                    self.t.initial_color
                };
                if higher.id == pa.id {
                    higher_colour
                } else {
                    higher_colour.invert()
                }
            }
            c => c,
        };
        if c == C::Black { (b, a) } else { (a, b) }
    }
}

/// Pairs one round in the given style; returns `(white, black)` index pairs
/// with the bye as `(i, i)`.
pub(crate) fn pair(t: &Tournament, input: &PairingInput, style: Style) -> Result<RoundPairing> {
    let f = Field::new(t, input);
    let mut order = f.available();
    // Score, then rank (Lim) or rating then rank (Dubov), highest first.
    order.sort_by(|&a, &b| {
        f.score(b).cmp(&f.score(a)).then_with(|| match style {
            Style::Lim => f.player(a).rank_index.cmp(&f.player(b).rank_index),
            Style::Dubov => f.ratings[b]
                .cmp(&f.ratings[a])
                .then_with(|| f.player(a).rank_index.cmp(&f.player(b).rank_index)),
        })
    });
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    if let Some(bye) = f.choose_bye(&order)? {
        order.retain(|&p| p != bye);
        pairs.push((bye, bye));
    }

    // Score groups, top first.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &p in &order {
        match groups.last_mut() {
            Some(g) if f.score(g[0]) == f.score(p) => g.push(p),
            _ => groups.push(vec![p]),
        }
    }

    let max = {
        let mut m = Wide::from_u64(1);
        m.shift_grow(92);
        m
    };
    let mut floaters: Vec<usize> = Vec::new();
    for gi in 0..groups.len() {
        let mut bracket = std::mem::take(&mut floaters);
        bracket.extend(groups[gi].iter().copied());
        let lower: Vec<usize> = groups[gi + 1..].iter().flatten().copied().collect();
        let prefs = in_bracket_preferences(&f, &bracket, style);
        let vertices: Vec<usize> = bracket.iter().chain(&lower).copied().collect();
        let m = bracket.len();
        let mut mc = Computer::new(vertices.len(), &max);
        for _ in 0..vertices.len() {
            mc.add_vertex();
        }
        for u in 0..vertices.len() {
            for v in u + 1..vertices.len() {
                let (a, b) = (vertices[u], vertices[v]);
                if !f.compatible(a, b) {
                    continue;
                }
                let mut w = COMPLETE;
                if v < m {
                    w += IN_BRACKET;
                    w += FLOAT_SELECTION * (2 * m - u - v) as u128;
                    if color_preferences_compatible(
                        f.player(a).color_preference,
                        f.player(b).color_preference,
                    ) {
                        w += COLOUR;
                    }
                    w += prefs[u][v];
                }
                mc.set_edge_weight(u, v, &wide(&max, w));
            }
        }
        mc.compute_matching();
        let mate = mc.get_matching();
        if mate.iter().enumerate().any(|(i, &j)| i == j) {
            return Err(Error::Invalid(
                "no valid pairing exists: the players cannot all be paired without breaking an absolute criterion".into(),
            ));
        }
        for u in 0..m {
            let v = mate[u];
            if v < m {
                if u < v {
                    pairs.push(f.colours(bracket[u], bracket[v]));
                }
            } else {
                floaters.push(bracket[u]);
            }
        }
    }
    if !floaters.is_empty() {
        return Err(Error::Invalid(
            "no valid pairing exists for the lowest score group".into(),
        ));
    }
    Ok(t.to_round(input, pairs))
}

/// The system's pairing-order preference for each pair inside a bracket
/// (higher is better), indexed by position in `bracket`.
fn in_bracket_preferences(f: &Field, bracket: &[usize], style: Style) -> Vec<Vec<u128>> {
    let m = bracket.len();
    let mut prefs = vec![vec![0u128; m]; m];
    let lim = |prefs: &mut Vec<Vec<u128>>| {
        let half = m / 2;
        for i in 0..half {
            for j in half..m {
                let d = (j - half).abs_diff(i);
                prefs[i][j] = (m - d.min(m - 1)) as u128;
                prefs[j][i] = prefs[i][j];
            }
        }
    };
    if style == Style::Lim || f.t.played_rounds == 0 {
        lim(&mut prefs);
        return prefs;
    }
    // Dubov: split into White- and Black-seekers; players without a
    // preference join the smaller side.
    let mut whites: Vec<usize> = Vec::new();
    let mut blacks: Vec<usize> = Vec::new();
    let mut neutral: Vec<usize> = Vec::new();
    for (i, &p) in bracket.iter().enumerate() {
        match f.player(p).color_preference {
            C::White => whites.push(i),
            C::Black => blacks.push(i),
            C::None => neutral.push(i),
        }
    }
    for i in neutral {
        if whites.len() <= blacks.len() {
            whites.push(i)
        } else {
            blacks.push(i)
        }
    }
    whites.sort_by(|&a, &b| {
        let (pa, pb) = (bracket[a], bracket[b]);
        f.aro(pa)
            .cmp(&f.aro(pb))
            .then_with(|| f.ratings[pb].cmp(&f.ratings[pa]))
            .then_with(|| f.player(pa).rank_index.cmp(&f.player(pb).rank_index))
    });
    blacks.sort_by(|&a, &b| {
        let (pa, pb) = (bracket[a], bracket[b]);
        f.ratings[pb]
            .cmp(&f.ratings[pa])
            .then_with(|| f.player(pa).rank_index.cmp(&f.player(pb).rank_index))
    });
    for (wi, &w) in whites.iter().enumerate() {
        for (bi, &b) in blacks.iter().enumerate() {
            let d = wi.abs_diff(bi);
            prefs[w][b] = (m - d.min(m - 1)) as u128;
            prefs[b][w] = prefs[w][b];
        }
    }
    prefs
}
