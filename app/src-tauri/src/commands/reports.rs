//! Calculated reports shown on screen (team standings, prize winners,
//! rating changes, norms) and publishing to a results server.

use crate::error::{CmdResult, CommandError};
use crate::state::AppState;
use co_core::PlayerId;
use serde::Serialize;
use tauri::{State, WebviewWindow};

#[tauri::command]
pub async fn team_standings(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<co_tiebreak::TeamStandings> {
    state.with_file(window.label(), |f| {
        let t = f.state();
        Ok(co_tiebreak::team_standings(t, t.paired_rounds())?)
    })
}

#[derive(Serialize)]
pub struct PrizeWinner {
    category: String,
    place: usize,
    prize: String,
    player: PlayerId,
}

/// Prize winners after the last paired round; each player wins at most one prize.
#[tauri::command]
pub async fn prize_winners(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<Vec<PrizeWinner>> {
    state.with_file(window.label(), |f| {
        let t = f.state();
        if t.paired_rounds() == 0 {
            return Ok(Vec::new());
        }
        Ok(co_io::report::prize_winners(t)?
            .into_iter()
            .map(|(ci, place, player)| {
                let c = &t.prize_categories[ci];
                PrizeWinner {
                    category: c.name.clone(),
                    place: place + 1,
                    prize: c.prizes[place].clone(),
                    player,
                }
            })
            .collect())
    })
}

#[tauri::command]
pub async fn rating_report(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<Vec<co_rating::RatingChangeRow>> {
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

/// Uploads the tournament to a publishing server; returns the public page.
#[tauri::command]
pub async fn publish_upload(
    window: WebviewWindow,
    state: State<'_, AppState>,
    server: String,
    token: String,
) -> CmdResult<String> {
    let base = server.trim().trim_end_matches('/').to_string();
    let url = reqwest::Url::parse(&base)
        .map_err(|e| CommandError::new("invalid", format!("not a web address: {e}")))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || (url.scheme() == "http" && local)) {
        return Err(CommandError::new(
            "invalid",
            "use an https:// address (plain http is only allowed for a server on this computer)",
        ));
    }
    let tournament = state.with_file(window.label(), |f| Ok(f.state().clone()))?;
    let id = tournament.id;
    let client = reqwest::Client::builder()
        .user_agent(concat!("ChessOrganiser/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| CommandError::new("io", e.to_string()))?;
    let body =
        serde_json::to_vec(&tournament).map_err(|e| CommandError::new("invalid", e.to_string()))?;
    let resp = client
        .put(format!("{base}/api/tournaments/{id}"))
        .bearer_auth(token.trim())
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| CommandError::new("io", format!("could not reach {base}: {e}")))?;
    match resp.status().as_u16() {
        200..=299 => Ok(format!("{base}/t/{id}")),
        401 => Err(CommandError::new(
            "invalid",
            "the server refused the upload token",
        )),
        503 => Err(CommandError::new(
            "invalid",
            "uploads are not enabled on this server",
        )),
        code => Err(CommandError::new(
            "io",
            format!(
                "the server answered {code}: {}",
                resp.text().await.unwrap_or_default()
            ),
        )),
    }
}
