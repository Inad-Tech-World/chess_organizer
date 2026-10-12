use crate::Phase;
use crate::ids::{PlayerId, RoundNo, TeamId};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors surfaced to callers of the core and the crates built on it.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A feature whose API exists but whose logic lands in a later phase.
    #[error("{feature} is planned for Phase {phase}")]
    NotYetImplemented { feature: &'static str, phase: Phase },
    #[error(transparent)]
    Rule(#[from] RuleError),
    #[error("{0}")]
    Invalid(String),
}

/// An event that would leave the tournament in an inconsistent state.
///
/// These are integrity checks only. Chess-rule checks such as repeat pairings
/// are reported by the pairing engines as warnings the arbiter may override.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuleError {
    #[error("the tournament has not been created yet")]
    NotCreated,
    #[error("the tournament already exists")]
    AlreadyCreated,
    #[error("player {0} already exists")]
    DuplicatePlayer(PlayerId),
    #[error("unknown player {0}")]
    UnknownPlayer(PlayerId),
    #[error("player {0} has already been paired and cannot be removed; withdraw them instead")]
    PlayerHasGames(PlayerId),
    #[error("player {0} appears more than once in round {1}")]
    PlayerPairedTwice(PlayerId, RoundNo),
    #[error("player {0} cannot play themselves")]
    SelfPairing(PlayerId),
    #[error("board {0} appears more than once")]
    DuplicateBoard(u16),
    #[error("expected round {expected}, got round {got}")]
    RoundOutOfSequence { expected: RoundNo, got: RoundNo },
    #[error("the tournament only has {0} rounds")]
    TooManyRounds(RoundNo),
    #[error("unknown round {0}")]
    UnknownRound(RoundNo),
    #[error("only the last round ({0}) can be deleted")]
    NotLastRound(RoundNo),
    #[error("round {round} has no game on board {board}")]
    UnknownBoard { round: RoundNo, board: u16 },
    #[error("round {0} already has results; delete them first")]
    RoundHasResults(RoundNo),
    #[error("the tournament type cannot change after round 1 has been paired")]
    KindLocked,
    #[error("cannot set {requested} rounds: {played} have already been paired")]
    RoundsBelowPlayed { requested: RoundNo, played: RoundNo },
    #[error("unknown team {0}")]
    UnknownTeam(TeamId),
    #[error("team {0} already exists")]
    DuplicateTeam(TeamId),
    #[error("unknown prize category {0}")]
    UnknownPrizeCategory(u32),
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
    #[error("undo/redo target {0} does not match the history")]
    HistoryMismatch(u64),
    #[error("{0}")]
    Other(String),
}
