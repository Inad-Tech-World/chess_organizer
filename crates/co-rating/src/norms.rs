//! Title norm checks (FIDE Handbook B.01, Title Regulations).
//!
//! The general requirements are checked: games, federation mix, titled
//! opponents, rating floor, average opponent rating and performance. Special
//! cases the regulations allow (national championships, events with many
//! foreign titled players, team events) are left to the arbiter, and every
//! unmet requirement is listed in plain language.

use crate::tables;
use co_core::{PlayerId, Result, Title, Tournament};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NormTitle {
    GM,
    IM,
    WGM,
    WIM,
}

impl NormTitle {
    pub const ALL: [NormTitle; 4] = [NormTitle::GM, NormTitle::IM, NormTitle::WGM, NormTitle::WIM];

    /// (minimum performance, minimum average opponent rating, rating floor).
    fn thresholds(self) -> (i32, i32, u16) {
        match self {
            NormTitle::GM => (2600, 2380, 2200),
            NormTitle::IM => (2450, 2230, 2050),
            NormTitle::WGM => (2400, 2180, 2000),
            NormTitle::WIM => (2250, 2030, 1850),
        }
    }

    /// Titles that count towards the "one third" requirement.
    fn qualifying_opponent(self, t: Title) -> bool {
        match self {
            NormTitle::GM => t == Title::GM,
            NormTitle::IM => matches!(t, Title::GM | Title::IM),
            NormTitle::WGM => matches!(t, Title::GM | Title::IM | Title::WGM),
            NormTitle::WIM => matches!(t, Title::GM | Title::IM | Title::WGM | Title::WIM),
        }
    }

    fn holder_titles(self) -> &'static [Title] {
        match self {
            NormTitle::GM => &[Title::GM],
            NormTitle::IM => &[Title::GM, Title::IM],
            NormTitle::WGM => &[Title::GM, Title::IM, Title::WGM],
            NormTitle::WIM => &[Title::GM, Title::IM, Title::WGM, Title::WIM],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NormReport {
    pub player: PlayerId,
    pub title: NormTitle,
    pub achieved: bool,
    pub games: u16,
    pub performance: u16,
    pub average_opponent_rating: u16,
    /// Requirements that were not met, in plain language.
    pub shortfalls: Vec<String>,
}

/// Checks every title norm for `player` over the rounds played so far.
pub fn norm_check(t: &Tournament, player: PlayerId) -> Result<Vec<NormReport>> {
    let Some(p) = t.player(player) else {
        return Ok(Vec::new());
    };
    let rt = t.settings.rating_type;
    let games = crate::rated_games(t, player);
    let own_fed = p.federation.clone().unwrap_or_default();
    let mut reports = Vec::new();
    for title in NormTitle::ALL {
        if matches!(title, NormTitle::WGM | NormTitle::WIM) && p.sex != Some(co_core::Sex::Female) {
            continue;
        }
        if p.title
            .is_some_and(|pt| title.holder_titles().contains(&pt))
        {
            continue; // already holds this title or a higher one
        }
        let (min_perf, min_ra, floor) = title.thresholds();
        let mut shortfalls = Vec::new();
        let n = games.len();
        if n < 9 {
            shortfalls.push(format!(
                "needs at least 9 games against rated opponents (has {n})"
            ));
        }
        // Federations.
        let feds: Vec<String> = games
            .iter()
            .map(|(o, _, _)| {
                t.player(*o)
                    .and_then(|x| x.federation.clone())
                    .unwrap_or_default()
            })
            .collect();
        let own_count = feds.iter().filter(|f| **f == own_fed).count();
        let mut distinct: Vec<&String> = feds.iter().filter(|f| **f != own_fed).collect();
        distinct.sort();
        distinct.dedup();
        if distinct.len() < 2 {
            shortfalls.push(
                "opponents must come from at least two federations other than the player's".into(),
            );
        }
        if n > 0 && own_count * 5 > n * 3 {
            shortfalls.push(
                "at most 3/5 of the opponents may come from the player's own federation".into(),
            );
        }
        let max_one_fed = feds
            .iter()
            .map(|f| feds.iter().filter(|g| *g == f).count())
            .max()
            .unwrap_or(0);
        if n > 0 && max_one_fed * 3 > n * 2 {
            shortfalls.push("at most 2/3 of the opponents may come from one federation".into());
        }
        // Titled opponents.
        let titles: Vec<Option<Title>> = games
            .iter()
            .map(|(o, _, _)| t.player(*o).and_then(|x| x.title))
            .collect();
        let titled = titles
            .iter()
            .filter(|tt| {
                matches!(
                    tt,
                    Some(Title::GM | Title::IM | Title::FM | Title::WGM | Title::WIM | Title::WFM)
                )
            })
            .count();
        if titled * 2 < n {
            shortfalls.push(format!(
                "at least half of the opponents must hold a title ({titled} of {n})"
            ));
        }
        let qualifying = titles
            .iter()
            .filter(|tt| tt.is_some_and(|x| title.qualifying_opponent(x)))
            .count();
        if qualifying < 3 || qualifying * 3 < n {
            shortfalls.push(format!(
                "at least one third (minimum 3) of the opponents must be {} ({qualifying} of {n})",
                match title {
                    NormTitle::GM => "GMs",
                    NormTitle::IM => "IMs or GMs",
                    NormTitle::WGM => "WGMs, IMs or GMs",
                    NormTitle::WIM => "WIMs, WGMs, IMs or GMs",
                }
            ));
        }
        // Ratings: one opponent below the floor is raised to it; unrated opponents count at the floor once.
        let mut ratings: Vec<i32> = games
            .iter()
            .map(|(_, r, _)| i32::from(r.unwrap_or(0)))
            .collect();
        ratings.sort_unstable();
        let below = ratings.iter().filter(|&&r| r < i32::from(floor)).count();
        if below > 1 {
            shortfalls.push(format!(
                "only one opponent may be rated below the {floor} floor ({below} are)"
            ));
        }
        if let Some(first) = ratings.first_mut()
            && *first < i32::from(floor)
        {
            *first = i32::from(floor);
        }
        let ra = if n > 0 {
            (ratings.iter().sum::<i32>() as f64 / n as f64).round() as i32
        } else {
            0
        };
        let score: f64 = games.iter().map(|(_, _, s)| s.as_f64()).sum();
        let perf = if n > 0 {
            ra + tables::dp(score / n as f64)
        } else {
            0
        };
        if ra < min_ra {
            shortfalls.push(format!("average opponent rating {ra} is below {min_ra}"));
        }
        if perf < min_perf {
            shortfalls.push(format!("performance {perf} is below {min_perf}"));
        }
        let _ = rt;
        reports.push(NormReport {
            player,
            title,
            achieved: shortfalls.is_empty(),
            games: n as u16,
            performance: perf.max(0) as u16,
            average_opponent_rating: ra.max(0) as u16,
            shortfalls,
        });
    }
    Ok(reports)
}
