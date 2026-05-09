//! axum router and handlers.

use crate::assets::Assets;
use crate::state::ViewerState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use loom_lens_core::GraphId;
use serde_json::json;
use std::path::PathBuf;

/// Build the axum router.
pub fn router(state: ViewerState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/r/:graph_id", get(spa_index))
        .route("/r/:graph_id/*rest", get(spa_index))
        .route("/api/graph/:graph_id", get(api_graph))
        .route("/api/graph/:graph_id/source/*file", get(api_source))
        .route("/_loom/assets/*path", get(serve_asset))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    Redirect::temporary("/healthz")
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "ok": true, "service": "loom-lens" }))
}

async fn spa_index() -> Response {
    match Assets::get("index.html") {
        Some(file) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "text/html; charset=utf-8".parse().unwrap(),
            );
            (headers, file.data.into_owned()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            "Loom Lens viewer SPA is not bundled in this binary.\n\
             Build the frontend first:  cd frontend && pnpm install && pnpm build\n\
             Then rebuild the binary:   cargo build --release\n",
        )
            .into_response(),
    }
}

async fn serve_asset(Path(path): Path<String>) -> Response {
    match Assets::get(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
            (headers, file.data.into_owned()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}

async fn api_graph(State(state): State<ViewerState>, Path(graph_id): Path<String>) -> Response {
    let id = GraphId(graph_id);
    match state.get(&id) {
        Some(graph) => Json(graph).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": 1001,
                    "message": "graph_id not found",
                    "hint": "Call analyze_repo first; graphs are cached LRU and may have been evicted."
                }
            })),
        )
            .into_response(),
    }
}

async fn api_source(
    State(state): State<ViewerState>,
    Path((graph_id, file)): Path<(String, String)>,
) -> Response {
    let id = GraphId(graph_id);
    let Some(graph) = state.get(&id) else {
        return (StatusCode::NOT_FOUND, "graph_id not found").into_response();
    };
    let candidate: PathBuf = graph.repo_root.join(&file);
    if !candidate.starts_with(&graph.repo_root) {
        // Defence in depth against `..` traversals beyond the path normaliser.
        return (StatusCode::BAD_REQUEST, "path traversal rejected").into_response();
    }
    match std::fs::read(&candidate) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&candidate).first_or_text_plain();
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
            (headers, bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "file not found in graph repo").into_response(),
    }
}
