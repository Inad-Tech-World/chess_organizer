use crate::error::{CmdResult, CommandError};
use crate::state::AppState;
use co_io::fide_list::{self, FideListEntry};
use co_io::national_list::{self, ColumnMapping, Preview};
use co_store::{PlayerRecord, Source};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn player_db_search(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> CmdResult<Vec<PlayerRecord>> {
    state.with_player_db(&app, |db| Ok(db.search(&query, limit.unwrap_or(50))?))
}

#[tauri::command]
pub async fn player_db_sources(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Vec<Source>> {
    state.with_player_db(&app, |db| Ok(db.sources()?))
}

/// Progress of a long import, sent to the window as `player-db-progress`.
#[derive(Clone, Serialize)]
struct Progress {
    stage: &'static str,
    count: usize,
}

fn fide_record(e: FideListEntry) -> PlayerRecord {
    PlayerRecord {
        fide_id: Some(e.fide_id),
        national_id: None,
        name: e.name,
        federation: Some(e.federation).filter(|f| !f.is_empty()),
        sex: e.sex,
        title: e.title,
        rating_standard: e.rating_standard,
        rating_rapid: e.rating_rapid,
        rating_blitz: e.rating_blitz,
        national_rating: None,
        k_factor: e.k_factor,
        birth_year: e.birth_year,
        club: None,
    }
}

/// Streams a FIDE list file into the player database (source "FIDE"),
/// without holding the whole list in memory.
fn import_fide_file(app: &AppHandle, path: &Path) -> CmdResult<usize> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<PlayerRecord>(4096);
    let path_owned = path.to_path_buf();
    let reader = std::thread::spawn(move || {
        fide_list::read_path(&path_owned, |e| {
            let _ = tx.send(fide_record(e));
        })
    });
    let period = co_core::chrono::Utc::now().format("%Y-%m").to_string();
    let progress_app = app.clone();
    let records = rx.into_iter().enumerate().map(move |(i, r)| {
        if i % 50_000 == 0 && i > 0 {
            let _ = progress_app.emit(
                "player-db-progress",
                Progress {
                    stage: "import",
                    count: i,
                },
            );
        }
        r
    });
    let state = app.state::<AppState>();
    let written = state.with_player_db(app, |db| {
        Ok(db.replace_source("FIDE", Some(&period), records)?)
    })?;
    let (_, read) = reader.join().map_err(|_| {
        CommandError::new("invalid", "the rating list reader stopped unexpectedly")
    })??;
    if read == 0 {
        return Err(CommandError::new(
            "invalid",
            "no players were found in this file",
        ));
    }
    Ok(written)
}

/// Imports a FIDE rating list (zip, XML or TXT) chosen by the arbiter.
#[tauri::command]
pub async fn player_db_import_fide(app: AppHandle, path: String) -> CmdResult<usize> {
    tauri::async_runtime::spawn_blocking(move || import_fide_file(&app, Path::new(&path)))
        .await
        .map_err(|e| CommandError::new("io", e.to_string()))?
}

/// Downloads FIDE's current combined list and imports it.
#[tauri::command]
pub async fn player_db_download_fide(app: AppHandle) -> CmdResult<usize> {
    let dir: PathBuf = app.path().app_cache_dir()?;
    std::fs::create_dir_all(&dir)?;
    let target = dir.join("players_list_xml.zip");
    let client = reqwest::Client::builder()
        .user_agent(concat!("ChessOrganiser/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| CommandError::new("io", e.to_string()))?;
    let mut resp = client
        .get(fide_list::FIDE_XML_URL)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| CommandError::new("io", format!("could not download the FIDE list: {e}")))?;
    let mut file = std::fs::File::create(&target)?;
    let mut downloaded = 0usize;
    let mut last_report = 0usize;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| CommandError::new("io", format!("download interrupted: {e}")))?
    {
        std::io::Write::write_all(&mut file, &chunk)?;
        downloaded += chunk.len();
        if downloaded - last_report > 1 << 20 {
            last_report = downloaded;
            let _ = app.emit(
                "player-db-progress",
                Progress {
                    stage: "download",
                    count: downloaded,
                },
            );
        }
    }
    drop(file);
    tauri::async_runtime::spawn_blocking(move || import_fide_file(&app, &target))
        .await
        .map_err(|e| CommandError::new("io", e.to_string()))?
}

/// The first rows of a national list with a guessed column mapping.
#[tauri::command]
pub async fn player_db_preview_national(path: String) -> CmdResult<Preview> {
    let text = national_list::decode(&std::fs::read(path)?);
    Ok(national_list::preview(&text, 8)?)
}

/// Imports a national list with the mapping the arbiter confirmed; `name`
/// labels the source (e.g. "UCF"), replacing an earlier import of it.
#[tauri::command]
pub async fn player_db_import_national(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    mapping: ColumnMapping,
    name: Option<String>,
) -> CmdResult<usize> {
    let text = national_list::decode(&std::fs::read(&path)?);
    let rows = national_list::read_csv(&text, &mapping)?;
    let records = rows.into_iter().map(|e| PlayerRecord {
        fide_id: e.fide_id,
        national_id: Some(e.national_id),
        name: e.name,
        federation: e.federation,
        sex: e.sex,
        title: e.title.as_deref().and_then(co_core::Title::parse),
        national_rating: e.rating,
        birth_year: e.birth_year,
        club: e.club,
        ..PlayerRecord::default()
    });
    let label = name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "National".into());
    let period = co_core::chrono::Utc::now().format("%Y-%m").to_string();
    state.with_player_db(&app, |db| {
        Ok(db.replace_source(&label, Some(&period), records)?)
    })
}
