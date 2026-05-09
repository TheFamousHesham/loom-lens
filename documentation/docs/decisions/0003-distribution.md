# 0003 — Distribution: GitHub Releases + npm wrapper + Homebrew tap

**Date:** Pre-Checkpoint 1
**Status:** Accepted (2026-05-09)
**Reviewed at checkpoint:** 1 ✓

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

## Refinements at Checkpoint 1

- **Conflict surfaced — crates.io name reservation.** The original draft says: *"register the `loom-lens` name on crates.io at Checkpoint 1 with a placeholder, before anyone else can."* This conflicts with the security rule in `CLAUDE.md` §6: *"Never publish to npm or crates.io without explicit user approval in chat."* A placeholder publish is still a publish.

  **Resolution:** the registration is **deferred to a user-approved action**, not auto-executed at Checkpoint 1. The user can: (a) register the placeholder manually now, (b) authorize the agent to run a one-time `cargo publish` of a 0.0.1 placeholder, or (c) accept the squat risk and register at v0.1.0 release. Same logic applies to the npm package name `loom-lens`. **The agent will not pre-register either name without explicit chat approval.**

- **Linux ARM64 added to the binary matrix.** `aarch64-unknown-linux-gnu` should be added to the release workflow alongside `x86_64-unknown-linux-gnu` for the increasingly common Graviton/RPi/Ampere user base. Cost: one matrix entry in `.github/workflows/release.yml`. Defer to M4 polish; not blocking.
- **Windows: still deferred.** No movement.
- **Binary integrity.** The release workflow will produce a `loom-lens-<target>.tar.gz` *and* a sibling `.tar.gz.sha256` file per target. The npm wrapper verifies the SHA-256 before extracting. Adds ~5 lines to the release workflow at M4. **Sigstore / cosign signing is deferred** — checksums are sufficient for v0.1; signing is a v0.2 hardening item that warrants its own ADR.
- **The Homebrew tap repository (`thefamoushesham/homebrew-tap`) is not yet created.** Creating it is a public action and falls in the same bucket as creating the main repo: deferred to user approval at the Checkpoint 1 review. Until then, the README's `brew install` line is aspirational.
- **MCP registry submission is M4 polish, not Checkpoint 1.** Listed in CHECKPOINTS.md as "optional" at M4. Confirmed.
