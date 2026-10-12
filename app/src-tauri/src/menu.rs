//! Native menu bar. It follows the arbiter's workflow: tournament, players,
//! rounds, lists, teams, prizes, ratings, publishing. Item ids are sent to the
//! focused window as a `menu` event; the UI decides what each one does.

use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime};

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let item = |id: &str, text: &str, accel: Option<&str>| {
        let mut b = MenuItemBuilder::with_id(id, text);
        if let Some(a) = accel {
            b = b.accelerator(a);
        }
        b.build(app)
    };
    let about = AboutMetadata {
        name: Some("Chess Organiser".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        license: Some("GPL-3.0-or-later".into()),
        website: Some("https://github.com/chess-organiser/chess-organiser".into()),
        ..AboutMetadata::default()
    };

    let mut file = SubmenuBuilder::new(app, "File")
        .item(&item("file.new", "New Tournament…", Some("CmdOrCtrl+N"))?)
        .item(&item("file.open", "Open…", Some("CmdOrCtrl+O"))?)
        .item(&item(
            "file.open-window",
            "Open in New Window…",
            Some("CmdOrCtrl+Shift+O"),
        )?)
        .separator()
        .item(&item(
            "file.setup",
            "Tournament Setup…",
            Some("CmdOrCtrl+I"),
        )?)
        .separator()
        .item(&item("file.import-trf", "Import TRF…", None)?)
        .item(&item(
            "file.import-other",
            "Import from Other Programs…",
            None,
        )?)
        .item(&item(
            "file.export-trf",
            "Export TRF…",
            Some("CmdOrCtrl+E"),
        )?)
        .item(&item(
            "file.save-as",
            "Save a Copy As…",
            Some("CmdOrCtrl+Shift+S"),
        )?)
        .separator()
        .item(&item("file.close", "Close Tournament", None)?);
    if !cfg!(target_os = "macos") {
        file = file
            .separator()
            .item(&item("app.settings", "Settings…", Some("CmdOrCtrl+,"))?)
            .quit();
    }

    let edit = SubmenuBuilder::new(app, "Edit")
        .item(&item(
            "edit.undo",
            "Undo Tournament Change",
            Some("CmdOrCtrl+Alt+Z"),
        )?)
        .item(&item(
            "edit.redo",
            "Redo Tournament Change",
            Some("CmdOrCtrl+Alt+Shift+Z"),
        )?)
        .separator()
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let players = SubmenuBuilder::new(app, "Players")
        .item(&item("nav.players", "Player List", Some("CmdOrCtrl+1"))?)
        .item(&item(
            "players.add",
            "Add Player…",
            Some("CmdOrCtrl+Shift+N"),
        )?)
        .item(&item("players.sort", "Sort Starting Rank", None)?)
        .separator()
        .item(&item(
            "nav.player-db",
            "Player Database",
            Some("CmdOrCtrl+Shift+D"),
        )?)
        .build()?;

    let rounds = SubmenuBuilder::new(app, "Rounds")
        .item(&item(
            "nav.rounds",
            "Pairings and Results",
            Some("CmdOrCtrl+2"),
        )?)
        .item(&item(
            "rounds.pair",
            "Pair Next Round",
            Some("CmdOrCtrl+Shift+P"),
        )?)
        .item(&item("rounds.manual", "Pair Next Round Manually…", None)?)
        .separator()
        .item(&item("rounds.check", "Check Pairings", None)?)
        .item(&item("rounds.delete", "Delete Last Round…", None)?)
        .build()?;

    let lists = SubmenuBuilder::new(app, "Lists")
        .item(&item("nav.lists", "All Lists", Some("CmdOrCtrl+3"))?)
        .separator()
        .item(&item("lists.starting-rank", "Starting Rank", None)?)
        .item(&item("lists.alphabetical", "Alphabetical List", None)?)
        .item(&item("lists.pairings", "Pairings", None)?)
        .item(&item("lists.standings", "Standings", None)?)
        .item(&item("lists.crosstable", "Crosstable", None)?)
        .item(&item("lists.scorecards", "Scorecards", None)?)
        .item(&item("lists.board-cards", "Board Cards", None)?)
        .build()?;

    let teams = SubmenuBuilder::new(app, "Teams")
        .item(&item(
            "nav.teams",
            "Teams and Line-ups",
            Some("CmdOrCtrl+4"),
        )?)
        .build()?;
    let prizes = SubmenuBuilder::new(app, "Prizes")
        .item(&item(
            "nav.prizes",
            "Prize Categories",
            Some("CmdOrCtrl+5"),
        )?)
        .item(&item("lists.prizes", "Prize List", None)?)
        .build()?;
    let ratings = SubmenuBuilder::new(app, "Ratings")
        .item(&item(
            "nav.ratings",
            "Rating Report and Norms",
            Some("CmdOrCtrl+6"),
        )?)
        .build()?;
    let publish = SubmenuBuilder::new(app, "Publish")
        .item(&item(
            "nav.publish",
            "Publish Results Online",
            Some("CmdOrCtrl+7"),
        )?)
        .build()?;
    let mut help = SubmenuBuilder::new(app, "Help").item(&item(
        "help.keyboard",
        "Keyboard Shortcuts",
        Some("F1"),
    )?);
    if !cfg!(target_os = "macos") {
        help = help.separator().about(Some(about.clone()));
    }

    let mut menu = MenuBuilder::new(app);
    if cfg!(target_os = "macos") {
        let app_menu = SubmenuBuilder::new(app, "Chess Organiser")
            .about(Some(about))
            .separator()
            .item(&item("app.settings", "Settings…", Some("CmdOrCtrl+,"))?)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?;
        menu = menu.item(&app_menu);
    }
    menu.item(&file.build()?)
        .item(&edit)
        .item(&players)
        .item(&rounds)
        .item(&lists)
        .item(&teams)
        .item(&prizes)
        .item(&ratings)
        .item(&publish)
        .item(&help.build()?)
        .build()
}

/// Forwards a menu click to the focused window (or the main window).
pub fn on_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let windows = app.webview_windows();
    let target = windows
        .values()
        .find(|w| w.is_focused().unwrap_or(false))
        .or_else(|| windows.get("main"))
        .or_else(|| windows.values().next());
    if let Some(w) = target {
        let _ = app.emit_to(
            EventTarget::webview_window(w.label()),
            "menu",
            event.id().0.clone(),
        );
    }
}
