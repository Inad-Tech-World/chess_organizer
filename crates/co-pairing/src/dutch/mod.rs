//! FIDE (Dutch) System, FIDE Handbook C.04.3 (2025 edition).
//!
//! A port of bbpPairings' Dutch engine (Apache-2.0, Jeremy Bierema; see
//! NOTICE). Players are processed bracket by bracket. Every compatible pair
//! is an edge whose weight packs the C.04.3 criteria into a wide integer in
//! strict priority order: completion and bye eligibility, pairs and scores in
//! this bracket and the next, the PAB assignee's unplayed games, colour
//! preferences, then repeated floaters. Maximum-weight matchings then decide
//! the moved-down players, their opponents, the exchanges, and finally each
//! pair, as the transposition and exchange order in C.04.3 requires.

// Index loops mirror the reference algorithm's vertex indices.
#![allow(clippy::needless_range_loop)]

use crate::matching::{Computer, Wide};
use crate::swiss_model::{
    C, Player, Tournament, choose_player_neutral_color, color_preferences_compatible,
};
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{Error, PairingSystem, Result};
use std::collections::HashMap;

pub struct DutchEngine;

impl PairingEngine for DutchEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Dutch
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        let t = Tournament::from_input(input)?;
        let pairs = compute_matching(&t)?;
        Ok(t.to_round(input, pairs))
    }
}

fn bits_to_represent(v: usize) -> usize {
    (usize::BITS - v.leading_zeros()) as usize
}

/// Absolute criteria C1 and C3 (and forbidden pairs).
fn compatible(p0: &Player, p1: &Player, t: &Tournament) -> bool {
    let top_score_threshold =
        (t.played_rounds as i64) * t.points_for_win.max(t.points_for_draw) / 2;
    let last_round = t.expected_rounds > 0 && t.played_rounds + 1 >= t.expected_rounds;
    !t.forbidden[p0.id].contains(&p1.id)
        && (!p0.absolute_color_preference()
            || !p1.absolute_color_preference()
            || p0.color_preference != p1.color_preference
            || (last_round
                && (t.score_acc(p0, 0) > top_score_threshold
                    || t.score_acc(p1, 0) > top_score_threshold)))
}

fn matching_is_complete(matching: &[usize], t: &Tournament, sorted: &[usize]) -> bool {
    let mut unmatched = false;
    for (vertex, &m) in matching.iter().enumerate() {
        if m == vertex {
            if unmatched || !t.eligible_for_bye(&t.players[sorted[m]]) {
                return false;
            }
            unmatched = true;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Float {
    Down,
    Up,
    None,
}

fn get_float(p: &Player, rounds_back: usize, t: &Tournament) -> Float {
    let m = &p.matches[t.played_rounds - rounds_back];
    if !m.played {
        return if t.get_points(p, m) > t.points_for_loss {
            Float::Down
        } else {
            Float::None
        };
    }
    let own = t.score_acc(p, rounds_back);
    let opp = t.score_acc(&t.players[m.opponent], rounds_back);
    if own > opp {
        Float::Down
    } else if own < opp {
        Float::Up
    } else {
        Float::None
    }
}

/// The edge weight under construction. With `max`, the weight being built is
/// the upper bound and grows as needed; otherwise shifts keep its width.
struct Ew<'a> {
    w: &'a mut Wide,
    max: bool,
}

impl Ew<'_> {
    fn shift(&mut self, n: usize) {
        if self.max {
            self.w.shift_grow(n);
        } else {
            self.w.shl(n);
        }
    }

    fn or(&mut self, cond: bool) {
        if !self.max && cond {
            self.w.or_small(1);
        }
    }
}

fn insert_color_bits(
    ew: &mut Ew,
    player: &Player,
    opponent: &Player,
    in_current: bool,
    bits: usize,
) {
    let mask = !ew.max && in_current;
    ew.shift(bits);
    ew.or(mask
        && (!player.absolute_color_imbalance()
            || !opponent.absolute_color_imbalance()
            || player.color_preference != opponent.color_preference));

    ew.shift(bits);
    ew.or(mask
        && (!player.absolute_color_preference()
            || !opponent.absolute_color_preference()
            || player.color_preference != opponent.color_preference
            || if player.color_imbalance == opponent.color_imbalance {
                player.repeated_color == C::None || player.repeated_color != opponent.repeated_color
            } else {
                let p = if player.color_imbalance > opponent.color_imbalance {
                    opponent
                } else {
                    player
                };
                p.repeated_color != player.color_preference.invert()
            }));

    ew.shift(bits);
    ew.or(mask && color_preferences_compatible(player.color_preference, opponent.color_preference));

    ew.shift(bits);
    ew.or(mask
        && ((!player.strong_color_preference && !player.absolute_color_preference())
            || (!opponent.strong_color_preference && !opponent.absolute_color_preference())
            || (player.absolute_color_preference() && opponent.absolute_color_preference())
            || player.color_preference != opponent.color_preference));
}

fn is_bye_candidate(p: &Player, t: &Tournament, bye_assignee_score: i64) -> bool {
    t.eligible_for_bye(p) && t.score_acc(p, 0) <= bye_assignee_score
}

struct Ctx<'a> {
    t: &'a Tournament,
    score_group_size_bits: usize,
    score_groups_shift: usize,
    score_group_shifts: HashMap<i64, usize>,
    unplayed_game_ranks: HashMap<usize, usize>,
}

impl Ctx<'_> {
    fn shift_of(&self, score: i64) -> usize {
        self.score_group_shifts[&score]
    }

    /// The basic edge weight between two players; with `max_out`, computes
    /// the upper bound into it instead.
    #[allow(clippy::too_many_arguments)]
    fn edge_weight(
        &self,
        higher: &Player,
        lower: &Player,
        lower_in_current: bool,
        lower_in_next: bool,
        bye_assignee_score: i64,
        single_downfloater_is_bye_assignee: bool,
        max_edge_weight: &mut Wide,
        max: bool,
    ) -> Wide {
        let t = self.t;
        let mut own = max_edge_weight.clone();
        let w: &mut Wide = if max { max_edge_weight } else { &mut own };
        w.and_small(0);
        let mut ew = Ew { w, max };

        if !max && !compatible(higher, lower, t) {
            return ew.w.clone();
        }

        // Completion and bye eligibility.
        if max {
            ew.w.or_small(2);
        } else {
            let v = 1
                + u64::from(!is_bye_candidate(higher, t, bye_assignee_score))
                + u64::from(!is_bye_candidate(lower, t, bye_assignee_score));
            ew.w.or_small(v);
        }

        let sgsb = self.score_group_size_bits;
        let sgs = self.score_groups_shift;

        // Maximize the number of pairs in the current bracket.
        ew.shift(sgsb);
        ew.or(lower_in_current);

        // Maximize the scores paired in the current bracket.
        ew.shift(sgs);
        if !max && lower_in_current {
            ew.w.or_bit(self.shift_of(t.score_acc(higher, 0)));
        }

        // Maximize the number of pairs in the next bracket.
        ew.shift(sgsb);
        ew.or(lower_in_next);

        // Maximize the scores paired in the next bracket.
        ew.shift(sgs);
        if !max && lower_in_next {
            ew.w.or_bit(self.shift_of(t.score_acc(higher, 0)));
        }

        // Minimize the unplayed games of the bye assignee.
        ew.shift(sgsb);
        ew.shift(sgsb);
        if !max && single_downfloater_is_bye_assignee {
            if t.score_acc(higher, 0) == bye_assignee_score {
                ew.w.or_small(self.unplayed_game_ranks[&higher.played_games] as u64);
            }
            if t.score_acc(lower, 0) == bye_assignee_score {
                ew.w.add_small(self.unplayed_game_ranks[&lower.played_games] as u64);
            }
        }

        // Colour preferences.
        insert_color_bits(&mut ew, lower, higher, lower_in_current, sgsb);

        let hs = t.score_acc(higher, 0);
        let ls = t.score_acc(lower, 0);
        if t.played_rounds > 0 {
            // Minimize downfloaters repeated from the previous round.
            ew.shift(sgsb);
            if !max && lower_in_current {
                ew.w.or_small(u64::from(get_float(lower, 1, t) == Float::Down));
                ew.w.add_small(u64::from(
                    hs <= ls && get_float(higher, 1, t) == Float::Down,
                ));
            }
            // Minimize upfloaters repeated from the previous round.
            ew.shift(sgsb);
            if !max && lower_in_current {
                ew.w.or_small(u64::from(!(hs > ls && get_float(lower, 1, t) == Float::Up)));
            }
        }
        if t.played_rounds > 1 {
            ew.shift(sgsb);
            if !max && lower_in_current {
                ew.w.or_small(u64::from(get_float(lower, 2, t) == Float::Down));
                ew.w.add_small(u64::from(
                    hs <= ls && get_float(higher, 2, t) == Float::Down,
                ));
            }
            ew.shift(sgsb);
            if !max && lower_in_current {
                ew.w.or_small(u64::from(!(hs > ls && get_float(lower, 2, t) == Float::Up)));
            }
        }
        if t.played_rounds > 0 {
            // Minimize the scores of downfloaters repeated from the previous round.
            ew.shift(sgs);
            if !max && lower_in_current {
                if get_float(lower, 1, t) == Float::Down {
                    ew.w.add_bit(self.shift_of(ls));
                }
                if get_float(higher, 1, t) == Float::Down {
                    ew.w.add_bit(self.shift_of(hs));
                }
            }
            // Minimize the scores of opponents of upfloaters repeated from the previous round.
            ew.shift(sgs);
            if !max && lower_in_current && !(get_float(lower, 1, t) == Float::Up && hs > ls) {
                ew.w.or_bit(self.shift_of(hs));
            }
        }
        if t.played_rounds > 1 {
            ew.shift(sgs);
            if !max && lower_in_current {
                if get_float(lower, 2, t) == Float::Down {
                    ew.w.add_bit(self.shift_of(ls));
                }
                if get_float(higher, 2, t) == Float::Down {
                    ew.w.add_bit(self.shift_of(hs));
                }
            }
            ew.shift(sgs);
            if !max && lower_in_current && !(get_float(lower, 2, t) == Float::Up && hs > ls) {
                ew.w.or_bit(self.shift_of(hs));
            }
        }

        // Room for the homogeneous/heterogeneous ordering requirements.
        ew.shift(sgsb);
        ew.shift(sgsb);
        ew.shift(sgsb);
        ew.shift(1);

        if max {
            // Room for the matching's two extra bits, then all ones.
            ew.w.shift_grow(2);
            ew.w.shr(1);
            ew.w.sub_small(1);
        }
        ew.w.clone()
    }
}

/// Colour for `player` against `opponent`.
fn choose_player_color(player: &Player, opponent: &Player, t: &Tournament) -> C {
    let result = choose_player_neutral_color(player, opponent);
    if result != C::None {
        return result;
    }
    let lower = t.accelerated_lower(player, opponent);
    if player.color_preference == C::None {
        if lower {
            if opponent.rank_index & 1 == 1 {
                t.initial_color
            } else {
                t.initial_color.invert()
            }
        } else if player.rank_index & 1 == 1 {
            t.initial_color.invert()
        } else {
            t.initial_color
        }
    } else if lower {
        opponent.color_preference.invert()
    } else {
        player.color_preference
    }
}

/// Pairs the round. Returns `(white, black)` index pairs; the PAB is
/// `(p, p)`.
pub(crate) fn compute_matching(t: &Tournament) -> Result<Vec<(usize, usize)>> {
    // Players to pair, by accelerated score then rank.
    let mut forbidden = t.forbidden.clone();
    let mut sorted: Vec<usize> = Vec::new();
    for p in &t.players {
        if p.valid {
            if p.matches.len() <= t.played_rounds {
                sorted.push(p.id);
            }
            for m in &p.matches {
                if m.played {
                    forbidden[p.id].insert(m.opponent);
                }
            }
        }
    }
    if sorted.is_empty() {
        return Ok(Vec::new());
    }
    let t_owned;
    let t = if forbidden != t.forbidden {
        let mut c = t.clone();
        c.forbidden = forbidden;
        t_owned = c;
        &t_owned
    } else {
        t
    };
    sorted.sort_by(|&a, &b| {
        let (pa, pb) = (&t.players[a], &t.players[b]);
        if t.accelerated_lower(pb, pa) {
            std::cmp::Ordering::Less
        } else if t.accelerated_lower(pa, pb) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    let score = |i: usize| t.score_acc(&t.players[sorted[i]], 0);

    // Bits needed per score group.
    let mut score_groups_shift = 0usize;
    let mut score_group_shifts: HashMap<i64, usize> = HashMap::new();
    let mut max_group_size = 0usize;
    let mut repeated = 0usize;
    for i in (0..sorted.len()).rev() {
        repeated += 1;
        let current = score(i);
        if i == 0 || current < score(i - 1) {
            let new_bits = bits_to_represent(repeated);
            score_group_shifts.insert(current, score_groups_shift);
            max_group_size = max_group_size.max(repeated);
            repeated = 0;
            score_groups_shift += new_bits;
        }
    }
    let score_group_size_bits = bits_to_represent(max_group_size);

    let mut ctx = Ctx {
        t,
        score_group_size_bits,
        score_groups_shift,
        score_group_shifts,
        unplayed_game_ranks: HashMap::new(),
    };

    let front = &t.players[sorted[0]];
    let mut max_edge_weight = Wide::from_u64(0);
    ctx.edge_weight(
        front,
        front,
        true,
        false,
        0,
        false,
        &mut max_edge_weight,
        true,
    );

    let n = sorted.len();
    if std::env::var_os("CO_DEBUG_WIDTH").is_some() {
        eprintln!("players {n} weight limbs {}", max_edge_weight.width());
    }
    let mut mc = Computer::new(n, &max_edge_weight);
    for _ in 0..n {
        mc.add_vertex();
    }

    let top = score(0);
    for pi in 0..n {
        let player = &t.players[sorted[pi]];
        let ps = t.score_acc(player, 0);
        for oi in 0..pi {
            let opponent = &t.players[sorted[oi]];
            if n & 1 == 1 {
                let mut w = max_edge_weight.zero_like();
                if compatible(player, opponent, t) {
                    w.or_small(
                        1 + u64::from(!t.eligible_for_bye(player))
                            + u64::from(!t.eligible_for_bye(opponent)),
                    );
                    w.shl(ctx.score_groups_shift);
                    w.or_small((ctx.shift_of(ps) + ctx.shift_of(t.score_acc(opponent, 0))) as u64);
                    w.shl(ctx.score_group_size_bits);
                    w.or_small(u64::from(ps >= top));
                }
                mc.set_edge_weight(pi, oi, &w);
            } else {
                let w = ctx.edge_weight(
                    opponent,
                    player,
                    false,
                    false,
                    0,
                    false,
                    &mut max_edge_weight,
                    false,
                );
                mc.set_edge_weight(pi, oi, &w);
            }
        }
    }

    // Is a pairing possible? Who gets the bye, and does C9 apply first?
    let mut bye_assignee_score = 0i64;
    let mut single_downfloater_is_bye_assignee;
    {
        mc.compute_matching();
        let matching = mc.get_matching();
        if !matching_is_complete(&matching, t, &sorted) {
            return Err(Error::Invalid(
                "No valid pairing exists: the players cannot all be paired without breaking an \
                 absolute criterion (repeat games, forbidden pairs, colours or byes)."
                    .into(),
            ));
        }
        if n & 1 == 1 {
            for (i, &m) in matching.iter().enumerate() {
                if m == i {
                    bye_assignee_score = score(i);
                    break;
                }
            }
            if bye_assignee_score >= top {
                single_downfloater_is_bye_assignee = true;
                for i in 0..n {
                    if score(i) < top {
                        break;
                    }
                    if score(matching[i]) < top {
                        single_downfloater_is_bye_assignee = false;
                        break;
                    }
                }
            } else {
                single_downfloater_is_bye_assignee = false;
            }
            let mut counts: Vec<usize> = (0..n)
                .filter(|&i| score(i) == bye_assignee_score)
                .map(|i| t.players[sorted[i]].played_games)
                .collect();
            counts.sort_unstable_by(|a, b| b.cmp(a));
            for (rank, c) in counts.into_iter().enumerate() {
                ctx.unplayed_game_ranks.insert(c, rank);
            }
            for pi in 0..n {
                let player = &t.players[sorted[pi]];
                for oi in 0..pi {
                    let opponent = &t.players[sorted[oi]];
                    let w = ctx.edge_weight(
                        opponent,
                        player,
                        false,
                        false,
                        bye_assignee_score,
                        single_downfloater_is_bye_assignee,
                        &mut max_edge_weight,
                        false,
                    );
                    mc.set_edge_weight(pi, oi, &w);
                }
            }
        } else {
            single_downfloater_is_bye_assignee = false;
        }
    }

    let mut matching_by_id: Vec<Option<usize>> = vec![None; t.players.len()];
    let mut players_by_index: Vec<usize> = Vec::new(); // player ids
    let mut vertex_indices: Vec<usize> = Vec::new();
    let mut next_group = 0usize; // index into sorted
    while next_group < n && score(next_group) >= top {
        players_by_index.push(sorted[next_group]);
        vertex_indices.push(vertex_indices.len());
        next_group += 1;
    }

    let mut matched = vec![false; n];
    let mut score_group_begin = 0usize;
    let mut score_group_begin_vertex = 0usize;
    let pscore = |id: usize| t.score_acc(&t.players[id], 0);

    while players_by_index.len() > 1 || next_group < n {
        let next_score_group_begin = players_by_index.len();
        let next_score_group_begin_vertex =
            score_group_begin_vertex + (next_score_group_begin - score_group_begin);
        let score_group_iterator = next_group;
        while next_group < n && score(next_group) >= score(score_group_iterator) {
            players_by_index.push(sorted[next_group]);
            let last = *vertex_indices.last().expect("non-empty");
            vertex_indices.push(last + 1);
            next_group += 1;
        }

        // Base edge weights within this bracket and the next.
        let size = players_by_index.len();
        let mut base: Vec<Vec<Wide>> = vec![Vec::new(); size];
        for larger in score_group_begin..size {
            for smaller in 0..larger {
                let w = ctx.edge_weight(
                    &t.players[players_by_index[smaller]],
                    &t.players[players_by_index[larger]],
                    larger < next_score_group_begin,
                    larger >= next_score_group_begin,
                    bye_assignee_score,
                    single_downfloater_is_bye_assignee,
                    &mut max_edge_weight,
                    false,
                );
                base[larger].push(w);
            }
        }
        for (oi, row) in base.iter().enumerate() {
            let ov = vertex_indices[oi];
            for (pi, w) in row.iter().enumerate() {
                mc.set_edge_weight(ov, vertex_indices[pi], w);
            }
        }

        let sgsb = ctx.score_group_size_bits;
        let edge_weight_computer = |base: &Vec<Vec<Wide>>,
                                    smaller: usize,
                                    larger: usize,
                                    smaller_rem: usize,
                                    rem_pairs: usize|
         -> Wide {
            let mut result = base[larger][smaller].clone();
            if !result.is_zero() {
                let mut addend = result.zero_like();
                addend.or_small(u64::from(smaller_rem < rem_pairs));
                addend.shl(sgsb);
                addend.shl(sgsb);
                addend.sub_small(smaller_rem as u64);
                addend.shl(1);
                result.add(addend.limbs());
            }
            result
        };

        mc.compute_matching();
        let mut stable = mc.get_matching();
        let in_bracket = |v: usize, sm: &Vec<usize>| {
            sm[v] >= score_group_begin_vertex && sm[v] < next_score_group_begin_vertex
        };

        // Choose the moved-down players to pair in this bracket.
        let mut moved_down_score = 0i64;
        let mut remaining_mdp = 0usize;
        let mut remaining_matched_mdp = 0usize;
        for pi in 0..score_group_begin {
            if pi == 0 || pscore(players_by_index[pi]) < moved_down_score {
                moved_down_score = pscore(players_by_index[pi]);
                remaining_matched_mdp = 0;
                remaining_mdp = 0;
                let mut mi = pi;
                while mi < size && pscore(players_by_index[mi]) >= moved_down_score {
                    remaining_mdp += 1;
                    if in_bracket(vertex_indices[mi], &stable) {
                        remaining_matched_mdp += 1;
                    }
                    mi += 1;
                }
            }
            if remaining_matched_mdp == 0 {
                continue;
            }
            let pv = vertex_indices[pi];
            if remaining_mdp <= remaining_matched_mdp {
                matched[pv] = true;
                continue;
            }
            remaining_mdp -= 1;
            if !in_bracket(pv, &stable) {
                for oi in score_group_begin..next_score_group_begin {
                    let mut w = base[oi][pi].clone();
                    if !w.is_zero() {
                        w.or_small(1);
                        mc.set_edge_weight(pv, vertex_indices[oi], &w);
                    }
                }
                mc.compute_matching();
                stable = mc.get_matching();
            }
            if in_bracket(pv, &stable) {
                matched[pv] = true;
                remaining_matched_mdp -= 1;
                for oi in score_group_begin..next_score_group_begin {
                    let mut w = base[oi][pi].clone();
                    if !w.is_zero() {
                        w.or_small((next_score_group_begin - score_group_begin) as u64);
                        w.inc();
                        mc.set_edge_weight(pv, vertex_indices[oi], &w);
                    }
                }
            }
        }

        // Choose the opponents of the moved-down players.
        for pi in 0..score_group_begin {
            let pv = vertex_indices[pi];
            if !matched[pv] {
                continue;
            }
            let mut addend = max_edge_weight.small_like(size as u64);
            let mut oi = next_score_group_begin;
            while oi > score_group_begin {
                oi -= 1;
                let ov = vertex_indices[oi];
                if matched[ov] {
                    continue;
                }
                let mut w = base[oi][pi].clone();
                if !w.is_zero() {
                    w.add(addend.limbs());
                    mc.set_edge_weight(pv, ov, &w);
                    addend.inc();
                }
            }
            mc.compute_matching();
            stable = mc.get_matching();
            let mv = stable[pv];
            matched[mv] = true;
            finalize_pair(pv, mv, &mut mc, &max_edge_weight);
        }

        // The remainder: residents still to be paired in this bracket.
        let mut remainder: Vec<usize> = Vec::new();
        let mut remainder_pairs = 0usize;
        for pi in score_group_begin..next_score_group_begin {
            let pv = vertex_indices[pi];
            if stable[pv] < score_group_begin_vertex {
                continue;
            }
            remainder.push(pi);
            if stable[pv] < pv {
                remainder_pairs += 1;
            }
        }
        let first_group_end = remainder_pairs; // index into remainder

        // Minimize exchanged players and the differences of exchanged BSNs.
        for &oi in &remainder {
            let ov = vertex_indices[oi];
            for (prem, &pi) in remainder.iter().enumerate() {
                if oi <= pi {
                    break;
                }
                let w = edge_weight_computer(&base, pi, oi, prem, remainder_pairs);
                mc.set_edge_weight(vertex_indices[pi], ov, &w);
            }
        }
        mc.compute_matching();
        stable = mc.get_matching();

        let is_exchanged_upper =
            |v: usize, sm: &Vec<usize>| sm[v] <= v || sm[v] >= next_score_group_begin_vertex;
        let is_s1_partner =
            |v: usize, sm: &Vec<usize>| sm[v] > v && sm[v] < next_score_group_begin_vertex;

        let exchange_count = remainder[..first_group_end]
            .iter()
            .filter(|&&pi| is_exchanged_upper(vertex_indices[pi], &stable))
            .count();

        // Exchange the lowest possible players of the upper group (S1).
        let mut exchanges_remaining = exchange_count;
        let mut prem = remainder_pairs;
        let mut pidx = first_group_end;
        while pidx > 0 && exchanges_remaining > 0 {
            pidx -= 1;
            prem -= 1;
            let pi = remainder[pidx];
            let pv = vertex_indices[pi];
            if is_s1_partner(pv, &stable) {
                for &oi in &remainder[pidx + 1..] {
                    let mut w = edge_weight_computer(&base, pi, oi, prem, remainder_pairs);
                    if !w.is_zero() {
                        w.sub_small(1);
                        mc.set_edge_weight(pv, vertex_indices[oi], &w);
                    }
                }
                mc.compute_matching();
                stable = mc.get_matching();
            }
            let exchange = is_exchanged_upper(pv, &stable);
            if exchange {
                exchanges_remaining -= 1;
            }
            for &oi in &remainder[pidx + 1..] {
                if exchange {
                    base[oi][pi].and_small(0);
                }
                let w = edge_weight_computer(&base, pi, oi, prem, remainder_pairs);
                mc.set_edge_weight(pv, vertex_indices[oi], &w);
            }
        }

        // Exchange the highest possible players of the lower group (S2).
        exchanges_remaining = exchange_count;
        let mut rem_index = remainder_pairs;
        let mut pidx = first_group_end;
        while pidx < remainder.len() && exchanges_remaining > 1 {
            let pi = remainder[pidx];
            let pv = vertex_indices[pi];
            let already = is_s1_partner(pv, &stable);
            if !already {
                for &oi in &remainder[pidx + 1..] {
                    let mut w = edge_weight_computer(&base, pi, oi, rem_index, remainder_pairs);
                    if !w.is_zero() {
                        w.add_small(1);
                        mc.set_edge_weight(pv, vertex_indices[oi], &w);
                    }
                }
                mc.compute_matching();
                stable = mc.get_matching();
            }
            let exchange = is_s1_partner(pv, &stable);
            if exchange {
                exchanges_remaining -= 1;
                for &oi in &remainder[..pidx] {
                    base[pi][oi].and_small(0);
                    let w = base[pi][oi].clone();
                    mc.set_edge_weight(pv, vertex_indices[oi], &w);
                }
                for oi in next_score_group_begin..size {
                    base[oi][pi].and_small(0);
                    let w = base[oi][pi].clone();
                    mc.set_edge_weight(pv, vertex_indices[oi], &w);
                }
            }
            if !already {
                for &oi in &remainder[pidx + 1..] {
                    let w = edge_weight_computer(&base, pi, oi, rem_index, remainder_pairs);
                    mc.set_edge_weight(pv, vertex_indices[oi], &w);
                }
            }
            rem_index += 1;
            pidx += 1;
        }

        // Finalize which players are exchanged, and reset the extra bits.
        for a in 0..remainder.len() {
            let pi = remainder[a];
            let pv = vertex_indices[pi];
            for &oi in &remainder[a + 1..] {
                let ov = vertex_indices[oi];
                if is_exchanged_upper(pv, &stable) || is_s1_partner(ov, &stable) {
                    base[oi][pi].and_small(0);
                }
                let w = base[oi][pi].clone();
                mc.set_edge_weight(pv, ov, &w);
            }
        }

        // Pair each player of the first group, preferring higher opponents.
        for &pi in &remainder {
            let pv = vertex_indices[pi];
            if is_s1_partner(pv, &stable) {
                let mut addend = 0u64;
                for &oi in remainder.iter().rev() {
                    let ov = vertex_indices[oi];
                    if oi <= pi || matched[ov] {
                        continue;
                    }
                    let mut w = base[oi][pi].clone();
                    if !w.is_zero() {
                        w.add_small(addend);
                        mc.set_edge_weight(pv, ov, &w);
                    }
                    addend += 1;
                }
                mc.compute_matching();
                stable = mc.get_matching();
                let mv = stable[pv];
                matched[pv] = true;
                matched[mv] = true;
                finalize_pair(pv, mv, &mut mc, &max_edge_weight);
            }
        }

        // Carry unpaired players to the next bracket.
        let mut new_players = Vec::new();
        let mut new_vertices = Vec::new();
        score_group_begin = 0;
        single_downfloater_is_bye_assignee = n & 1 == 1
            && score_group_iterator < n
            && bye_assignee_score >= score(score_group_iterator);
        for pi in 0..players_by_index.len() {
            let pv = vertex_indices[pi];
            if pi < next_score_group_begin && matched[pv] {
                let a = players_by_index[pi];
                let b = sorted[stable[pv]];
                matching_by_id[a] = Some(b);
                matching_by_id[b] = Some(a);
            } else {
                new_players.push(players_by_index[pi]);
                new_vertices.push(pv);
                if pi < next_score_group_begin {
                    score_group_begin += 1;
                }
                if single_downfloater_is_bye_assignee
                    && score_group_iterator < n
                    && pscore(sorted[stable[pv]]) < score(score_group_iterator)
                {
                    single_downfloater_is_bye_assignee = false;
                }
            }
        }
        players_by_index = new_players;
        vertex_indices = new_vertices;
        score_group_begin_vertex = next_score_group_begin_vertex;
    }

    let mut result = Vec::new();
    for &id in &sorted {
        match matching_by_id[id] {
            Some(m) => {
                if id < m {
                    let c = choose_player_color(&t.players[id], &t.players[m], t);
                    result.push(if c == C::White { (id, m) } else { (m, id) });
                }
            }
            None => result.push((id, id)),
        }
    }
    Ok(result)
}

/// Fixes the pair (v0, v1): their edge keeps `default` weight, every other
/// edge of theirs becomes zero.
pub(crate) fn finalize_pair(v0: usize, v1: usize, mc: &mut Computer, default: &Wide) {
    let zero = default.zero_like();
    for u in 0..mc.size() {
        if v0 != u {
            mc.set_edge_weight(v0, u, if u == v1 { default } else { &zero });
        }
        if v1 != u {
            mc.set_edge_weight(v1, u, if u == v0 { default } else { &zero });
        }
    }
}
