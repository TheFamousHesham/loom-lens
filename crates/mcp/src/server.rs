//! Stdio JSON-RPC server. Implements `initialize`, `tools/list`, and a
//! working `tools/call analyze_repo` (parser → graph → viewer cache → URL).
//! Other tools return `MethodNotFound` until M2/M3.

use crate::protocol::{codes, Request, Response};
use crate::tools::tool_descriptors;
use loom_lens_core::{analyze_repo, DiscoveryOpts};
use loom_lens_viewer::ViewerState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::warn;

/// Shared server state. Behind `Arc<Mutex<>>` so handlers can mutate the
/// graph cache without external sync.
pub struct ServerState {
    /// Server software version (set by the binary at startup).
    pub version: String,
    /// In-memory LRU of analysed graphs (shared with the HTTP viewer task).
    pub viewer: ViewerState,
    /// Public base URL the agent should hand back to the user (built from
    /// the bind address; e.g., `http://127.0.0.1:7000`).
    pub viewer_base: String,
}

/// Run the stdio JSON-RPC loop until EOF on stdin.
pub async fn run_stdio(state: Arc<Mutex<ServerState>>) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Request>(trimmed) {
            Ok(req) => handle(req, state.clone()).await,
            Err(e) => Some(Response::err(
                Value::Null,
                codes::INVALID_PARAMS,
                format!("malformed JSON-RPC: {e}"),
                None,
            )),
        };
        if let Some(resp) = response {
            let body = serde_json::to_vec(&resp)?;
            stdout.write_all(&body).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

async fn handle(req: Request, state: Arc<Mutex<ServerState>>) -> Option<Response> {
    let id = req.id.clone()?;
    match req.method.as_str() {
        "initialize" => {
            let v = state.lock().await.version.clone();
            Some(Response::ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "loom-lens", "version": v }
                }),
            ))
        }
        "tools/list" => Some(Response::ok(id, json!({ "tools": tool_descriptors() }))),
        "tools/call" => Some(handle_tool_call(id, req.params, state).await),
        "ping" => Some(Response::ok(id, json!({}))),
        other => Some(Response::err(
            id,
            codes::METHOD_NOT_FOUND,
            format!("method not implemented: {other}"),
            None,
        )),
    }
}

async fn handle_tool_call(id: Value, params: Value, state: Arc<Mutex<ServerState>>) -> Response {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);
    match name {
        "analyze_repo" => analyze_repo_tool(id, args, state).await,
        "query_graph" | "get_function_context" | "compare_hashes" => Response::err(
            id,
            codes::METHOD_NOT_FOUND,
            format!("{name} is not yet implemented in v0.1.0-alpha.1"),
            None,
        ),
        other => Response::err(
            id,
            codes::METHOD_NOT_FOUND,
            format!("unknown tool: {other}"),
            None,
        ),
    }
}

async fn analyze_repo_tool(id: Value, args: Value, state: Arc<Mutex<ServerState>>) -> Response {
    let path = match args.get("path").and_then(Value::as_str) {
        Some(p) => PathBuf::from(p),
        None => {
            return Response::err(
                id,
                codes::INVALID_PARAMS,
                "missing required parameter: path",
                None,
            );
        }
    };
    if !path.is_dir() {
        return Response::err(
            id,
            codes::REPO_NOT_FOUND,
            format!("repo path is not a directory: {}", path.display()),
            None,
        );
    }

    let max_files = args
        .get("max_files")
        .and_then(Value::as_u64)
        .unwrap_or(10_000) as usize;
    let opts = DiscoveryOpts {
        languages: vec![],
        max_files,
    };

    // Run the (CPU-bound) parse in a blocking task so the stdio loop
    // doesn't stall while we walk a large repo.
    let analyze_path = path.clone();
    let result = tokio::task::spawn_blocking(move || analyze_repo(&analyze_path, &opts)).await;

    let graph = match result {
        Ok(Ok(g)) => g,
        Ok(Err(e)) => {
            warn!(?e, "analyze_repo failed");
            return Response::err(id, codes::INTERNAL, format!("analyze failed: {e}"), None);
        }
        Err(e) => {
            return Response::err(
                id,
                codes::INTERNAL,
                format!("analyze task panicked: {e}"),
                None,
            );
        }
    };

    let summary = serde_json::to_value(&graph.summary).unwrap_or(Value::Null);
    let viewer_base = state.lock().await.viewer_base.clone();
    let viewer_url = format!("{viewer_base}/r/{}", graph.graph_id);
    let graph_id = graph.graph_id.0.clone();

    state.lock().await.viewer.put(graph);

    let payload = json!({
        "graph_id": graph_id,
        "viewer_url": viewer_url,
        "summary": summary,
    });

    Response::ok(
        id,
        json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&payload).unwrap_or_default()
            }],
            "isError": false,
            "_meta": payload
        }),
    )
}
