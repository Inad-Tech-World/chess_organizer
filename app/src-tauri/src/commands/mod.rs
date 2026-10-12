//! Tauri commands, grouped like the menus. Each command resolves the
//! tournament by the calling window, turns the request into an event, and
//! returns the new view.

pub mod lists;
pub mod planned;
pub mod player_db;
pub mod players;
pub mod rounds;
pub mod teams_prizes;
pub mod tournament;

use crate::error::CmdResult;
use crate::state::{AppState, TournamentView};
use co_core::Event;
use tauri::WebviewWindow;

/// Appends `event` to the window's tournament and returns the new view.
pub(crate) fn apply(
    state: &AppState,
    window: &WebviewWindow,
    event: Event,
) -> CmdResult<TournamentView> {
    state.with_file(window.label(), |f| {
        f.append(event)?;
        Ok(TournamentView::of(f))
    })
}
