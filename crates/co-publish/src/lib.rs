//! Publishing server: players follow live pairings and standings on their
//! phones, as Chess-Results does for Swiss-Manager.
//!
//! The desktop app uploads a tournament snapshot (`PUT /api/tournaments/{id}`
//! with a bearer token). Pages are rendered from it with the same `co-io`
//! lists used for printing, in any supported language (`?lang=fr`).
//! Snapshots are kept in memory and, with a data directory, on disk, so
//! they survive restarts. Uploads can be removed with `DELETE`.
//!
//! Routes:
//! - `GET /` – published tournaments
//! - `GET /t/{id}` – one tournament: rounds and lists
//! - `GET /t/{id}/{list}` and `/t/{id}/{list}/{round}` – a list
//! - `GET /api/tournaments` – JSON index
//! - `PUT`/`DELETE /api/tournaments/{id}` – upload or withdraw (token)

use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use co_core::{RoundNo, Tournament, Uuid};
use co_i18n::Translator;
use co_io::report::{self, ListKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Largest accepted upload: a 2,000-player, 13-round event is well under this.
pub const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Default)]
pub struct AppState {
    tournaments: Arc<RwLock<HashMap<Uuid, Tournament>>>,
    /// Bearer token required for uploads. `None` disables uploads entirely.
    upload_token: Option<String>,
    /// Where snapshots are stored, one JSON file per tournament.
    data_dir: Option<PathBuf>,
}

impl AppState {
    pub fn new(upload_token: Option<String>) -> Self {
        Self {
            tournaments: Arc::default(),
            upload_token,
            data_dir: None,
        }
    }

    /// Like [`AppState::new`], persisting snapshots in `dir` and loading the
    /// ones already there.
    pub fn with_data_dir(upload_token: Option<String>, dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&dir)?;
        let mut map = HashMap::new();
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "json") {
                match std::fs::read(&path).map(|b| serde_json::from_slice::<Tournament>(&b)) {
                    Ok(Ok(t)) => {
                        map.insert(t.id, t);
                    }
                    _ => eprintln!("skipping unreadable snapshot {}", path.display()),
                }
            }
        }
        Ok(Self {
            tournaments: Arc::new(RwLock::new(map)),
            upload_token,
            data_dir: Some(dir),
        })
    }

    fn snapshot_path(&self, id: Uuid) -> Option<PathBuf> {
        self.data_dir.as_ref().map(|d| d.join(format!("{id}.json")))
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/", get(home))
        .route("/api/tournaments", get(index_json))
        .route("/api/tournaments/{id}", put(upload).delete(withdraw))
        .route("/t/{id}", get(overview))
        .route("/t/{id}/{list}", get(list_page))
        .route("/t/{id}/{list}/{round}", get(round_list_page))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .with_state(state)
}

fn authorized(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, &'static str)> {
    let Some(expected) = &state.upload_token else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "uploads are not enabled on this server",
        ));
    };
    let given = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    // Constant-time comparison so the token cannot be guessed byte by byte.
    let ok = given.is_some_and(|g| {
        g.len() == expected.len()
            && g.bytes()
                .zip(expected.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    });
    if ok {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, "wrong upload token"))
    }
}

async fn upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(tournament): Json<Tournament>,
) -> Response {
    if let Err(r) = authorized(&state, &headers) {
        return r.into_response();
    }
    if tournament.id != id {
        return (StatusCode::BAD_REQUEST, "id mismatch").into_response();
    }
    if let Some(path) = state.snapshot_path(id) {
        let write = serde_json::to_vec(&tournament)
            .map_err(std::io::Error::other)
            .and_then(|bytes| {
                let tmp = path.with_extension("json.tmp");
                std::fs::write(&tmp, bytes)?;
                std::fs::rename(&tmp, &path)
            });
        if let Err(e) = write {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not store the snapshot: {e}"),
            )
                .into_response();
        }
    }
    state
        .tournaments
        .write()
        .expect("lock poisoned")
        .insert(id, tournament);
    StatusCode::NO_CONTENT.into_response()
}

async fn withdraw(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    if let Err(r) = authorized(&state, &headers) {
        return r.into_response();
    }
    let removed = state
        .tournaments
        .write()
        .expect("lock poisoned")
        .remove(&id)
        .is_some();
    if let Some(path) = state.snapshot_path(id) {
        let _ = std::fs::remove_file(path);
    }
    if removed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
    .into_response()
}

#[derive(Serialize)]
struct Summary {
    id: Uuid,
    name: String,
    city: String,
    rounds: RoundNo,
    paired: RoundNo,
}

async fn index_json(State(state): State<AppState>) -> Json<Vec<Summary>> {
    let map = state.tournaments.read().expect("lock poisoned");
    let mut list: Vec<Summary> = map
        .values()
        .map(|t| Summary {
            id: t.id,
            name: t.info.name.clone(),
            city: t.info.city.clone(),
            rounds: t.settings.rounds,
            paired: t.paired_rounds(),
        })
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Json(list)
}

#[derive(Deserialize, Default)]
struct Lang {
    lang: Option<String>,
}

impl Lang {
    fn code(&self) -> &str {
        match self.lang.as_deref() {
            Some(l) if co_i18n::SUPPORTED.contains(&l) => l,
            _ => "en",
        }
    }

    fn query(&self) -> String {
        match self.lang.as_deref() {
            Some(l) if l != "en" && co_i18n::SUPPORTED.contains(&l) => format!("?lang={l}"),
            _ => String::new(),
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A small mobile-friendly page around `body`.
fn shell(title: &str, lang: &str, body: &str) -> Response {
    let dir = if co_i18n::is_rtl(lang) { "rtl" } else { "ltr" };
    let html = format!(
        r#"<!doctype html><html lang="{lang}" dir="{dir}"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>{t}</title>
<style>body{{font-family:system-ui,sans-serif;margin:0 auto;max-width:860px;padding:16px;color:#1c1c1c;background:#fff}}
h1{{font-size:1.4rem}}a{{color:#8a5a00}}ul{{padding-inline-start:20px;line-height:1.8}}
.rounds a{{display:inline-block;margin:0 6px 6px 0;padding:4px 10px;border:1px solid #d9c9a3;border-radius:6px;text-decoration:none}}
@media (prefers-color-scheme:dark){{body{{background:#161616;color:#eee}}a{{color:#f0c060}}}}</style></head>
<body>{body}<p style="opacity:.6;font-size:.8rem">Chess Organiser</p></body></html>"#,
        t = esc(title)
    );
    let mut r = Html(html).into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    r
}

async fn home(State(state): State<AppState>, Query(lang): Query<Lang>) -> Response {
    let map = state.tournaments.read().expect("lock poisoned");
    let mut items: Vec<&Tournament> = map.values().collect();
    items.sort_by(|a, b| a.info.name.cmp(&b.info.name));
    let tr = Translator::new(lang.code());
    let mut body = format!("<h1>{}</h1><ul>", esc(&tr.t("publish-index-title")));
    for t in items {
        body.push_str(&format!(
            r#"<li><a href="/t/{}{}">{}</a> {}</li>"#,
            t.id,
            lang.query(),
            esc(&t.info.name),
            esc(&t.info.city)
        ));
    }
    body.push_str("</ul>");
    shell(&tr.t("publish-index-title"), lang.code(), &body)
}

async fn overview(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(lang): Query<Lang>,
) -> Response {
    let map = state.tournaments.read().expect("lock poisoned");
    let Some(t) = map.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let tr = Translator::new(lang.code());
    let q = lang.query();
    let paired = t.paired_rounds();
    let team = t.kind.is_team();
    let mut body = format!("<h1>{}</h1>", esc(&t.info.name));
    if !t.info.city.is_empty() {
        body.push_str(&format!("<p>{}</p>", esc(&t.info.city)));
    }
    if paired > 0 {
        body.push_str(&format!(
            "<h2>{}</h2><p class=rounds>",
            esc(&tr.t("publish-pairings"))
        ));
        let list = if team { "team-pairings" } else { "pairings" };
        for r in 1..=paired {
            body.push_str(&format!(r#"<a href="/t/{id}/{list}/{r}{q}">{r}</a>"#));
        }
        body.push_str("</p>");
    }
    body.push_str("<ul>");
    let mut links: Vec<(&str, String)> = vec![("starting-rank", tr.t("list-starting-rank"))];
    if paired > 0 {
        if team {
            links.push((
                "team-standings",
                tr.t_args(
                    "list-team-standings",
                    &[("round", u32::from(paired).into())],
                ),
            ));
        }
        links.push((
            "standings",
            tr.t_args("list-standings", &[("round", u32::from(paired).into())]),
        ));
        links.push((
            "crosstable",
            tr.t_args("list-crosstable", &[("round", u32::from(paired).into())]),
        ));
    }
    if team {
        links.push(("teams", tr.t("list-team-roster")));
    }
    if !t.prize_categories.is_empty() && paired > 0 {
        links.push(("prizes", tr.t("list-prizes")));
    }
    for (path, label) in links {
        body.push_str(&format!(
            r#"<li><a href="/t/{id}/{path}{q}">{}</a></li>"#,
            esc(&label)
        ));
    }
    body.push_str("</ul>");
    shell(&t.info.name, lang.code(), &body)
}

/// The list named in a URL, for round `round` (default: the last paired).
fn list_kind(name: &str, t: &Tournament, round: Option<RoundNo>) -> Option<ListKind> {
    let last = t.paired_rounds().max(1);
    let r = round.unwrap_or(last);
    Some(match name {
        "starting-rank" => ListKind::StartingRank,
        "alphabetical" => ListKind::Alphabetical,
        "pairings" => ListKind::Pairings { round: r },
        "standings" => ListKind::Standings { round: r },
        "crosstable" => ListKind::Crosstable { round: r },
        "prizes" => ListKind::Prizes,
        "teams" => ListKind::TeamRosters,
        "team-pairings" => ListKind::TeamPairings { round: r },
        "team-standings" => ListKind::TeamStandings { round: r },
        _ => return None,
    })
}

fn render(state: &AppState, id: Uuid, list: &str, round: Option<RoundNo>, lang: &Lang) -> Response {
    let map = state.tournaments.read().expect("lock poisoned");
    let Some(t) = map.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(kind) = list_kind(list, t, round) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let rendered = report::build(kind, t, &Translator::new(lang.code()))
        .and_then(|r| co_io::render::html::render(&r));
    match rendered {
        Ok(html) => {
            let mut r = Html(html).into_response();
            r.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            r
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn list_page(
    State(state): State<AppState>,
    Path((id, list)): Path<(Uuid, String)>,
    Query(lang): Query<Lang>,
) -> Response {
    render(&state, id, &list, None, &lang)
}

async fn round_list_page(
    State(state): State<AppState>,
    Path((id, list, round)): Path<(Uuid, String, RoundNo)>,
    Query(lang): Query<Lang>,
) -> Response {
    render(&state, id, &list, Some(round), &lang)
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

    fn get(path: String) -> Request<Body> {
        Request::get(path).body(Body::empty()).unwrap()
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
            send(&app, upload_req(&t, Some("wrong!"))).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            send(&app, upload_req(&t, Some("secret"))).await.0,
            StatusCode::NO_CONTENT
        );

        let (status, html) = send(&app, get(format!("/t/{}/standings", t.id))).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Okello, David"));

        let (status, html) = send(&app, get(format!("/t/{}", t.id))).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("/pairings/3"));
        assert!(html.contains("crosstable"));

        let (status, html) = send(&app, get(format!("/t/{}/pairings/2?lang=fr", t.id))).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("lang=\"fr\""));

        assert_eq!(
            send(&app, get(format!("/t/{}/pairings/9", t.id))).await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            send(&app, get(format!("/t/{}/nonsense", t.id))).await.0,
            StatusCode::NOT_FOUND
        );

        let (_, index) = send(&app, get("/api/tournaments".into())).await;
        assert!(index.contains(&t.id.to_string()));
        let (_, home) = send(&app, get("/".into())).await;
        assert!(home.contains(&t.info.name));
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

    #[tokio::test]
    async fn snapshots_survive_a_restart_and_can_be_withdrawn() {
        let dir = tempfile::tempdir().unwrap();
        let t = co_io::trf::read(SAMPLE).unwrap().tournament;
        {
            let app = router(AppState::with_data_dir(Some("s".into()), dir.path().into()).unwrap());
            assert_eq!(
                send(&app, upload_req(&t, Some("s"))).await.0,
                StatusCode::NO_CONTENT
            );
        }
        let app = router(AppState::with_data_dir(Some("s".into()), dir.path().into()).unwrap());
        assert_eq!(
            send(&app, get(format!("/t/{}", t.id))).await.0,
            StatusCode::OK
        );
        let del = Request::delete(format!("/api/tournaments/{}", t.id))
            .header("authorization", "Bearer s")
            .body(Body::empty())
            .unwrap();
        assert_eq!(send(&app, del).await.0, StatusCode::NO_CONTENT);
        assert_eq!(
            send(&app, get(format!("/t/{}", t.id))).await.0,
            StatusCode::NOT_FOUND
        );
        assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
    }
}
