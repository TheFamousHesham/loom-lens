//! Minimal stdio JSON-RPC server. One request → one response, line-delimited.
//! M1 implements `initialize`, `tools/list`, and a stub `tools/call analyze_repo`
//! that returns a placeholder until the parser pipeline is wired in the next pass.

use crate::protocol::{codes, Request, Response};
use crate::tools::tool_descriptors;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

/// Server-wide mutable state. Held inside an `Arc<Mutex<>>` so request handling
/// is single-threaded against the cache (M1 has no parallel requests in flight).
#[derive(Default)]
pub struct ServerState {
    /// Server software version (set by the binary at startup).
    pub version: String,
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
        "tools/call" => Some(handle_tool_call(id, req.params)),
        "ping" => Some(Response::ok(id, json!({}))),
        // Notifications carrying no id (e.g., `notifications/initialized`)
        // never reach here because we already checked id above.
        other => Some(Response::err(
            id,
            codes::METHOD_NOT_FOUND,
            format!("method not implemented in v0.1.0-alpha.1: {other}"),
            None,
        )),
    }
}

fn handle_tool_call(id: Value, params: Value) -> Response {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    match name {
        "analyze_repo" => {
            // M1 placeholder: return a deterministic graph_id and a near-empty
            // summary so the protocol round-trip is exercisable. The real
            // implementation lands in the next commit (parser → CodeGraph).
            Response::ok(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": "analyze_repo: parser pipeline lands in the next M1 commit"
                    }],
                    "isError": false,
                    "_meta": {
                        "viewer_url": "http://localhost:7000/r/000000000000",
                        "graph_id": "000000000000"
                    }
                }),
            )
        }
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
