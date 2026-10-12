//! FIDE rating list import.
//!
//! FIDE publishes monthly lists as zipped XML (`players_list_xml.zip`) and
//! fixed-width text (`players_list.zip`). Both are streamed entry by entry so
//! the full list (well over a million players) never sits in memory.

use crate::Result;
use co_core::{Error as CoreError, Sex, Title};
use quick_xml::events::Event as XmlEvent;
use std::io::{BufRead, BufReader, Read, Seek};

/// Where FIDE publishes the combined standard/rapid/blitz list.
pub const FIDE_XML_URL: &str = "https://ratings.fide.com/download/players_list_xml.zip";
pub const FIDE_TXT_URL: &str = "https://ratings.fide.com/download/players_list.zip";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FideListEntry {
    pub fide_id: u32,
    pub name: String,
    pub federation: String,
    pub sex: Option<Sex>,
    pub title: Option<Title>,
    pub rating_standard: Option<u16>,
    pub rating_rapid: Option<u16>,
    pub rating_blitz: Option<u16>,
    pub k_factor: Option<u8>,
    pub birth_year: Option<u16>,
    /// FIDE flag, e.g. `i` (inactive), `w` (woman), `wi`.
    pub flag: Option<String>,
}

fn invalid(msg: impl Into<String>) -> crate::Error {
    CoreError::Invalid(msg.into()).into()
}

fn rating(s: &str) -> Option<u16> {
    s.trim().parse::<u16>().ok().filter(|r| *r > 0)
}

fn sex(s: &str) -> Option<Sex> {
    match s.trim() {
        "M" | "m" => Some(Sex::Male),
        "F" | "f" => Some(Sex::Female),
        _ => None,
    }
}

/// The open title takes precedence over the women's title (a WGM who is also
/// an IM is listed as IM).
fn best_title(open: &str, women: &str) -> Option<Title> {
    Title::parse(open).or_else(|| Title::parse(women))
}

/// Streams entries from FIDE's XML list, returning the number read.
pub fn read_xml<R: BufRead>(reader: R, mut on_entry: impl FnMut(FideListEntry)) -> Result<usize> {
    let mut xml = quick_xml::Reader::from_reader(reader);
    // Not trimmed: text around `&amp;` arrives in separate events.
    xml.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut field = String::new();
    let mut text = String::new();
    let mut fields: Vec<(String, String)> = Vec::with_capacity(24);
    let mut in_player = false;
    let mut count = 0;
    loop {
        let ev = xml.read_event_into(&mut buf).map_err(|e| {
            invalid(format!(
                "FIDE list XML at byte {}: {e}",
                xml.buffer_position()
            ))
        })?;
        match ev {
            XmlEvent::Start(e) => {
                let name = e.local_name().as_ref().to_ascii_lowercase();
                if name == "player" {
                    in_player = true;
                    fields.clear();
                } else if in_player {
                    field = name;
                    text.clear();
                }
            }
            XmlEvent::Text(t) if in_player => text.push_str(&t.xml10_content()),
            XmlEvent::CData(t) if in_player => text.push_str(&t),
            XmlEvent::GeneralRef(r) if in_player => {
                if let Ok(Some(c)) = r.resolve_char_ref() {
                    text.push(c);
                } else {
                    text.push_str(match &*r {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "apos" => "'",
                        "quot" => "\"",
                        _ => "",
                    });
                }
            }
            XmlEvent::End(e) => {
                let name = e.local_name().as_ref().to_ascii_lowercase();
                if name == "player" {
                    in_player = false;
                    if let Some(entry) = entry_from_xml(&fields) {
                        on_entry(entry);
                        count += 1;
                    }
                } else if in_player && name == field {
                    fields.push((std::mem::take(&mut field), text.trim().to_string()));
                    text.clear();
                }
            }
            XmlEvent::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(count)
}

fn entry_from_xml(fields: &[(String, String)]) -> Option<FideListEntry> {
    let get = |k: &str| {
        fields
            .iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.as_str())
    };
    let fide_id = get("fideid").parse().ok()?;
    let name = get("name").to_string();
    if name.is_empty() {
        return None;
    }
    // Single-type lists (e.g. the standard list) use <rating>/<k>; the combined
    // list adds rapid_ and blitz_ fields.
    let k = get("k");
    let k = if k.is_empty() { get("rating_k") } else { k };
    Some(FideListEntry {
        fide_id,
        name,
        federation: get("country").to_string(),
        sex: sex(get("sex")),
        title: best_title(get("title"), get("w_title")),
        rating_standard: rating(get("rating")),
        rating_rapid: rating(get("rapid_rating")),
        rating_blitz: rating(get("blitz_rating")),
        k_factor: k.parse().ok(),
        birth_year: get("birthday")
            .get(..4)
            .and_then(|y| y.parse().ok())
            .filter(|y| *y > 1850),
        flag: Some(get("flag").to_string()).filter(|f| !f.is_empty()),
    })
}

/// Column positions taken from the TXT list's header line.
struct TxtLayout {
    cols: Vec<(String, usize)>,
}

impl TxtLayout {
    fn parse(header: &[u8]) -> Option<Self> {
        let line = decode(header);
        let mut cols = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b' ' {
                i += 1;
                continue;
            }
            let start = i;
            while i < bytes.len() && bytes[i] != b' ' {
                i += 1;
            }
            let mut label = line[start..i].to_string();
            // "ID Number" is the only label containing a space.
            if label == "ID" && line[i..].starts_with(" Number") {
                i += " Number".len();
                label = "ID Number".into();
            }
            cols.push((label, start));
        }
        cols.iter()
            .any(|(l, _)| l == "ID Number")
            .then_some(Self { cols })
    }

    fn field(&self, line: &[char], label: impl Fn(&str) -> bool) -> String {
        let Some(i) = self.cols.iter().position(|(l, _)| label(l)) else {
            return String::new();
        };
        let start = self.cols[i].1.min(line.len());
        let end = self
            .cols
            .get(i + 1)
            .map_or(line.len(), |c| c.1)
            .min(line.len());
        line[start..end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }
}

/// UTF-8 when valid, otherwise Latin-1 (older FIDE files).
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_end_matches(['\r', '\n']).to_string(),
        Err(_) => bytes
            .iter()
            .map(|&b| b as char)
            .collect::<String>()
            .trim_end_matches(['\r', '\n'])
            .to_string(),
    }
}

/// A single-type list labels its rating column with the period, e.g. `OCT26`.
fn is_period_label(l: &str) -> bool {
    l.len() == 5
        && l[..3].chars().all(|c| c.is_ascii_uppercase())
        && l[3..].chars().all(|c| c.is_ascii_digit())
}

/// Streams entries from FIDE's fixed-width TXT list. A single-type list's
/// rating column (labelled with the period) is read as the standard rating.
pub fn read_txt<R: BufRead>(
    mut reader: R,
    mut on_entry: impl FnMut(FideListEntry),
) -> Result<usize> {
    let mut raw = Vec::new();
    reader
        .read_until(b'\n', &mut raw)
        .map_err(|e| invalid(e.to_string()))?;
    let layout = TxtLayout::parse(&raw)
        .ok_or_else(|| invalid("not a FIDE rating list (no 'ID Number' header)"))?;
    let mut count = 0;
    loop {
        raw.clear();
        if reader
            .read_until(b'\n', &mut raw)
            .map_err(|e| invalid(e.to_string()))?
            == 0
        {
            break;
        }
        // Columns are counted in characters, so names with accents keep the
        // later columns aligned.
        let line: Vec<char> = decode(&raw).chars().collect();
        let f = |l: &str| layout.field(&line, |x| x == l);
        let Ok(fide_id) = f("ID Number").parse::<u32>() else {
            continue;
        };
        let name = f("Name");
        if name.is_empty() {
            continue;
        }
        let standard = {
            let s = f("SRtng");
            if s.is_empty() {
                layout.field(&line, is_period_label)
            } else {
                s
            }
        };
        let k = {
            let s = f("SK");
            if s.is_empty() { f("K") } else { s }
        };
        on_entry(FideListEntry {
            fide_id,
            name,
            federation: f("Fed"),
            sex: sex(&f("Sex")),
            title: best_title(&f("Tit"), &f("WTit")),
            rating_standard: rating(&standard),
            rating_rapid: rating(&f("RRtng")),
            rating_blitz: rating(&f("BRtng")),
            k_factor: k.parse().ok(),
            birth_year: f("B-day").parse().ok().filter(|y| *y > 1850),
            flag: Some(f("Flag")).filter(|s| !s.is_empty()),
        });
        count += 1;
    }
    Ok(count)
}

/// Which file inside a downloaded zip was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormat {
    Xml,
    Txt,
}

/// Reads the first `.xml` or `.txt` list inside a zip archive.
pub fn read_zip<R: Read + Seek>(
    reader: R,
    on_entry: impl FnMut(FideListEntry),
) -> Result<(ListFormat, usize)> {
    let mut zip =
        zip::ZipArchive::new(reader).map_err(|e| invalid(format!("not a zip file: {e}")))?;
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|e| invalid(e.to_string()))?;
        let name = file
            .name()
            .map_err(|e| invalid(e.to_string()))?
            .to_ascii_lowercase();
        if name.ends_with(".xml") {
            return Ok((ListFormat::Xml, read_xml(BufReader::new(file), on_entry)?));
        }
        if name.ends_with(".txt") {
            return Ok((ListFormat::Txt, read_txt(BufReader::new(file), on_entry)?));
        }
    }
    Err(invalid("the zip file contains no .xml or .txt rating list"))
}

/// Reads a list from a file: zip, XML or TXT, recognised by content.
pub fn read_path(
    path: &std::path::Path,
    on_entry: impl FnMut(FideListEntry),
) -> Result<(ListFormat, usize)> {
    let mut file =
        std::fs::File::open(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    let mut magic = [0u8; 5];
    let n = file.read(&mut magic).map_err(|e| invalid(e.to_string()))?;
    file.rewind().map_err(|e| invalid(e.to_string()))?;
    if n >= 2 && &magic[..2] == b"PK" {
        return read_zip(file, on_entry);
    }
    let reader = BufReader::with_capacity(1 << 16, file);
    if n >= 5 && (&magic[..5] == b"<?xml" || magic[0] == b'<') {
        Ok((ListFormat::Xml, read_xml(reader, on_entry)?))
    } else {
        Ok((ListFormat::Txt, read_txt(reader, on_entry)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<playerslist>
<player><fideid>99000001</fideid><name>Okello, David</name><country>UGA</country><sex>M</sex><title>FM</title><w_title></w_title><o_title></o_title><foa_title></foa_title><rating>2310</rating><games>9</games><k>20</k><rapid_rating>2290</rapid_rating><rapid_games>0</rapid_games><rapid_k>20</rapid_k><blitz_rating></blitz_rating><blitz_games></blitz_games><blitz_k></blitz_k><birthday>1998</birthday><flag></flag></player>
<player><fideid>99000002</fideid><name>O&apos;Neill, Anne &amp; Co</name><country>IRL</country><sex>F</sex><title></title><w_title>WIM</w_title><rating>0</rating><k>40</k><birthday>2010</birthday><flag>wi</flag></player>
</playerslist>"#;

    #[test]
    fn reads_xml() {
        let mut got = Vec::new();
        assert_eq!(read_xml(XML.as_bytes(), |e| got.push(e)).unwrap(), 2);
        assert_eq!(got[0].fide_id, 99_000_001);
        assert_eq!(got[0].name, "Okello, David");
        assert_eq!(got[0].title, Some(Title::FM));
        assert_eq!(got[0].rating_standard, Some(2310));
        assert_eq!(got[0].rating_rapid, Some(2290));
        assert_eq!(got[0].rating_blitz, None);
        assert_eq!(got[0].k_factor, Some(20));
        assert_eq!(got[0].birth_year, Some(1998));
        assert_eq!(got[1].name, "O'Neill, Anne & Co");
        assert_eq!(got[1].sex, Some(Sex::Female));
        assert_eq!(got[1].title, Some(Title::WIM));
        assert_eq!(got[1].rating_standard, None);
        assert_eq!(got[1].flag.as_deref(), Some("wi"));
    }

    const HEADER: &str = "ID Number      Name                                                         Fed Sex Tit  WTit OTit           FOA SRtng SGm SK RRtng RGm Rk BRtng BGm BK B-day Flag ";

    /// A row with each value placed under its header label, as FIDE writes them.
    fn row(header: &str, values: &[(&str, &str)]) -> String {
        let mut line = vec![' '; header.len()];
        for (label, v) in values {
            let at = header.find(label).unwrap();
            for (i, c) in v.chars().enumerate() {
                line[at + i] = c;
            }
        }
        line.into_iter().collect()
    }

    fn txt() -> String {
        [
            HEADER.to_string(),
            row(
                HEADER,
                &[
                    ("ID Number", "99000001"),
                    ("Name", "Okello, David"),
                    ("Fed", "UGA"),
                    ("Sex", "M"),
                    ("Tit ", "FM"),
                    ("SRtng", "2310"),
                    ("SGm", "9"),
                    ("SK", "20"),
                    ("RRtng", "2290"),
                    ("RGm", "0"),
                    ("Rk", "20"),
                    ("B-day", "1998"),
                ],
            ),
            row(
                HEADER,
                &[
                    ("ID Number", "99000002"),
                    ("Name", "Nakató, Grace"),
                    ("Fed", "UGA"),
                    ("Sex", "F"),
                    ("WTit", "WIM"),
                    ("SRtng", "2105"),
                    ("SK", "40"),
                    ("B-day", "2010"),
                    ("Flag", "wi"),
                ],
            ),
        ]
        .join("\n")
    }

    #[test]
    fn reads_txt() {
        let mut got = Vec::new();
        assert_eq!(read_txt(txt().as_bytes(), |e| got.push(e)).unwrap(), 2);
        assert_eq!(got[0].name, "Okello, David");
        assert_eq!(got[0].federation, "UGA");
        assert_eq!(got[0].title, Some(Title::FM));
        assert_eq!(got[0].rating_standard, Some(2310));
        assert_eq!(got[0].rating_rapid, Some(2290));
        assert_eq!(got[0].k_factor, Some(20));
        assert_eq!(got[0].birth_year, Some(1998));
        assert_eq!(got[1].name, "Nakató, Grace");
        assert_eq!(got[1].title, Some(Title::WIM));
        assert_eq!(got[1].rating_standard, Some(2105));
        assert_eq!(got[1].sex, Some(Sex::Female));
        assert_eq!(got[1].flag.as_deref(), Some("wi"));
    }

    fn txt_single() -> String {
        let h = "ID Number      Name                                                         Fed Sex Tit  WTit OTit           FOA OCT26 Gms  K  B-day Flag ";
        format!(
            "{h}\n{}",
            row(
                h,
                &[
                    ("ID Number", "99000001"),
                    ("Name", "Okello, David"),
                    ("OCT26", "2310"),
                    ("Gms", "9"),
                    (" K ", " 20"),
                    ("B-day", "1998")
                ]
            )
        )
    }

    #[test]
    fn single_list_period_column_is_the_rating() {
        let mut got = Vec::new();
        read_txt(txt_single().as_bytes(), |e| got.push(e)).unwrap();
        assert_eq!(got[0].rating_standard, Some(2310));
        assert_eq!(got[0].k_factor, Some(20));
    }

    #[test]
    fn reads_zip() {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            z.start_file(
                "players_list_xml_foa.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            z.write_all(XML.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf.set_position(0);
        let mut n = 0;
        assert_eq!(read_zip(buf, |_| n += 1).unwrap(), (ListFormat::Xml, 2));
        assert_eq!(n, 2);
    }
}
