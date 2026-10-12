use super::apply;
use crate::error::CmdResult;
use crate::state::{AppState, TournamentView};
use co_core::{ByeKind, Event, Player, PlayerId, RoundNo, Title};
use co_store::PlayerRecord;
use tauri::{State, WebviewWindow};

/// Adds a player; the id, starting rank and (after round 1) late-entry
/// round are assigned here so the UI cannot get them wrong.
fn add(state: &AppState, window: &WebviewWindow, mut player: Player) -> CmdResult<TournamentView> {
    let (id, rank, next_round) = state.with_file(window.label(), |f| {
        let t = f.state();
        Ok((
            t.next_player_id(),
            t.players.len() as u32 + 1,
            t.paired_rounds() + 1,
        ))
    })?;
    player.id = id;
    player.starting_rank = rank;
    player.joined_round = player.joined_round.max(next_round);
    apply(state, window, Event::PlayerAdded { player })
}

#[tauri::command]
pub async fn player_add(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: Player,
) -> CmdResult<TournamentView> {
    add(&state, &window, player)
}

#[tauri::command]
pub async fn player_add_from_record(
    window: WebviewWindow,
    state: State<'_, AppState>,
    record: PlayerRecord,
    late_entry_bye: Option<ByeKind>,
) -> CmdResult<TournamentView> {
    let mut player = record.to_player(PlayerId(0));
    if let Some(k) = late_entry_bye {
        player.late_entry_bye = k;
    }
    add(&state, &window, player)
}

#[tauri::command]
pub async fn player_update(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: Player,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::PlayerUpdated { player })
}

#[tauri::command]
pub async fn player_remove(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: PlayerId,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::PlayerRemoved { player })
}

#[tauri::command]
pub async fn player_withdraw(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: PlayerId,
    from_round: RoundNo,
) -> CmdResult<TournamentView> {
    apply(
        &state,
        &window,
        Event::PlayerWithdrawn { player, from_round },
    )
}

#[tauri::command]
pub async fn player_reenter(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: PlayerId,
) -> CmdResult<TournamentView> {
    apply(&state, &window, Event::PlayerReentered { player })
}

#[tauri::command]
pub async fn player_set_bye(
    window: WebviewWindow,
    state: State<'_, AppState>,
    player: PlayerId,
    round: RoundNo,
    kind: Option<ByeKind>,
) -> CmdResult<TournamentView> {
    apply(
        &state,
        &window,
        Event::ByeSet {
            player,
            round,
            kind,
        },
    )
}

/// FIDE C.04.2 initial order: rating, then title, then name.
#[tauri::command]
pub async fn players_sort(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    let order = state.with_file(window.label(), |f| {
        let t = f.state();
        let rt = t.settings.rating_type;
        let title_rank = |t: Option<Title>| {
            t.and_then(|t| {
                [
                    Title::GM,
                    Title::IM,
                    Title::WGM,
                    Title::FM,
                    Title::WIM,
                    Title::CM,
                    Title::WFM,
                    Title::WCM,
                ]
                .iter()
                .position(|x| *x == t)
            })
            .unwrap_or(usize::MAX)
        };
        let mut players: Vec<&Player> = t.players.iter().collect();
        players.sort_by(|a, b| {
            b.rating(rt)
                .unwrap_or(0)
                .cmp(&a.rating(rt).unwrap_or(0))
                .then(title_rank(a.title).cmp(&title_rank(b.title)))
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(players.into_iter().map(|p| p.id).collect())
    })?;
    apply(&state, &window, Event::StartingRanksSet { order })
}
