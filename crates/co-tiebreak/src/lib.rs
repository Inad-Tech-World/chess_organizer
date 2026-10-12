//! Tie-breaks (FIDE Handbook C.07, edition effective 1 March 2026) and
//! standings.
//!
//! Unplayed rounds follow Article 16. Each unplayed round falls in one of
//! five categories:
//! 1. pairing-allocated or full-point bye
//! 2. forfeit win
//! 3. requested bye followed by a played round
//! 4. forfeit loss
//! 5. requested bye with no played round after it, or in the last round
//!
//! When opponents' scores are used (Buchholz, Sonneborn-Berger, …),
//! category 5 counts as a draw (16.3). A player's own unplayed rounds count
//! as games against a dummy with the player's score, capped (16.4). Cut
//! modifiers drop a voluntary unplayed round's contribution first (16.5).

pub mod team;

use co_core::{
    ByeKind, Color, Pairing, PlayerId, Points, Result, RoundNo, SideResult, TiebreakKind,
    TiebreakSpec, Tournament, scoring,
};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

pub use team::{TeamStandingRow, TeamStandings, team_standings};

/// A tie-break value in hundredths (Sonneborn-Berger can have quarters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TbValue {
    Value(i64),
    /// Not computable for this participant (e.g. rating tie-breaks without rated opponents).
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

    fn raw(self) -> i64 {
        match self {
            TbValue::Value(v) => v,
            TbValue::Unavailable => i64::MIN,
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
        self.raw().cmp(&other.raw())
    }
}

/// What happened to a player in one round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Played over the board (including unrated games).
    Played {
        opp: PlayerId,
        color: Color,
        side: SideResult,
    },
    /// Forfeit win or loss against the scheduled opponent.
    Forfeit {
        opp: PlayerId,
        side: SideResult,
    },
    Bye(ByeKind),
    /// Not paired: absent, withdrawn, or before a late entry joined.
    Absent(ByeKind),
    /// Paired, no result yet.
    Pending,
}

#[derive(Debug, Clone, Copy)]
struct Rec {
    kind: Kind,
    points: Points,
    /// Unplayed-round category (Article 16.2), 0 for played games.
    category: u8,
}

impl Rec {
    fn is_vur(&self) -> bool {
        matches!(self.category, 3..=5)
    }

    fn opponent(&self) -> Option<PlayerId> {
        match self.kind {
            Kind::Played { opp, .. } | Kind::Forfeit { opp, .. } => Some(opp),
            _ => None,
        }
    }
}

fn is_requested(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Bye(ByeKind::Half | ByeKind::Zero) | Kind::Absent(ByeKind::Half | ByeKind::Zero)
    )
}

pub struct TiebreakContext<'a> {
    pub tournament: &'a Tournament,
    pub after_round: RoundNo,
    pub scores: BTreeMap<PlayerId, Points>,
    recs: HashMap<PlayerId, Vec<Rec>>,
    adjusted: HashMap<PlayerId, Points>,
    fore: bool,
}

impl<'a> TiebreakContext<'a> {
    pub fn new(tournament: &'a Tournament, after_round: RoundNo) -> Self {
        Self::build(tournament, after_round, false)
    }

    /// The context Fore Buchholz uses: final-round games count as draws.
    fn fore(tournament: &'a Tournament, after_round: RoundNo) -> Self {
        Self::build(tournament, after_round, true)
    }

    fn build(t: &'a Tournament, after_round: RoundNo, fore: bool) -> Self {
        let ps = &t.settings.points;
        let rounds: Vec<_> = t.rounds.iter().take(usize::from(after_round)).collect();
        let indexes: Vec<_> = rounds.iter().map(|r| r.index()).collect();
        let mut recs: HashMap<PlayerId, Vec<Rec>> = HashMap::new();
        for p in &t.players {
            let mut v = Vec::with_capacity(rounds.len());
            for (i, round) in rounds.iter().enumerate() {
                let last = i + 1 == rounds.len();
                let entry = indexes[i].get(&p.id).copied();
                let (mut kind, mut points) = match entry {
                    Some(g @ Pairing::Game { result, .. }) => {
                        let (color, opp) = g.side_of(p.id).expect("game involves player");
                        match result {
                            Some(r) => {
                                let side = r.for_color(color);
                                let k = if side.is_played() {
                                    Kind::Played { opp, color, side }
                                } else {
                                    Kind::Forfeit { opp, side }
                                };
                                (k, scoring::side_points(side, ps))
                            }
                            None => (Kind::Pending, Points::ZERO),
                        }
                    }
                    Some(Pairing::Bye { kind, .. }) => {
                        (Kind::Bye(*kind), scoring::bye_points(*kind, ps))
                    }
                    None if round.number < p.joined_round => (
                        Kind::Absent(p.late_entry_bye),
                        scoring::bye_points(p.late_entry_bye, ps),
                    ),
                    None => (Kind::Absent(ByeKind::Zero), ps.zero_bye),
                };
                if fore
                    && last
                    && let Some(g @ Pairing::Game { .. }) = entry
                {
                    let (color, opp) = g.side_of(p.id).expect("game involves player");
                    kind = Kind::Played {
                        opp,
                        color,
                        side: SideResult::Draw,
                    };
                    points = ps.draw;
                }
                v.push(Rec {
                    kind,
                    points,
                    category: 0,
                });
            }
            // Categories (16.2).
            let n = v.len();
            for i in 0..n {
                v[i].category = match v[i].kind {
                    Kind::Bye(ByeKind::PairingAllocated | ByeKind::Full)
                    | Kind::Absent(ByeKind::Full | ByeKind::PairingAllocated) => 1,
                    Kind::Forfeit { side, .. } if side.is_win() => 2,
                    Kind::Forfeit { .. } => 4,
                    k if is_requested(k) => {
                        let played_later = v[i + 1..].iter().any(|r| {
                            !is_requested(r.kind)
                                && !matches!(r.kind, Kind::Forfeit { side, .. } if !side.is_win())
                        });
                        if i + 1 == n || !played_later { 5 } else { 3 }
                    }
                    _ => 0,
                };
            }
            recs.insert(p.id, v);
        }
        let adjusted = recs
            .iter()
            .map(|(id, v)| {
                let s = v
                    .iter()
                    .map(|r| if r.category == 5 { ps.draw } else { r.points })
                    .sum();
                (*id, s)
            })
            .collect();
        Self {
            tournament: t,
            after_round,
            scores: scoring::scores_after(t, after_round),
            recs,
            adjusted,
            fore,
        }
    }

    fn adj(&self, p: PlayerId) -> i64 {
        i64::from(self.adjusted.get(&p).copied().unwrap_or_default().0)
    }

    fn recs(&self, p: PlayerId) -> &[Rec] {
        self.recs.get(&p).map_or(&[], Vec::as_slice)
    }

    /// Score of the dummy opponent for one of `p`'s unplayed rounds (16.4).
    fn dummy(&self, p: PlayerId, rec: &Rec) -> i64 {
        let own = self.adj(p);
        match rec.kind {
            Kind::Forfeit { opp, .. } => own.min(self.adj(opp)),
            _ => {
                let cap =
                    i64::from(self.tournament.settings.points.draw.0) * i64::from(self.after_round);
                own.min(cap)
            }
        }
    }

    fn rating(&self, p: PlayerId) -> Option<i64> {
        self.tournament
            .player(p)
            .and_then(|x| x.rating(self.tournament.settings.rating_type))
            .map(i64::from)
    }

    /// Over-the-board games: (opponent, standard score in hundredths).
    fn otb(&self, p: PlayerId) -> Vec<(PlayerId, i64)> {
        self.recs(p)
            .iter()
            .filter_map(|r| match r.kind {
                Kind::Played { opp, side, .. } => Some((opp, standard_hundredths(side))),
                _ => None,
            })
            .collect()
    }
}

fn standard_hundredths(side: SideResult) -> i64 {
    match side {
        SideResult::Win | SideResult::UnratedWin | SideResult::ForfeitWin => 100,
        SideResult::Draw | SideResult::UnratedDraw => 50,
        _ => 0,
    }
}

/// Removes contributions per the cut/median modifiers, cutting voluntary
/// unplayed rounds first among the lowest (16.5).
fn apply_cuts(mut v: Vec<(i64, bool)>, cut_lowest: u8, cut_highest: u8) -> i64 {
    for _ in 0..cut_lowest {
        let idx = v
            .iter()
            .enumerate()
            .filter(|(_, (_, vur))| *vur)
            .min_by_key(|(_, (x, _))| *x)
            .map(|(i, _)| i)
            .or_else(|| {
                v.iter()
                    .enumerate()
                    .min_by_key(|(_, (x, _))| *x)
                    .map(|(i, _)| i)
            });
        match idx {
            Some(i) => {
                v.remove(i);
            }
            None => break,
        }
    }
    for _ in 0..cut_highest {
        match v
            .iter()
            .enumerate()
            .max_by_key(|(_, (x, _))| *x)
            .map(|(i, _)| i)
        {
            Some(i) => {
                v.remove(i);
            }
            None => break,
        }
    }
    v.iter().map(|(x, _)| x).sum()
}

fn avg_round_half_up(sum: i64, n: i64) -> i64 {
    if n == 0 {
        0
    } else {
        (2 * sum + n).div_euclid(2 * n)
    }
}

fn buchholz(ctx: &TiebreakContext, p: PlayerId, m: &co_core::TiebreakModifiers) -> i64 {
    let contributions: Vec<(i64, bool)> = ctx
        .recs(p)
        .iter()
        .filter(|r| !matches!(r.kind, Kind::Pending))
        .map(|r| {
            let value = match r.kind {
                Kind::Played { opp, .. } => ctx.adj(opp),
                Kind::Forfeit { opp, .. } if m.forfeits_as_played => ctx.adj(opp),
                _ => ctx.dummy(p, r),
            };
            (value * 10, r.is_vur())
        })
        .collect();
    apply_cuts(contributions, m.cut_lowest, m.cut_highest)
}

fn sonneborn_berger(ctx: &TiebreakContext, p: PlayerId, m: &co_core::TiebreakModifiers) -> i64 {
    let contributions: Vec<(i64, bool)> = ctx
        .recs(p)
        .iter()
        .filter(|r| !matches!(r.kind, Kind::Pending))
        .map(|r| {
            let base = match r.kind {
                Kind::Played { opp, .. } => ctx.adj(opp),
                Kind::Forfeit { opp, .. } if m.forfeits_as_played => ctx.adj(opp),
                _ => ctx.dummy(p, r),
            };
            (base * i64::from(r.points.0), r.is_vur())
        })
        .collect();
    apply_cuts(contributions, m.cut_lowest, m.cut_highest)
}

fn rated_otb(ctx: &TiebreakContext, p: PlayerId) -> Vec<(i64, i64)> {
    ctx.otb(p)
        .into_iter()
        .filter_map(|(o, s)| ctx.rating(o).map(|r| (r, s)))
        .collect()
}

fn tpr_of(ctx: &TiebreakContext, p: PlayerId) -> Option<i64> {
    let games = rated_otb(ctx, p);
    if games.is_empty() {
        return None;
    }
    let n = games.len() as i64;
    let aro = avg_round_half_up(games.iter().map(|g| g.0).sum(), n);
    let score: i64 = games.iter().map(|g| g.1).sum();
    Some(aro + i64::from(co_rating::tables::dp(score as f64 / (100 * n) as f64)))
}

fn ptp_of(ctx: &TiebreakContext, p: PlayerId) -> Option<i64> {
    let games = rated_otb(ctx, p);
    if games.is_empty() {
        return None;
    }
    let score: i64 = games.iter().map(|g| g.1).sum();
    let lo = games.iter().map(|g| g.0).min().unwrap_or(0);
    let hi = games.iter().map(|g| g.0).max().unwrap_or(0);
    if score == 0 {
        return Some(lo - 800);
    }
    (lo - 800..=hi + 800).find(|&r| {
        let expected: i64 = games
            .iter()
            .map(|g| i64::from(co_rating::tables::expected_hundredths((r - g.0) as i32)))
            .sum();
        expected >= score
    })
}

fn count(n: usize) -> i64 {
    n as i64 * 100
}

/// Computes one tie-break for one player. Direct encounter depends on the
/// tied group and is computed by [`standings`].
pub fn compute(spec: &TiebreakSpec, ctx: &TiebreakContext, player: PlayerId) -> Result<TbValue> {
    let m = &spec.modifiers;
    let t = ctx.tournament;
    let recs = ctx.recs(player);
    let win = t.settings.points.win;
    let v = match spec.kind {
        TiebreakKind::Buchholz => buchholz(ctx, player, m),
        TiebreakKind::ForeBuchholz => {
            if ctx.fore {
                buchholz(ctx, player, m)
            } else {
                let fc = TiebreakContext::fore(t, ctx.after_round);
                buchholz(&fc, player, m)
            }
        }
        TiebreakKind::AverageOpponentsBuchholz => {
            let plain = co_core::TiebreakModifiers::default();
            let fore_ctx;
            let c = if m.fore {
                fore_ctx = TiebreakContext::fore(t, ctx.after_round);
                &fore_ctx
            } else {
                ctx
            };
            let opps: Vec<i64> =
                ctx.otb(player).iter().map(|(o, _)| buchholz(c, *o, &plain)).collect();
            avg_round_half_up(opps.iter().sum(), opps.len() as i64)
        }
        TiebreakKind::SonnebornBerger => sonneborn_berger(ctx, player, m),
        TiebreakKind::ProgressiveScore => {
            let mut cum = Points::ZERO;
            let mut values = Vec::new();
            for r in recs {
                cum += r.points;
                values.push(i64::from(cum.0) * 10);
            }
            let cut = usize::from(m.cut_lowest).min(values.len());
            values[cut..].iter().sum()
        }
        TiebreakKind::Koya => {
            let max = i64::from(win.0) * i64::from(ctx.after_round);
            let threshold = max / 2 + i64::from(m.koya_limit.unwrap_or(0)) * 5;
            recs.iter()
                .filter_map(|r| r.opponent().map(|o| (o, r)))
                .filter(|(o, r)| {
                    ctx.adj(*o) >= threshold
                        && (matches!(r.kind, Kind::Played { .. })
                            || m.forfeits_as_played
                            || t.kind.is_round_robin())
                })
                .map(|(_, r)| i64::from(r.points.0) * 10)
                .sum()
        }
        TiebreakKind::Wins => count(recs.iter().filter(|r| win > Points::ZERO && r.points >= win).count()),
        TiebreakKind::WonGames => count(
            recs.iter()
                .filter(|r| matches!(r.kind, Kind::Played { side, .. } if side.is_win()))
                .count(),
        ),
        TiebreakKind::GamesWithBlack => count(
            recs.iter()
                .filter(|r| matches!(r.kind, Kind::Played { color: Color::Black, .. }))
                .count(),
        ),
        TiebreakKind::WinsWithBlack => count(
            recs.iter()
                .filter(|r| {
                    matches!(r.kind, Kind::Played { color: Color::Black, side, .. } if side.is_win())
                })
                .count(),
        ),
        TiebreakKind::RoundsElectedToPlay => {
            (i64::from(ctx.after_round) - recs.iter().filter(|r| r.is_vur()).count() as i64) * 100
        }
        TiebreakKind::StandardPoints => recs
            .iter()
            .map(|r| match r.kind {
                Kind::Played { side, .. } | Kind::Forfeit { side, .. } => {
                    if side.is_win() {
                        100
                    } else if matches!(side, SideResult::Draw | SideResult::UnratedDraw)
                        // A double forfeit counts as equal scores.
                        || (side == SideResult::ForfeitLoss
                            && r.opponent().is_some_and(|o| {
                                ctx.recs(o).iter().any(|x| {
                                    matches!(x.kind, Kind::Forfeit { opp, side: SideResult::ForfeitLoss } if opp == player)
                                })
                            }))
                    {
                        50
                    } else {
                        0
                    }
                }
                Kind::Pending => 0,
                _ => match r.points.cmp(&t.settings.points.draw) {
                    Ordering::Greater => 100,
                    Ordering::Equal => 50,
                    Ordering::Less => 0,
                },
            })
            .sum(),
        TiebreakKind::PairingNumber => {
            -i64::from(t.player(player).map_or(u32::MAX, |p| p.starting_rank)) * 100
        }
        TiebreakKind::Rating => ctx.rating(player).unwrap_or(0) * 100,
        TiebreakKind::AverageRatingOfOpponents => {
            let ratings: Vec<(i64, bool)> =
                rated_otb(ctx, player).into_iter().map(|(r, _)| (r, false)).collect();
            let n = ratings.len() as i64 - i64::from(m.cut_lowest) - i64::from(m.cut_highest);
            if n <= 0 {
                return Ok(TbValue::Unavailable);
            }
            avg_round_half_up(apply_cuts(ratings, m.cut_lowest, m.cut_highest), n) * 100
        }
        TiebreakKind::TournamentPerformanceRating => match tpr_of(ctx, player) {
            Some(v) => v * 100,
            None => return Ok(TbValue::Unavailable),
        },
        TiebreakKind::PerfectTournamentPerformance => match ptp_of(ctx, player) {
            Some(v) => v * 100,
            None => return Ok(TbValue::Unavailable),
        },
        TiebreakKind::AveragePerformanceRatingOfOpponents
        | TiebreakKind::AveragePerfectPerformanceOfOpponents => {
            let values: Vec<i64> = ctx
                .otb(player)
                .iter()
                .filter_map(|(o, _)| {
                    if spec.kind == TiebreakKind::AveragePerformanceRatingOfOpponents {
                        tpr_of(ctx, *o)
                    } else {
                        ptp_of(ctx, *o)
                    }
                })
                .collect();
            if values.is_empty() {
                return Ok(TbValue::Unavailable);
            }
            avg_round_half_up(values.iter().sum(), values.len() as i64) * 100
        }
        // Direct encounter needs the tied group; team tie-breaks live in `team`.
        _ => return Ok(TbValue::Unavailable),
    };
    Ok(TbValue::Value(v))
}

/// Direct encounter among `group` (C.07 Article 6): points each scored
/// against the others, averaged over repeated meetings. In Swiss events, if
/// not everyone met, only a player certain to lead the group is separated.
fn direct_encounter(
    ctx: &TiebreakContext,
    group: &[PlayerId],
    forfeits: bool,
) -> HashMap<PlayerId, i64> {
    let mut met: HashMap<(PlayerId, PlayerId), (i64, i64)> = HashMap::new();
    for &p in group {
        for r in ctx.recs(p) {
            let (opp, pts) = match r.kind {
                Kind::Played { opp, side, .. } => (opp, standard_hundredths(side)),
                Kind::Forfeit { opp, side } if forfeits => (opp, standard_hundredths(side)),
                _ => continue,
            };
            if group.contains(&opp) {
                let e = met.entry((p, opp)).or_insert((0, 0));
                e.0 += pts;
                e.1 += 1;
            }
        }
    }
    let scores: HashMap<PlayerId, i64> = group
        .iter()
        .map(|&p| {
            let total = group
                .iter()
                .filter(|o| **o != p)
                .filter_map(|o| met.get(&(p, *o)).map(|(s, n)| s / n))
                .sum();
            (p, total)
        })
        .collect();
    let all_met = group
        .iter()
        .all(|a| group.iter().all(|b| a == b || met.contains_key(&(*a, *b))));
    if all_met || ctx.tournament.kind.is_round_robin() {
        return scores;
    }
    // 6.3: only a player certain to lead, whatever the missing games, is separated.
    let max_possible = |p: PlayerId| {
        let missing = group
            .iter()
            .filter(|o| **o != p && !met.contains_key(&(p, **o)))
            .count() as i64;
        scores[&p] + missing * 100
    };
    let leader = group.iter().copied().find(|&p| {
        group
            .iter()
            .all(|&o| o == p || scores[&p] > max_possible(o))
    });
    group
        .iter()
        .map(|&p| (p, i64::from(Some(p) == leader) * 100))
        .collect()
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
    /// Column labels, e.g. `BH/C1`.
    pub tiebreak_labels: Vec<String>,
    pub rows: Vec<StandingRow>,
}

type Values = HashMap<PlayerId, Vec<TbValue>>;

/// Orders a tied `group` by the criteria from `level` on. Direct encounter
/// is computed for the group itself and re-applied to sub-ties (6.2).
fn resolve(
    group: Vec<PlayerId>,
    level: usize,
    specs: &[TiebreakSpec],
    ctx: &TiebreakContext,
    values: &mut Values,
    start_rank: &dyn Fn(PlayerId) -> u32,
    out: &mut Vec<PlayerId>,
) {
    if group.len() <= 1 || level >= specs.len() {
        let mut g = group;
        g.sort_by_key(|p| start_rank(*p));
        out.extend(g);
        return;
    }
    if specs[level].kind == TiebreakKind::DirectEncounter {
        let de = direct_encounter(ctx, &group, specs[level].modifiers.forfeits_as_played);
        for (p, v) in de {
            if let Some(row) = values.get_mut(&p) {
                row[level] = TbValue::Value(v);
            }
        }
    }
    let mut g = group;
    g.sort_by(|a, b| values[b][level].cmp(&values[a][level]));
    let total = g.len();
    let mut i = 0;
    while i < g.len() {
        let mut j = i + 1;
        while j < g.len() && values[&g[j]][level] == values[&g[i]][level] {
            j += 1;
        }
        let sub: Vec<PlayerId> = g[i..j].to_vec();
        let next = if specs[level].kind == TiebreakKind::DirectEncounter
            && sub.len() > 1
            && sub.len() < total
        {
            level
        } else {
            level + 1
        };
        resolve(sub, next, specs, ctx, values, start_rank, out);
        i = j;
    }
}

/// Individual standings after `after_round` rounds.
pub fn standings(t: &Tournament, after_round: RoundNo) -> Result<Standings> {
    let ctx = TiebreakContext::new(t, after_round);
    let specs = &t.settings.tiebreaks;
    let mut values: Values = HashMap::new();
    for p in &t.players {
        let mut row = Vec::with_capacity(specs.len());
        for spec in specs {
            row.push(compute(spec, &ctx, p.id)?);
        }
        values.insert(p.id, row);
    }
    let start_rank = |id: PlayerId| t.player(id).map_or(u32::MAX, |p| p.starting_rank);
    let mut order: Vec<PlayerId> = t.players.iter().map(|p| p.id).collect();
    order.sort_by(|a, b| {
        ctx.scores[b]
            .cmp(&ctx.scores[a])
            .then(start_rank(*a).cmp(&start_rank(*b)))
    });
    let mut sorted = Vec::with_capacity(order.len());
    let mut i = 0;
    while i < order.len() {
        let mut j = i + 1;
        while j < order.len() && ctx.scores[&order[j]] == ctx.scores[&order[i]] {
            j += 1;
        }
        resolve(
            order[i..j].to_vec(),
            0,
            specs,
            &ctx,
            &mut values,
            &start_rank,
            &mut sorted,
        );
        i = j;
    }
    let mut rows: Vec<StandingRow> = sorted
        .into_iter()
        .map(|p| StandingRow {
            rank: 0,
            tied_with_previous: false,
            player: p,
            points: ctx.scores[&p],
            tiebreaks: values.remove(&p).unwrap_or_default(),
        })
        .collect();
    for i in 0..rows.len() {
        rows[i].rank = i as u32 + 1;
        if i > 0 {
            rows[i].tied_with_previous =
                rows[i].points == rows[i - 1].points && rows[i].tiebreaks == rows[i - 1].tiebreaks;
        }
    }
    Ok(Standings {
        after_round,
        tiebreak_labels: specs.iter().map(TiebreakSpec::label).collect(),
        rows,
    })
}

#[cfg(test)]
mod tests;
