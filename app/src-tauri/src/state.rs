//! Process-wide state: the tournament open in each window, the player
//! database, recent files and files the OS asked us to open.

use crate::error::{CmdResult, CommandError};
use co_core::{PlayerId, Points, RoundNo, Tournament};
use co_store::{PlayerDb, TournamentFile};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Default)]
pub struct AppState {
    /// Window label -> tournament open in that window.
    pub open: Mutex<HashMap<String, TournamentFile>>,
    pub player_db: Mutex<Option<PlayerDb>>,
    /// `.cot` files passed on the command line or by the OS, not yet opened.
    pub pending_files: Mutex<Vec<PathBuf>>,
}

impl AppState {
    pub fn with_file<T>(
        &self,
        label: &str,
        f: impl FnOnce(&mut TournamentFile) -> CmdResult<T>,
    ) -> CmdResult<T> {
        let mut map = self.open.lock().expect("state lock");
        let file = map.get_mut(label).ok_or_else(CommandError::no_tournament)?;
        f(file)
    }

    pub fn with_player_db<R: Runtime, T>(
        &self,
        app: &AppHandle<R>,
        f: impl FnOnce(&mut PlayerDb) -> CmdResult<T>,
    ) -> CmdResult<T> {
        let mut slot = self.player_db.lock().expect("state lock");
        if slot.is_none() {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            *slot = Some(PlayerDb::open(dir.join("players.db"))?);
        }
        f(slot.as_mut().expect("just opened"))
    }

    /// Whether `path` is already open in some window; returns that window's label.
    pub fn window_for(&self, path: &Path) -> Option<String> {
        let map = self.open.lock().expect("state lock");
        map.iter()
            .find(|(_, f)| same_file(f.path(), path))
            .map(|(label, _)| label.clone())
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Everything a window needs to draw its tournament.
#[derive(Serialize)]
pub struct TournamentView {
    pub tournament: Tournament,
    pub path: String,
    pub can_undo: bool,
    pub can_redo: bool,
    /// Kind of the event an undo would revert, e.g. `ResultEntered`.
    pub undo_label: Option<&'static str>,
    /// Scores after all paired rounds (pending games count 0).
    pub scores: BTreeMap<PlayerId, Points>,
    /// `round_scores[i]`: scores after round `i + 1`.
    pub round_scores: Vec<BTreeMap<PlayerId, Points>>,
    pub next_round: RoundNo,
}

impl TournamentView {
    pub fn of(file: &TournamentFile) -> Self {
        let t = file.state();
        let round_scores = co_core::scoring::cumulative_scores(t);
        Self {
            scores: round_scores
                .last()
                .cloned()
                .unwrap_or_else(|| co_core::scoring::scores_after(t, 0)),
            round_scores,
            next_round: t.paired_rounds() + 1,
            tournament: t.clone(),
            path: file.path().display().to_string(),
            can_undo: file.can_undo(),
            can_redo: file.can_redo(),
            undo_label: file.undo_target().map(|e| e.event.kind()),
        }
    }
}

const RECENT_MAX: usize = 10;

fn recent_path<R: Runtime>(app: &AppHandle<R>) -> CmdResult<PathBuf> {
    Ok(app.path().app_config_dir()?.join("recent.json"))
}

pub fn recent_files<R: Runtime>(app: &AppHandle<R>) -> Vec<String> {
    recent_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| Path::new(p).exists())
        .collect()
}

pub fn remember_recent<R: Runtime>(app: &AppHandle<R>, path: &Path) {
    let entry = path.display().to_string();
    let mut list = recent_files(app);
    list.retain(|p| *p != entry);
    list.insert(0, entry);
    list.truncate(RECENT_MAX);
    if let Ok(p) = recent_path(app) {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, serde_json::to_string_pretty(&list).unwrap_or_default());
    }
}
