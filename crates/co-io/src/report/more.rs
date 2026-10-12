//! Crosstables, scorecards, board cards, prize lists, rating report and the
//! team lists.

use super::{Align, Column, Report, col, empty, fmt_points, rating_str, title_str};
use crate::Result;
use co_core::{
    ByeKind, Color, Pairing, Player, PlayerId, PrizeCategory, Round, RoundNo, SideResult,
    Tournament, scoring,
};
use co_i18n::Translator;
use std::collections::{HashMap, HashSet};

fn side_code(side: SideResult) -> &'static str {
    match side {
        SideResult::Win | SideResult::UnratedWin => "1",
        SideResult::Draw | SideResult::UnratedDraw => "½",
        SideResult::Loss | SideResult::UnratedLoss => "0",
        SideResult::ForfeitWin => "+",
        SideResult::ForfeitLoss => "-",
    }
}

fn bye_code(kind: ByeKind) -> &'static str {
    match kind {
        ByeKind::PairingAllocated => "PAB",
        ByeKind::Full => "1 bye",
        ByeKind::Half => "½ bye",
        ByeKind::Zero => "–",
    }
}

fn name(t: &Tournament, id: PlayerId) -> String {
    t.player(id).map(|p| p.name.clone()).unwrap_or_default()
}

/// Cell for `player` in `round`: opponent number, colour and result.
fn cross_cell(round: &Round, player: PlayerId, number: &HashMap<PlayerId, u32>) -> String {
    match round.entry_for(player) {
        Some(g @ Pairing::Game { result, .. }) => {
            let (color, opp) = g.side_of(player).expect("game involves player");
            let c = if color == Color::White { "w" } else { "b" };
            let r = result.map_or("", |r| side_code(r.for_color(color)));
            format!("{}{c}{r}", number.get(&opp).copied().unwrap_or(0))
        }
        Some(Pairing::Bye { kind, .. }) => bye_code(*kind).to_string(),
        None => String::new(),
    }
}

pub(super) fn crosstable(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let s = co_tiebreak::standings(t, round)?;
    let title = tr.t_args("list-crosstable", &[("round", u32::from(round).into())]);
    if t.kind.is_round_robin() {
        return Ok(rr_crosstable(t, tr, round, title, &s));
    }
    let place: HashMap<PlayerId, u32> = s.rows.iter().map(|r| (r.player, r.rank)).collect();
    let mut columns = vec![
        col(tr, "col-rank", Align::Right, 4),
        col(tr, "col-name", Align::Left, 20),
        col(tr, "col-rating", Align::Right, 5),
        col(tr, "col-fed", Align::Center, 4),
    ];
    for r in 1..=round {
        columns.push(Column {
            label: format!("{r}"),
            align: Align::Center,
            width: 5,
        });
    }
    columns.push(col(tr, "col-points", Align::Right, 4));
    columns.extend(s.tiebreak_labels.iter().map(|l| Column {
        label: l.clone(),
        align: Align::Right,
        width: 5,
    }));
    let mut rep = empty(title, columns);
    for row in &s.rows {
        let Some(p) = t.player(row.player) else {
            continue;
        };
        let mut cells = vec![
            row.rank.to_string(),
            p.name.clone(),
            rating_str(t, p),
            p.federation.clone().unwrap_or_default(),
        ];
        for rd in t.rounds.iter().take(usize::from(round)) {
            cells.push(cross_cell(rd, p.id, &place));
        }
        cells.push(fmt_points(row.points));
        cells.extend(row.tiebreaks.iter().map(|v| v.display()));
        rep.rows.push(cells);
    }
    rep.ltr_table = true;
    Ok(rep)
}

/// Round robins print the classic matrix, players by starting rank.
fn rr_crosstable(
    t: &Tournament,
    tr: &Translator,
    round: RoundNo,
    title: String,
    s: &co_tiebreak::Standings,
) -> Report {
    let players = t.players_by_rank();
    let place: HashMap<PlayerId, u32> = s.rows.iter().map(|r| (r.player, r.rank)).collect();
    let mut columns = vec![
        col(tr, "col-start-rank", Align::Right, 3),
        col(tr, "col-name", Align::Left, 20),
        col(tr, "col-rating", Align::Right, 5),
    ];
    for i in 1..=players.len() {
        columns.push(Column {
            label: i.to_string(),
            align: Align::Center,
            width: 3,
        });
    }
    columns.push(col(tr, "col-points", Align::Right, 4));
    columns.push(col(tr, "col-rank", Align::Right, 3));
    let mut rep = empty(title, columns);
    let tbs: HashMap<PlayerId, Vec<String>> = s
        .rows
        .iter()
        .map(|r| (r.player, r.tiebreaks.iter().map(|v| v.display()).collect()))
        .collect();
    for (i, p) in players.iter().enumerate() {
        let mut cells = vec![(i + 1).to_string(), p.name.clone(), rating_str(t, p)];
        for (j, q) in players.iter().enumerate() {
            if i == j {
                cells.push("×".into());
                continue;
            }
            let results: Vec<&str> = t
                .rounds
                .iter()
                .take(usize::from(round))
                .filter_map(|rd| match rd.entry_for(p.id) {
                    Some(
                        g @ Pairing::Game {
                            result: Some(r), ..
                        },
                    ) => {
                        let (c, opp) = g.side_of(p.id)?;
                        (opp == q.id).then(|| side_code(r.for_color(c)))
                    }
                    _ => None,
                })
                .collect();
            cells.push(results.join(" "));
        }
        let points = s
            .rows
            .iter()
            .find(|r| r.player == p.id)
            .map(|r| r.points)
            .unwrap_or_default();
        cells.push(fmt_points(points));
        cells.push(place.get(&p.id).map(|r| r.to_string()).unwrap_or_default());
        let _ = &tbs;
        rep.rows.push(cells);
    }
    rep.ltr_table = true;
    rep
}

pub(super) fn scorecards(t: &Tournament, tr: &Translator) -> Report {
    let mut rep = empty(
        tr.t("list-scorecards"),
        vec![
            col(tr, "col-start-rank", Align::Right, 4),
            col(tr, "col-name", Align::Left, 18),
            col(tr, "col-round", Align::Right, 3),
            col(tr, "col-color", Align::Center, 4),
            col(tr, "col-opponent", Align::Left, 18),
            col(tr, "col-rating", Align::Right, 5),
            col(tr, "col-result", Align::Center, 5),
            col(tr, "col-points", Align::Right, 4),
        ],
    );
    let cumulative = scoring::cumulative_scores(t);
    for p in t.players_by_rank() {
        for (i, rd) in t.rounds.iter().enumerate() {
            let (color, opp, result) = match rd.entry_for(p.id) {
                Some(g @ Pairing::Game { result, .. }) => {
                    let (c, o) = g.side_of(p.id).expect("game involves player");
                    let cs = if c == Color::White {
                        tr.t("color-short-white")
                    } else {
                        tr.t("color-short-black")
                    };
                    (
                        cs,
                        Some(o),
                        result.map_or(String::new(), |r| side_code(r.for_color(c)).to_string()),
                    )
                }
                Some(Pairing::Bye { kind, .. }) => {
                    (String::new(), None, bye_code(*kind).to_string())
                }
                None => (String::new(), None, String::new()),
            };
            let opp_name = opp.map(|o| name(t, o)).unwrap_or_default();
            let opp_rating = opp
                .and_then(|o| t.player(o))
                .map(|q| rating_str(t, q))
                .unwrap_or_default();
            let score = cumulative[i].get(&p.id).copied().unwrap_or_default();
            rep.rows.push(vec![
                if i == 0 {
                    p.starting_rank.to_string()
                } else {
                    String::new()
                },
                if i == 0 {
                    p.name.clone()
                } else {
                    String::new()
                },
                rd.number.to_string(),
                color,
                opp_name,
                opp_rating,
                result,
                fmt_points(score),
            ]);
        }
    }
    rep
}

pub(super) fn board_cards(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let Some(rd) = t.round(round) else {
        return Err(co_core::Error::Invalid(format!("round {round} has not been paired")).into());
    };
    let mut rep = empty(
        tr.t_args(
            "list-board-cards-round",
            &[("round", u32::from(round).into())],
        ),
        vec![
            col(tr, "col-board", Align::Center, 4),
            col(tr, "col-white", Align::Left, 20),
            col(tr, "col-rating", Align::Right, 5),
            col(tr, "col-result", Align::Center, 8),
            col(tr, "col-black", Align::Left, 20),
            col(tr, "col-rating", Align::Right, 5),
        ],
    );
    let mut games: Vec<&Pairing> = rd.pairings.iter().filter(|p| p.board().is_some()).collect();
    games.sort_by_key(|p| p.board());
    for g in games {
        if let Pairing::Game {
            board,
            white,
            black,
            ..
        } = *g
        {
            let rating = |id| t.player(id).map(|p| rating_str(t, p)).unwrap_or_default();
            rep.rows.push(vec![
                board.to_string(),
                name(t, white),
                rating(white),
                "____ – ____".into(),
                name(t, black),
                rating(black),
            ]);
        }
    }
    rep.ltr_table = true;
    Ok(rep)
}

pub(super) fn rating_report(t: &Tournament, tr: &Translator) -> Result<Report> {
    let rows = co_rating::tournament_rating_changes(t)?;
    let mut rep = empty(
        tr.t("list-rating-report"),
        vec![
            col(tr, "col-start-rank", Align::Right, 4),
            col(tr, "col-name", Align::Left, 20),
            col(tr, "col-rating", Align::Right, 5),
            col(tr, "col-k", Align::Right, 3),
            col(tr, "col-games", Align::Right, 4),
            col(tr, "col-score", Align::Right, 4),
            col(tr, "col-expected", Align::Right, 5),
            col(tr, "col-change", Align::Right, 6),
            col(tr, "col-aro", Align::Right, 5),
            col(tr, "col-performance", Align::Right, 5),
        ],
    );
    for r in rows {
        let Some(p) = t.player(r.player) else {
            continue;
        };
        let change = match (r.rating, r.initial_rating_estimate) {
            (Some(_), _) => format!("{:+.1}", r.change),
            (None, Some(est)) => tr.t_args(
                "rating-initial-estimate",
                &[("rating", u32::from(est).into())],
            ),
            (None, None) => String::new(),
        };
        rep.rows.push(vec![
            p.starting_rank.to_string(),
            p.name.clone(),
            r.rating.map(|x| x.to_string()).unwrap_or_default(),
            r.k.map(|k| k.to_string()).unwrap_or_default(),
            r.games.to_string(),
            format!("{}", r.score),
            if r.rating.is_some() {
                format!("{:.2}", r.expected)
            } else {
                String::new()
            },
            change,
            r.average_opponent_rating
                .map(|x| x.to_string())
                .unwrap_or_default(),
            r.performance.map(|x| x.to_string()).unwrap_or_default(),
        ]);
    }
    rep.notes.push(tr.t("rating-report-note"));
    Ok(rep)
}

fn eligible(p: &Player, c: &PrizeCategory, t: &Tournament) -> bool {
    let f = &c.filter;
    let rating = p.rating(t.settings.rating_type);
    if f.unrated_only && rating.is_some() {
        return false;
    }
    let r = rating.unwrap_or(0);
    let born = p.birth_year();
    !(f.max_rating.is_some_and(|max| r > max)
        || f.min_rating.is_some_and(|min| r < min)
        || (f.sex.is_some() && p.sex != f.sex)
        || f.born_from
            .is_some_and(|from| born.is_none_or(|y| y < from))
        || f.born_until
            .is_some_and(|until| born.is_none_or(|y| y > until))
        || f.federation
            .as_ref()
            .is_some_and(|fed| p.federation.as_deref() != Some(fed.as_str()))
        || f.club
            .as_ref()
            .is_some_and(|club| p.club.as_deref() != Some(club.as_str())))
}

/// Prize winners: categories in their listed order, each player wins at most one prize.
pub fn prize_winners(t: &Tournament) -> Result<Vec<(usize, usize, PlayerId)>> {
    let s = co_tiebreak::standings(t, t.paired_rounds())?;
    let mut awarded: HashSet<PlayerId> = HashSet::new();
    let mut out = Vec::new();
    for (ci, c) in t.prize_categories.iter().enumerate() {
        let mut place = 0;
        for row in &s.rows {
            if place >= c.prizes.len() {
                break;
            }
            let Some(p) = t.player(row.player) else {
                continue;
            };
            if awarded.contains(&p.id) || !eligible(p, c, t) {
                continue;
            }
            awarded.insert(p.id);
            out.push((ci, place, p.id));
            place += 1;
        }
    }
    Ok(out)
}

pub(super) fn prizes(t: &Tournament, tr: &Translator) -> Result<Report> {
    let s = co_tiebreak::standings(t, t.paired_rounds())?;
    let points: HashMap<PlayerId, String> = s
        .rows
        .iter()
        .map(|r| (r.player, fmt_points(r.points)))
        .collect();
    let rank: HashMap<PlayerId, u32> = s.rows.iter().map(|r| (r.player, r.rank)).collect();
    let mut rep = empty(
        tr.t("list-prizes"),
        vec![
            col(tr, "col-category", Align::Left, 14),
            col(tr, "col-place", Align::Right, 4),
            col(tr, "col-prize", Align::Left, 12),
            col(tr, "col-name", Align::Left, 20),
            col(tr, "col-rank", Align::Right, 4),
            col(tr, "col-points", Align::Right, 4),
        ],
    );
    for (ci, place, pid) in prize_winners(t)? {
        let c = &t.prize_categories[ci];
        rep.rows.push(vec![
            if place == 0 {
                c.name.clone()
            } else {
                String::new()
            },
            (place + 1).to_string(),
            c.prizes[place].clone(),
            name(t, pid),
            rank.get(&pid).map(|r| r.to_string()).unwrap_or_default(),
            points.get(&pid).cloned().unwrap_or_default(),
        ]);
    }
    rep.notes.push(tr.t("prizes-one-per-player"));
    Ok(rep)
}

pub(super) fn team_rosters(t: &Tournament, tr: &Translator) -> Report {
    let mut rep = empty(
        tr.t("list-team-roster"),
        vec![
            col(tr, "col-team", Align::Left, 16),
            col(tr, "col-board", Align::Right, 3),
            col(tr, "col-title", Align::Left, 4),
            col(tr, "col-name", Align::Left, 20),
            col(tr, "col-fed", Align::Center, 4),
            col(tr, "col-rating", Align::Right, 5),
        ],
    );
    for team in &t.teams {
        for (i, id) in team.members.iter().enumerate() {
            let Some(p) = t.player(*id) else { continue };
            rep.rows.push(vec![
                if i == 0 {
                    team.name.clone()
                } else {
                    String::new()
                },
                (i + 1).to_string(),
                title_str(p),
                p.name.clone(),
                p.federation.clone().unwrap_or_default(),
                rating_str(t, p),
            ]);
        }
    }
    rep
}

pub(super) fn team_pairings(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let Some(rd) = t.round(round) else {
        return Err(co_core::Error::Invalid(format!("round {round} has not been paired")).into());
    };
    let team_name = |id| t.team(id).map(|x| x.name.clone()).unwrap_or_default();
    let ps = &t.settings.points;
    let mut rep = empty(
        tr.t_args("list-team-pairings", &[("round", u32::from(round).into())]),
        vec![
            col(tr, "col-board", Align::Right, 4),
            col(tr, "col-white", Align::Left, 20),
            col(tr, "col-result", Align::Center, 7),
            col(tr, "col-black", Align::Left, 20),
        ],
    );
    for m in &rd.matches {
        match m.away {
            None => rep.rows.push(vec![
                m.number.to_string(),
                team_name(m.home),
                String::new(),
                tr.t("bye-pairing-allocated"),
            ]),
            Some(away) => {
                let (mut hg, mut ag) = (co_core::Points::ZERO, co_core::Points::ZERO);
                for g in &m.games {
                    if let Pairing::Game {
                        white,
                        result: Some(r),
                        ..
                    } = *g
                    {
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
                }
                rep.rows.push(vec![
                    format!("{}", m.number),
                    team_name(m.home),
                    format!("{} – {}", fmt_points(hg), fmt_points(ag)),
                    team_name(away),
                ]);
                for g in &m.games {
                    if let Pairing::Game {
                        board,
                        white,
                        black,
                        result,
                    } = *g
                    {
                        rep.rows.push(vec![
                            format!("{}.{board}", m.number),
                            name(t, white),
                            result.map(|r| r.code()).unwrap_or_default(),
                            name(t, black),
                        ]);
                    }
                }
            }
        }
    }
    rep.ltr_table = true;
    Ok(rep)
}

pub(super) fn team_standings(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let s = co_tiebreak::team_standings(t, round)?;
    let mut columns = vec![
        col(tr, "col-rank", Align::Right, 4),
        col(tr, "col-team", Align::Left, 20),
        col(tr, "col-mp", Align::Right, 4),
        col(tr, "col-gp", Align::Right, 4),
    ];
    columns.extend(s.tiebreak_labels.iter().map(|l| Column {
        label: l.clone(),
        align: Align::Right,
        width: 5,
    }));
    let mut rep = empty(
        tr.t_args("list-team-standings", &[("round", u32::from(round).into())]),
        columns,
    );
    for row in &s.rows {
        let mut cells = vec![
            if row.tied_with_previous {
                String::new()
            } else {
                row.rank.to_string()
            },
            t.team(row.team).map(|x| x.name.clone()).unwrap_or_default(),
            fmt_points(row.match_points),
            fmt_points(row.game_points),
        ];
        cells.extend(row.tiebreaks.iter().map(|v| v.display()));
        rep.rows.push(cells);
    }
    Ok(rep)
}
