//! Chess Organiser desktop app: a thin Tauri shell over the core crates.

mod commands;
mod error;
mod menu;
mod state;

use state::AppState;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, WindowEvent};

pub fn run() {
    // Windows and Linux pass double-clicked files as arguments.
    let pending: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| p.extension().is_some_and(|e| e == "cot") && p.is_file())
        .collect();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(AppState {
            pending_files: Mutex::new(pending),
            ..AppState::default()
        })
        .menu(menu::build)
        .on_menu_event(menu::on_event)
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                let state = window.state::<AppState>();
                let removed = state
                    .open
                    .lock()
                    .expect("state lock")
                    .remove(window.label());
                if let Some(file) = removed {
                    let _ = file.checkpoint();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::tournament::defaults,
            commands::tournament::tournament_current,
            commands::tournament::tournament_new,
            commands::tournament::tournament_open,
            commands::tournament::tournament_import_trf,
            commands::tournament::tournament_import_swiss_manager,
            commands::tournament::open_window,
            commands::tournament::tournament_close,
            commands::tournament::tournament_save_as,
            commands::tournament::tournament_export_trf,
            commands::tournament::tournament_undo,
            commands::tournament::tournament_redo,
            commands::tournament::update_info,
            commands::tournament::update_settings,
            commands::tournament::change_kind,
            commands::tournament::recent_files,
            commands::tournament::take_pending_files,
            commands::players::player_add,
            commands::players::player_add_from_record,
            commands::players::player_update,
            commands::players::player_remove,
            commands::players::player_withdraw,
            commands::players::player_reenter,
            commands::players::player_set_bye,
            commands::players::players_sort,
            commands::player_db::player_db_search,
            commands::player_db::player_db_sources,
            commands::player_db::player_db_import_fide,
            commands::player_db::player_db_import_national,
            commands::rounds::round_pair_next,
            commands::rounds::round_pair_manual,
            commands::rounds::round_check,
            commands::rounds::round_delete_last,
            commands::rounds::round_swap_colors,
            commands::rounds::result_set,
            commands::lists::list_html,
            commands::lists::list_print,
            commands::lists::list_export,
            commands::teams_prizes::team_add,
            commands::teams_prizes::team_update,
            commands::teams_prizes::team_remove,
            commands::teams_prizes::prize_category_save,
            commands::teams_prizes::prize_category_remove,
            commands::planned::team_standings,
            commands::planned::rating_report,
            commands::planned::norm_check,
            commands::planned::publish_upload,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Chess Organiser");

    app.run(|_app, _event| {
        // macOS delivers double-clicked files as an event instead of arguments.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = &_event {
            use tauri::Emitter;
            let state = _app.state::<AppState>();
            let mut pending = state.pending_files.lock().expect("state lock");
            pending.extend(urls.iter().filter_map(|u| u.to_file_path().ok()));
            drop(pending);
            let _ = _app.emit("files-opened", ());
        }
    });
}
