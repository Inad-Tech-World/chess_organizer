use crate::error::CmdResult;
use crate::state::AppState;
use co_core::Tournament;
use co_i18n::Translator;
use co_io::render::{self, Format};
use co_io::report::{self, ListKind};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_opener::OpenerExt;

fn snapshot(state: &AppState, window: &WebviewWindow) -> CmdResult<Tournament> {
    state.with_file(window.label(), |f| Ok(f.state().clone()))
}

fn html(t: &Tournament, kind: ListKind, lang: &str) -> CmdResult<String> {
    let r = report::build(kind, t, &Translator::new(lang))?;
    Ok(render::html::render(&r)?)
}

/// The list as a standalone HTML page, for the in-app preview.
#[tauri::command]
pub async fn list_html(
    window: WebviewWindow,
    state: State<'_, AppState>,
    kind: ListKind,
    lang: String,
) -> CmdResult<String> {
    html(&snapshot(&state, &window)?, kind, &lang)
}

/// Opens the list in the system browser, whose print dialog also saves PDF
/// on macOS and Windows.
#[tauri::command]
pub async fn list_print(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    kind: ListKind,
    lang: String,
) -> CmdResult<()> {
    let page = html(&snapshot(&state, &window)?, kind, &lang)?;
    let dir = app.path().app_cache_dir()?.join("print");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.html", window.label()));
    std::fs::write(&path, page)?;
    app.opener()
        .open_path(path.display().to_string(), None::<&str>)
        .map_err(|e| crate::error::CommandError::new("io", e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn list_export(
    window: WebviewWindow,
    state: State<'_, AppState>,
    kind: ListKind,
    format: Format,
    lang: String,
    path: String,
) -> CmdResult<()> {
    let t = snapshot(&state, &window)?;
    let r = report::build(kind, &t, &Translator::new(&lang))?;
    let bytes = render::render(&r, format)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
