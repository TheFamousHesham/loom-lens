//! Frontend assets, embedded at build time.
//!
//! `rust-embed` walks `frontend/dist/` at compile time and bakes the files into
//! the binary. If the directory doesn't exist (a fresh checkout before the
//! frontend has been built), the embed is empty — `viewer` still compiles and
//! the SPA routes return 404 with a hint until `pnpm build` runs.

use rust_embed::RustEmbed;

/// Embedded SPA bundle. Empty until `frontend/dist/` is populated.
#[derive(RustEmbed)]
#[folder = "../../frontend/dist/"]
pub struct Assets;
