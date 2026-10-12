use super::apply;
use crate::error::{CmdResult, CommandError};
use crate::state::{self, AppState, TournamentView};
use co_core::{Event, Settings, TournamentInfo, TournamentKind, Uuid};
use co_store::TournamentFile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

fn cot_path(path: &str) -> PathBuf {
    let mut p = PathBuf::from(path);
    if p.extension().is_none_or(|e| e != TournamentFile::EXTENSION) {
        p.set_extension(TournamentFile::EXTENSION);
    }
    p
}

fn window_title(file: &TournamentFile) -> String {
    let name = &file.state().info.name;
    if name.is_empty() {
        "Chess Organiser".into()
    } else {
        format!("{name} — Chess Organiser")
    }
}

/// Binds `file` to `window`, closing whatever the window had open.
fn install(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &AppState,
    file: TournamentFile,
) -> CmdResult<TournamentView> {
    state::remember_recent(app, file.path());
    let _ = window.set_title(&window_title(&file));
    let view = TournamentView::of(&file);
    let previous = state
        .open
        .lock()
        .expect("state lock")
        .insert(window.label().to_string(), file);
    if let Some(prev) = previous {
        let _ = prev.checkpoint();
    }
    Ok(view)
}

#[derive(Serialize)]
pub struct Defaults {
    info: TournamentInfo,
    settings: Settings,
}

/// Default tournament settings, so the UI never duplicates them.
#[tauri::command]
pub async fn defaults() -> CmdResult<Defaults> {
    Ok(Defaults {
        info: TournamentInfo::default(),
        settings: Settings::default(),
    })
}

#[tauri::command]
pub async fn tournament_current(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<Option<TournamentView>> {
    let map = state.open.lock().expect("state lock");
    Ok(map.get(window.label()).map(TournamentView::of))
}

#[derive(Deserialize)]
pub struct NewTournament {
    path: String,
    info: TournamentInfo,
    settings: Settings,
    kind: TournamentKind,
}

#[tauri::command]
pub async fn tournament_new(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    args: NewTournament,
) -> CmdResult<TournamentView> {
    let file = TournamentFile::create(
        cot_path(&args.path),
        Event::TournamentCreated {
            id: Uuid::new_v4(),
            info: args.info,
            settings: args.settings,
            kind: args.kind,
        },
    )?;
    install(&app, &window, &state, file)
}

#[tauri::command]
pub async fn tournament_open(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<TournamentView> {
    let path = PathBuf::from(path);
    if let Some(label) = state.window_for(&path) {
        if let Some(other) = app.get_webview_window(&label) {
            let _ = other.set_focus();
        }
        if label != window.label() {
            return Err(CommandError::new(
                "invalid",
                "That tournament is already open in another window.",
            ));
        }
    }
    let file = TournamentFile::open(&path)?;
    install(&app, &window, &state, file)
}

#[derive(Serialize)]
pub struct ImportResult {
    view: TournamentView,
    warnings: Vec<String>,
}

#[tauri::command]
pub async fn tournament_import_trf(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    trf_path: String,
    cot: String,
) -> CmdResult<ImportResult> {
    let text = std::fs::read_to_string(&trf_path)?;
    let import = co_io::trf::read(&text).map_err(co_io::Error::from)?;
    let source = PathBuf::from(&trf_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file = TournamentFile::create(
        cot_path(&cot),
        Event::TournamentImported {
            tournament: Box::new(import.tournament),
            source,
        },
    )?;
    Ok(ImportResult {
        view: install(&app, &window, &state, file)?,
        warnings: import.warnings,
    })
}

/// Imports a file exported by another pairing program (TRF or a player
/// list) into a new tournament file.
#[tauri::command]
pub async fn tournament_import_swiss_manager(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    path: String,
    cot: String,
) -> CmdResult<ImportResult> {
    let bytes = std::fs::read(&path)?;
    let import = co_io::swiss_manager_import::import(&bytes)?;
    let source = PathBuf::from(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file = TournamentFile::create(
        cot_path(&cot),
        Event::TournamentImported {
            tournament: Box::new(import.tournament),
            source,
        },
    )?;
    Ok(ImportResult {
        view: install(&app, &window, &state, file)?,
        warnings: import.warnings,
    })
}

/// Opens a new window, optionally with a tournament already loaded.
#[tauri::command]
pub async fn open_window(
    app: AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> CmdResult<()> {
    let label = format!("t-{}", Uuid::new_v4().simple());
    let mut title = "Chess Organiser".to_string();
    if let Some(path) = path {
        let path = PathBuf::from(path);
        if let Some(existing) = state.window_for(&path) {
            if let Some(w) = app.get_webview_window(&existing) {
                w.set_focus()?;
            }
            return Ok(());
        }
        let file = TournamentFile::open(&path)?;
        state::remember_recent(&app, file.path());
        title = window_title(&file);
        state
            .open
            .lock()
            .expect("state lock")
            .insert(label.clone(), file);
    }
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(1280.0, 820.0)
        .min_inner_size(960.0, 600.0)
        .build()?;
    Ok(())
}

#[tauri::command]
pub async fn tournament_close(window: WebviewWindow, state: State<'_, AppState>) -> CmdResult<()> {
    let removed = state
        .open
        .lock()
        .expect("state lock")
        .remove(window.label());
    if let Some(file) = removed {
        file.checkpoint()?;
    }
    let _ = window.set_title("Chess Organiser");
    Ok(())
}

#[tauri::command]
pub async fn tournament_save_as(
    window: WebviewWindow,
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<()> {
    state.with_file(window.label(), |f| Ok(f.save_as(cot_path(&path))?))
}

#[tauri::command]
pub async fn tournament_export_trf(
    window: WebviewWindow,
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<()> {
    let text = state.with_file(window.label(), |f| Ok(co_io::trf::write(f.state())))?;
    std::fs::write(path, text)?;
    Ok(())
}

#[tauri::command]
pub async fn tournament_undo(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    state.with_file(window.label(), |f| {
        f.undo()?;
        Ok(TournamentView::of(f))
    })
}

#[tauri::command]
pub async fn tournament_redo(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    state.with_file(window.label(), |f| {
        f.redo()?;
        Ok(TournamentView::of(f))
    })
}

#[tauri::command]
pub async fn update_info(
    window: WebviewWindow,
    state: State<'_, AppState>,
    info: TournamentInfo,
) -> CmdResult<TournamentView> {
    let view = apply(&state, &window, Event::InfoUpdated { info })?;
    let _ = window.set_title(&if view.tournament.info.name.is_empty() {
        "Chess Organiser".to_string()
    } else {
        format!("{} — Chess Organiser", view.tournament.info.name)
    });
    Ok(view)
}

#[tauri::command]
pub async fn update_settings(
    window: WebviewWindow,
    state: State<'_, AppState>,
    settings: Settings,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::SettingsUpdated { settings })
}

#[tauri::command]
pub async fn change_kind(
    window: WebviewWindow,
    state: State<'_, AppState>,
    kind: TournamentKind,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::KindChanged { kind })
}

#[tauri::command]
pub async fn recent_files(app: AppHandle) -> CmdResult<Vec<String>> {
    Ok(state::recent_files(&app))
}

/// Files passed by the OS (double-click, command line) waiting to be opened.
#[tauri::command]
pub async fn take_pending_files(state: State<'_, AppState>) -> CmdResult<Vec<String>> {
    let mut pending = state.pending_files.lock().expect("state lock");
    Ok(pending.drain(..).map(|p| p.display().to_string()).collect())
}

/// Creates a fresh copy of the sample tournament (in the app's data folder)
/// and fills the player database with the sample list, for trying things out.
#[tauri::command]
pub async fn sample_load(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    use co_io::sample;
    use co_store::PlayerRecord;

    let records: Vec<PlayerRecord> = sample::sample_people()
        .iter()
        .enumerate()
        .map(|(i, p)| PlayerRecord {
            fide_id: p.rating.map(|_| sample::sample_fide_id(i)),
            name: p.name.to_string(),
            federation: Some(p.federation.to_string()),
            sex: Some(p.sex),
            title: p.title,
            rating_standard: p.rating,
            rating_rapid: p.rating.map(|r| r.saturating_sub(35)),
            rating_blitz: p.rating.map(|r| r.saturating_sub(60)),
            birth_year: Some(p.birth_year),
            club: p.club.map(str::to_string),
            ..PlayerRecord::default()
        })
        .collect();
    state.with_player_db(&app, |db| {
        Ok(db.replace_source("Sample list (fictional players)", Some("demo"), records)?)
    })?;

    let dir = app.path().app_data_dir()?.join("samples");
    std::fs::create_dir_all(&dir)?;
    let path = (1..)
        .map(|n| {
            let suffix = if n == 1 {
                String::new()
            } else {
                format!(" {n}")
            };
            dir.join(format!("Kampala Rapid Open 2026 (sample){suffix}.cot"))
        })
        .find(|p| !p.exists())
        .expect("a free file name");
    let mut events = sample::tournament_events().into_iter();
    let mut file = TournamentFile::create(&path, events.next().expect("creation event"))?;
    for ev in events {
        file.append(ev)?;
    }
    file.checkpoint()?;
    install(&app, &window, &state, file)
}
