//! Tie-break configuration (FIDE Handbook C.07, TRF-2026 "Mandatory
//! Tie-Breaks"). The calculations live in `co-tiebreak`; the catalogue lives
//! here so tournament files stay stable while calculations land phase by phase.
//!
//! A tie-break is written as a rank order descriptor, as in TRF records
//! 202/212: `NAME[:MP|:GP][/variant]...`, e.g. `BH/C1`, `SB/P`, `KS/L+1`,
//! `BH:GP/M1/P`.

use crate::Phase;
use crate::model::TeamScore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TiebreakKind {
    // Individuals only.
    DirectEncounter,
    GamesWithBlack,
    WinsWithBlack,
    RoundsElectedToPlay,
    SonnebornBerger,
    AverageRatingOfOpponents,
    TournamentPerformanceRating,
    PerfectTournamentPerformance,
    AveragePerformanceRatingOfOpponents,
    AveragePerfectPerformanceOfOpponents,
    // Individuals and teams.
    Wins,
    WonGames,
    ProgressiveScore,
    Buchholz,
    AverageOpponentsBuchholz,
    ForeBuchholz,
    Koya,
    // Teams only.
    BoardCount,
    TopBoardResult,
    BottomBoardElimination,
    MatchPointsVsGamePoints,
    TeamSbMatchMatch,
    TeamSbMatchGame,
    TeamSbGameMatch,
    TeamSbGameGame,
    ExtendedDirectEncounter,
    EdeBoardCountTopBoard,
    EdeBoardCountBottomBoard,
    EdeTopBoard,
    EdeBottomBoard,
    StrengthCompensatedScore,
    /// Secondary team score (game points when ranking by match points, or
    /// vice versa). Not a C.07 code; written to TRF as `OTHER_SEC`.
    SecondaryScore,
    // Added in the March 2026 edition of C.07.
    /// Standard points: rounds won against the scheduled opponent, half for equal (7.7).
    StandardPoints,
    /// Pairing number, lower first (7.8).
    PairingNumber,
    /// Rating, higher first (10.6).
    Rating,
}

impl TiebreakKind {
    pub const ALL: [TiebreakKind; 35] = [
        Self::DirectEncounter,
        Self::GamesWithBlack,
        Self::WinsWithBlack,
        Self::RoundsElectedToPlay,
        Self::SonnebornBerger,
        Self::AverageRatingOfOpponents,
        Self::TournamentPerformanceRating,
        Self::PerfectTournamentPerformance,
        Self::AveragePerformanceRatingOfOpponents,
        Self::AveragePerfectPerformanceOfOpponents,
        Self::Wins,
        Self::WonGames,
        Self::ProgressiveScore,
        Self::Buchholz,
        Self::AverageOpponentsBuchholz,
        Self::ForeBuchholz,
        Self::Koya,
        Self::BoardCount,
        Self::TopBoardResult,
        Self::BottomBoardElimination,
        Self::MatchPointsVsGamePoints,
        Self::TeamSbMatchMatch,
        Self::TeamSbMatchGame,
        Self::TeamSbGameMatch,
        Self::TeamSbGameGame,
        Self::ExtendedDirectEncounter,
        Self::EdeBoardCountTopBoard,
        Self::EdeBoardCountBottomBoard,
        Self::EdeTopBoard,
        Self::EdeBottomBoard,
        Self::StrengthCompensatedScore,
        Self::SecondaryScore,
        Self::StandardPoints,
        Self::PairingNumber,
        Self::Rating,
    ];

    /// Acronym used in C.07, on printed lists and in TRF 202/212.
    pub fn acronym(self) -> &'static str {
        match self {
            Self::DirectEncounter => "DE",
            Self::GamesWithBlack => "BPG",
            Self::WinsWithBlack => "BWG",
            Self::RoundsElectedToPlay => "REP",
            Self::SonnebornBerger => "SB",
            Self::AverageRatingOfOpponents => "ARO",
            Self::TournamentPerformanceRating => "TPR",
            Self::PerfectTournamentPerformance => "PTP",
            Self::AveragePerformanceRatingOfOpponents => "APRO",
            Self::AveragePerfectPerformanceOfOpponents => "APPO",
            Self::Wins => "WIN",
            Self::WonGames => "WON",
            Self::ProgressiveScore => "PS",
            Self::Buchholz => "BH",
            Self::AverageOpponentsBuchholz => "AOB",
            Self::ForeBuchholz => "FB",
            Self::Koya => "KS",
            Self::BoardCount => "BC",
            Self::TopBoardResult => "TBR",
            Self::BottomBoardElimination => "BBE",
            Self::MatchPointsVsGamePoints => "MPvGP",
            Self::TeamSbMatchMatch => "EMMSB",
            Self::TeamSbMatchGame => "EMGSB",
            Self::TeamSbGameMatch => "EGMSB",
            Self::TeamSbGameGame => "EGGSB",
            Self::ExtendedDirectEncounter => "EDE",
            Self::EdeBoardCountTopBoard => "EDEBT",
            Self::EdeBoardCountBottomBoard => "EDEBB",
            Self::EdeTopBoard => "EDET",
            Self::EdeBottomBoard => "EDEB",
            Self::StrengthCompensatedScore => "SSSC",
            Self::SecondaryScore => "OTHER_SEC",
            Self::StandardPoints => "STD",
            Self::PairingNumber => "TPN",
            Self::Rating => "RTNG",
        }
    }

    pub fn from_acronym(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|k| k.acronym().eq_ignore_ascii_case(s))
    }

    pub fn for_individuals(self) -> bool {
        !matches!(
            self,
            Self::BoardCount
                | Self::TopBoardResult
                | Self::BottomBoardElimination
                | Self::MatchPointsVsGamePoints
                | Self::TeamSbMatchMatch
                | Self::TeamSbMatchGame
                | Self::TeamSbGameMatch
                | Self::TeamSbGameGame
                | Self::ExtendedDirectEncounter
                | Self::EdeBoardCountTopBoard
                | Self::EdeBoardCountBottomBoard
                | Self::EdeTopBoard
                | Self::EdeBottomBoard
                | Self::StrengthCompensatedScore
                | Self::SecondaryScore
        )
    }

    pub fn for_teams(self) -> bool {
        !matches!(
            self,
            Self::DirectEncounter
                | Self::GamesWithBlack
                | Self::WinsWithBlack
                | Self::RoundsElectedToPlay
                | Self::SonnebornBerger
                | Self::AverageRatingOfOpponents
                | Self::TournamentPerformanceRating
                | Self::PerfectTournamentPerformance
                | Self::AveragePerformanceRatingOfOpponents
                | Self::AveragePerfectPerformanceOfOpponents
                | Self::Rating
        )
    }

    /// Phase in which the calculation is scheduled to land.
    pub fn phase(self) -> Phase {
        match self {
            Self::DirectEncounter
            | Self::Wins
            | Self::ProgressiveScore
            | Self::Buchholz
            | Self::SonnebornBerger => 1,
            k if k.for_individuals() => 2,
            _ => 3,
        }
    }
}

/// Variants and options of a tie-break (TRF-2026 "Mandatory Tie-Breaks").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct TiebreakModifiers {
    /// `/Cn`: drop the n lowest values.
    pub cut_lowest: u8,
    /// With `cut_lowest`, `/Mn` (median): also drop the n highest values.
    pub cut_highest: u8,
    /// `/L±n`: Koya limit in half points above/below 50%.
    pub koya_limit: Option<i8>,
    /// `/Kx`: SSSC normalising factor.
    pub sssc_factor: Option<u8>,
    /// `/P`: forfeited games count as played against the scheduled opponent.
    pub forfeits_as_played: bool,
    /// `/F`: use Fore Buchholz instead of Buchholz (AOB, SSSC).
    pub fore: bool,
    /// `:MP` / `:GP`: team score the calculation uses.
    pub team_score: Option<TeamScore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TiebreakSpec {
    pub kind: TiebreakKind,
    pub modifiers: TiebreakModifiers,
}

impl TiebreakSpec {
    pub fn plain(kind: TiebreakKind) -> Self {
        Self {
            kind,
            modifiers: TiebreakModifiers::default(),
        }
    }

    /// Rank order descriptor, e.g. `BH/C1`, `BH:GP/M1/P`. Also the column label.
    pub fn label(&self) -> String {
        let m = &self.modifiers;
        let mut s = self.kind.acronym().to_string();
        match m.team_score {
            Some(TeamScore::MatchPoints) => s.push_str(":MP"),
            Some(TeamScore::GamePoints) => s.push_str(":GP"),
            None => {}
        }
        match (m.cut_lowest, m.cut_highest) {
            (0, _) => {}
            (l, h) if l == h => s.push_str(&format!("/M{l}")),
            (l, _) => s.push_str(&format!("/C{l}")),
        }
        if let Some(l) = m.koya_limit {
            s.push_str(&format!("/L{l:+}"));
        }
        if m.fore {
            s.push_str("/F");
        }
        if m.forfeits_as_played {
            s.push_str("/P");
        }
        if let Some(k) = m.sssc_factor {
            s.push_str(&format!("/K{k}"));
        }
        s
    }

    /// Parses a rank order descriptor. Case-insensitive.
    pub fn parse(code: &str) -> Option<Self> {
        let mut parts = code.trim().split('/');
        let head = parts.next()?;
        let (name, score) = match head.split_once(':') {
            Some((n, s)) => (n, Some(s)),
            None => (head, None),
        };
        let kind = TiebreakKind::from_acronym(name)?;
        let mut m = TiebreakModifiers {
            team_score: match score.map(str::to_ascii_uppercase).as_deref() {
                None => None,
                Some("MP") => Some(TeamScore::MatchPoints),
                Some("GP") => Some(TeamScore::GamePoints),
                Some(_) => return None,
            },
            ..TiebreakModifiers::default()
        };
        for part in parts {
            let upper = part.to_ascii_uppercase();
            let (flag, rest) = upper.split_at(1.min(upper.len()));
            match flag {
                "C" => m.cut_lowest = rest.parse().ok()?,
                "M" => {
                    let n = rest.parse().ok()?;
                    m.cut_lowest = n;
                    m.cut_highest = n;
                }
                "L" => m.koya_limit = Some(rest.parse().ok()?),
                "K" => m.sssc_factor = Some(rest.parse().ok()?),
                "P" if rest.is_empty() => m.forfeits_as_played = true,
                "F" if rest.is_empty() => m.fore = true,
                _ => return None,
            }
        }
        Some(Self { kind, modifiers: m })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_round_trip() {
        for code in [
            "BH/C1",
            "BH:GP/M1/P",
            "KS/L+1",
            "KS:MP/L-2",
            "SSSC/F/P/K4",
            "DE",
            "SB/C2/P",
        ] {
            let spec = TiebreakSpec::parse(code).unwrap_or_else(|| panic!("{code}"));
            assert_eq!(spec.label(), code);
        }
        assert_eq!(TiebreakSpec::parse("bh/c1").unwrap().label(), "BH/C1");
        assert!(TiebreakSpec::parse("XYZ").is_none());
    }

    #[test]
    fn phases() {
        assert_eq!(TiebreakKind::Buchholz.phase(), 1);
        assert_eq!(TiebreakKind::Koya.phase(), 2);
        assert_eq!(TiebreakKind::BoardCount.phase(), 3);
        assert!(TiebreakKind::Buchholz.for_teams() && TiebreakKind::Buchholz.for_individuals());
    }
}
