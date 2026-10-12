//! National rating list import from CSV with a user-chosen column mapping.
//!
//! Federations publish lists in many shapes, so the arbiter picks which
//! column holds which field. [`preview`] shows the first rows and a guessed
//! mapping; [`read_csv`] then reads every row with the mapping confirmed.

use crate::Result;
use co_core::{Error as CoreError, Sex};
use serde::{Deserialize, Serialize};

/// Which CSV column (0-based) holds each field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ColumnMapping {
    pub national_id: Option<usize>,
    pub name: Option<usize>,
    /// Lists that split the name: "Surname, First name" is built from both.
    pub surname: Option<usize>,
    pub first_name: Option<usize>,
    pub rating: Option<usize>,
    pub fide_id: Option<usize>,
    pub federation: Option<usize>,
    pub sex: Option<usize>,
    pub birth_year: Option<usize>,
    pub club: Option<usize>,
    pub title: Option<usize>,
    /// The first row holds column names.
    pub has_header: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NationalEntry {
    pub national_id: String,
    pub name: String,
    pub rating: Option<u16>,
    pub fide_id: Option<u32>,
    pub federation: Option<String>,
    pub sex: Option<Sex>,
    pub club: Option<String>,
    pub birth_year: Option<u16>,
    pub title: Option<String>,
}

/// The first rows of a file, for the mapping dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub delimiter: char,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub mapping: ColumnMapping,
}

/// Picks the delimiter that splits the first line into the most columns.
fn sniff_delimiter(text: &str) -> u8 {
    let first = text.lines().next().unwrap_or("");
    b";,\t|"
        .iter()
        .copied()
        .max_by_key(|d| first.bytes().filter(|b| b == d).count())
        .unwrap_or(b',')
}

fn reader(text: &str) -> csv::Reader<&[u8]> {
    csv::ReaderBuilder::new()
        .delimiter(sniff_delimiter(text))
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes())
}

/// Strips a UTF-8 BOM and decodes Latin-1 files (common in federation exports).
pub fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// Guesses the mapping from header names in several languages.
pub fn guess_mapping(headers: &[String]) -> ColumnMapping {
    let norm: Vec<String> = headers
        .iter()
        .map(|h| {
            h.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect()
        })
        .collect();
    let find = |keys: &[&str]| norm.iter().position(|h| keys.iter().any(|k| h == k));
    let mut m = ColumnMapping {
        national_id: find(&[
            "id",
            "nationalid",
            "natid",
            "code",
            "memberid",
            "member",
            "membership",
            "licence",
            "license",
            "nr",
            "no",
            "number",
            "codigo",
            "matricule",
        ]),
        name: find(&[
            "name",
            "fullname",
            "player",
            "playername",
            "nom",
            "nombre",
            "nome",
            "jina",
        ]),
        surname: find(&[
            "surname",
            "lastname",
            "familyname",
            "nomdefamille",
            "apellido",
            "apellidos",
            "sobrenome",
        ]),
        first_name: find(&[
            "firstname",
            "givenname",
            "forename",
            "prenom",
            "prénom",
            "nombredepila",
            "primeironome",
        ]),
        rating: find(&[
            "rating",
            "elo",
            "rtg",
            "nationalrating",
            "natrating",
            "rating2",
            "puntos",
            "classement",
        ]),
        fide_id: find(&["fideid", "fide", "idfide", "fidecode", "fideno"]),
        federation: find(&[
            "fed",
            "federation",
            "country",
            "nat",
            "pays",
            "pais",
            "país",
        ]),
        sex: find(&["sex", "gender", "sexe", "sexo", "s"]),
        birth_year: find(&[
            "birthyear",
            "born",
            "birth",
            "yob",
            "birthdate",
            "dob",
            "byear",
            "anneenaissance",
            "naissance",
            "nacimiento",
            "nascimento",
        ]),
        club: find(&["club", "clubname", "team", "clube"]),
        title: find(&["title", "tit", "titre", "titulo"]),
        has_header: false,
    };
    // A first row with recognisable column names is a header row.
    m.has_header = [
        m.national_id,
        m.name,
        m.surname,
        m.first_name,
        m.rating,
        m.fide_id,
        m.federation,
        m.sex,
        m.birth_year,
        m.club,
        m.title,
    ]
    .iter()
    .any(Option::is_some);
    if m.name.is_some() {
        m.surname = None;
        m.first_name = None;
    }
    m
}

/// Reads the first `limit` rows and guesses a mapping.
pub fn preview(text: &str, limit: usize) -> Result<Preview> {
    let mut rows = Vec::new();
    for rec in reader(text).records().take(limit + 1) {
        let rec = rec.map_err(|e| CoreError::Invalid(format!("CSV: {e}")))?;
        rows.push(rec.iter().map(str::to_string).collect::<Vec<_>>());
    }
    let first = rows.first().cloned().unwrap_or_default();
    let mapping = guess_mapping(&first);
    let headers = if mapping.has_header {
        rows.remove(0);
        first
    } else {
        rows.truncate(limit);
        (1..=first.len()).map(|i| format!("{i}")).collect()
    };
    Ok(Preview {
        delimiter: sniff_delimiter(text) as char,
        headers,
        rows,
        mapping,
    })
}

fn year(s: &str) -> Option<u16> {
    // "1998", "1998-04-02", "02.04.1998", "02/04/1998"
    let digits: Vec<&str> = s
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| p.len() == 4)
        .collect();
    digits
        .first()
        .and_then(|y| y.parse().ok())
        .filter(|y| (1850..=2100).contains(y))
}

/// Reads every row with `mapping`. Rows without a name are skipped; rows
/// without a national id get their row number as id.
pub fn read_csv(text: &str, mapping: &ColumnMapping) -> Result<Vec<NationalEntry>> {
    if mapping.name.is_none() && mapping.surname.is_none() {
        return Err(
            CoreError::Invalid("choose the column that holds the player's name".into()).into(),
        );
    }
    let mut out = Vec::new();
    for (i, rec) in reader(text).records().enumerate() {
        if i == 0 && mapping.has_header {
            continue;
        }
        let rec = rec.map_err(|e| CoreError::Invalid(format!("CSV row {}: {e}", i + 1)))?;
        let get = |c: Option<usize>| {
            c.and_then(|c| rec.get(c))
                .map(str::trim)
                .filter(|s| !s.is_empty())
        };
        let name = match (
            get(mapping.name),
            get(mapping.surname),
            get(mapping.first_name),
        ) {
            (Some(n), _, _) => n.to_string(),
            (None, Some(s), Some(f)) => format!("{s}, {f}"),
            (None, Some(s), None) => s.to_string(),
            (None, None, Some(f)) => f.to_string(),
            (None, None, None) => continue,
        };
        out.push(NationalEntry {
            national_id: get(mapping.national_id)
                .map_or_else(|| (i + 1).to_string(), str::to_string),
            name,
            rating: get(mapping.rating)
                .and_then(|r| r.parse::<f32>().ok())
                .map(|r| r.round() as u16)
                .filter(|r| *r > 0),
            fide_id: get(mapping.fide_id)
                .and_then(|r| r.parse().ok())
                .filter(|id| *id > 0),
            federation: get(mapping.federation).map(|s| s.to_ascii_uppercase()),
            sex: get(mapping.sex).and_then(|s| match s.to_lowercase().as_str() {
                "m" | "male" | "man" | "h" | "homme" | "hombre" | "masculino" | "1" => {
                    Some(Sex::Male)
                }
                "f" | "w" | "female" | "woman" | "femme" | "mujer" | "feminino" | "femenino"
                | "2" => Some(Sex::Female),
                _ => None,
            }),
            club: get(mapping.club).map(str::to_string),
            birth_year: get(mapping.birth_year).and_then(year),
            title: get(mapping.title).map(str::to_string),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "Member ID;Surname;First name;Rating;FIDE ID;Club;Born;Sex\n\
UG-001;Okello;David;2310;99000001;Kampala Knights;02.04.1998;M\n\
UG-002;Nakato;Grace;2105;;Mengo CC;2010;F\n\
;;;;;;;\n";

    #[test]
    fn guesses_mapping_and_reads_rows() {
        let p = preview(CSV, 10).unwrap();
        assert_eq!(p.delimiter, ';');
        assert_eq!(p.headers[0], "Member ID");
        assert_eq!(p.rows.len(), 3);
        let m = &p.mapping;
        assert!(m.has_header);
        assert_eq!(
            (m.national_id, m.surname, m.first_name, m.rating),
            (Some(0), Some(1), Some(2), Some(3))
        );
        assert_eq!(
            (m.fide_id, m.club, m.birth_year, m.sex),
            (Some(4), Some(5), Some(6), Some(7))
        );

        let rows = read_csv(CSV, m).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Okello, David");
        assert_eq!(rows[0].national_id, "UG-001");
        assert_eq!(rows[0].rating, Some(2310));
        assert_eq!(rows[0].fide_id, Some(99_000_001));
        assert_eq!(rows[0].birth_year, Some(1998));
        assert_eq!(rows[1].fide_id, None);
        assert_eq!(rows[1].sex, Some(Sex::Female));
    }

    #[test]
    fn headerless_files_use_column_numbers() {
        let p = preview("1,Okello David,2310\n2,Nakato Grace,2105\n", 10).unwrap();
        assert!(!p.mapping.has_header);
        assert_eq!(p.headers, ["1", "2", "3"]);
        assert_eq!(p.rows.len(), 2);
        let m = ColumnMapping {
            national_id: Some(0),
            name: Some(1),
            rating: Some(2),
            ..Default::default()
        };
        let rows = read_csv("1,Okello David,2310\n", &m).unwrap();
        assert_eq!(rows[0].name, "Okello David");
    }

    #[test]
    fn a_name_column_is_required() {
        assert!(read_csv(CSV, &ColumnMapping::default()).is_err());
    }

    #[test]
    fn latin1_is_decoded() {
        assert_eq!(decode(b"Nakat\xf3"), "Nakató");
        assert_eq!(decode(b"\xEF\xBB\xBFabc"), "abc");
    }
}
