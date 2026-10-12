use crate::Result;
use crate::schema::{self, Kind};
use co_core::chrono::Utc;
use co_core::{Player, PlayerId, Ratings, Sex, Title};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One player from an imported rating list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub fide_id: Option<u32>,
    pub national_id: Option<String>,
    pub name: String,
    pub federation: Option<String>,
    pub sex: Option<Sex>,
    pub title: Option<Title>,
    pub rating_standard: Option<u16>,
    pub rating_rapid: Option<u16>,
    pub rating_blitz: Option<u16>,
    pub national_rating: Option<u16>,
    pub k_factor: Option<u8>,
    pub birth_year: Option<u16>,
    pub club: Option<String>,
}

impl PlayerRecord {
    /// A tournament player pre-filled from this record.
    pub fn to_player(&self, id: PlayerId) -> Player {
        let mut p = Player::new(id, self.name.clone());
        p.fide_id = self.fide_id;
        p.national_id = self.national_id.clone();
        p.federation = self.federation.clone();
        p.sex = self.sex;
        p.title = self.title;
        p.ratings = Ratings {
            fide_standard: self.rating_standard,
            fide_rapid: self.rating_rapid,
            fide_blitz: self.rating_blitz,
            national: self.national_rating,
        };
        p.k_factor = self.k_factor;
        p.birth_date = self.birth_year.map(|y| y.to_string());
        p.club = self.club.clone();
        p
    }
}

/// An imported list, e.g. "FIDE" for period "2026-10".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub period: Option<String>,
    pub imported_at: String,
    pub players: i64,
}

pub struct PlayerDb {
    conn: Connection,
}

impl PlayerDb {
    /// Opens the database at `path`, creating it if needed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        schema::migrate(&mut conn, Kind::PlayerDb, true)?;
        Ok(Self { conn })
    }

    /// Replaces every player of list `name` with `records` in one transaction.
    /// Importing the monthly FIDE list again therefore updates it in place.
    pub fn replace_source(
        &mut self,
        name: &str,
        period: Option<&str>,
        records: impl IntoIterator<Item = PlayerRecord>,
    ) -> Result<usize> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM sources WHERE name = ?1", [name])?;
        tx.execute(
            "INSERT INTO sources (name, period, imported_at) VALUES (?1, ?2, ?3)",
            params![name, period, Utc::now().to_rfc3339()],
        )?;
        let source_id = tx.last_insert_rowid();
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO players (source_id, fide_id, national_id, name, federation, sex,
                     title, rating_standard, rating_rapid, rating_blitz, national_rating,
                     k_factor, birth_year, club)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            )?;
            for r in records {
                stmt.execute(params![
                    source_id,
                    r.fide_id,
                    r.national_id,
                    r.name,
                    r.federation,
                    r.sex.map(sex_code),
                    r.title.map(Title::as_str),
                    r.rating_standard,
                    r.rating_rapid,
                    r.rating_blitz,
                    r.national_rating,
                    r.k_factor,
                    r.birth_year,
                    r.club,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Finds players by FIDE id, national id, or name prefix words
    /// (accent-insensitive), best-rated first.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<PlayerRecord>> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        const COLS: &str = "p.fide_id, p.national_id, p.name, p.federation, p.sex, p.title,
             p.rating_standard, p.rating_rapid, p.rating_blitz, p.national_rating,
             p.k_factor, p.birth_year, p.club";
        let limit = limit as i64;
        if q.chars().all(|c| c.is_ascii_digit()) {
            let sql = format!(
                "SELECT {COLS} FROM players p WHERE p.fide_id = ?1 OR p.national_id = ?2 LIMIT ?3"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![q.parse::<i64>().unwrap_or(-1), q, limit], record)?;
            return Ok(rows.collect::<Result<_, _>>()?);
        }
        // Each word becomes a quoted prefix term: `okello d` -> "okello"* "d"*
        let fts: String = q
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|w| !w.is_empty())
            .map(|w| format!("\"{}\"*", w.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        let sql = format!(
            "SELECT {COLS} FROM players_fts f JOIN players p ON p.id = f.rowid
             WHERE players_fts MATCH ?1
             ORDER BY COALESCE(p.rating_standard, p.national_rating, 0) DESC, p.name
             LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts, limit], record)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn get_by_fide_id(&self, fide_id: u32) -> Result<Option<PlayerRecord>> {
        Ok(self
            .conn
            .query_row(
                "SELECT fide_id, national_id, name, federation, sex, title, rating_standard,
                     rating_rapid, rating_blitz, national_rating, k_factor, birth_year, club
                 FROM players WHERE fide_id = ?1 LIMIT 1",
                [fide_id],
                record,
            )
            .optional()?)
    }

    pub fn sources(&self) -> Result<Vec<Source>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.name, s.period, s.imported_at,
                    (SELECT COUNT(*) FROM players p WHERE p.source_id = s.id)
             FROM sources s ORDER BY s.name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Source {
                id: r.get(0)?,
                name: r.get(1)?,
                period: r.get(2)?,
                imported_at: r.get(3)?,
                players: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

fn sex_code(s: Sex) -> &'static str {
    match s {
        Sex::Male => "M",
        Sex::Female => "F",
    }
}

fn record(r: &Row) -> rusqlite::Result<PlayerRecord> {
    Ok(PlayerRecord {
        fide_id: r.get(0)?,
        national_id: r.get(1)?,
        name: r.get(2)?,
        federation: r.get(3)?,
        sex: r
            .get::<_, Option<String>>(4)?
            .and_then(|s| match s.as_str() {
                "M" => Some(Sex::Male),
                "F" => Some(Sex::Female),
                _ => None,
            }),
        title: r
            .get::<_, Option<String>>(5)?
            .and_then(|t| Title::parse(&t)),
        rating_standard: r.get(6)?,
        rating_rapid: r.get(7)?,
        rating_blitz: r.get(8)?,
        national_rating: r.get(9)?,
        k_factor: r.get(10)?,
        birth_year: r.get(11)?,
        club: r.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(fide_id: u32, name: &str, rating: u16) -> PlayerRecord {
        PlayerRecord {
            fide_id: Some(fide_id),
            name: name.into(),
            federation: Some("UGA".into()),
            rating_standard: Some(rating),
            ..PlayerRecord::default()
        }
    }

    #[test]
    fn search_by_name_prefix_and_id() {
        let mut db = PlayerDb::open_in_memory().unwrap();
        db.replace_source(
            "FIDE",
            Some("2026-10"),
            [
                rec(1, "Okello, David", 2310),
                rec(2, "Okello, Daniel", 2400),
                rec(3, "Müller, Jörg", 2200),
            ],
        )
        .unwrap();

        let hits = db.search("okel d", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].name, "Okello, Daniel", "best rated first");
        assert_eq!(
            db.search("muller", 10).unwrap()[0].fide_id,
            Some(3),
            "accent-insensitive"
        );
        assert_eq!(db.search("3", 10).unwrap()[0].name, "Müller, Jörg");
        assert_eq!(db.get_by_fide_id(1).unwrap().unwrap().name, "Okello, David");
    }

    #[test]
    fn reimport_replaces_the_list() {
        let mut db = PlayerDb::open_in_memory().unwrap();
        db.replace_source("FIDE", Some("2026-09"), [rec(1, "Old Name", 2000)])
            .unwrap();
        db.replace_source("FIDE", Some("2026-10"), [rec(1, "New Name", 2050)])
            .unwrap();
        assert!(db.search("old", 5).unwrap().is_empty());
        let sources = db.sources().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].period.as_deref(), Some("2026-10"));
        assert_eq!(sources[0].players, 1);
    }
}
