//! Printable lists as renderer-neutral tables.
//!
//! Each list is built once as a [`Report`]; the renderers in
//! [`crate::render`] turn it into HTML (screen, print, publishing site), PDF,
//! CSV or XLSX, so every output shows the same content.

mod more;
pub use more::prize_winners;

use crate::Result;
use co_core::{ByeKind, Pairing, Player, PlayerId, Points, RoundNo, Tournament, scoring};
use co_i18n::Translator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ListKind {
    StartingRank,
    Alphabetical,
    Pairings { round: RoundNo },
    Standings { round: RoundNo },
    Crosstable { round: RoundNo },
    Scorecards,
    BoardCards { round: RoundNo },
    Prizes,
    TeamRosters,
    RatingReport,
    TeamPairings { round: RoundNo },
    TeamStandings { round: RoundNo },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub label: String,
    pub align: Align,
    /// Relative width hint for paged output.
    pub width: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub lang: String,
    pub rtl: bool,
    /// Tournament name.
    pub heading: String,
    /// List name, e.g. "Standings after round 5".
    pub title: String,
    /// Label/value lines under the heading (city, dates, arbiter).
    pub info: Vec<(String, String)>,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
    pub notes: Vec<String>,
    pub footer: String,
    /// Keep the table left-to-right in RTL languages (pairings: White left).
    pub ltr_table: bool,
}

/// Builds a list in the language of `tr`.
pub fn build(kind: ListKind, t: &Tournament, tr: &Translator) -> Result<Report> {
    let mut report = match kind {
        ListKind::StartingRank => player_list(t, tr, false),
        ListKind::Alphabetical => player_list(t, tr, true),
        ListKind::Pairings { round } => pairings(t, tr, round)?,
        ListKind::Standings { round } => standings(t, tr, round)?,
        ListKind::Crosstable { round } => more::crosstable(t, tr, round)?,
        ListKind::Scorecards => more::scorecards(t, tr),
        ListKind::BoardCards { round } => more::board_cards(t, tr, round)?,
        ListKind::RatingReport => more::rating_report(t, tr)?,
        ListKind::Prizes => more::prizes(t, tr)?,
        ListKind::TeamRosters => more::team_rosters(t, tr),
        ListKind::TeamPairings { round } => more::team_pairings(t, tr, round)?,
        ListKind::TeamStandings { round } => more::team_standings(t, tr, round)?,
    };
    report.lang = tr.lang().to_string();
    report.rtl = tr.is_rtl();
    report.heading = t.info.name.clone();
    report.info = info_lines(t, tr);
    report.footer = tr.t("footer-generated");
    Ok(report)
}

pub(crate) fn empty(title: String, columns: Vec<Column>) -> Report {
    Report {
        lang: String::new(),
        rtl: false,
        heading: String::new(),
        title,
        info: Vec::new(),
        columns,
        rows: Vec::new(),
        notes: Vec::new(),
        footer: String::new(),
        ltr_table: false,
    }
}

pub(crate) fn col(tr: &Translator, id: &str, align: Align, width: u16) -> Column {
    Column {
        label: tr.t(id),
        align,
        width,
    }
}

fn info_lines(t: &Tournament, tr: &Translator) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if !t.info.city.is_empty() {
        out.push((tr.t("info-city"), t.info.city.clone()));
    }
    let fmt = |d: co_core::chrono::NaiveDate| d.format("%Y-%m-%d").to_string();
    match (t.info.start_date, t.info.end_date) {
        (Some(s), Some(e)) if s != e => {
            out.push((tr.t("info-dates"), format!("{} – {}", fmt(s), fmt(e))))
        }
        (Some(s), _) => out.push((tr.t("info-dates"), fmt(s))),
        _ => {}
    }
    if !t.info.chief_arbiter.is_empty() {
        out.push((tr.t("info-chief-arbiter"), t.info.chief_arbiter.clone()));
    }
    if !t.info.time_control.is_empty() {
        out.push((tr.t("info-time-control"), t.info.time_control.clone()));
    }
    out
}

/// Points for print: `3`, `2½`, `½`, or a decimal for other point systems.
pub fn fmt_points(p: Points) -> String {
    let (whole, tenths) = (p.0 / 10, p.0.rem_euclid(10));
    match tenths {
        0 => whole.to_string(),
        5 if whole == 0 => "½".to_string(),
        5 => format!("{whole}½"),
        _ => p.to_string(),
    }
}

pub(crate) fn title_str(p: &Player) -> String {
    p.title.map(|t| t.as_str().to_string()).unwrap_or_default()
}

pub(crate) fn rating_str(t: &Tournament, p: &Player) -> String {
    p.rating(t.settings.rating_type)
        .map(|r| r.to_string())
        .unwrap_or_default()
}

fn player_list(t: &Tournament, tr: &Translator, alphabetical: bool) -> Report {
    let mut players = t.players_by_rank();
    if alphabetical {
        players.sort_by_key(|p| p.name.to_lowercase());
    }
    let title = tr.t(if alphabetical {
        "list-alphabetical"
    } else {
        "list-starting-rank"
    });
    let mut r = empty(
        title,
        vec![
            col(tr, "col-start-rank", Align::Right, 4),
            col(tr, "col-title", Align::Left, 4),
            col(tr, "col-name", Align::Left, 24),
            col(tr, "col-fed", Align::Center, 4),
            col(tr, "col-rating", Align::Right, 5),
            col(tr, "col-fide-id", Align::Right, 9),
            col(tr, "col-club", Align::Left, 16),
        ],
    );
    r.rows = players
        .into_iter()
        .map(|p| {
            vec![
                p.starting_rank.to_string(),
                title_str(p),
                p.name.clone(),
                p.federation.clone().unwrap_or_default(),
                rating_str(t, p),
                p.fide_id.map(|id| id.to_string()).unwrap_or_default(),
                p.club.clone().unwrap_or_default(),
            ]
        })
        .collect();
    r
}

fn pairings(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let Some(rd) = t.round(round) else {
        return Err(co_core::Error::Invalid(format!("round {round} has not been paired")).into());
    };
    let before = scoring::scores_after(t, round - 1);
    let mut r = empty(
        tr.t_args("list-pairings", &[("round", u32::from(round).into())]),
        vec![
            col(tr, "col-board", Align::Right, 3),
            col(tr, "col-start-rank", Align::Right, 4),
            col(tr, "col-white", Align::Left, 22),
            col(tr, "col-rating", Align::Right, 5),
            col(tr, "col-points", Align::Center, 4),
            col(tr, "col-result", Align::Center, 6),
            col(tr, "col-points", Align::Center, 4),
            col(tr, "col-start-rank", Align::Right, 4),
            col(tr, "col-black", Align::Left, 22),
            col(tr, "col-rating", Align::Right, 5),
        ],
    );
    r.ltr_table = true;
    let name = |id: PlayerId| t.player(id).map(|p| p.name.clone()).unwrap_or_default();
    let rank = |id: PlayerId| {
        t.player(id)
            .map(|p| p.starting_rank.to_string())
            .unwrap_or_default()
    };
    let rating = |id: PlayerId| t.player(id).map(|p| rating_str(t, p)).unwrap_or_default();
    let pts = |id: PlayerId| fmt_points(before.get(&id).copied().unwrap_or_default());

    let mut games: Vec<&Pairing> = rd.pairings.iter().filter(|p| p.board().is_some()).collect();
    games.sort_by_key(|p| p.board());
    for g in games {
        if let Pairing::Game {
            board,
            white,
            black,
            result,
        } = *g
        {
            r.rows.push(vec![
                board.to_string(),
                rank(white),
                name(white),
                rating(white),
                pts(white),
                result.map(|x| x.code()).unwrap_or_default(),
                pts(black),
                rank(black),
                name(black),
                rating(black),
            ]);
        }
    }
    for p in &rd.pairings {
        if let Pairing::Bye { player, kind } = *p {
            let label = match kind {
                ByeKind::PairingAllocated => "bye-pairing-allocated",
                ByeKind::Half => "bye-half",
                ByeKind::Full => "bye-full",
                ByeKind::Zero => "bye-zero",
            };
            r.rows.push(vec![
                String::new(),
                rank(player),
                name(player),
                rating(player),
                pts(player),
                fmt_points(scoring::bye_points(kind, &t.settings.points)),
                String::new(),
                String::new(),
                tr.t(label),
                String::new(),
            ]);
        }
    }
    Ok(r)
}

fn standings(t: &Tournament, tr: &Translator, round: RoundNo) -> Result<Report> {
    let s = co_tiebreak::standings(t, round)?;
    let mut columns = vec![
        col(tr, "col-rank", Align::Right, 4),
        col(tr, "col-start-rank", Align::Right, 4),
        col(tr, "col-title", Align::Left, 4),
        col(tr, "col-name", Align::Left, 24),
        col(tr, "col-fed", Align::Center, 4),
        col(tr, "col-rating", Align::Right, 5),
        col(tr, "col-points", Align::Right, 5),
    ];
    columns.extend(s.tiebreak_labels.iter().map(|l| Column {
        label: l.clone(),
        align: Align::Right,
        width: 5,
    }));
    let mut r = empty(
        tr.t_args("list-standings", &[("round", u32::from(round).into())]),
        columns,
    );
    for row in &s.rows {
        let Some(p) = t.player(row.player) else {
            continue;
        };
        let mut cells = vec![
            if row.tied_with_previous {
                String::new()
            } else {
                row.rank.to_string()
            },
            p.starting_rank.to_string(),
            title_str(p),
            p.name.clone(),
            p.federation.clone().unwrap_or_default(),
            rating_str(t, p),
            fmt_points(row.points),
        ];
        cells.extend(row.tiebreaks.iter().map(|v| v.display()));
        r.rows.push(cells);
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_points() {
        assert_eq!(fmt_points(Points(30)), "3");
        assert_eq!(fmt_points(Points(25)), "2½");
        assert_eq!(fmt_points(Points(5)), "½");
        assert_eq!(fmt_points(Points(33)), "3.3");
    }
}
