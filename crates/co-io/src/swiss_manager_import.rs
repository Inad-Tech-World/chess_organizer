//! Migration from files *exported* by Swiss-Manager and similar programs.
//!
//! Only documented export formats are read: the FIDE Tournament Report File
//! (TRF) that every rated event is exported to, and player lists saved as
//! CSV or tab-separated text. Nothing here is derived from any program's
//! binaries or private file layouts (see docs/legal.md).

use crate::Result;
use crate::national_list::{self, ColumnMapping};
use co_core::{Error as CoreError, Player, PlayerId, Ratings, Tournament, TournamentInfo};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ImportedFormat {
    Trf,
    PlayerList,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub tournament: Tournament,
    pub format: ImportedFormat,
    pub warnings: Vec<String>,
}

fn looks_like_trf(text: &str) -> bool {
    text.lines()
        .take(200)
        .any(|l| l.starts_with("001 ") || l.starts_with("012 "))
}

/// Reads an exported file: TRF, or a player list (CSV / tab-separated).
pub fn import(bytes: &[u8]) -> Result<Import> {
    // Exports from Windows programs are often Windows-1252 rather than UTF-8.
    let text = national_list::decode(bytes);
    if looks_like_trf(&text) {
        let t = crate::trf::read(&text)?;
        return Ok(Import {
            tournament: t.tournament,
            format: ImportedFormat::Trf,
            warnings: t.warnings,
        });
    }
    let preview = national_list::preview(&text, 0)?;
    let mut mapping = preview.mapping.clone();
    if mapping.name.is_none() && mapping.surname.is_none() {
        return Err(CoreError::Invalid(
            "this file is neither a TRF file nor a player list with a name column".into(),
        )
        .into());
    }
    // Player lists often number players in the first column; never use it as an id.
    if mapping.national_id == Some(0) && preview.headers.first().is_some_and(|h| is_rank_header(h))
    {
        mapping.national_id = None;
    }
    players_from_list(&text, &mapping)
}

fn is_rank_header(h: &str) -> bool {
    matches!(
        h.trim().to_lowercase().trim_end_matches('.'),
        "no" | "nr" | "sno" | "rk" | "#" | "tpn" | "snr"
    )
}

fn players_from_list(text: &str, mapping: &ColumnMapping) -> Result<Import> {
    let rows = national_list::read_csv(text, mapping)?;
    if rows.is_empty() {
        return Err(CoreError::Invalid("the player list is empty".into()).into());
    }
    let mut t = Tournament::create(&co_core::Event::TournamentCreated {
        id: co_core::Uuid::new_v4(),
        info: TournamentInfo {
            name: "Imported tournament".into(),
            ..TournamentInfo::default()
        },
        settings: co_core::Settings::default(),
        kind: co_core::TournamentKind::default(),
    })
    .map_err(CoreError::from)?;
    let mut warnings = Vec::new();
    for (i, r) in rows.into_iter().enumerate() {
        let id = PlayerId(i as u32 + 1);
        let mut p = Player::new(id, r.name);
        p.fide_id = r.fide_id;
        p.national_id = if mapping.national_id.is_some() {
            Some(r.national_id)
        } else {
            None
        };
        p.federation = r.federation;
        p.sex = r.sex;
        p.club = r.club;
        p.birth_date = r.birth_year.map(|y| y.to_string());
        p.title = r.title.as_deref().and_then(co_core::Title::parse);
        // A single rating column is taken as the FIDE standard rating when
        // the player has a FIDE id, otherwise as a national rating.
        p.ratings = if p.fide_id.is_some() {
            Ratings {
                fide_standard: r.rating,
                ..Ratings::default()
            }
        } else {
            Ratings {
                national: r.rating,
                ..Ratings::default()
            }
        };
        if let Err(e) = t.apply(&co_core::Event::PlayerAdded { player: p }) {
            warnings.push(format!("row {}: {e}", i + 2));
        }
    }
    Ok(Import {
        tournament: t,
        format: ImportedFormat::PlayerList,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_trf_exports() {
        let trf = include_str!("../../../testdata/trf/sample.trf");
        let i = import(trf.as_bytes()).unwrap();
        assert_eq!(i.format, ImportedFormat::Trf);
        assert_eq!(i.tournament.players.len(), 7);
    }

    #[test]
    fn reads_player_lists() {
        let csv: &[u8] = b"No.\tName\tFideID\tFED\tRtg\tClub\n1\tOkello, David\t99000001\tUGA\t2310\tKampala Knights\n2\tNakat\xf3, Grace\t\tUGA\t1650\tMengo\n";
        let i = import(csv).unwrap();
        assert_eq!(i.format, ImportedFormat::PlayerList);
        let ps = &i.tournament.players;
        assert_eq!(ps.len(), 2);
        assert_eq!(ps[0].name, "Okello, David");
        assert_eq!(ps[0].ratings.fide_standard, Some(2310));
        assert_eq!(ps[0].national_id, None);
        assert_eq!(ps[1].name, "Nakató, Grace");
        assert_eq!(ps[1].ratings.national, Some(1650));
    }

    #[test]
    fn rejects_unknown_files() {
        assert!(import(b"\x00\x01binary").is_err());
    }
}
