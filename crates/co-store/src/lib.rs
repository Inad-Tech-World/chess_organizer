//! Storage.
//!
//! * [`TournamentFile`]: one `.cot` file per tournament, an SQLite database
//!   holding the append-only event log. Every event commits in its own
//!   transaction with full sync, so the file survives a crash mid-round and
//!   can be copied or e-mailed as a single file.
//! * [`PlayerDb`]: the arbiter's local player database (FIDE and national
//!   rating lists), shared by all tournaments.

mod player_db;
mod schema;
mod tournament_file;

pub use player_db::{PlayerDb, PlayerRecord, Source};
pub use tournament_file::TournamentFile;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("corrupt record: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Core(#[from] co_core::Error),
    #[error(transparent)]
    Rule(#[from] co_core::RuleError),
    #[error("{0}")]
    Format(String),
}

pub type Result<T, E = StoreError> = std::result::Result<T, E>;
