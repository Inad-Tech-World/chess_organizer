use super::apply;
use crate::error::{CmdResult, CommandError};
use crate::state::{AppState, TournamentView};
use co_core::pairing_view::PairingInput;
use co_core::{Event, GameResult, Pairing, PairingMethod, PlayerId, RoundNo};
use co_pairing::manual::manual_round;
use serde::Serialize;
use tauri::{State, WebviewWindow};

/// Pairs the next round with the tournament's pairing system.
#[tauri::command]
pub async fn round_pair_next(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    let event = state.with_file(window.label(), |f| {
        let t = f.state();
        let system = t.kind.pairing_system();
        let round = co_pairing::engine_for(system).pair(&PairingInput::for_next_round(t))?;
        Ok(Event::RoundPaired {
            round: t.paired_rounds() + 1,
            pairings: round.pairings,
            matches: round.matches,
            method: PairingMethod::Engine { system },
        })
    })?;
    apply(&state, &window, event)
}

#[derive(Serialize)]
pub struct ManualOutcome {
    /// False when violations were found and `force` was not set.
    applied: bool,
    violations: Vec<String>,
    view: TournamentView,
}

/// Pairs the next round by hand. Rule violations are returned for the
/// arbiter to confirm; with `force` the round is paired anyway.
#[tauri::command]
pub async fn round_pair_manual(
    window: WebviewWindow,
    state: State<'_, AppState>,
    games: Vec<(PlayerId, PlayerId)>,
    pab: Option<PlayerId>,
    force: bool,
) -> CmdResult<ManualOutcome> {
    state.with_file(window.label(), |f| {
        let t = f.state();
        let input = PairingInput::for_next_round(t);
        let (round, violations) = manual_round(&input, &games, pab);
        let violations: Vec<String> = violations.iter().map(ToString::to_string).collect();
        let applied = violations.is_empty() || force;
        if applied {
            f.append(Event::RoundPaired {
                round: input.round,
                pairings: round.pairings,
                matches: round.matches,
                method: PairingMethod::Manual,
            })?;
        }
        Ok(ManualOutcome {
            applied,
            violations,
            view: TournamentView::of(f),
        })
    })
}

/// Lists rule violations in an existing round's pairings.
#[tauri::command]
pub async fn round_check(
    window: WebviewWindow,
    state: State<'_, AppState>,
    round: RoundNo,
) -> CmdResult<Vec<String>> {
    state.with_file(window.label(), |f| {
        let t = f.state();
        let r = t
            .round(round)
            .ok_or_else(|| CommandError::new("invalid", format!("round {round} is not paired")))?;
        let engine = co_pairing::engine_for(t.kind.pairing_system());
        let input = PairingInput::for_existing_round(t, round);
        Ok(engine
            .check(&input, &r.pairings)
            .iter()
            .map(ToString::to_string)
            .collect())
    })
}

#[tauri::command]
pub async fn round_delete_last(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> CmdResult<TournamentView> {
    let round = state.with_file(window.label(), |f| Ok(f.state().paired_rounds()))?;
    apply(&state, &window, Event::RoundDeleted { round })
}

/// Swaps colours on one board (manual override; only before a result exists).
#[tauri::command]
pub async fn round_swap_colors(
    window: WebviewWindow,
    state: State<'_, AppState>,
    round: RoundNo,
    board: u16,
) -> CmdResult<TournamentView> {
    let pairings = state.with_file(window.label(), |f| {
        let r = f
            .state()
            .round(round)
            .ok_or_else(|| CommandError::new("invalid", format!("round {round} is not paired")))?;
        let mut pairings = r.pairings.clone();
        for p in &mut pairings {
            if let Pairing::Game {
                board: b,
                white,
                black,
                result,
            } = p
                && *b == board
            {
                if result.is_some() {
                    return Err(CommandError::new(
                        "invalid",
                        "Clear the result before swapping colours.",
                    ));
                }
                std::mem::swap(white, black);
            }
        }
        Ok(pairings)
    })?;
    apply(&state, &window, Event::PairingsEdited { round, pairings })
}

#[tauri::command]
pub async fn result_set(
    window: WebviewWindow,
    state: State<'_, AppState>,
    round: RoundNo,
    board: u16,
    result: Option<GameResult>,
) -> CmdResult<TournamentView> {
    apply(
        &state,
        &window,
        Event::ResultEntered {
            round,
            board,
            result,
        },
    )
}

/// Result of one board in a team match.
#[tauri::command]
pub async fn team_result_set(
    window: WebviewWindow,
    state: State<'_, AppState>,
    round: RoundNo,
    match_number: u16,
    board: u16,
    result: Option<GameResult>,
) -> CmdResult<TournamentView> {
    apply(
        &state,
        &window,
        Event::TeamGameResultEntered {
            round,
            match_number,
            board,
            result,
        },
    )
}

/// One team's line-up (board order) in a match; the board games are rebuilt.
#[tauri::command]
pub async fn lineup_set(
    window: WebviewWindow,
    state: State<'_, AppState>,
    round: RoundNo,
    match_number: u16,
    team: co_core::TeamId,
    lineup: Vec<PlayerId>,
) -> CmdResult<TournamentView> {
    apply(
        &state,
        &window,
        Event::LineupSet {
            round,
            match_number,
            team,
            lineup,
        },
    )
}
