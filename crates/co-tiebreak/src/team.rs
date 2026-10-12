//! Team standings and team tie-breaks (C.07 Articles 11–13).
//!
//! Each team's match result per round gives match points (MP: win / draw /
//! loss) and game points (GP: sum of its players' points). The primary
//! score is MP or GP according to the tournament settings; the other is the
//! secondary score.

use crate::{TbValue, TiebreakContext};
use co_core::{
    Pairing, Points, Result, RoundNo, TeamId, TeamScore, TiebreakKind, TiebreakSpec, Tournament,
    scoring,
};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;

/// One team's result in one round.
#[derive(Debug, Clone)]
struct MatchRec {
    opponent: Option<TeamId>,
    mp: Points,
    gp: Points,
    /// Game points per board (index 0 = board 1).
    boards: Vec<Points>,
    complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamStandingRow {
    pub rank: u32,
    pub tied_with_previous: bool,
    pub team: TeamId,
    pub match_points: Points,
    pub game_points: Points,
    pub tiebreaks: Vec<TbValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamStandings {
    pub after_round: RoundNo,
    pub primary: TeamScore,
    pub tiebreak_labels: Vec<String>,
    pub rows: Vec<TeamStandingRow>,
}

struct TeamCtx<'a> {
    t: &'a Tournament,
    after_round: RoundNo,
    recs: HashMap<TeamId, Vec<MatchRec>>,
    mp: HashMap<TeamId, Points>,
    gp: HashMap<TeamId, Points>,
}

impl<'a> TeamCtx<'a> {
    fn new(t: &'a Tournament, after_round: RoundNo, fore: bool) -> Self {
        let ts = &t.settings.team;
        let ps = &t.settings.points;
        let mut recs: HashMap<TeamId, Vec<MatchRec>> =
            t.teams.iter().map(|team| (team.id, Vec::new())).collect();
        let rounds: Vec<_> = t.rounds.iter().take(usize::from(after_round)).collect();
        for (ri, round) in rounds.iter().enumerate() {
            let last = ri + 1 == rounds.len();
            for m in &round.matches {
                match m.away {
                    None => {
                        // Pairing-allocated bye: a win on every board.
                        let boards = vec![ps.win; usize::from(ts.boards)];
                        let gp = boards.iter().copied().sum();
                        if let Some(v) = recs.get_mut(&m.home) {
                            v.push(MatchRec {
                                opponent: None,
                                mp: ts.match_win,
                                gp,
                                boards,
                                complete: true,
                            });
                        }
                    }
                    Some(away) => {
                        let mut home_boards = vec![Points::ZERO; usize::from(ts.boards)];
                        let mut away_boards = vec![Points::ZERO; usize::from(ts.boards)];
                        let mut complete = true;
                        for g in &m.games {
                            if let Pairing::Game {
                                board,
                                white,
                                result,
                                ..
                            } = *g
                            {
                                let b = usize::from(board).saturating_sub(1);
                                let home_is_white = m.home_lineup.contains(&white);
                                let (hw, aw) = match (fore && last, result) {
                                    (true, _) => (ps.draw, ps.draw),
                                    (false, Some(r)) => {
                                        let w = scoring::side_points(r.white, ps);
                                        let bl = scoring::side_points(r.black, ps);
                                        if home_is_white { (w, bl) } else { (bl, w) }
                                    }
                                    (false, None) => {
                                        complete = false;
                                        (Points::ZERO, Points::ZERO)
                                    }
                                };
                                if b < home_boards.len() {
                                    home_boards[b] = hw;
                                    away_boards[b] = aw;
                                }
                            }
                        }
                        let hg: Points = home_boards.iter().copied().sum();
                        let ag: Points = away_boards.iter().copied().sum();
                        let (hm, am) = match hg.cmp(&ag) {
                            Ordering::Greater => (ts.match_win, ts.match_loss),
                            Ordering::Less => (ts.match_loss, ts.match_win),
                            Ordering::Equal => (ts.match_draw, ts.match_draw),
                        };
                        if let Some(v) = recs.get_mut(&m.home) {
                            v.push(MatchRec {
                                opponent: Some(away),
                                mp: hm,
                                gp: hg,
                                boards: home_boards,
                                complete,
                            });
                        }
                        if let Some(v) = recs.get_mut(&away) {
                            v.push(MatchRec {
                                opponent: Some(m.home),
                                mp: am,
                                gp: ag,
                                boards: away_boards,
                                complete,
                            });
                        }
                    }
                }
            }
        }
        let mp = recs
            .iter()
            .map(|(k, v)| (*k, v.iter().map(|r| r.mp).sum()))
            .collect();
        let gp = recs
            .iter()
            .map(|(k, v)| (*k, v.iter().map(|r| r.gp).sum()))
            .collect();
        Self {
            t,
            after_round,
            recs,
            mp,
            gp,
        }
    }

    fn score(&self, team: TeamId, which: TeamScore) -> i64 {
        let map = if which == TeamScore::MatchPoints {
            &self.mp
        } else {
            &self.gp
        };
        i64::from(map.get(&team).copied().unwrap_or_default().0)
    }

    fn primary(&self) -> TeamScore {
        self.t.settings.team.primary_score
    }

    fn secondary(&self) -> TeamScore {
        match self.primary() {
            TeamScore::MatchPoints => TeamScore::GamePoints,
            TeamScore::GamePoints => TeamScore::MatchPoints,
        }
    }

    fn recs(&self, team: TeamId) -> &[MatchRec] {
        self.recs.get(&team).map_or(&[], Vec::as_slice)
    }

    fn buchholz(&self, team: TeamId, which: TeamScore, cut_low: u8, cut_high: u8) -> i64 {
        let own = self.score(team, which);
        let v: Vec<(i64, bool)> = self
            .recs(team)
            .iter()
            .map(|r| (r.opponent.map_or(own, |o| self.score(o, which)) * 10, false))
            .collect();
        crate::apply_cuts(v, cut_low, cut_high)
    }
}

fn lexicographic(values: &[i64], base: i64) -> i64 {
    values.iter().fold(0i128, |acc, &v| {
        (acc * i128::from(base) + i128::from(v)).min(i128::from(i64::MAX))
    }) as i64
}

fn compute(spec: &TiebreakSpec, c: &TeamCtx, team: TeamId) -> TbValue {
    let m = &spec.modifiers;
    let which = m.team_score.unwrap_or(c.primary());
    let recs = c.recs(team);
    let base = i64::from(c.t.settings.points.win.0) * i64::from(c.after_round.max(1)) + 1;
    let v = match spec.kind {
        TiebreakKind::SecondaryScore | TiebreakKind::MatchPointsVsGamePoints => {
            c.score(team, c.secondary()) * 10
        }
        TiebreakKind::Buchholz => c.buchholz(team, which, m.cut_lowest, m.cut_highest),
        TiebreakKind::ForeBuchholz => {
            let f = TeamCtx::new(c.t, c.after_round, true);
            f.buchholz(team, which, m.cut_lowest, m.cut_highest)
        }
        TiebreakKind::AverageOpponentsBuchholz => {
            let opps: Vec<i64> = recs
                .iter()
                .filter_map(|r| r.opponent)
                .map(|o| c.buchholz(o, which, 0, 0))
                .collect();
            crate::avg_round_half_up(opps.iter().sum(), opps.len() as i64)
        }
        TiebreakKind::TeamSbMatchMatch
        | TiebreakKind::TeamSbMatchGame
        | TiebreakKind::TeamSbGameMatch
        | TiebreakKind::TeamSbGameGame => {
            let (opp_kind, own_kind) = match spec.kind {
                TiebreakKind::TeamSbMatchMatch => (TeamScore::MatchPoints, TeamScore::MatchPoints),
                TiebreakKind::TeamSbMatchGame => (TeamScore::MatchPoints, TeamScore::GamePoints),
                TiebreakKind::TeamSbGameMatch => (TeamScore::GamePoints, TeamScore::MatchPoints),
                _ => (TeamScore::GamePoints, TeamScore::GamePoints),
            };
            let own_total = c.score(team, opp_kind);
            let v: Vec<(i64, bool)> = recs
                .iter()
                .map(|r| {
                    let opp = r.opponent.map_or(own_total, |o| c.score(o, opp_kind));
                    let earned = if own_kind == TeamScore::MatchPoints {
                        r.mp
                    } else {
                        r.gp
                    };
                    (opp * i64::from(earned.0), false)
                })
                .collect();
            crate::apply_cuts(v, m.cut_lowest, m.cut_highest)
        }
        TiebreakKind::Wins | TiebreakKind::WonGames => {
            recs.iter()
                .filter(|r| r.mp == c.t.settings.team.match_win)
                .count() as i64
                * 100
        }
        TiebreakKind::ProgressiveScore => {
            let mut cum = 0i64;
            let mut values = Vec::new();
            for r in recs {
                cum += i64::from(if which == TeamScore::MatchPoints {
                    r.mp.0
                } else {
                    r.gp.0
                });
                values.push(cum * 10);
            }
            let cut = usize::from(m.cut_lowest).min(values.len());
            values[cut..].iter().sum()
        }
        TiebreakKind::Koya => {
            let max_mp = i64::from(c.t.settings.team.match_win.0) * i64::from(c.after_round);
            let threshold = max_mp / 2 + i64::from(m.koya_limit.unwrap_or(0)) * 5;
            recs.iter()
                .filter(|r| {
                    r.opponent
                        .is_some_and(|o| c.score(o, TeamScore::MatchPoints) >= threshold)
                })
                .map(|r| {
                    i64::from(if which == TeamScore::MatchPoints {
                        r.mp.0
                    } else {
                        r.gp.0
                    }) * 10
                })
                .sum()
        }
        TiebreakKind::BoardCount => {
            // Lower is better: store negated.
            -recs
                .iter()
                .flat_map(|r| {
                    r.boards
                        .iter()
                        .enumerate()
                        .map(|(b, p)| (b as i64 + 1) * i64::from(p.0))
                })
                .sum::<i64>()
                * 10
        }
        TiebreakKind::TopBoardResult => {
            let boards = usize::from(c.t.settings.team.boards);
            let per: Vec<i64> = (0..boards)
                .map(|b| {
                    recs.iter()
                        .map(|r| i64::from(r.boards.get(b).copied().unwrap_or_default().0))
                        .sum()
                })
                .collect();
            lexicographic(&per, base)
        }
        TiebreakKind::BottomBoardElimination => {
            let boards = usize::from(c.t.settings.team.boards);
            let per: Vec<i64> = (1..boards)
                .rev()
                .map(|keep| {
                    recs.iter()
                        .map(|r| {
                            r.boards
                                .iter()
                                .take(keep)
                                .map(|p| i64::from(p.0))
                                .sum::<i64>()
                        })
                        .sum()
                })
                .collect();
            lexicographic(&per, base * boards as i64)
        }
        TiebreakKind::StrengthCompensatedScore => {
            let k = i64::from(m.sssc_factor.unwrap_or(4)).max(1);
            let bh = if m.fore {
                TeamCtx::new(c.t, c.after_round, true).buchholz(team, c.primary(), 0, 0)
            } else {
                c.buchholz(team, c.primary(), 0, 0)
            };
            c.score(team, c.secondary()) * 10 + bh / k
        }
        // Extended direct encounter is group-dependent (see `resolve`).
        _ => return TbValue::Unavailable,
    };
    TbValue::Value(v)
}

/// EDE (13.3): direct encounter on the primary score, then the secondary.
fn extended_direct_encounter(c: &TeamCtx, group: &[TeamId]) -> HashMap<TeamId, i64> {
    group
        .iter()
        .map(|&team| {
            let (mut prim, mut sec) = (0i64, 0i64);
            for r in c.recs(team) {
                if r.opponent.is_some_and(|o| group.contains(&o)) {
                    let (p, s) = if c.primary() == TeamScore::MatchPoints {
                        (r.mp, r.gp)
                    } else {
                        (r.gp, r.mp)
                    };
                    prim += i64::from(p.0);
                    sec += i64::from(s.0);
                }
            }
            (team, prim * 10_000 + sec)
        })
        .collect()
}

/// Team standings after `after_round` rounds.
pub fn team_standings(t: &Tournament, after_round: RoundNo) -> Result<TeamStandings> {
    let c = TeamCtx::new(t, after_round, false);
    let specs: Vec<TiebreakSpec> = t
        .settings
        .tiebreaks
        .iter()
        .copied()
        .filter(|s| s.kind.for_teams())
        .collect();
    let mut values: HashMap<TeamId, Vec<TbValue>> = t
        .teams
        .iter()
        .map(|team| {
            (
                team.id,
                specs.iter().map(|s| compute(s, &c, team.id)).collect(),
            )
        })
        .collect();
    let primary = |team: TeamId| c.score(team, c.primary());
    let mut order: Vec<TeamId> = t.teams.iter().map(|team| team.id).collect();
    order.sort_by(|a, b| primary(*b).cmp(&primary(*a)).then(a.0.cmp(&b.0)));

    fn resolve(
        group: Vec<TeamId>,
        level: usize,
        specs: &[TiebreakSpec],
        c: &TeamCtx,
        values: &mut HashMap<TeamId, Vec<TbValue>>,
        out: &mut Vec<TeamId>,
    ) {
        if group.len() <= 1 || level >= specs.len() {
            let mut g = group;
            g.sort_by_key(|t| t.0);
            out.extend(g);
            return;
        }
        if matches!(
            specs[level].kind,
            TiebreakKind::ExtendedDirectEncounter
                | TiebreakKind::EdeBoardCountTopBoard
                | TiebreakKind::EdeBoardCountBottomBoard
                | TiebreakKind::EdeTopBoard
                | TiebreakKind::EdeBottomBoard
        ) {
            for (team, v) in extended_direct_encounter(c, &group) {
                if let Some(row) = values.get_mut(&team) {
                    row[level] = TbValue::Value(v);
                }
            }
        }
        let mut g = group;
        g.sort_by(|a, b| values[b][level].cmp(&values[a][level]));
        let mut i = 0;
        while i < g.len() {
            let mut j = i + 1;
            while j < g.len() && values[&g[j]][level] == values[&g[i]][level] {
                j += 1;
            }
            resolve(g[i..j].to_vec(), level + 1, specs, c, values, out);
            i = j;
        }
    }

    let mut sorted = Vec::new();
    let mut i = 0;
    while i < order.len() {
        let mut j = i + 1;
        while j < order.len() && primary(order[j]) == primary(order[i]) {
            j += 1;
        }
        resolve(
            order[i..j].to_vec(),
            0,
            &specs,
            &c,
            &mut values,
            &mut sorted,
        );
        i = j;
    }
    let mut rows: Vec<TeamStandingRow> = sorted
        .into_iter()
        .map(|team| TeamStandingRow {
            rank: 0,
            tied_with_previous: false,
            team,
            match_points: c.mp.get(&team).copied().unwrap_or_default(),
            game_points: c.gp.get(&team).copied().unwrap_or_default(),
            tiebreaks: values.remove(&team).unwrap_or_default(),
        })
        .collect();
    for i in 0..rows.len() {
        rows[i].rank = i as u32 + 1;
        if i > 0 {
            let same_primary = match c.primary() {
                TeamScore::MatchPoints => rows[i].match_points == rows[i - 1].match_points,
                TeamScore::GamePoints => rows[i].game_points == rows[i - 1].game_points,
            };
            rows[i].tied_with_previous = same_primary && rows[i].tiebreaks == rows[i - 1].tiebreaks;
        }
    }
    let _ = TiebreakContext::new;
    let _ = c.recs.values().all(|v| v.iter().all(|r| r.complete));
    Ok(TeamStandings {
        after_round,
        primary: c.primary(),
        tiebreak_labels: specs.iter().map(TiebreakSpec::label).collect(),
        rows,
    })
}
