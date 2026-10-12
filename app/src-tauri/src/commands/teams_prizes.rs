//! Teams (Phase 3) and prize categories. Editing works now; team pairing,
//! team standings and prize lists are planned.

use super::apply;
use crate::error::CmdResult;
use crate::state::{AppState, TournamentView};
use co_core::{Event, PrizeCategory, Team, TeamId};
use tauri::{State, WebviewWindow};

#[tauri::command]
pub async fn team_add(
    window: WebviewWindow,
    state: State<'_, AppState>,
    mut team: Team,
) -> CmdResult<TournamentView> {
    team.id = state.with_file(window.label(), |f| Ok(f.state().next_team_id()))?;
    apply(&state, &window, Event::TeamAdded { team })
}

#[tauri::command]
pub async fn team_update(
    window: WebviewWindow,
    state: State<'_, AppState>,
    team: Team,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::TeamUpdated { team })
}

#[tauri::command]
pub async fn team_remove(
    window: WebviewWindow,
    state: State<'_, AppState>,
    team: TeamId,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::TeamRemoved { team })
}

#[tauri::command]
pub async fn prize_category_save(
    window: WebviewWindow,
    state: State<'_, AppState>,
    mut category: PrizeCategory,
) -> CmdResult<TournamentView> {
    let (exists, next) = state.with_file(window.label(), |f| {
        let t = f.state();
        Ok((
            t.prize_categories.iter().any(|c| c.id == category.id),
            t.next_prize_category_id(),
        ))
    })?;
    if exists {
        apply(&state, &window, Event::PrizeCategoryUpdated { category })
    } else {
        category.id = next;
        apply(&state, &window, Event::PrizeCategoryAdded { category })
    }
}

#[tauri::command]
pub async fn prize_category_remove(
    window: WebviewWindow,
    state: State<'_, AppState>,
    category: co_core::PrizeCategoryId,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::PrizeCategoryRemoved { category })
}
