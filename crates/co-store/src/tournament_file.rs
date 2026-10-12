use crate::schema::{self, Kind};
use crate::{Result, StoreError};
use co_core::chrono::{DateTime, Utc};
use co_core::history::{self, HistoryCursor};
use co_core::{Envelope, Event, RuleError, Tournament};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use std::path::{Path, PathBuf};

/// An open `.cot` tournament file and the state rebuilt from it.
pub struct TournamentFile {
    conn: Connection,
    path: PathBuf,
    log: Vec<Envelope>,
    cursor: HistoryCursor,
    state: Tournament,
}

impl TournamentFile {
    pub const EXTENSION: &'static str = "cot";

    /// Creates a new file whose first event is `first`
    /// (`TournamentCreated` or `TournamentImported`).
    pub fn create(path: impl AsRef<Path>, first: Event) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            return Err(StoreError::Format(format!(
                "{} already exists",
                path.display()
            )));
        }
        let state = Tournament::create(&first)?;
        let mut conn = connect(path, true)?;
        schema::migrate(&mut conn, Kind::Tournament, true)?;
        let tx = conn.transaction()?;
        for (k, v) in [
            ("format", "chess-organiser-tournament"),
            ("app_version", env!("CARGO_PKG_VERSION")),
            ("tournament_id", &state.id.to_string()),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
        tx.commit()?;
        let mut file = Self {
            conn,
            path: path.to_path_buf(),
            log: Vec::new(),
            cursor: HistoryCursor::default(),
            state,
        };
        let env = file.insert(first)?;
        file.cursor.push(&env)?;
        file.log.push(env);
        Ok(file)
    }

    /// Opens an existing file and replays its log.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut conn = connect(path, false)?;
        schema::migrate(&mut conn, Kind::Tournament, false)?;
        let log = {
            let mut stmt = conn.prepare("SELECT seq, at, payload FROM events ORDER BY seq")?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            let mut log = Vec::new();
            for row in rows {
                let (seq, at, payload) = row?;
                log.push(Envelope {
                    seq: seq as u64,
                    at: parse_time(&at)?,
                    event: serde_json::from_str(&payload)?,
                });
            }
            log
        };
        let last = log.last().map_or(0, |e| e.seq);
        let snapshot: Option<String> = conn
            .query_row(
                "SELECT state FROM snapshots WHERE seq = ?1",
                [last as i64],
                |r| r.get(0),
            )
            .optional()?;
        let (state, cursor) = match snapshot {
            Some(json) => (serde_json::from_str(&json)?, HistoryCursor::from_log(&log)?),
            None => history::replay(&log)?,
        };
        Ok(Self {
            conn,
            path: path.to_path_buf(),
            log,
            cursor,
            state,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn state(&self) -> &Tournament {
        &self.state
    }

    /// The full audit trail, including undo/redo markers.
    pub fn log(&self) -> &[Envelope] {
        &self.log
    }

    /// Validates `event` against the current state, then commits it.
    pub fn append(&mut self, event: Event) -> Result<&Tournament> {
        if event.is_history_marker() {
            return Err(StoreError::Format("use undo() or redo()".into()));
        }
        let mut next = self.state.clone();
        next.apply(&event)?;
        let env = self.insert(event)?;
        self.cursor.push(&env)?;
        self.log.push(env);
        self.state = next;
        Ok(&self.state)
    }

    pub fn can_undo(&self) -> bool {
        self.cursor.next_undo().is_some()
    }

    pub fn can_redo(&self) -> bool {
        self.cursor.next_redo().is_some()
    }

    /// The event the next undo would revert.
    pub fn undo_target(&self) -> Option<&Envelope> {
        let seq = self.cursor.next_undo()?;
        self.log.iter().find(|e| e.seq == seq)
    }

    pub fn undo(&mut self) -> Result<&Tournament> {
        let target = self.cursor.next_undo().ok_or(RuleError::NothingToUndo)?;
        self.append_marker(Event::Undo { target })
    }

    pub fn redo(&mut self) -> Result<&Tournament> {
        let target = self.cursor.next_redo().ok_or(RuleError::NothingToRedo)?;
        self.append_marker(Event::Redo { target })
    }

    fn append_marker(&mut self, marker: Event) -> Result<&Tournament> {
        let mut log = self.log.clone();
        log.push(Envelope {
            seq: self.next_seq(),
            at: Utc::now(),
            event: marker.clone(),
        });
        let (state, cursor) = history::replay(&log)?;
        let env = self.insert(marker)?;
        self.log.push(env);
        self.cursor = cursor;
        self.state = state;
        Ok(&self.state)
    }

    /// Stores the current state so the next open skips the replay, and so
    /// other tools can read the state without the reducer.
    pub fn checkpoint(&self) -> Result<()> {
        let json = serde_json::to_string(&self.state)?;
        let seq = self.log.last().map_or(0, |e| e.seq) as i64;
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM snapshots", [])?;
        tx.execute(
            "INSERT INTO snapshots (seq, state) VALUES (?1, ?2)",
            params![seq, json],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Writes a consistent copy to `path` (which must not exist).
    pub fn save_as(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            return Err(StoreError::Format(format!(
                "{} already exists",
                path.display()
            )));
        }
        self.checkpoint()?;
        let target = path
            .to_str()
            .ok_or_else(|| StoreError::Format("path is not valid UTF-8".into()))?;
        self.conn.execute("VACUUM INTO ?1", [target])?;
        Ok(())
    }

    fn next_seq(&self) -> u64 {
        self.log.last().map_or(1, |e| e.seq + 1)
    }

    fn insert(&mut self, event: Event) -> Result<Envelope> {
        let env = Envelope {
            seq: self.next_seq(),
            at: Utc::now(),
            event,
        };
        let payload = serde_json::to_string(&env.event)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO events (seq, at, kind, payload) VALUES (?1, ?2, ?3, ?4)",
            params![
                env.seq as i64,
                env.at.to_rfc3339(),
                env.event.kind(),
                payload
            ],
        )?;
        tx.commit()?;
        Ok(env)
    }
}

fn connect(path: &Path, create: bool) -> Result<Connection> {
    let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    if create {
        flags |= OpenFlags::SQLITE_OPEN_CREATE;
    }
    let conn = Connection::open_with_flags(path, flags)?;
    // A rollback journal (not WAL) keeps the tournament a single file;
    // FULL sync makes each committed event durable.
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    Ok(conn)
}

fn parse_time(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| StoreError::Format(format!("invalid timestamp '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use co_core::*;

    fn created() -> Event {
        Event::TournamentCreated {
            id: Uuid::new_v4(),
            info: TournamentInfo {
                name: "Test Open".into(),
                ..TournamentInfo::default()
            },
            settings: Settings::default(),
            kind: TournamentKind::default(),
        }
    }

    fn add(file: &mut TournamentFile, id: u32) {
        file.append(Event::PlayerAdded {
            player: Player::new(PlayerId(id), format!("Player {id}")),
        })
        .unwrap();
    }

    #[test]
    fn events_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.cot");
        {
            let mut f = TournamentFile::create(&path, created()).unwrap();
            add(&mut f, 1);
            add(&mut f, 2);
            // Dropped without checkpoint, as after a crash.
        }
        let f = TournamentFile::open(&path).unwrap();
        assert_eq!(f.state().info.name, "Test Open");
        assert_eq!(f.state().players.len(), 2);
        assert_eq!(f.log().len(), 3);
    }

    #[test]
    fn rejected_events_are_not_stored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.cot");
        let mut f = TournamentFile::create(&path, created()).unwrap();
        add(&mut f, 1);
        let err = f
            .append(Event::PlayerAdded {
                player: Player::new(PlayerId(1), "Dup"),
            })
            .err()
            .unwrap();
        assert!(matches!(
            err,
            StoreError::Rule(RuleError::DuplicatePlayer(_))
        ));
        drop(f);
        assert_eq!(TournamentFile::open(&path).unwrap().log().len(), 2);
    }

    #[test]
    fn undo_redo_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.cot");
        let mut f = TournamentFile::create(&path, created()).unwrap();
        add(&mut f, 1);
        add(&mut f, 2);
        assert!(!f.can_redo());
        f.undo().unwrap();
        assert_eq!(f.state().players.len(), 1);
        assert!(f.can_redo());
        drop(f);

        let mut f = TournamentFile::open(&path).unwrap();
        assert_eq!(f.state().players.len(), 1);
        f.redo().unwrap();
        assert_eq!(f.state().players.len(), 2);
        f.undo().unwrap();
        f.undo().unwrap();
        assert_eq!(f.state().players.len(), 0);
        assert!(!f.can_undo(), "creation cannot be undone");
        assert!(f.undo().is_err());
    }

    #[test]
    fn checkpoint_and_save_as() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.cot");
        let copy = dir.path().join("copy.cot");
        let mut f = TournamentFile::create(&path, created()).unwrap();
        add(&mut f, 1);
        f.save_as(&copy).unwrap();
        add(&mut f, 2);

        let c = TournamentFile::open(&copy).unwrap();
        assert_eq!(c.state().players.len(), 1);
        let o = TournamentFile::open(&path).unwrap();
        assert_eq!(
            o.state().players.len(),
            2,
            "stale snapshot must not be used"
        );
    }

    #[test]
    fn refuses_foreign_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("other.db");
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE x (y);")
            .unwrap();
        assert!(matches!(
            TournamentFile::open(&path),
            Err(StoreError::Format(_))
        ));
        assert!(TournamentFile::create(&path, created()).is_err());
    }
}
