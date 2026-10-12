//! FIDE Tournament Report File (TRF).
//!
//! Writes TRF-2026 and reads TRF-2026, TRF-16 and the bbpPairings/JaVaFo
//! extensions (`XXR`, `XXC`, `BBx`). TRF identifies players by starting rank,
//! so on import `PlayerId(n)` is the player with starting rank `n`.
//!
//! Team records (013, 310, 3xx, 80x) are read for team names and rosters
//! only; full team support lands in Phase 3.

pub mod bbp;
mod columns;
mod reader;
mod writer;

pub use reader::{TrfError, TrfImport, read};
pub use writer::{trf_ids, write};

use co_core::{
    AccelerationMethod, ByeKind, SideResult, SwissSystem, TeamScore, Title, TournamentKind,
};

/// First column of round `r`'s block in an `001` record.
pub(crate) fn round_col(round: u16) -> usize {
    92 + 10 * (usize::from(round) - 1)
}

pub(crate) fn title_code(t: Title) -> &'static str {
    t.as_str()
}

/// Accepts TRF-2026 titles (`GM`, `WGM`) and TRF-16 codes (`g`, `wg`).
pub(crate) fn parse_title(s: &str) -> Option<Title> {
    Title::parse(s).or(match s {
        "g" => Some(Title::GM),
        "m" => Some(Title::IM),
        "f" => Some(Title::FM),
        "c" => Some(Title::CM),
        "wg" => Some(Title::WGM),
        "wm" => Some(Title::WIM),
        "wf" => Some(Title::WFM),
        "wc" => Some(Title::WCM),
        _ => None,
    })
}

pub(crate) fn result_char(r: SideResult) -> char {
    match r {
        SideResult::Win => '1',
        SideResult::Draw => '=',
        SideResult::Loss => '0',
        SideResult::ForfeitWin => '+',
        SideResult::ForfeitLoss => '-',
        SideResult::UnratedWin => 'W',
        SideResult::UnratedDraw => 'D',
        SideResult::UnratedLoss => 'L',
    }
}

pub(crate) fn parse_result_char(c: char) -> Option<SideResult> {
    Some(match c.to_ascii_uppercase() {
        '1' => SideResult::Win,
        '=' => SideResult::Draw,
        '0' => SideResult::Loss,
        '+' => SideResult::ForfeitWin,
        '-' => SideResult::ForfeitLoss,
        'W' => SideResult::UnratedWin,
        'D' => SideResult::UnratedDraw,
        'L' => SideResult::UnratedLoss,
        _ => return None,
    })
}

pub(crate) fn bye_char(k: ByeKind) -> char {
    match k {
        ByeKind::PairingAllocated => 'U',
        ByeKind::Half => 'H',
        ByeKind::Full => 'F',
        ByeKind::Zero => 'Z',
    }
}

/// Bye kind for a round block with opponent `0000`.
pub(crate) fn parse_bye_char(c: char) -> ByeKind {
    match c.to_ascii_uppercase() {
        'U' => ByeKind::PairingAllocated,
        'H' | '=' => ByeKind::Half,
        'F' | '+' | '1' | 'W' => ByeKind::Full,
        _ => ByeKind::Zero,
    }
}

/// Record 192 code (TRF-2026 Tournament Type Code Table).
pub(crate) fn type_code(
    kind: TournamentKind,
    acceleration: Option<AccelerationMethod>,
    team_score: TeamScore,
) -> &'static str {
    let baku = acceleration == Some(AccelerationMethod::Baku);
    match kind {
        TournamentKind::Swiss { system } => match (system, baku) {
            (SwissSystem::Dutch, false) => "FIDE_DUTCH_2025",
            (SwissSystem::Dutch, true) => "FIDE_DUTCH_2025_BAKU",
            (SwissSystem::Dubov, false) => "FIDE_DUBOV",
            (SwissSystem::Dubov, true) => "FIDE_DUBOV_BAKU",
            (SwissSystem::Burstein, false) => "FIDE_BURSTEIN",
            (SwissSystem::Burstein, true) => "FIDE_BURSTEIN_BAKU",
            (SwissSystem::Lim | SwissSystem::Monrad, _) => "CUSTOM_SWISS",
        },
        TournamentKind::RoundRobin { double: false } => "FIDE_ROUNDROBIN",
        TournamentKind::RoundRobin { double: true } => "FIDE_DOUBLEROUNDROBIN",
        TournamentKind::TeamSwiss { .. } => match (team_score, baku) {
            (TeamScore::MatchPoints, false) => "FIDE_TEAM_TYPEA_MP_GP",
            (TeamScore::MatchPoints, true) => "FIDE_TEAM_TYPEA_MP_GP_BAKU",
            (TeamScore::GamePoints, _) => "FIDE_TEAM_TYPEA_GP_MP",
        },
        TournamentKind::TeamRoundRobin { double: false } => "FIDE_TEAM_ROUNDROBIN",
        TournamentKind::TeamRoundRobin { double: true } => "FIDE_TEAM_DOUBLEROUNDROBIN",
    }
}

/// Best-effort reading of a record 192 code.
pub(crate) fn parse_type_code(code: &str) -> Option<(TournamentKind, bool)> {
    let c = code.trim().to_ascii_uppercase();
    let baku = c.contains("BAKU");
    let kind = if c.starts_with("FIDE_DUTCH") {
        TournamentKind::Swiss {
            system: SwissSystem::Dutch,
        }
    } else if c.starts_with("FIDE_DUBOV") {
        TournamentKind::Swiss {
            system: SwissSystem::Dubov,
        }
    } else if c.starts_with("FIDE_BURSTEIN") {
        TournamentKind::Swiss {
            system: SwissSystem::Burstein,
        }
    } else if c == "CUSTOM_SWISS" {
        TournamentKind::Swiss {
            system: SwissSystem::Monrad,
        }
    } else if c.contains("TEAM") && c.contains("ROUNDROBIN") {
        TournamentKind::TeamRoundRobin {
            double: c.contains("DOUBLE") || c.ends_with("_G2"),
        }
    } else if c.contains("TEAM") {
        TournamentKind::TeamSwiss {
            system: SwissSystem::Dutch,
        }
    } else if c.contains("ROUNDROBIN") {
        TournamentKind::RoundRobin {
            double: c.contains("DOUBLE") || c.ends_with("_G2"),
        }
    } else {
        return None;
    };
    Some((kind, baku))
}

#[cfg(test)]
mod tests;
