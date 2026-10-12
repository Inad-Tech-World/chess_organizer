use crate::error::CmdResult;
use crate::state::AppState;
use co_io::national_list::ColumnMapping;
use co_store::{PlayerRecord, Source};
use std::io::BufReader;
use tauri::{AppHandle, State};

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

/// Imports a FIDE rating list (XML or TXT) downloaded by the arbiter.
#[tauri::command]
pub async fn player_db_import_fide(path: String) -> CmdResult<usize> {
    let reader = BufReader::new(std::fs::File::open(&path)?);
    let count = if path.to_ascii_lowercase().ends_with(".txt") {
        co_io::fide_list::read_txt(reader, |_| {})?
    } else {
        co_io::fide_list::read_xml(reader, |_| {})?
    };
    Ok(count)
}

#[tauri::command]
pub async fn player_db_import_national(path: String) -> CmdResult<usize> {
    let text = std::fs::read_to_string(path)?;
    Ok(co_io::national_list::read_csv(&text, &ColumnMapping::default())?.len())
}
