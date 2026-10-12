use super::columns::LineBuf;
use super::{bye_char, result_char, round_col, title_code, type_code};
use co_core::*;
use std::collections::{BTreeMap, HashMap};

/// TRF player ids: position in starting-rank order, from 1.
pub fn trf_ids(t: &Tournament) -> HashMap<PlayerId, u32> {
    t.players_by_rank()
        .into_iter()
        .enumerate()
        .map(|(i, p)| (p.id, i as u32 + 1))
        .collect()
}

/// Writes the tournament as TRF-2026.
pub fn write(t: &Tournament) -> String {
    let ids = trf_ids(t);
    let rt = t.settings.rating_type;
    let mut out: Vec<String> = Vec::new();
    let mut text = |code: &str, value: &str| {
        if !value.is_empty() {
            out.push(format!("{code} {value}"));
        }
    };
    let date = |d: Option<chrono::NaiveDate>| {
        d.map(|d| d.format("%Y/%m/%d").to_string())
            .unwrap_or_default()
    };

    text("012", &t.info.name);
    text("022", &t.info.city);
    text("032", &t.info.federation);
    text("042", &date(t.info.start_date));
    text("052", &date(t.info.end_date));
    text("062", &t.players.len().to_string());
    let rated = t
        .players
        .iter()
        .filter(|p| p.ratings.fide(rt).is_some())
        .count();
    text("072", &rated.to_string());
    if t.kind.is_team() {
        text("082", &t.teams.len().to_string());
    }
    text("092", &kind_description(t.kind));
    text("102", &t.info.chief_arbiter);
    for deputy in &t.info.deputy_arbiters {
        text("112", deputy);
    }
    text("122", &t.info.time_control);

    if t.info.round_dates.iter().any(Option::is_some) {
        let mut b = LineBuf::new("132");
        for (i, d) in t.info.round_dates.iter().enumerate() {
            if let Some(d) = d {
                let c = round_col(i as RoundNo + 1);
                b.put_left(c, c + 7, &d.format("%y/%m/%d").to_string());
            }
        }
        out.push(b.finish());
    }
    out.push(format!("142 {}", t.settings.rounds));
    out.push(format!(
        "152 {}",
        match t.settings.initial_color {
            Color::White => "W",
            Color::Black => "B",
        }
    ));
    if t.settings.points != PointSystem::default() {
        out.push(point_system_line(&t.settings.points));
    }
    out.push(format!(
        "192 {}",
        type_code(
            t.kind,
            t.settings.acceleration,
            t.settings.team.primary_score
        )
    ));
    if !t.settings.tiebreaks.is_empty() {
        let codes: Vec<String> = t
            .settings
            .tiebreaks
            .iter()
            .map(TiebreakSpec::label)
            .collect();
        out.push(format!("202 {}", codes.join(",")));
    }

    let ranks: HashMap<PlayerId, u32> = co_tiebreak::standings(t, t.paired_rounds())
        .map(|s| s.rows.iter().map(|r| (r.player, r.rank)).collect())
        .unwrap_or_default();
    let indexes: Vec<_> = t.rounds.iter().map(|r| r.index()).collect();
    let scores = scoring::scores_after(t, t.paired_rounds());
    for p in t.players_by_rank() {
        let rank = ranks.get(&p.id).copied().unwrap_or(0);
        out.push(player_line(t, p, &ids, &indexes, scores[&p.id], rank));
    }

    // Requested byes for rounds not yet paired (needed for in-tournament exchange).
    let mut future: BTreeMap<(RoundNo, char), Vec<u32>> = BTreeMap::new();
    for p in &t.players {
        for (&round, &kind) in &p.byes {
            if round > t.paired_rounds() && kind != ByeKind::PairingAllocated {
                future
                    .entry((round, bye_char(kind)))
                    .or_default()
                    .push(ids[&p.id]);
            }
        }
    }
    for ((round, kind), mut players) in future {
        players.sort_unstable();
        let mut b = LineBuf::new("240");
        b.put_char(5, kind);
        b.put_right(7, 9, &format!("{round:03}"));
        put_ids(&mut b, 11, &players);
        out.push(b.finish());
    }

    for (points, round, first, last) in acceleration_ranges(t, &ids) {
        let mut b = LineBuf::new("250");
        b.put_right(10, 13, &points.to_string());
        b.put_right(15, 17, &format!("{round:03}"));
        b.put_right(19, 21, &format!("{round:03}"));
        b.put_right(23, 26, &format!("{first:04}"));
        b.put_right(28, 31, &format!("{last:04}"));
        out.push(b.finish());
    }

    for group in &t.settings.forbidden_groups {
        let mut members: Vec<u32> = group
            .members
            .iter()
            .filter_map(|m| ids.get(m).copied())
            .collect();
        if members.len() < 2 {
            continue;
        }
        members.sort_unstable();
        let mut b = LineBuf::new("260");
        b.put_right(5, 7, "001");
        b.put_right(9, 11, &format!("{:03}", t.settings.rounds));
        put_ids(&mut b, 13, &members);
        out.push(b.finish());
    }

    // TRF-2026 team records: team number, name and members' pairing numbers
    // in board order. The optional summary columns (42-72) are left blank;
    // they are recomputed from the games by any reader.
    for (i, team) in t.teams.iter().enumerate() {
        let members: Vec<u32> = team
            .members
            .iter()
            .filter_map(|m| ids.get(m).copied())
            .collect();
        let mut b = LineBuf::new("310");
        b.put_right(5, 7, &format!("{:03}", i + 1));
        b.put_left(9, 40, &team.name);
        put_ids(&mut b, 74, &members);
        out.push(b.finish());
    }

    let mut s = out.join("\n");
    s.push('\n');
    s
}

fn player_line(
    t: &Tournament,
    p: &Player,
    ids: &HashMap<PlayerId, u32>,
    indexes: &[HashMap<PlayerId, &Pairing>],
    points: Points,
    rank: u32,
) -> String {
    let mut b = LineBuf::new("001");
    b.put_right(5, 8, &ids[&p.id].to_string());
    match p.sex {
        Some(Sex::Male) => b.put_char(10, 'm'),
        Some(Sex::Female) => b.put_char(10, 'w'),
        None => {}
    }
    if let Some(title) = p.title {
        b.put_right(11, 13, title_code(title));
    }
    b.put_left(15, 47, &p.name);
    if let Some(r) = p.ratings.fide(t.settings.rating_type) {
        b.put_right(49, 52, &r.to_string());
    }
    if let Some(fed) = &p.federation {
        b.put_left(54, 56, fed);
    }
    if let Some(id) = p.fide_id {
        b.put_right(58, 68, &id.to_string());
    }
    if let Some(birth) = &p.birth_date {
        b.put_left(70, 79, birth);
    }
    b.put_right(81, 84, &points.to_string());
    b.put_right(86, 89, &rank.to_string());

    for (round, index) in t.rounds.iter().zip(indexes) {
        let c = round_col(round.number);
        let (opponent, color, result) = match index.get(&p.id).copied() {
            Some(game @ Pairing::Game { result, .. }) => {
                let (color, opp) = game.side_of(p.id).expect("game involves player");
                let res = result.map_or(' ', |r| result_char(r.for_color(color)));
                let c = if color == Color::White { 'w' } else { 'b' };
                (ids[&opp].to_string(), c, res)
            }
            Some(Pairing::Bye { kind, .. }) => ("0000".to_string(), '-', bye_char(*kind)),
            None if round.number < p.joined_round => {
                ("0000".to_string(), '-', bye_char(p.late_entry_bye))
            }
            // A team bye: every member of the team gets the pairing-allocated bye.
            None if round.matches.iter().any(|m| {
                m.away.is_none() && t.team(m.home).is_some_and(|tm| tm.members.contains(&p.id))
            }) =>
            {
                ("0000".to_string(), '-', bye_char(ByeKind::PairingAllocated))
            }
            None => ("0000".to_string(), '-', 'Z'),
        };
        b.put_right(c, c + 3, &opponent);
        b.put_char(c + 5, color);
        b.put_char(c + 7, result);
    }
    b.finish()
}

/// Record 162 with W, D, L, A, P (columns 6 + 9k).
fn point_system_line(ps: &PointSystem) -> String {
    let mut b = LineBuf::new("162");
    let entries = [
        ('W', ps.win),
        ('D', ps.draw),
        ('L', ps.loss),
        ('A', ps.zero_bye),
        ('P', ps.pairing_bye),
    ];
    for (k, (symbol, points)) in entries.into_iter().enumerate() {
        let col = 6 + 9 * k;
        b.put_char(col, symbol);
        b.put_right(col + 1, col + 4, &points.to_string());
    }
    b.finish()
}

fn put_ids(b: &mut LineBuf, start: usize, ids: &[u32]) {
    for (i, id) in ids.iter().enumerate() {
        let c = start + 5 * i;
        b.put_right(c, c + 3, &id.to_string());
    }
}

/// Groups acceleration points into (points, round, first id, last id) runs.
fn acceleration_ranges(
    t: &Tournament,
    ids: &HashMap<PlayerId, u32>,
) -> Vec<(Points, RoundNo, u32, u32)> {
    let mut by_round: BTreeMap<(RoundNo, Points), Vec<u32>> = BTreeMap::new();
    for p in &t.players {
        for (i, pts) in p.acceleration.iter().enumerate() {
            if *pts != Points::ZERO {
                by_round
                    .entry((i as RoundNo + 1, *pts))
                    .or_default()
                    .push(ids[&p.id]);
            }
        }
    }
    let mut out = Vec::new();
    for ((round, pts), mut players) in by_round {
        players.sort_unstable();
        let mut start = players[0];
        let mut prev = start;
        for &id in &players[1..] {
            if id != prev + 1 {
                out.push((pts, round, start, prev));
                start = id;
            }
            prev = id;
        }
        out.push((pts, round, start, prev));
    }
    out
}

fn kind_description(kind: TournamentKind) -> String {
    match kind {
        TournamentKind::Swiss { system } => format!("Swiss System ({system:?})"),
        TournamentKind::RoundRobin { double: false } => "Round Robin".into(),
        TournamentKind::RoundRobin { double: true } => "Double Round Robin".into(),
        TournamentKind::TeamSwiss { .. } => "Team Swiss System".into(),
        TournamentKind::TeamRoundRobin { double: false } => "Team Round Robin".into(),
        TournamentKind::TeamRoundRobin { double: true } => "Team Double Round Robin".into(),
    }
}
