//! `co-publish`: serves live pairings and standings uploaded by the desktop app.
//!
//! Environment: `CO_PUBLISH_ADDR` (default `127.0.0.1:8787`) and
//! `CO_PUBLISH_TOKEN` (uploads are refused unless set).

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
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("co-publish listening on http://{addr}");
    axum::serve(listener, router(AppState::new(token))).await?;
    Ok(())
}
