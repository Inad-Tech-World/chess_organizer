//! In-app updates. Builds without the `updater` feature (development and
//! unsigned builds) report that updates are not available.

use crate::error::CmdResult;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
pub struct UpdateInfo {
    /// Whether this build can update itself.
    enabled: bool,
    current: String,
    /// The newer version on offer, if any.
    available: Option<String>,
    notes: Option<String>,
}

#[tauri::command]
pub async fn update_check(app: AppHandle) -> CmdResult<UpdateInfo> {
    let current = app.package_info().version.to_string();
    #[cfg(feature = "updater")]
    {
        use tauri_plugin_updater::UpdaterExt;
        let io =
            |e: tauri_plugin_updater::Error| crate::error::CommandError::new("io", e.to_string());
        let update = app.updater().map_err(io)?.check().await.map_err(io)?;
        Ok(UpdateInfo {
            enabled: true,
            current,
            available: update.as_ref().map(|u| u.version.clone()),
            notes: update.and_then(|u| u.body),
        })
    }
    #[cfg(not(feature = "updater"))]
    Ok(UpdateInfo {
        enabled: false,
        current,
        available: None,
        notes: None,
    })
}

/// Downloads and installs the available update, then restarts.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> CmdResult<()> {
    #[cfg(feature = "updater")]
    {
        use tauri_plugin_updater::UpdaterExt;
        let io =
            |e: tauri_plugin_updater::Error| crate::error::CommandError::new("io", e.to_string());
        if let Some(update) = app.updater().map_err(io)?.check().await.map_err(io)? {
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(io)?;
            app.restart();
        }
        Ok(())
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = app;
        Err(crate::error::CommandError::new(
            "invalid",
            "this build cannot update itself",
        ))
    }
}
