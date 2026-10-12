//! Commands whose screens exist but whose logic lands in a later phase.
//! They call the real (stubbed) APIs so the UI shows the right phase, and
//! start working without UI changes as each phase fills its stub in.

use crate::error::CmdResult;
use crate::state::AppState;
use co_core::{PlayerId, not_yet};
use tauri::{State, WebviewWindow};

#[tauri::command]
pub async fn team_standings(window: WebviewWindow, state: State<'_, AppState>) -> CmdResult<()> {
    state.with_file(window.label(), |f| {
        let t = f.state();
        co_tiebreak::team_standings(t, t.paired_rounds())?;
        Ok(())
    })
}

#[tauri::command]
pub async fn rating_report(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<Vec<(PlayerId, f64)>> {
    state.with_file(window.label(), |f| {
        Ok(co_rating::tournament_rating_changes(f.state())?)
    })
}

#[tauri::command]
pub async fn norm_check(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: PlayerId,
) -> CmdResult<Vec<co_rating::NormReport>> {
    state.with_file(window.label(), |f| {
        Ok(co_rating::norm_check(f.state(), player)?)
    })
}

#[tauri::command]
pub async fn publish_upload(server: String, _token: String) -> CmdResult<()> {
    let _ = server;
    Ok(not_yet!("Publishing to a results site", 5)?)
}
