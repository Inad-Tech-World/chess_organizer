//! The tournament state, covering every tournament type from all phases.

use crate::ids::{PlayerId, PrizeCategoryId, RoundNo, TeamId};
use crate::points::Points;
use crate::tiebreak_spec::TiebreakSpec;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Full state of one tournament, rebuilt by replaying its event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Tournament {
    pub id: Uuid,
    pub info: TournamentInfo,
    pub settings: Settings,
    pub kind: TournamentKind,
    /// Ordered by ascending id.
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    /// `rounds[i]` is round `i + 1`.
    pub rounds: Vec<Round>,
    pub prize_categories: Vec<PrizeCategory>,
}

impl Tournament {
    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players
            .binary_search_by_key(&id, |p| p.id)
            .ok()
            .map(|i| &self.players[i])
    }

    pub fn player_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players
            .binary_search_by_key(&id, |p| p.id)
            .ok()
            .map(|i| &mut self.players[i])
    }

    pub fn team(&self, id: TeamId) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    pub fn round(&self, number: RoundNo) -> Option<&Round> {
        self.rounds.get(usize::from(number).checked_sub(1)?)
    }

    pub fn round_mut(&mut self, number: RoundNo) -> Option<&mut Round> {
        self.rounds.get_mut(usize::from(number).checked_sub(1)?)
    }

    /// Number of rounds that have been paired so far.
    pub fn paired_rounds(&self) -> RoundNo {
        self.rounds.len() as RoundNo
    }

    /// Highest round whose games all have results, or 0.
    pub fn completed_rounds(&self) -> RoundNo {
        self.rounds.iter().take_while(|r| r.is_complete()).count() as RoundNo
    }

    /// Next free player id. Ids are never reused within a tournament.
    pub fn next_player_id(&self) -> PlayerId {
        PlayerId(self.players.last().map_or(1, |p| p.id.0 + 1))
    }

    pub fn next_team_id(&self) -> TeamId {
        TeamId(self.teams.iter().map(|t| t.id.0).max().unwrap_or(0) + 1)
    }

    pub fn next_prize_category_id(&self) -> PrizeCategoryId {
        PrizeCategoryId(
            self.prize_categories
                .iter()
                .map(|c| c.id.0)
                .max()
                .unwrap_or(0)
                + 1,
        )
    }

    /// Players ordered by starting rank.
    pub fn players_by_rank(&self) -> Vec<&Player> {
        let mut v: Vec<&Player> = self.players.iter().collect();
        v.sort_by_key(|p| (p.starting_rank, p.id));
        v
    }

    /// Whether `player` appears in any round's pairings.
    pub fn has_games(&self, player: PlayerId) -> bool {
        self.rounds.iter().any(|r| r.entry_for(player).is_some())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct TournamentInfo {
    pub name: String,
    pub city: String,
    /// Three-letter FIDE federation code of the host federation.
    pub federation: String,
    pub venue: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub chief_arbiter: String,
    pub deputy_arbiters: Vec<String>,
    pub organizer: String,
    /// Free-text time control, e.g. "90 min + 30 s/move".
    pub time_control: String,
    /// Scheduled date of each round, index 0 = round 1.
    pub round_dates: Vec<Option<NaiveDate>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum TournamentKind {
    Swiss { system: SwissSystem },
    RoundRobin { double: bool },
    TeamSwiss { system: SwissSystem },
    TeamRoundRobin { double: bool },
}

impl TournamentKind {
    pub fn is_team(self) -> bool {
        matches!(self, Self::TeamSwiss { .. } | Self::TeamRoundRobin { .. })
    }

    pub fn pairing_system(self) -> PairingSystem {
        match self {
            Self::Swiss { system } => system.into(),
            Self::RoundRobin { .. } => PairingSystem::Berger,
            Self::TeamSwiss { .. } => PairingSystem::TeamSwiss,
            Self::TeamRoundRobin { .. } => PairingSystem::TeamRoundRobin,
        }
    }
}

impl Default for TournamentKind {
    fn default() -> Self {
        Self::Swiss {
            system: SwissSystem::Dutch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SwissSystem {
    /// FIDE (Dutch) System, C.04.3.
    Dutch,
    /// FIDE Dubov System, C.04.4.1.
    Dubov,
    /// FIDE Burstein System, C.04.4.2.
    Burstein,
    /// FIDE Lim System, C.04.4.3.
    Lim,
    /// Simple Monrad-style Swiss for club play.
    Monrad,
}

/// Every pairing method the program can run, individual or team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum PairingSystem {
    Dutch,
    Dubov,
    Burstein,
    Lim,
    Monrad,
    /// Round robin using FIDE Berger tables, C.05 Annex 1.
    Berger,
    TeamSwiss,
    TeamRoundRobin,
}

impl From<SwissSystem> for PairingSystem {
    fn from(s: SwissSystem) -> Self {
        match s {
            SwissSystem::Dutch => Self::Dutch,
            SwissSystem::Dubov => Self::Dubov,
            SwissSystem::Burstein => Self::Burstein,
            SwissSystem::Lim => Self::Lim,
            SwissSystem::Monrad => Self::Monrad,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum RatingType {
    #[default]
    Standard,
    Rapid,
    Blitz,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Color {
    #[default]
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Settings {
    pub rounds: RoundNo,
    pub points: PointSystem,
    pub rating_type: RatingType,
    /// Whether the event is submitted to FIDE for rating.
    pub fide_rated: bool,
    pub acceleration: Option<AccelerationMethod>,
    pub tiebreaks: Vec<TiebreakSpec>,
    /// Players in the same group are never paired against each other
    /// (same club, same family, explicit forbidden pairs).
    pub forbidden_groups: Vec<ForbiddenGroup>,
    /// Colour of the top-ranked player on board 1 in round 1.
    pub initial_color: Color,
    pub team: TeamSettings,
}

impl Default for Settings {
    fn default() -> Self {
        use crate::tiebreak_spec::{TiebreakKind, TiebreakModifiers};
        Self {
            rounds: 7,
            points: PointSystem::default(),
            rating_type: RatingType::Standard,
            fide_rated: false,
            acceleration: None,
            tiebreaks: vec![
                TiebreakSpec {
                    kind: TiebreakKind::Buchholz,
                    modifiers: TiebreakModifiers {
                        cut_lowest: 1,
                        ..TiebreakModifiers::default()
                    },
                },
                TiebreakSpec::plain(TiebreakKind::Buchholz),
                TiebreakSpec::plain(TiebreakKind::SonnebornBerger),
                TiebreakSpec::plain(TiebreakKind::DirectEncounter),
            ],
            forbidden_groups: Vec::new(),
            initial_color: Color::White,
            team: TeamSettings::default(),
        }
    }
}

/// Points awarded per outcome. Defaults are the standard 1 / ½ / 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct PointSystem {
    pub win: Points,
    pub draw: Points,
    pub loss: Points,
    pub forfeit_win: Points,
    pub forfeit_loss: Points,
    /// Pairing-allocated bye.
    pub pairing_bye: Points,
    pub half_bye: Points,
    pub full_bye: Points,
    pub zero_bye: Points,
}

impl Default for PointSystem {
    fn default() -> Self {
        Self {
            win: Points::ONE,
            draw: Points::HALF,
            loss: Points::ZERO,
            forfeit_win: Points::ONE,
            forfeit_loss: Points::ZERO,
            pairing_bye: Points::ONE,
            half_bye: Points::HALF,
            full_bye: Points::ONE,
            zero_bye: Points::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum AccelerationMethod {
    /// Baku acceleration, C.04.5.1 (Phase 2).
    Baku,
    /// Per-player acceleration points entered by the arbiter or read from TRF `XXA` lines.
    Manual,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ForbiddenGroup {
    pub name: String,
    pub members: Vec<PlayerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct TeamSettings {
    pub boards: u8,
    pub reserves: u8,
    pub match_win: Points,
    pub match_draw: Points,
    pub match_loss: Points,
    pub primary_score: TeamScore,
    pub color_preference: TeamColorPreference,
}

/// Colour-preference type of the FIDE Team Pairing System.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TeamColorPreference {
    #[default]
    TypeA,
    TypeB,
    None,
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            boards: 4,
            reserves: 1,
            match_win: Points(20),
            match_draw: Points::ONE,
            match_loss: Points::ZERO,
            primary_score: TeamScore::MatchPoints,
            color_preference: TeamColorPreference::TypeA,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TeamScore {
    #[default]
    MatchPoints,
    GamePoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Title {
    GM,
    IM,
    FM,
    CM,
    WGM,
    WIM,
    WFM,
    WCM,
}

impl Title {
    pub const ALL: [Title; 8] = [
        Title::GM,
        Title::IM,
        Title::FM,
        Title::CM,
        Title::WGM,
        Title::WIM,
        Title::WFM,
        Title::WCM,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Title::GM => "GM",
            Title::IM => "IM",
            Title::FM => "FM",
            Title::CM => "CM",
            Title::WGM => "WGM",
            Title::WIM => "WIM",
            Title::WFM => "WFM",
            Title::WCM => "WCM",
        }
    }

    pub fn parse(s: &str) -> Option<Title> {
        let s = s.trim().to_ascii_uppercase();
        Title::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Ratings {
    pub fide_standard: Option<u16>,
    pub fide_rapid: Option<u16>,
    pub fide_blitz: Option<u16>,
    pub national: Option<u16>,
}

impl Ratings {
    pub fn fide(&self, rt: RatingType) -> Option<u16> {
        match rt {
            RatingType::Standard => self.fide_standard,
            RatingType::Rapid => self.fide_rapid,
            RatingType::Blitz => self.fide_blitz,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum PlayerStatus {
    #[default]
    Active,
    /// Not paired from `from_round` onwards.
    Withdrawn { from_round: RoundNo },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Player {
    pub id: PlayerId,
    /// "Surname, Firstname", as on the FIDE list.
    pub name: String,
    pub sex: Option<Sex>,
    pub title: Option<Title>,
    /// Three-letter FIDE federation code.
    pub federation: Option<String>,
    pub fide_id: Option<u32>,
    pub national_id: Option<String>,
    /// "YYYY" or "YYYY/MM/DD", as in TRF.
    pub birth_date: Option<String>,
    pub club: Option<String>,
    pub ratings: Ratings,
    pub k_factor: Option<u8>,
    pub starting_rank: u32,
    pub status: PlayerStatus,
    /// Byes requested for future rounds; `Zero` marks a pause / known absence.
    #[serde(with = "round_keys")]
    #[cfg_attr(feature = "ts", ts(as = "BTreeMap<RoundNo, ByeKind>"))]
    pub byes: BTreeMap<RoundNo, ByeKind>,
    /// First round the player takes part in (late entries have > 1).
    pub joined_round: RoundNo,
    /// Bye credited for each round before `joined_round`.
    pub late_entry_bye: ByeKind,
    pub team: Option<TeamId>,
    /// Acceleration points per round, index 0 = round 1 (TRF `XXA`).
    pub acceleration: Vec<Points>,
}

impl Player {
    pub fn new(id: PlayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            sex: None,
            title: None,
            federation: None,
            fide_id: None,
            national_id: None,
            birth_date: None,
            club: None,
            ratings: Ratings::default(),
            k_factor: None,
            starting_rank: id.0,
            status: PlayerStatus::Active,
            byes: BTreeMap::new(),
            joined_round: 1,
            late_entry_bye: ByeKind::Zero,
            team: None,
            acceleration: Vec::new(),
        }
    }

    pub fn rating(&self, rt: RatingType) -> Option<u16> {
        self.ratings.fide(rt).or(self.ratings.national)
    }

    pub fn birth_year(&self) -> Option<u16> {
        self.birth_date.as_deref()?.get(..4)?.parse().ok()
    }

    /// Whether the player should be considered for pairing in `round`.
    pub fn is_available(&self, round: RoundNo) -> bool {
        let withdrawn =
            matches!(self.status, PlayerStatus::Withdrawn { from_round } if round >= from_round);
        !withdrawn && round >= self.joined_round && !self.byes.contains_key(&round)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub short_name: Option<String>,
    pub federation: Option<String>,
    /// Members in board order.
    pub members: Vec<PlayerId>,
    pub captain: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum PairingMethod {
    Manual,
    Engine { system: PairingSystem },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Round {
    pub number: RoundNo,
    /// Individual games and byes. Empty for team events, which use `matches`.
    pub pairings: Vec<Pairing>,
    pub matches: Vec<TeamMatch>,
    pub method: PairingMethod,
}

impl Round {
    /// The pairing (game or bye) involving `player`, if any.
    pub fn entry_for(&self, player: PlayerId) -> Option<&Pairing> {
        self.all_pairings().find(|p| p.involves(player))
    }

    /// Individual pairings plus every game inside team matches.
    pub fn all_pairings(&self) -> impl Iterator<Item = &Pairing> {
        self.pairings
            .iter()
            .chain(self.matches.iter().flat_map(|m| m.games.iter()))
    }

    /// Each player's entry in this round, for repeated lookups.
    pub fn index(&self) -> std::collections::HashMap<PlayerId, &Pairing> {
        let mut map = std::collections::HashMap::new();
        for p in self.all_pairings() {
            for id in p.players() {
                map.insert(id, p);
            }
        }
        map
    }

    pub fn game_on_board(&self, board: u16) -> Option<&Pairing> {
        self.pairings.iter().find(|p| p.board() == Some(board))
    }

    pub fn is_complete(&self) -> bool {
        self.all_pairings().all(|p| match p {
            Pairing::Game { result, .. } => result.is_some(),
            Pairing::Bye { .. } => true,
        })
    }

    pub fn has_results(&self) -> bool {
        self.all_pairings().any(|p| {
            matches!(
                p,
                Pairing::Game {
                    result: Some(_),
                    ..
                }
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Pairing {
    Game {
        board: u16,
        white: PlayerId,
        black: PlayerId,
        result: Option<GameResult>,
    },
    Bye {
        player: PlayerId,
        kind: ByeKind,
    },
}

impl Pairing {
    pub fn board(&self) -> Option<u16> {
        match self {
            Pairing::Game { board, .. } => Some(*board),
            Pairing::Bye { .. } => None,
        }
    }

    pub fn players(&self) -> impl Iterator<Item = PlayerId> {
        let (a, b) = match *self {
            Pairing::Game { white, black, .. } => (white, Some(black)),
            Pairing::Bye { player, .. } => (player, None),
        };
        std::iter::once(a).chain(b)
    }

    pub fn involves(&self, player: PlayerId) -> bool {
        self.players().any(|p| p == player)
    }

    /// The colour `player` had and their opponent, for games.
    pub fn side_of(&self, player: PlayerId) -> Option<(Color, PlayerId)> {
        match *self {
            Pairing::Game { white, black, .. } if white == player => Some((Color::White, black)),
            Pairing::Game { white, black, .. } if black == player => Some((Color::Black, white)),
            _ => None,
        }
    }
}

/// Kinds of round without an over-the-board opponent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ByeKind {
    /// Pairing-allocated bye (TRF `U`).
    PairingAllocated,
    /// Requested half-point bye (TRF `H`).
    Half,
    /// Full-point bye (TRF `F`).
    Full,
    /// Zero-point bye / known absence (TRF `Z`).
    Zero,
}

/// What one side of a game scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SideResult {
    Win,
    Draw,
    Loss,
    ForfeitWin,
    ForfeitLoss,
    /// Played but not rated (game lasted less than one move): TRF `W`/`D`/`L`.
    UnratedWin,
    UnratedDraw,
    UnratedLoss,
}

impl SideResult {
    pub fn is_played(self) -> bool {
        !matches!(self, SideResult::ForfeitWin | SideResult::ForfeitLoss)
    }

    pub fn is_rated(self) -> bool {
        matches!(self, SideResult::Win | SideResult::Draw | SideResult::Loss)
    }

    pub fn is_win(self) -> bool {
        matches!(
            self,
            SideResult::Win | SideResult::ForfeitWin | SideResult::UnratedWin
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct GameResult {
    pub white: SideResult,
    pub black: SideResult,
}

impl GameResult {
    pub const WHITE_WINS: Self = Self::new(SideResult::Win, SideResult::Loss);
    pub const DRAW: Self = Self::new(SideResult::Draw, SideResult::Draw);
    pub const BLACK_WINS: Self = Self::new(SideResult::Loss, SideResult::Win);
    pub const WHITE_FORFEIT_WIN: Self = Self::new(SideResult::ForfeitWin, SideResult::ForfeitLoss);
    pub const BLACK_FORFEIT_WIN: Self = Self::new(SideResult::ForfeitLoss, SideResult::ForfeitWin);
    pub const DOUBLE_FORFEIT: Self = Self::new(SideResult::ForfeitLoss, SideResult::ForfeitLoss);

    pub const fn new(white: SideResult, black: SideResult) -> Self {
        Self { white, black }
    }

    pub fn for_color(self, color: Color) -> SideResult {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    /// Whether the game was actually played over the board.
    pub fn is_played(self) -> bool {
        self.white.is_played() && self.black.is_played()
    }

    /// Short display code, e.g. `1-0`, `½-½`, `+/-`.
    pub fn code(self) -> String {
        fn side(r: SideResult) -> &'static str {
            match r {
                SideResult::Win | SideResult::UnratedWin => "1",
                SideResult::Draw | SideResult::UnratedDraw => "½",
                SideResult::Loss | SideResult::UnratedLoss => "0",
                SideResult::ForfeitWin => "+",
                SideResult::ForfeitLoss => "-",
            }
        }
        let sep = if self.is_played() { "-" } else { "/" };
        format!("{}{sep}{}", side(self.white), side(self.black))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TeamMatch {
    pub number: u16,
    pub home: TeamId,
    /// `None` when the home team has a bye.
    pub away: Option<TeamId>,
    /// Players fielded by each team, in board order.
    pub home_lineup: Vec<PlayerId>,
    pub away_lineup: Vec<PlayerId>,
    /// Board games built from the line-ups; boards are numbered 1.. within the match.
    pub games: Vec<Pairing>,
}

impl TeamMatch {
    /// Rebuilds `games` from the line-ups. The home team has White on odd
    /// boards. Results are kept for boards whose players did not change.
    pub fn rebuild_games(&mut self) {
        let old = std::mem::take(&mut self.games);
        for (i, (&home, &away)) in self.home_lineup.iter().zip(&self.away_lineup).enumerate() {
            let board = i as u16 + 1;
            let (white, black) = if i % 2 == 0 {
                (home, away)
            } else {
                (away, home)
            };
            let result = old.iter().find_map(|g| match *g {
                Pairing::Game {
                    board: b,
                    white: w,
                    black: bl,
                    result,
                } if b == board && w == white && bl == black => result,
                _ => None,
            });
            self.games.push(Pairing::Game {
                board,
                white,
                black,
                result,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PrizeCategory {
    pub id: PrizeCategoryId,
    pub name: String,
    pub filter: PrizeFilter,
    /// Prize descriptions in place order, e.g. ["500 USD", "300 USD"].
    pub prizes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct PrizeFilter {
    pub min_rating: Option<u16>,
    pub max_rating: Option<u16>,
    pub sex: Option<Sex>,
    /// Born in or after this year (juniors).
    pub born_from: Option<u16>,
    /// Born in or before this year (seniors).
    pub born_until: Option<u16>,
    pub federation: Option<String>,
    pub club: Option<String>,
    pub unrated_only: bool,
}

/// Serde for maps keyed by round. JSON object keys are strings, and serde
/// cannot turn them back into integers inside internally tagged enums such
/// as `Event`, so keys are parsed explicitly.
mod round_keys {
    use super::RoundNo;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;

    pub fn serialize<S: Serializer, V: Serialize>(
        map: &BTreeMap<RoundNo, V>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        map.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
        d: D,
    ) -> Result<BTreeMap<RoundNo, V>, D::Error> {
        BTreeMap::<String, V>::deserialize(d)?
            .into_iter()
            .map(|(k, v)| k.parse().map(|k| (k, v)).map_err(D::Error::custom))
            .collect()
    }
}
