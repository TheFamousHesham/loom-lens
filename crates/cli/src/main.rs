//! Loom Lens CLI binary. Two subcommands at M1:
//!
//! - `serve`   — start the MCP server on stdio (this is what Claude Code launches).
//! - `analyze` — analyse a repo and print a viewer URL (standalone mode).

#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
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
    /// Bind address for the HTTP viewer (when started via `analyze`).
    #[arg(long, default_value = DEFAULT_BIND, global = true)]
    bind: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the MCP server on stdio (default for Claude Code integration).
    Serve,

    /// Analyse a repo and print a viewer URL. Used standalone or for smoke tests.
    Analyze {
        /// Path to the repo root.
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logs go to stderr so stdout stays clean for JSON-RPC.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve => run_serve().await,
        Command::Analyze { path } => run_analyze(path, &cli.bind).await,
    }
}

async fn run_serve() -> Result<()> {
    info!(version = VERSION, "loom-lens MCP server starting on stdio");
    let state = Arc::new(Mutex::new(ServerState {
        version: VERSION.to_string(),
    }));
    run_stdio(state).await
}

async fn run_analyze(path: PathBuf, bind: &str) -> Result<()> {
    info!(?path, "analyze invoked (parser pipeline lands in next M1 commit)");
    let viewer_state = ViewerState::new();
    println!("Viewer ready at http://{bind}/healthz");
    serve(bind, viewer_state).await?;
    Ok(())
}
