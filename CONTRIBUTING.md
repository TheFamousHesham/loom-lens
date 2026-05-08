# Contributing to Loom Lens

Thanks for your interest. The development model for this project is unusual, so please read this before opening a PR.

## How this project is built

Loom Lens is being built largely by Claude Code working autonomously between defined checkpoints. The maintainer reviews progress at each checkpoint, makes architectural decisions, and steers the project. Between checkpoints, the agent owns the working tree.

This means:

- **The codebase changes in large bursts**, not in a steady stream of small commits from many contributors.
- **External PRs may conflict with in-progress agent work** that hasn't yet been pushed.
- **The maintainer handles all PR responses personally**; the agent does not respond to issues or comments.

## Before opening a PR

**Open a discussion first.** Use [GitHub Discussions](https://github.com/TheFamousHesham/loom-lens/discussions) to:

1. Describe the bug, feature, or improvement.
2. Wait for maintainer response before writing code.
3. The maintainer may say "yes please," "we're already on it," "this conflicts with the current architecture," or "this is great but please wait until v0.2."

This is not bureaucracy; it's because the development cadence is unusual and we want to avoid you wasting time on work that won't merge.

## What we welcome

- **Bug reports** with clear reproduction steps.
- **New language support** following the pattern in `documentation/docs/effect-rules/` (write the rules document; the implementation can come later).
- **Effect rule improvements** for existing languages — false positives and false negatives are bugs, please report them with the offending code snippet.
- **Documentation improvements**, especially examples and screenshots.
- **Frontend improvements** to the viewer if you have UX expertise.
- **Architectural feedback** in Discussions — the design is not finalized.

## What we don't currently want

- Large feature additions outside the v0.1 roadmap (see `documentation/CHECKPOINTS.md`).
- Style/formatting-only PRs.
- Cosmetic README changes.
- Build system overhauls.

These are not "bad" — they're "not for this phase."

## Code requirements (if a PR is invited)

- **Conventional commits:** `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`, `security:`.
- **Rust:** `cargo fmt` and `cargo clippy --all-targets -- -D warnings` clean.
- **TypeScript:** Prettier and ESLint clean; `pnpm typecheck` passes.
- **No `any` in TS, no `unwrap()` in Rust outside tests.**
- **Tests for new functionality.**
- **Updated documentation** if the change is user-facing.

## Reporting security issues

**Do not open a public issue for security vulnerabilities.** Email the maintainer at [contact via GitHub profile] or use GitHub's private vulnerability reporting feature. We will acknowledge within 7 days.

## Code of conduct

Be kind, be precise, be honest. Don't be a jerk. Standard adult expectations apply.

## License

By contributing, you agree your contribution is licensed under the project's MIT license.
