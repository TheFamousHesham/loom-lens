//! JSON-RPC 2.0 envelope types used on the MCP stdio transport.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    /// Always `"2.0"`.
    pub jsonrpc: String,
    /// Method name (e.g., `"initialize"`, `"tools/list"`, `"tools/call"`).
    pub method: String,
    /// Method-specific params.
    #[serde(default)]
    pub params: Value,
    /// Request id; missing on notifications.
    #[serde(default)]
    pub id: Option<Value>,
}

/// Response envelope.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    /// Always `"2.0"`.
    pub jsonrpc: &'static str,
    /// Echoed request id.
    pub id: Value,
    /// Result body (mutually exclusive with `error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error body (mutually exclusive with `result`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// Error body.
#[derive(Debug, Clone, Serialize)]
pub struct RpcError {
    /// Numeric error code (see `documentation/docs/api.md`).
    pub code: i64,
    /// Human-readable message.
    pub message: String,
    /// Structured detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Response {
    /// Build a success response.
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response.
    pub fn err(id: Value, code: i64, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data,
            }),
        }
    }
}

/// Standard JSON-RPC error codes.
pub mod codes {
    /// Method not found / not implemented yet.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// Invalid params for a recognised method.
    pub const INVALID_PARAMS: i64 = -32602;
    /// Internal error.
    pub const INTERNAL: i64 = -32603;
    /// Repo path not found.
    pub const REPO_NOT_FOUND: i64 = 1000;
    /// `graph_id` not in cache (call `analyze_repo` first).
    pub const GRAPH_NOT_FOUND: i64 = 1001;
    /// Repo too large for current limits.
    pub const REPO_TOO_LARGE: i64 = 1003;
}
