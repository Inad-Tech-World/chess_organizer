//! `co-publish`: serves live pairings and standings uploaded by the desktop app.
//!
//! Environment:
//! - `CO_PUBLISH_ADDR` (default `127.0.0.1:8787`)
//! - `CO_PUBLISH_TOKEN`: uploads are refused unless set
//! - `CO_PUBLISH_DATA`: directory for snapshots, so they survive restarts
//!   (default: memory only)
//!
//! Put it behind a TLS-terminating reverse proxy (Caddy, nginx) on the
//! public internet; the bearer token must not travel over plain HTTP.

use co_publish::{AppState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::var("CO_PUBLISH_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let token = std::env::var("CO_PUBLISH_TOKEN")
        .ok()
        .filter(|t| !t.is_empty());
    if token.is_none() {
        eprintln!("CO_PUBLISH_TOKEN is not set: uploads are disabled");
    }
    let state = match std::env::var_os("CO_PUBLISH_DATA") {
        Some(dir) => AppState::with_data_dir(token, dir.into())?,
        None => AppState::new(token),
    };
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("co-publish listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
