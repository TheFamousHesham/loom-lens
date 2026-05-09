//! Loom Lens HTTP viewer. Embeds the React/Cytoscape frontend at build time
//! (when `frontend/dist/` exists) and serves graph JSON over HTTP.
//!
//! M1 surface:
//! - `GET /` — index redirect.
//! - `GET /r/:graph_id` — viewer SPA (404s gracefully if frontend isn't bundled).
//! - `GET /api/graph/:graph_id` — graph JSON.
//! - `GET /api/graph/:graph_id/source/*file` — raw source for a file in the graph.
//! - `GET /healthz` — liveness.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod assets;
mod routes;
mod state;

pub use routes::router;
pub use state::ViewerState;

/// Default bind address (per ARCHITECTURE.md §1).
pub const DEFAULT_BIND: &str = "127.0.0.1:7000";

/// Bind and serve until shutdown.
///
/// Lets the `cli` crate avoid taking a direct axum dependency — viewer owns
/// the wire-format and binding choices.
pub async fn serve(bind: &str, state: ViewerState) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    let app = router(state);
    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
