//! Loom Lens CLI binary.
//!
//! - `serve`   — start the MCP server on stdio plus the HTTP viewer on a TCP
//!               port. analyze_repo populates the viewer cache; the agent
//!               (Claude Code) returns the viewer URL to the user.
//! - `analyze` — one-shot: parse a repo, populate the cache, print the
//!               viewer URL, then keep the HTTP server alive until Ctrl-C.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use loom_lens_core::{analyze_repo, DiscoveryOpts};
use loom_lens_mcp::{run_stdio, ServerState};
use loom_lens_viewer::{serve, ViewerState, DEFAULT_BIND};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "loom-lens", version = VERSION, about, long_about = None)]
struct Cli {
    /// Bind address for the HTTP viewer.
    #[arg(long, default_value = DEFAULT_BIND, global = true)]
    bind: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the MCP server on stdio (default for Claude Code integration).
    Serve,

    /// Analyse a repo and print a viewer URL.
    Analyze {
        /// Path to the repo root.
        path: PathBuf,
        /// Don't keep the HTTP viewer running after analysis (machine-readable mode).
        #[arg(long)]
        no_serve: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logs go to stderr so stdout stays clean for JSON-RPC.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve => run_serve(&cli.bind).await,
        Command::Analyze { path, no_serve } => run_analyze(path, &cli.bind, no_serve).await,
    }
}

async fn run_serve(bind: &str) -> Result<()> {
    info!(
        version = VERSION,
        bind, "loom-lens MCP server + viewer starting"
    );
    let viewer_state = ViewerState::new();
    let viewer_base = format!("http://{bind}");
    let server_state = Arc::new(Mutex::new(ServerState {
        version: VERSION.to_string(),
        viewer: viewer_state.clone(),
        viewer_base,
    }));

    // Start the HTTP viewer in a background task.
    let bind_owned = bind.to_string();
    let viewer_handle = tokio::spawn(async move {
        if let Err(e) = serve(&bind_owned, viewer_state).await {
            tracing::error!(error = %e, "viewer task exited");
        }
    });

    let serve_result = run_stdio(server_state).await;
    viewer_handle.abort();
    serve_result
}

async fn run_analyze(path: PathBuf, bind: &str, no_serve: bool) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("could not resolve {}", path.display()))?;
    info!(?canonical, "analyzing");

    let opts = DiscoveryOpts::default();
    let path_for_blocking = canonical.clone();
    let graph = tokio::task::spawn_blocking(move || analyze_repo(&path_for_blocking, &opts))
        .await
        .map_err(|e| anyhow::anyhow!("analyze task panicked: {e}"))??;

    let viewer_state = ViewerState::new();
    let viewer_url = format!("http://{bind}/r/{}", graph.graph_id);
    let summary = graph.summary.clone();
    let graph_id = graph.graph_id.clone();
    viewer_state.put(graph);

    println!(
        "analyzed: {} files, {} functions, {} modules, {} types in {} ms",
        summary.files, summary.functions, summary.modules, summary.types, summary.elapsed_ms
    );
    let imp_pct = if summary.imports_total > 0 {
        100 * summary.imports_resolved / summary.imports_total
    } else {
        0
    };
    let call_pct = if summary.calls_total > 0 {
        100 * summary.calls_resolved / summary.calls_total
    } else {
        0
    };
    println!(
        "resolution: {}/{} imports ({}%), {}/{} calls ({}%)",
        summary.imports_resolved, summary.imports_total, imp_pct,
        summary.calls_resolved, summary.calls_total, call_pct,
    );
    if !summary.parse_errors.is_empty() {
        println!(
            "parse errors: {} files (graph still built from parseable subset)",
            summary.parse_errors.len()
        );
    }
    println!("graph_id:   {graph_id}");
    println!("viewer_url: {viewer_url}");

    if no_serve {
        return Ok(());
    }

    println!("\nServing the viewer at http://{bind}/ — Ctrl-C to stop.");
    serve(bind, viewer_state).await?;
    Ok(())
}
