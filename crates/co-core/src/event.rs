//! Every change to a tournament is recorded as an [`Event`].
//!
//! The event log is append-only: it is the tournament's audit trail and the
//! basis for unlimited undo/redo (see [`crate::history`]).

use crate::ids::{PlayerId, PrizeCategoryId, RoundNo, TeamId};
use crate::model::{
    ByeKind, GameResult, Pairing, PairingMethod, Player, PrizeCategory, Settings, Team, TeamMatch,
    Tournament, TournamentInfo, TournamentKind,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Event {
    TournamentCreated {
        id: Uuid,
        info: TournamentInfo,
        settings: Settings,
        kind: TournamentKind,
    },
    /// Creation from another program's file (e.g. TRF); carries the full state.
    TournamentImported {
        tournament: Box<Tournament>,
        source: String,
    },
    InfoUpdated {
        info: TournamentInfo,
    },
    SettingsUpdated {
        settings: Settings,
    },
    KindChanged {
        kind: TournamentKind,
    },

    PlayerAdded {
        player: Player,
    },
    PlayerUpdated {
        player: Player,
    },
    PlayerRemoved {
        player: PlayerId,
    },
    PlayerWithdrawn {
        player: PlayerId,
        from_round: RoundNo,
    },
    PlayerReentered {
        player: PlayerId,
    },
    /// Request (or with `None`, cancel) a bye or pause for a future round.
    ByeSet {
        player: PlayerId,
        round: RoundNo,
        kind: Option<ByeKind>,
    },
    /// New starting-rank order; players not listed keep their relative order after these.
    StartingRanksSet {
        order: Vec<PlayerId>,
    },

    RoundPaired {
        round: RoundNo,
        pairings: Vec<Pairing>,
        matches: Vec<TeamMatch>,
        method: PairingMethod,
    },
    /// Manual override of an existing round's pairings.
    PairingsEdited {
        round: RoundNo,
        pairings: Vec<Pairing>,
    },
    RoundDeleted {
        round: RoundNo,
    },
    ResultEntered {
        round: RoundNo,
        board: u16,
        result: Option<GameResult>,
    },

    TeamAdded {
        team: Team,
    },
    TeamUpdated {
        team: Team,
    },
    TeamRemoved {
        team: TeamId,
    },
    /// Line-up (board order) of one team in one match of a round.
    LineupSet {
        round: RoundNo,
        match_number: u16,
        team: TeamId,
        lineup: Vec<PlayerId>,
    },
    TeamGameResultEntered {
        round: RoundNo,
        match_number: u16,
        board: u16,
        result: Option<GameResult>,
    },

    PrizeCategoryAdded {
        category: PrizeCategory,
    },
    PrizeCategoryUpdated {
        category: PrizeCategory,
    },
    PrizeCategoryRemoved {
        category: PrizeCategoryId,
    },

    /// Undo the effective event with sequence number `target`.
    Undo {
        target: u64,
    },
    /// Redo the previously undone event `target`.
    Redo {
        target: u64,
    },
}

impl Event {
    /// Stable name stored alongside the payload, useful for querying the log.
    pub fn kind(&self) -> &'static str {
        match self {
            Event::TournamentCreated { .. } => "TournamentCreated",
            Event::TournamentImported { .. } => "TournamentImported",
            Event::InfoUpdated { .. } => "InfoUpdated",
            Event::SettingsUpdated { .. } => "SettingsUpdated",
            Event::KindChanged { .. } => "KindChanged",
            Event::PlayerAdded { .. } => "PlayerAdded",
            Event::PlayerUpdated { .. } => "PlayerUpdated",
            Event::PlayerRemoved { .. } => "PlayerRemoved",
            Event::PlayerWithdrawn { .. } => "PlayerWithdrawn",
            Event::PlayerReentered { .. } => "PlayerReentered",
            Event::ByeSet { .. } => "ByeSet",
            Event::StartingRanksSet { .. } => "StartingRanksSet",
            Event::RoundPaired { .. } => "RoundPaired",
            Event::PairingsEdited { .. } => "PairingsEdited",
            Event::RoundDeleted { .. } => "RoundDeleted",
            Event::ResultEntered { .. } => "ResultEntered",
            Event::TeamAdded { .. } => "TeamAdded",
            Event::TeamUpdated { .. } => "TeamUpdated",
            Event::TeamRemoved { .. } => "TeamRemoved",
            Event::LineupSet { .. } => "LineupSet",
            Event::TeamGameResultEntered { .. } => "TeamGameResultEntered",
            Event::PrizeCategoryAdded { .. } => "PrizeCategoryAdded",
            Event::PrizeCategoryUpdated { .. } => "PrizeCategoryUpdated",
            Event::PrizeCategoryRemoved { .. } => "PrizeCategoryRemoved",
            Event::Undo { .. } => "Undo",
            Event::Redo { .. } => "Redo",
        }
    }

    pub fn is_history_marker(&self) -> bool {
        matches!(self, Event::Undo { .. } | Event::Redo { .. })
    }
}

/// An event as stored in the log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Envelope {
    /// 1-based, strictly increasing position in the log.
    pub seq: u64,
    pub at: DateTime<Utc>,
    pub event: Event,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::PlayerId;

    #[test]
    fn events_round_trip_through_json() {
        let mut player = Player::new(PlayerId(3), "Bye, Requested");
        player.byes.insert(4, ByeKind::Half);
        let events = [
            Event::PlayerAdded {
                player: player.clone(),
            },
            Event::TournamentImported {
                tournament: Box::new(Tournament {
                    players: vec![player],
                    ..Tournament::create(&Event::TournamentCreated {
                        id: Uuid::nil(),
                        info: TournamentInfo::default(),
                        settings: Settings::default(),
                        kind: TournamentKind::default(),
                    })
                    .unwrap()
                }),
                source: "test".into(),
            },
        ];
        for ev in events {
            let json = serde_json::to_string(&ev).unwrap();
            assert_eq!(serde_json::from_str::<Event>(&json).unwrap(), ev, "{json}");
        }
    }
}
