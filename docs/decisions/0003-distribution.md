# 0003 — Distribution: GitHub Releases + npm wrapper + Homebrew tap

**Date:** Pre-Checkpoint 1
**Status:** Proposed
**Reviewed at checkpoint:** 1 (pending)

## Context

Loom Lens is a single Rust binary plus an embedded frontend. Users come from three communities:

- **Claude Code users** (primary) — most have Node and use `npm` for global tools.
- **Rust developers** — comfortable with `cargo install`.
- **macOS power users** — prefer Homebrew.

Each community has different expectations. We need a distribution story that hits all three without maintaining three separate build pipelines.

## Decision

**Three distribution channels, one source of truth (GitHub Releases):**

1. **GitHub Releases** — pre-built binaries for `x86_64-linux-gnu`, `aarch64-darwin`, `x86_64-darwin`. Built by the manual-dispatch release workflow. The canonical artifacts.
2. **npm wrapper package** (`loom-lens` on npm) — a small Node package that downloads the appropriate GitHub Release binary on `npm install -g`. Pattern proven by `esbuild`, `swc`, `biome`.
3. **Homebrew tap** (`thefamoushesham/homebrew-tap`) — Formula references the GitHub Release for macOS users.
4. **Cargo** (`cargo install loom-lens` from crates.io) — builds from source; slower but works on any Rust-supported platform.

Windows binaries deferred to v0.2 unless real demand emerges.

## Alternatives considered

- **npm only.** Rejected because it forces Node as a dependency for non-Node users and because the wrapper must download binaries anyway.
- **Cargo only.** Rejected because compiling from source on every install is slow (60+ seconds) and many target users don't have Rust toolchains.
- **Docker only.** Niche audience; doesn't fit a "plugin you install once and forget" UX. Could ship a Docker image as an extra channel later.
- **Self-hosted installer script (`curl … | bash`).** Rejected on security grounds; we should not encourage what we forbid in `CLAUDE.md`.
- **AUR / Flatpak / Snap.** Out of scope for v0.1.

## Consequences

### Positive
- Each user community installs through familiar tooling.
- One source of truth (GitHub Releases) means one build pipeline.
- The npm wrapper is ~30 lines of code; trivial to maintain.
- Cargo path serves as an emergency fallback if the binaries break.

### Negative
- Three things to update on each release. Mitigation: release workflow handles all three from one dispatch.
- Homebrew tap requires a separate repo; small ongoing maintenance.
- The npm wrapper is an attack surface (typo-squatters, supply chain attacks). Mitigation: 2FA on the npm account; provenance attestation via npm's GitHub integration once available.

### Risks
- **Binary tampering on GitHub Releases.** Mitigation: ship SHA-256 checksums alongside binaries; the npm wrapper verifies before installing.
- **Cargo crate name squatting.** Mitigation: register the `loom-lens` name on crates.io at Checkpoint 1 with a placeholder, before anyone else can.
- **Homebrew tap discovery.** A custom tap is less discoverable than `homebrew-core`. Mitigation: README documents the tap clearly; submit to `homebrew-core` once project is mature (v0.3+).

## References

- esbuild's npm-wraps-binary pattern: https://esbuild.github.io/getting-started/#install-the-binary-executable
- npm provenance: https://docs.npmjs.com/generating-provenance-statements
- Homebrew tap docs: https://docs.brew.sh/Taps
