//! Tie-breaks (FIDE Handbook C.07) and standings.
//!
//! [`standings`] already ranks by points with a starting-rank fallback. Each
//! tie-break calculation lands in its phase (see `TiebreakKind::phase`);
//! until then its column shows as unavailable and is listed in
//! [`Standings::pending`].

use co_core::{
    Error, PlayerId, Points, Result, RoundNo, TeamId, TiebreakSpec, Tournament, not_yet, scoring,
};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A tie-break value in hundredths, so Sonneborn-Berger's quarter points stay exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TbValue {
    Value(i64),
    /// Not yet computable (feature planned for a later phase).
    Unavailable,
}

impl TbValue {
    pub fn from_points(p: Points) -> Self {
        TbValue::Value(i64::from(p.0) * 10)
    }

    /// Display with up to two decimals, e.g. `12.5`, `7.25`, `2380`.
    pub fn display(self) -> String {
        match self {
            TbValue::Unavailable => "–".into(),
            TbValue::Value(v) if v % 100 == 0 => format!("{}", v / 100),
            TbValue::Value(v) if v % 10 == 0 => format!("{:.1}", v as f64 / 100.0),
            TbValue::Value(v) => format!("{:.2}", v as f64 / 100.0),
        }
    }
}

impl PartialOrd for TbValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TbValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (TbValue::Value(a), TbValue::Value(b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }
}

pub struct TiebreakContext<'a> {
    pub tournament: &'a Tournament,
    pub after_round: RoundNo,
    pub scores: BTreeMap<PlayerId, Points>,
}

impl<'a> TiebreakContext<'a> {
    pub fn new(tournament: &'a Tournament, after_round: RoundNo) -> Self {
        Self {
            tournament,
            after_round,
            scores: scoring::scores_after(tournament, after_round),
        }
    }
}

/// Computes one tie-break for one player.
pub fn compute(spec: &TiebreakSpec, _ctx: &TiebreakContext, _player: PlayerId) -> Result<TbValue> {
    not_yet!(spec.kind.acronym(), spec.kind.phase())
}

#[derive(Debug, Clone, Serialize)]
pub struct StandingRow {
    /// 1-based place.
    pub rank: u32,
    /// Same points and tie-breaks as the row above.
    pub tied_with_previous: bool,
    pub player: PlayerId,
    pub points: Points,
    pub tiebreaks: Vec<TbValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Standings {
    pub after_round: RoundNo,
    /// Column labels, e.g. `BH-C1`.
    pub tiebreak_labels: Vec<String>,
    pub rows: Vec<StandingRow>,
    /// Tie-breaks not computed yet, with the phase they land in.
    pub pending: Vec<(String, u8)>,
}

/// Individual standings after `after_round` rounds.
pub fn standings(t: &Tournament, after_round: RoundNo) -> Result<Standings> {
    let ctx = TiebreakContext::new(t, after_round);
    let specs = &t.settings.tiebreaks;
    let mut pending = Vec::new();
    let mut values: BTreeMap<PlayerId, Vec<TbValue>> = BTreeMap::new();
    for spec in specs {
        for p in &t.players {
            let v = match compute(spec, &ctx, p.id) {
                Ok(v) => v,
                Err(Error::NotYetImplemented { phase, .. }) => {
                    if !pending.iter().any(|(l, _)| *l == spec.label()) {
                        pending.push((spec.label(), phase));
                    }
                    TbValue::Unavailable
                }
                Err(e) => return Err(e),
            };
            values.entry(p.id).or_default().push(v);
        }
    }

    let mut rows: Vec<StandingRow> = t
        .players
        .iter()
        .map(|p| StandingRow {
            rank: 0,
            tied_with_previous: false,
            player: p.id,
            points: ctx.scores[&p.id],
            tiebreaks: values.remove(&p.id).unwrap_or_default(),
        })
        .collect();
    let start_rank = |id: PlayerId| t.player(id).map_or(u32::MAX, |p| p.starting_rank);
    rows.sort_by(|a, b| {
        b.points
            .cmp(&a.points)
            .then_with(|| b.tiebreaks.cmp(&a.tiebreaks))
            .then_with(|| start_rank(a.player).cmp(&start_rank(b.player)))
    });
    for i in 0..rows.len() {
        rows[i].rank = i as u32 + 1;
        if i > 0 {
            let tied = rows[i].points == rows[i - 1].points
                && rows[i].tiebreaks.cmp(&rows[i - 1].tiebreaks) == Ordering::Equal;
            rows[i].tied_with_previous = tied;
        }
    }
    Ok(Standings {
        after_round,
        tiebreak_labels: specs.iter().map(TiebreakSpec::label).collect(),
        rows,
        pending,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamStandingRow {
    pub rank: u32,
    pub team: TeamId,
    pub match_points: Points,
    pub game_points: Points,
    pub tiebreaks: Vec<TbValue>,
}

/// Team standings (Phase 3).
pub fn team_standings(_t: &Tournament, _after_round: RoundNo) -> Result<Vec<TeamStandingRow>> {
    not_yet!("Team standings", 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use co_core::*;

    #[test]
    fn ranks_by_points_then_starting_rank() {
        let mut t = Tournament::create(&Event::TournamentCreated {
            id: Uuid::nil(),
            info: TournamentInfo::default(),
            settings: Settings::default(),
            kind: TournamentKind::default(),
        })
        .unwrap();
        for i in 1..=4 {
            t.apply(&Event::PlayerAdded {
                player: Player::new(PlayerId(i), format!("P{i}")),
            })
            .unwrap();
        }
        t.apply(&Event::RoundPaired {
            round: 1,
            pairings: vec![
                Pairing::Game {
                    board: 1,
                    white: PlayerId(1),
                    black: PlayerId(2),
                    result: Some(GameResult::BLACK_WINS),
                },
                Pairing::Game {
                    board: 2,
                    white: PlayerId(3),
                    black: PlayerId(4),
                    result: Some(GameResult::DRAW),
                },
            ],
            matches: vec![],
            method: PairingMethod::Manual,
        })
        .unwrap();
        let s = standings(&t, 1).unwrap();
        let order: Vec<u32> = s.rows.iter().map(|r| r.player.0).collect();
        assert_eq!(order, vec![2, 3, 4, 1]);
        // Default settings list four tie-breaks, all pending until Phase 1.
        assert_eq!(s.pending.len(), 4);
        assert!(s.rows[2].tied_with_previous);
    }
}
