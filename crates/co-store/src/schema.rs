//! Schema versions and migrations for both database kinds.

use crate::{Result, StoreError};
use rusqlite::Connection;

/// SQLite `application_id` of `.cot` files ("COT1").
pub const TOURNAMENT_APP_ID: i32 = 0x434F_5431;
/// SQLite `application_id` of the player database ("COP1").
pub const PLAYER_DB_APP_ID: i32 = 0x434F_5031;

const TOURNAMENT_MIGRATIONS: &[&str] = &[
    // v1
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
     CREATE TABLE events (
         seq INTEGER PRIMARY KEY,
         at TEXT NOT NULL,
         kind TEXT NOT NULL,
         payload TEXT NOT NULL
     );
     CREATE TABLE snapshots (seq INTEGER PRIMARY KEY, state TEXT NOT NULL);",
];

const PLAYER_DB_MIGRATIONS: &[&str] = &[
    // v1
    "CREATE TABLE sources (
         id INTEGER PRIMARY KEY,
         name TEXT NOT NULL UNIQUE,
         period TEXT,
         imported_at TEXT NOT NULL
     );
     CREATE TABLE players (
         id INTEGER PRIMARY KEY,
         source_id INTEGER REFERENCES sources(id) ON DELETE CASCADE,
         fide_id INTEGER,
         national_id TEXT,
         name TEXT NOT NULL,
         federation TEXT,
         sex TEXT,
         title TEXT,
         rating_standard INTEGER,
         rating_rapid INTEGER,
         rating_blitz INTEGER,
         national_rating INTEGER,
         k_factor INTEGER,
         birth_year INTEGER,
         club TEXT
     );
     CREATE INDEX players_fide_id ON players(fide_id);
     CREATE INDEX players_national_id ON players(national_id);
     CREATE VIRTUAL TABLE players_fts USING fts5(
         name, content='players', content_rowid='id',
         tokenize='unicode61 remove_diacritics 2'
     );
     CREATE TRIGGER players_ai AFTER INSERT ON players BEGIN
         INSERT INTO players_fts(rowid, name) VALUES (new.id, new.name);
     END;
     CREATE TRIGGER players_ad AFTER DELETE ON players BEGIN
         INSERT INTO players_fts(players_fts, rowid, name) VALUES ('delete', old.id, old.name);
     END;
     CREATE TRIGGER players_au AFTER UPDATE ON players BEGIN
         INSERT INTO players_fts(players_fts, rowid, name) VALUES ('delete', old.id, old.name);
         INSERT INTO players_fts(rowid, name) VALUES (new.id, new.name);
     END;",
];

pub enum Kind {
    Tournament,
    PlayerDb,
}

/// Brings a database to the current schema, refusing files from other
/// programs or from newer versions of this one. An empty database is only
/// initialised when `allow_fresh` is set (i.e. when creating a new file).
pub fn migrate(conn: &mut Connection, kind: Kind, allow_fresh: bool) -> Result<()> {
    let (app_id, migrations, what) = match kind {
        Kind::Tournament => (TOURNAMENT_APP_ID, TOURNAMENT_MIGRATIONS, "tournament file"),
        Kind::PlayerDb => (PLAYER_DB_APP_ID, PLAYER_DB_MIGRATIONS, "player database"),
    };
    let current: i32 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let version = usize::try_from(version).unwrap_or(usize::MAX);
    let fresh = allow_fresh && current == 0 && version == 0;
    if !fresh && current != app_id {
        return Err(StoreError::Format(format!(
            "this is not a Chess Organiser {what}"
        )));
    }
    if version > migrations.len() {
        return Err(StoreError::Format(format!(
            "this {what} was saved by a newer version of Chess Organiser; please update"
        )));
    }
    let tx = conn.transaction()?;
    for sql in &migrations[version..] {
        tx.execute_batch(sql)?;
    }
    tx.pragma_update(None, "application_id", app_id)?;
    tx.pragma_update(None, "user_version", migrations.len() as i64)?;
    tx.commit()?;
    Ok(())
}
