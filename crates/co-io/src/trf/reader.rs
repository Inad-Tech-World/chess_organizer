use super::columns::Line;
use super::{parse_bye_char, parse_result_char, parse_title, parse_type_code, round_col};
use co_core::chrono::NaiveDate;
use co_core::*;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum TrfError {
    #[error("TRF line {line}: {message}")]
    Syntax { line: usize, message: String },
    #[error("TRF content is inconsistent: {0}")]
    Rule(#[from] RuleError),
}

/// A tournament read from TRF, plus anything that was skipped or looked wrong.
#[derive(Debug)]
pub struct TrfImport {
    pub tournament: Tournament,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct RoundBlock {
    opponent: u32,
    color: char,
    result: char,
}

struct PlayerLine {
    player: Player,
    declared_points: Option<Points>,
    blocks: Vec<RoundBlock>,
}

struct TeamRecord {
    name: String,
    members: Vec<u32>,
}

/// Reads a TRF file (TRF-2026, TRF-16 or bbpPairings TRF(x)).
pub fn read(text: &str) -> Result<TrfImport, TrfError> {
    let mut info = TournamentInfo::default();
    let mut settings = Settings::default();
    let mut kind = TournamentKind::default();
    let mut baku = false;
    let mut declared_rounds: Option<RoundNo> = None;
    let mut players: Vec<PlayerLine> = Vec::new();
    let mut bye_records: Vec<(ByeKind, RoundNo, Vec<u32>)> = Vec::new();
    let mut accel: Vec<(Points, RoundNo, RoundNo, u32, u32)> = Vec::new();
    let mut forbidden: Vec<Vec<u32>> = Vec::new();
    let mut teams: Vec<TeamRecord> = Vec::new();
    let mut national: BTreeMap<u32, u16> = BTreeMap::new();
    let mut warnings = Vec::new();

    for (idx, raw) in text.lines().enumerate() {
        let lineno = idx + 1;
        let raw = raw.trim_end_matches('\r');
        if raw.trim().is_empty() || raw.starts_with("###") {
            continue;
        }
        let line = Line::new(raw);
        let code = line.field(1, 3);
        let syntax = |message: String| TrfError::Syntax {
            line: lineno,
            message,
        };
        match code.as_str() {
            "001" => players.push(parse_player(&line).map_err(syntax)?),
            "012" => info.name = line.rest(5),
            "022" => info.city = line.rest(5),
            "032" => info.federation = line.rest(5),
            "042" => info.start_date = parse_date(&line.rest(5)),
            "052" => info.end_date = parse_date(&line.rest(5)),
            "102" => info.chief_arbiter = line.rest(5),
            "112" => info.deputy_arbiters.push(line.rest(5)),
            "122" => info.time_control = line.rest(5),
            "132" => {
                let mut r = 1;
                while round_col(r) <= line.len() {
                    let c = round_col(r);
                    let date = NaiveDate::parse_from_str(&line.field(c, c + 7), "%y/%m/%d").ok();
                    info.round_dates.push(date);
                    r += 1;
                }
                while info.round_dates.last() == Some(&None) {
                    info.round_dates.pop();
                }
            }
            "142" | "XXR" => {
                declared_rounds = Some(
                    line.rest(5)
                        .parse()
                        .map_err(|_| syntax("invalid number of rounds".into()))?,
                );
            }
            "152" => match line.rest(5).chars().next().map(|c| c.to_ascii_uppercase()) {
                Some('W') => settings.initial_color = Color::White,
                Some('B') => settings.initial_color = Color::Black,
                _ => warnings.push(format!("line {lineno}: unknown initial colour")),
            },
            "XXC" => {
                let rest = line.rest(5).to_ascii_lowercase();
                if rest.contains("black1") {
                    settings.initial_color = Color::Black;
                } else if rest.contains("white1") {
                    settings.initial_color = Color::White;
                }
            }
            "162" => parse_point_system(&line, &mut settings.points).map_err(syntax)?,
            "BBW" | "BBD" | "BBL" | "BBZ" | "BBF" | "BBU" => {
                let v: f64 = line
                    .rest(5)
                    .parse()
                    .map_err(|_| syntax(format!("invalid points in {code}")))?;
                let v = Points::from_f64(v);
                let ps = &mut settings.points;
                match code.as_str() {
                    "BBW" => (ps.win, ps.forfeit_win, ps.full_bye) = (v, v, v),
                    "BBD" => (ps.draw, ps.half_bye) = (v, v),
                    "BBL" => ps.loss = v,
                    "BBZ" => ps.zero_bye = v,
                    "BBF" => ps.forfeit_loss = v,
                    _ => ps.pairing_bye = v,
                }
            }
            "192" => match parse_type_code(&line.rest(5)) {
                Some((k, b)) => (kind, baku) = (k, b),
                None => warnings.push(format!(
                    "line {lineno}: unknown tournament type '{}'",
                    line.rest(5)
                )),
            },
            "202" | "212" => {
                let mut specs = Vec::new();
                for code in line.rest(5).split(',').map(str::trim) {
                    if code.is_empty() || code.eq_ignore_ascii_case("PTS") {
                        continue;
                    }
                    match TiebreakSpec::parse(code) {
                        Some(s) => specs.push(s),
                        None => warnings.push(format!("line {lineno}: unknown tie-break '{code}'")),
                    }
                }
                settings.tiebreaks = specs;
            }
            "240" => {
                let kind = match line.char_at(5).to_ascii_uppercase() {
                    'F' => ByeKind::Full,
                    'H' => ByeKind::Half,
                    'Z' => ByeKind::Zero,
                    c => return Err(syntax(format!("unknown bye type '{c}'"))),
                };
                let round = parse_num(&line.field(7, 9)).map_err(syntax)?;
                bye_records.push((kind, round as RoundNo, ids_from(&line, 11)));
            }
            "250" => {
                let points = line.field(10, 13).parse::<f64>().map(Points::from_f64);
                let points = points.map_err(|_| syntax("invalid acceleration points".into()))?;
                let first_round = parse_num(&line.field(15, 17)).map_err(syntax)? as RoundNo;
                let last_round = parse_num(&line.field(19, 21)).map_err(syntax)? as RoundNo;
                let first = parse_num(&line.field(23, 26)).map_err(syntax)?;
                let last = parse_num(&line.field(28, 31)).map_err(syntax)?;
                accel.push((points, first_round, last_round, first, last));
            }
            "260" => forbidden.push(ids_from(&line, 13)),
            "013" => teams.push(TeamRecord {
                name: line.field(5, 36),
                members: ids_from(&line, 37),
            }),
            "310" => teams.push(TeamRecord {
                name: line.field(9, 40),
                members: ids_from(&line, 74),
            }),
            "062" | "072" | "082" | "092" | "172" | "182" | "222" | "352" | "362" => {
                // Derived or informative; recomputed on export.
            }
            c if c.starts_with("XX") => {
                warnings.push(format!("line {lineno}: extension record {c} ignored"))
            }
            c if c.len() == 3
                && c.chars().all(|ch| ch.is_ascii_uppercase())
                && line.field(5, 8).parse::<u32>().is_ok() =>
            {
                // National Rating Support record: same layout as 001.
                let rank = line.field(5, 8).parse::<u32>().unwrap_or(0);
                if let Ok(r) = line.field(49, 52).parse::<u16>() {
                    national.insert(rank, r);
                }
            }
            c => warnings.push(format!("line {lineno}: record {c} ignored")),
        }
    }

    players.sort_by_key(|p| p.player.id);
    for w in players.windows(2) {
        if w[0].player.id == w[1].player.id {
            return Err(TrfError::Syntax {
                line: 0,
                message: format!("starting rank {} appears twice", w[0].player.id),
            });
        }
    }
    let known: HashSet<u32> = players.iter().map(|p| p.player.id.0).collect();

    let present_rounds = players.iter().map(|p| p.blocks.len()).max().unwrap_or(0) as RoundNo;
    let last_bye_round = bye_records.iter().map(|b| b.1).max().unwrap_or(0);
    settings.rounds = declared_rounds
        .unwrap_or(if present_rounds > 0 {
            present_rounds
        } else {
            settings.rounds
        })
        .max(present_rounds)
        .max(last_bye_round);
    if baku {
        settings.acceleration = Some(AccelerationMethod::Baku);
    } else if !accel.is_empty() {
        settings.acceleration = Some(AccelerationMethod::Manual);
    }

    for (points, first_round, last_round, first, last) in accel {
        for pl in players
            .iter_mut()
            .filter(|p| (first..=last).contains(&p.player.id.0))
        {
            let acc = &mut pl.player.acceleration;
            if acc.len() < usize::from(last_round) {
                acc.resize(usize::from(last_round), Points::ZERO);
            }
            for r in first_round..=last_round {
                acc[usize::from(r) - 1] += points;
            }
        }
    }
    for pl in &mut players {
        if let Some(r) = national.get(&pl.player.id.0) {
            pl.player.ratings.national = Some(*r);
        }
    }

    let forbidden_groups: Vec<ForbiddenGroup> = forbidden
        .into_iter()
        .map(|ids| ForbiddenGroup {
            name: "TRF 260".into(),
            members: ids
                .into_iter()
                .filter(|id| {
                    let ok = known.contains(id);
                    if !ok {
                        warnings.push(format!("record 260: unknown player {id} dropped"));
                    }
                    ok
                })
                .map(PlayerId)
                .collect(),
        })
        .collect();

    let mut t = Tournament::create(&Event::TournamentCreated {
        id: Uuid::new_v4(),
        info,
        settings: settings.clone(),
        kind,
    })?;
    for (i, team) in teams.iter().enumerate() {
        t.apply(&Event::TeamAdded {
            team: Team {
                id: TeamId(i as u32 + 1),
                name: team.name.clone(),
                short_name: None,
                federation: None,
                members: Vec::new(),
                captain: None,
            },
        })?;
    }
    for pl in &players {
        let mut player = pl.player.clone();
        player.team = teams
            .iter()
            .position(|tr| tr.members.contains(&player.id.0))
            .map(|i| TeamId(i as u32 + 1));
        t.apply(&Event::PlayerAdded { player })?;
    }
    for (i, team) in teams.iter().enumerate() {
        let mut updated = t.teams[i].clone();
        updated.members = team
            .members
            .iter()
            .filter(|id| known.contains(id))
            .map(|id| PlayerId(*id))
            .collect();
        t.apply(&Event::TeamUpdated { team: updated })?;
    }
    if !forbidden_groups.is_empty() {
        let mut s = t.settings.clone();
        s.forbidden_groups = forbidden_groups;
        t.apply(&Event::SettingsUpdated { settings: s })?;
    }

    for r in 1..=present_rounds {
        let pairings = build_round(&t, &players, r, &mut warnings)?;
        t.apply(&Event::RoundPaired {
            round: r,
            pairings,
            matches: Vec::new(),
            method: PairingMethod::Manual,
        })?;
    }

    for (kind, round, ids) in bye_records {
        if round <= present_rounds {
            continue; // Already recorded in the 001 records.
        }
        for id in ids {
            if !known.contains(&id) {
                warnings.push(format!("record 240: unknown player {id} ignored"));
                continue;
            }
            t.apply(&Event::ByeSet {
                player: PlayerId(id),
                round,
                kind: Some(kind),
            })?;
        }
    }

    for pl in &players {
        if let Some(declared) = pl.declared_points {
            let computed = scoring::score_after(&t, pl.player.id, present_rounds);
            if declared != computed {
                warnings.push(format!(
                    "player {}: file says {declared} points, computed {computed} (check the point system)",
                    pl.player.id
                ));
            }
        }
    }

    Ok(TrfImport {
        tournament: t,
        warnings,
    })
}

fn build_round(
    t: &Tournament,
    players: &[PlayerLine],
    r: RoundNo,
    warnings: &mut Vec<String>,
) -> Result<Vec<Pairing>, TrfError> {
    let idx = usize::from(r) - 1;
    let block_of = |id: u32| {
        players
            .binary_search_by_key(&PlayerId(id), |p| p.player.id)
            .ok()
            .and_then(|i| players[i].blocks.get(idx).copied())
    };
    let mut games: Vec<(PlayerId, PlayerId, Option<GameResult>)> = Vec::new();
    let mut byes = Vec::new();
    let mut seen = HashSet::new();
    for pl in players {
        let Some(b) = pl.blocks.get(idx).copied() else {
            continue;
        };
        let me = pl.player.id.0;
        if b.opponent == 0 {
            byes.push(Pairing::Bye {
                player: pl.player.id,
                kind: parse_bye_char(b.result),
            });
            continue;
        }
        let Some(theirs) = block_of(b.opponent) else {
            return Err(TrfError::Syntax {
                line: 0,
                message: format!("round {r}: player {me} meets unknown player {}", b.opponent),
            });
        };
        let points_back = theirs.opponent == me;
        let (white, black, white_block, black_block) = match b.color.to_ascii_lowercase() {
            'w' => (me, b.opponent, Some(b), points_back.then_some(theirs)),
            'b' if points_back && theirs.color.eq_ignore_ascii_case(&'w') => continue,
            'b' => (b.opponent, me, None, Some(b)),
            _ if me < b.opponent => (me, b.opponent, Some(b), points_back.then_some(theirs)),
            _ => continue,
        };
        if !points_back {
            warnings.push(format!(
                "round {r}: player {me} lists {} as opponent, but not vice versa",
                b.opponent
            ));
        }
        if !seen.insert(white) || !seen.insert(black) {
            warnings.push(format!("round {r}: game {white}-{black} listed twice"));
            continue;
        }
        let wr = white_block.and_then(|b| parse_result_char(b.result));
        let br = black_block.and_then(|b| parse_result_char(b.result));
        let result = match (wr, br) {
            (Some(w), Some(b)) => Some(GameResult::new(w, b)),
            (Some(w), None) => Some(GameResult::new(w, inverse(w))),
            (None, Some(b)) => Some(GameResult::new(inverse(b), b)),
            (None, None) => None,
        };
        games.push((PlayerId(white), PlayerId(black), result));
    }

    // TRF has no board numbers: order boards by score group, then rank.
    let scores = scoring::scores_after(t, r - 1);
    let rank = |id: PlayerId| t.player(id).map_or(u32::MAX, |p| p.starting_rank);
    games.sort_by_key(|&(w, b, _)| {
        let (sw, sb) = (scores[&w], scores[&b]);
        (Reverse(sw.max(sb)), Reverse(sw + sb), rank(w).min(rank(b)))
    });
    let mut pairings: Vec<Pairing> = games
        .into_iter()
        .enumerate()
        .map(|(i, (white, black, result))| Pairing::Game {
            board: i as u16 + 1,
            white,
            black,
            result,
        })
        .collect();
    pairings.extend(byes);
    Ok(pairings)
}

fn inverse(r: SideResult) -> SideResult {
    match r {
        SideResult::Win => SideResult::Loss,
        SideResult::Loss => SideResult::Win,
        SideResult::Draw => SideResult::Draw,
        SideResult::ForfeitWin => SideResult::ForfeitLoss,
        SideResult::ForfeitLoss => SideResult::ForfeitWin,
        SideResult::UnratedWin => SideResult::UnratedLoss,
        SideResult::UnratedLoss => SideResult::UnratedWin,
        SideResult::UnratedDraw => SideResult::UnratedDraw,
    }
}

fn parse_player(line: &Line) -> Result<PlayerLine, String> {
    let rank: u32 = parse_num(&line.field(5, 8))?;
    if rank == 0 {
        return Err("starting rank must be at least 1".into());
    }
    let mut p = Player::new(PlayerId(rank), line.field(15, 47));
    p.starting_rank = rank;
    p.sex = match line.char_at(10).to_ascii_lowercase() {
        'm' => Some(Sex::Male),
        'w' | 'f' => Some(Sex::Female),
        _ => None,
    };
    p.title = parse_title(&line.field(11, 13));
    p.ratings.fide_standard = line.field(49, 52).parse().ok().filter(|r| *r > 0);
    p.federation = Some(line.field(54, 56)).filter(|s| !s.is_empty());
    p.fide_id = line.field(58, 68).parse().ok().filter(|id| *id > 0);
    p.birth_date = Some(line.field(70, 79)).filter(|s| !s.is_empty());
    let declared_points = line.field(81, 84).parse::<f64>().ok().map(Points::from_f64);

    let mut blocks = Vec::new();
    let mut r: RoundNo = 1;
    while round_col(r) <= line.len() {
        let c = round_col(r);
        let opp = line.field(c, c + 3);
        let opponent = if opp.is_empty() {
            0
        } else {
            opp.parse()
                .map_err(|_| format!("round {r}: invalid opponent '{opp}'"))?
        };
        blocks.push(RoundBlock {
            opponent,
            color: line.char_at(c + 5),
            result: line.char_at(c + 7),
        });
        r += 1;
    }
    Ok(PlayerLine {
        player: p,
        declared_points,
        blocks,
    })
}

/// Record 162: symbol at column 6 + 9k, points in the next four columns.
fn parse_point_system(line: &Line, ps: &mut PointSystem) -> Result<(), String> {
    let mut values = BTreeMap::new();
    let mut col = 6;
    while col <= line.len() {
        let symbol = line.char_at(col).to_ascii_uppercase();
        if symbol != ' ' {
            let v: f64 = line
                .field(col + 1, col + 4)
                .parse()
                .map_err(|_| format!("invalid points for '{symbol}'"))?;
            values.insert(symbol, Points::from_f64(v));
        }
        col += 9;
    }
    for (symbol, v) in &values {
        match symbol {
            'W' => (ps.win, ps.forfeit_win, ps.full_bye) = (*v, *v, *v),
            'D' => (ps.draw, ps.half_bye) = (*v, *v),
            'L' => ps.loss = *v,
            'A' => (ps.zero_bye, ps.forfeit_loss) = (*v, *v),
            'P' => ps.pairing_bye = *v,
            'X' => {}
            other => return Err(format!("unknown point symbol '{other}'")),
        }
    }
    if !values.contains_key(&'P') {
        ps.pairing_bye = ps.win;
    }
    Ok(())
}

fn parse_num(s: &str) -> Result<u32, String> {
    s.parse()
        .map_err(|_| format!("expected a number, found '{s}'"))
}

/// Player ids in 4-character fields starting at `col`, every 5 columns.
fn ids_from(line: &Line, mut col: usize) -> Vec<u32> {
    let mut ids = Vec::new();
    while col <= line.len() {
        if let Ok(id) = line.field(col, col + 3).parse::<u32>()
            && id > 0
        {
            ids.push(id);
        }
        col += 5;
    }
    ids
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    [
        "%Y/%m/%d", "%Y-%m-%d", "%Y.%m.%d", "%d.%m.%Y", "%d/%m/%Y", "%y/%m/%d",
    ]
    .iter()
    .find_map(|f| NaiveDate::parse_from_str(s, f).ok())
}
