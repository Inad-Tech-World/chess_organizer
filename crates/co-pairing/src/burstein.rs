//! FIDE Burstein System, FIDE Handbook C.04.4.2.
//!
//! A port of bbpPairings' Burstein engine (Apache-2.0, Jeremy Bierema; see
//! NOTICE). Players are sorted into score groups and, within a group, by
//! Sonneborn-Berger, Buchholz and median Buchholz (unplayed games counted
//! as described in the rules). A maximum-weight matching first fixes which
//! groups are paired and who floats; then, from the top of each group, every
//! player is matched to the lowest-placed opponent that keeps the matching
//! complete. In rounds 1 and 2 the top half are accelerated by one win, as
//! bbpPairings does when no other acceleration is given.

// Index loops mirror the reference algorithm's vertex indices.
#![allow(clippy::needless_range_loop)]

use crate::dutch::finalize_pair;
use crate::matching::{Computer, Wide};
use crate::swiss_model::{
    C, Ms, Player, Tournament, choose_player_neutral_color, color_preferences_compatible,
};
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::{Availability, PairingInput};
use co_core::scoring::RoundOutcome;
use co_core::{ByeKind, Error, PairingSystem, Result};
use std::cmp::Ordering;
use std::collections::HashSet;

pub struct BursteinEngine;

impl PairingEngine for BursteinEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Burstein
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        let input = with_default_acceleration(input);
        let t = Tournament::from_input(&input)?;
        let pairs = compute_matching(&t)?;
        Ok(t.to_round(&input, pairs))
    }
}

/// Rounds 1 and 2: the top half (by starting rank) of the players who take
/// part in round 1 get a win's worth of virtual points.
fn with_default_acceleration(input: &PairingInput) -> PairingInput {
    let mut input = input.clone();
    if input.round > 2 || input.players.iter().any(|p| !p.accelerations.is_empty()) {
        return input;
    }
    let first_round = input.round == 1;
    let participates: Vec<bool> = input
        .players
        .iter()
        .map(|p| {
            if first_round {
                p.availability == Availability::Available
            } else {
                matches!(
                    p.history.first().map(|h| h.outcome),
                    Some(RoundOutcome::Game { .. } | RoundOutcome::Bye(ByeKind::PairingAllocated))
                )
            }
        })
        .collect();
    let mut bound = participates.iter().filter(|&&x| x).count();
    if bound <= 1 {
        return input;
    }
    let win = input.points.win;
    let rounds = usize::from(input.round);
    for (p, part) in input.players.iter_mut().zip(participates) {
        p.accelerations = vec![win; rounds];
        p.acceleration = win;
        if part {
            bound -= 2;
            if bound <= 1 {
                break;
            }
        }
    }
    input
}

// Edge weight layout, as in bbpPairings built for at most 9999 players.
const PREFERENCE_SIZE: u64 = 9998;
const COLOR_COUNT_SIZE: u64 = 5000;
const SAME_SCORE_GROUP_SIZE: u64 = 5000;
const SAME_SCORE_GROUP_MULTIPLIER: u64 = PREFERENCE_SIZE * COLOR_COUNT_SIZE;
const COMPATIBLE_MULTIPLIER: u64 = SAME_SCORE_GROUP_MULTIPLIER * SAME_SCORE_GROUP_SIZE;
const COLOR_MULTIPLIER: u64 = PREFERENCE_SIZE;
const MAX_EDGE_WEIGHT: u64 =
    COMPATIBLE_MULTIPLIER + SAME_SCORE_GROUP_MULTIPLIER + PREFERENCE_SIZE + PREFERENCE_SIZE - 1;

fn acceleration(t: &Tournament, p: &Player) -> i64 {
    p.accelerations.get(t.played_rounds).copied().unwrap_or(0)
}

/// An unplayed game counts as a draw.
fn adjusted_points(t: &Tournament, p: &Player, m: &crate::swiss_model::Match) -> i64 {
    if m.played {
        t.get_points(p, m)
    } else {
        t.points_for_draw
    }
}

/// The virtual opponent's score for an unplayed game.
fn virtual_opponent_score(t: &Tournament, p: &Player, m: &crate::swiss_model::Match) -> i64 {
    match m.score {
        Ms::Loss => t.points_for_win,
        Ms::Draw => t.points_for_draw,
        Ms::Win => {
            if m.opponent == p.id && m.participated && t.points_for_pab < t.points_for_win {
                if t.points_for_pab < t.points_for_draw {
                    t.points_for_win
                } else {
                    t.points_for_draw
                }
            } else {
                t.points_for_forfeit_loss
            }
        }
    }
}

fn virtual_start(t: &Tournament, p: &Player) -> i128 {
    (t.played_rounds as i128 - 1) * i128::from(t.points_for_draw) + i128::from(acceleration(t, p))
}

fn sonneborn_berger(t: &Tournament, p: &Player, adjusted: &[i64]) -> i128 {
    if !p.valid {
        return 0;
    }
    let mut result: i128 = 0;
    let mut virtual_points = virtual_start(t, p);
    for m in p.matches.iter().take(t.played_rounds) {
        let pts = i128::from(t.get_points(p, m));
        if m.played {
            result += i128::from(adjusted[m.opponent]) * pts;
        } else {
            result += pts * (virtual_points + i128::from(virtual_opponent_score(t, p, m)));
        }
        virtual_points += pts - i128::from(t.points_for_draw);
    }
    result
}

fn buchholz(t: &Tournament, p: &Player, adjusted: &[i64], median: bool) -> i128 {
    if !p.valid || (median && t.played_rounds <= 2) {
        return 0;
    }
    let mut result: i128 = 0;
    let (mut min, mut max) = (i128::MAX, 0i128);
    let mut virtual_points = virtual_start(t, p);
    for m in p.matches.iter().take(t.played_rounds) {
        let adjustment = if m.played {
            i128::from(adjusted[m.opponent])
        } else {
            virtual_points + i128::from(virtual_opponent_score(t, p, m))
        };
        result += adjustment;
        min = min.min(adjustment);
        max = max.max(adjustment);
        virtual_points += i128::from(t.get_points(p, m)) - i128::from(t.points_for_draw);
    }
    if median {
        result -= min + max;
    }
    result
}

/// Accelerated score, tie-breaks and rank used to order a score group.
#[derive(Debug, Clone, Copy)]
struct Metric {
    score: i128,
    sb: i128,
    bh: i128,
    med: i128,
    rank: usize,
}

impl Metric {
    fn less(&self, that: &Metric) -> bool {
        if self.score == that.score {
            (self.sb, self.bh, self.med, that.rank) < (that.sb, that.bh, that.med, self.rank)
        } else {
            (
                self.sb,
                self.bh * self.score,
                self.med * self.score,
                that.rank,
            ) < (
                that.sb,
                that.bh * that.score,
                that.med * that.score,
                self.rank,
            )
        }
    }

    fn cmp_desc(a: &Metric, b: &Metric) -> Ordering {
        if b.less(a) {
            Ordering::Less
        } else if a.less(b) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

fn edge_weight(
    p0: &Player,
    p1: &Player,
    forbidden: &[HashSet<usize>],
    same_score_group: bool,
    use_due_color: bool,
) -> u64 {
    if forbidden[p0.id].contains(&p1.id)
        || (p0.absolute_color_preference()
            && p1.absolute_color_preference()
            && p0.color_preference == p1.color_preference)
    {
        return 0;
    }
    COMPATIBLE_MULTIPLIER
        + u64::from(same_score_group) * SAME_SCORE_GROUP_MULTIPLIER
        + u64::from(
            same_score_group
                && use_due_color
                && color_preferences_compatible(p0.color_preference, p1.color_preference),
        ) * COLOR_MULTIPLIER
}

/// Every player in the matched groups is paired (except one in the lowest)
/// and at most one player floats from each group.
fn matching_is_valid(mc: &Computer, groups: &[usize]) -> bool {
    let matching = mc.get_matching();
    debug_assert_eq!(groups[0], 0);
    let last = *groups.last().expect("groups");
    let second_last = groups[groups.len() - 2];
    let mut it = 0;
    let mut unmatched = 0;
    let mut group_begin = 0;
    for (vertex, &matched) in matching.iter().enumerate() {
        if vertex >= last {
            return true;
        }
        if vertex >= groups[it] {
            unmatched = 0;
            group_begin = vertex;
            loop {
                it += 1;
                if vertex < groups[it] {
                    break;
                }
            }
        }
        if vertex == matched && vertex < second_last {
            return false;
        }
        if vertex == matched || matched < group_begin || matched >= groups[it] {
            unmatched += 1;
            if unmatched > if groups[it] & 1 == 1 { 2 } else { 1 } {
                return false;
            }
        }
    }
    true
}

fn choose_player_color(p: &Player, o: &Player, t: &Tournament, metrics: &[Metric]) -> C {
    let neutral = choose_player_neutral_color(p, o);
    if neutral != C::None {
        return neutral;
    }
    if p.color_preference == C::None {
        if p.rank_index < o.rank_index {
            if p.rank_index & 1 == 1 {
                t.initial_color.invert()
            } else {
                t.initial_color
            }
        } else if o.rank_index & 1 == 1 {
            t.initial_color
        } else {
            t.initial_color.invert()
        }
    } else if metrics[p.id].less(&metrics[o.id]) {
        o.color_preference.invert()
    } else {
        p.color_preference
    }
}

/// `(white, black)` player indexes; the bye is `(i, i)`.
pub(crate) fn compute_matching(t: &Tournament) -> Result<Vec<(usize, usize)>> {
    let w = |v: u64| Wide::from_u64(v);
    let mut forbidden = t.forbidden.clone();
    let mut adjusted = Vec::with_capacity(t.players.len());
    let mut sorted: Vec<usize> = Vec::new();
    for p in &t.players {
        let mut a = 0;
        if p.valid {
            if p.matches.len() <= t.played_rounds {
                sorted.push(p.id);
            }
            a = acceleration(t, p);
            for (i, m) in p.matches.iter().enumerate() {
                if i < t.played_rounds {
                    a += adjusted_points(t, p, m);
                }
                if m.played {
                    forbidden[p.id].insert(m.opponent);
                }
            }
        }
        adjusted.push(a);
    }
    let metrics: Vec<Metric> = t
        .players
        .iter()
        .map(|p| Metric {
            score: i128::from(t.score_acc(p, 0)),
            sb: sonneborn_berger(t, p, &adjusted),
            bh: buchholz(t, p, &adjusted, false),
            med: buchholz(t, p, &adjusted, true),
            rank: p.rank_index,
        })
        .collect();
    sorted.sort_by(|&a, &b| {
        let (pa, pb) = (&t.players[a], &t.players[b]);
        t.score_acc(pb, 0)
            .cmp(&t.score_acc(pa, 0))
            .then_with(|| Metric::cmp_desc(&metrics[a], &metrics[b]))
    });

    let mut result: Vec<(usize, usize)> = Vec::new();
    if sorted.len() % 2 == 1 {
        let eligible = |p: &Player| {
            !p.matches
                .iter()
                .any(|m| !m.played && m.participated && m.score == Ms::Win)
        };
        let pos = sorted
            .iter()
            .rposition(|&i| eligible(&t.players[i]))
            .ok_or_else(|| {
                Error::Invalid("no player is eligible for the pairing-allocated bye".into())
            })?;
        let bye = sorted.remove(pos);
        result.push((bye, bye));
    }
    let labels = sorted;
    let n = labels.len();
    let mut mc = Computer::new(n, &w(MAX_EDGE_WEIGHT));
    for _ in 0..n {
        mc.add_vertex();
    }
    let player = |v: usize| &t.players[labels[v]];

    // Group boundaries; a boundary listed twice means no float below it.
    let mut groups: Vec<usize> = vec![0, 0];
    let mut valid = true;
    while *groups.last().unwrap() < n {
        groups.push(*groups.last().unwrap());
        loop {
            let group_begin = *groups.last().unwrap();
            loop {
                let back = *groups.last().unwrap();
                let len = groups.len();
                for v in groups[len - 3]..back {
                    let weight = edge_weight(
                        player(v),
                        player(back),
                        &forbidden,
                        v >= groups[len - 2],
                        false,
                    );
                    mc.set_edge_weight(back, v, &w(weight));
                }
                *groups.last_mut().unwrap() += 1;
                let back = *groups.last().unwrap();
                if !(back < n
                    && t.score_acc(player(group_begin), 0) == t.score_acc(player(back), 0))
                {
                    break;
                }
            }
            let back = *groups.last().unwrap();
            if back & 1 == 1 {
                // A virtual opponent so nobody above stays unpaired.
                for v in groups[groups.len() - 2]..back {
                    mc.set_edge_weight(back, v, &w(COMPATIBLE_MULTIPLIER));
                }
            }
            mc.compute_matching();
            valid = matching_is_valid(&mc, &groups);
            if !(back < n && !valid) {
                break;
            }
        }
        let back = *groups.last().unwrap();
        if back < n && back & 1 == 0 {
            groups.push(back);
        }
    }

    // Merge the lowest groups until the matching is valid.
    while groups.len() > 3 && !valid {
        groups.pop();
        let boundary = groups.pop().unwrap();
        let group_begin = groups.pop().unwrap();
        for outer in *groups.last().unwrap()..boundary {
            for inner in boundary..n {
                let weight = edge_weight(
                    player(outer),
                    player(inner),
                    &forbidden,
                    outer >= group_begin,
                    false,
                );
                mc.set_edge_weight(outer, inner, &w(weight));
            }
        }
        mc.compute_matching();
        groups.push(group_begin);
        groups.push(n);
        valid = matching_is_valid(&mc, &groups);
    }
    if !valid {
        return Err(Error::Invalid(
            "no valid pairing exists: the players cannot all be paired without breaking an absolute criterion".into(),
        ));
    }

    // From the top of each group, take the lowest opponent that keeps the matching.
    let mut matched: Vec<Option<usize>> = vec![None; t.players.len()];
    let one = w(1);
    let mut group_begin = groups[0];
    let mut floater: Option<usize> = None;
    for &group_end in &groups[1..] {
        if group_begin == group_end {
            continue;
        }
        let mut full: Vec<usize> = (group_begin..group_end).collect();
        if let Some(f) = floater.take() {
            full.push(f);
        }
        full.sort_by(|&a, &b| Metric::cmp_desc(&metrics[labels[a]], &metrics[labels[b]]));
        for (i, &v) in full.iter().enumerate() {
            for &nb in full[i + 1..].iter().rev() {
                mc.set_edge_weight(
                    v,
                    nb,
                    &w(edge_weight(player(v), player(nb), &forbidden, true, true)),
                );
            }
        }
        for (i, &v) in full.iter().enumerate() {
            if matched[labels[v]].is_some() {
                continue;
            }
            let mut priority = 1;
            for &nb in &full[i + 1..] {
                if matched[labels[nb]].is_none() {
                    let weight = edge_weight(player(v), player(nb), &forbidden, true, true);
                    if weight != 0 {
                        mc.set_edge_weight(v, nb, &w(weight + priority));
                        priority += 1;
                    }
                }
            }
            mc.compute_matching();
            let m = mc.get_matching()[v];
            if m >= group_end {
                floater = Some(v);
            } else {
                matched[labels[v]] = Some(labels[m]);
                matched[labels[m]] = Some(labels[v]);
                let (p, o) = (player(v), player(m));
                result.push(if choose_player_color(p, o, t, &metrics) == C::White {
                    (p.id, o.id)
                } else {
                    (o.id, p.id)
                });
                finalize_pair(v, m, &mut mc, &one);
            }
        }
        group_begin = group_end;
    }
    Ok(result)
}
