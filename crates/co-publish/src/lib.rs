//! Publishing server (Phase 5): players follow live pairings and standings
//! on their phones, as Chess-Results does for Swiss-Manager.
//!
//! The desktop app uploads a tournament snapshot; pages are rendered from it
//! with the same `co-io` lists used for printing. This is the skeleton:
//! upload, health check and two pages. Accounts, multiple organisers and
//! caching come in Phase 5.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use co_core::{RoundNo, Tournament, Uuid};
use co_i18n::Translator;
use co_io::report::{self, ListKind};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct AppState {
    tournaments: Arc<RwLock<HashMap<Uuid, Tournament>>>,
    /// Bearer token required for uploads. `None` disables uploads entirely.
    upload_token: Option<String>,
}

impl AppState {
    pub fn new(upload_token: Option<String>) -> Self {
        Self {
            tournaments: Arc::default(),
            upload_token,
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/tournaments/{id}", put(upload))
        .route("/t/{id}/standings", get(standings))
        .route("/t/{id}/pairings/{round}", get(pairings))
        .with_state(state)
}

async fn upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(tournament): Json<Tournament>,
) -> Response {
    let Some(expected) = &state.upload_token else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "uploads are not enabled on this server",
        )
            .into_response();
    };
    let given = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if given != Some(expected.as_str()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if tournament.id != id {
        return (StatusCode::BAD_REQUEST, "id mismatch").into_response();
    }
    state
        .tournaments
        .write()
        .expect("lock poisoned")
        .insert(id, tournament);
    StatusCode::NO_CONTENT.into_response()
}

fn page(state: &AppState, id: Uuid, kind: impl FnOnce(&Tournament) -> ListKind) -> Response {
    let map = state.tournaments.read().expect("lock poisoned");
    let Some(t) = map.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let rendered = report::build(kind(t), t, &Translator::new("en"))
        .and_then(|r| co_io::render::html::render(&r));
    match rendered {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn standings(State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    page(&state, id, |t| ListKind::Standings {
        round: t.paired_rounds().max(1),
    })
}

async fn pairings(
    State(state): State<AppState>,
    Path((id, round)): Path<(Uuid, RoundNo)>,
) -> Response {
    page(&state, id, |_| ListKind::Pairings { round })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    const SAMPLE: &str = include_str!("../../../testdata/trf/sample.trf");

    async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    fn upload_req(t: &Tournament, token: Option<&str>) -> Request<Body> {
        let mut b = Request::put(format!("/api/tournaments/{}", t.id))
            .header("content-type", "application/json");
        if let Some(tok) = token {
            b = b.header("authorization", format!("Bearer {tok}"));
        }
        b.body(Body::from(serde_json::to_vec(t).unwrap())).unwrap()
    }

    #[tokio::test]
    async fn upload_then_view() {
        let app = router(AppState::new(Some("secret".into())));
        let t = co_io::trf::read(SAMPLE).unwrap().tournament;

        assert_eq!(
            send(&app, upload_req(&t, None)).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            send(&app, upload_req(&t, Some("secret"))).await.0,
            StatusCode::NO_CONTENT
        );

        let (status, html) = send(
            &app,
            Request::get(format!("/t/{}/standings", t.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Okello, David"));

        let (status, _) = send(
            &app,
            Request::get(format!("/t/{}/pairings/9", t.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn uploads_disabled_without_token() {
        let app = router(AppState::new(None));
        let t = co_io::trf::read(SAMPLE).unwrap().tournament;
        assert_eq!(
            send(&app, upload_req(&t, Some("anything"))).await.0,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
