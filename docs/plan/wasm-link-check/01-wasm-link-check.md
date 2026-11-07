# Ticket: The WebAssembly check links the portable crates

## What to build

The WebAssembly step of `scripts/check-build.sh` proves that the portable crates link for
`wasm32-unknown-unknown`, not only that they type-check. Today it runs `cargo check`, which never
links, so a link error, such as a second `getrandom` custom backend beside the one `nectar-rough`
registers (ADR-0020), would reach a WebAssembly host unnoticed. A review of PR #14 built a
`cdylib` calling the rough engine by hand and it linked; nothing in the repository repeats that.

## Acceptance criteria

- [ ] Given the repository at `main`, when `scripts/check-build.sh` runs, then its WebAssembly
  step builds and links an artifact for `wasm32-unknown-unknown` that calls the core, the
  renderers and the railroad and rough engines, and passes.
- [ ] Given a second `getrandom::register_custom_getrandom!` in that build, when the step runs,
  then it fails with the link error.
- [ ] Given the step, when it runs in CI, then its time stays within a few minutes more than
  today's step.

## Read first

- Touches: `scripts/check-build.sh`, possibly a small test crate or example under `crates/`,
  `Cargo.toml`
- ADRs: ADR-0002 (checks as scripts), ADR-0009 (portable crates), ADR-0014 (the build script),
  ADR-0020 (getrandom's custom backend)
- Whether a new crate or an example is the simpler artifact is the session's call; a new crate
  needs its place in `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human: the shape of the artifact (an example, a test crate, or `cargo build` of a
`cdylib`) and its cost in CI time are choices the maintainer weighs.
