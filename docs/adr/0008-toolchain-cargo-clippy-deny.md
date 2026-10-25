# ADR-0008: Cargo builds and tests Nectar, rustfmt, Clippy and cargo-deny are its first sensors, run by one script

- **Status:** 🟡 proposed
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** `Cargo.toml`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-all.sh`, the `build` job of `.github/workflows/checks.yml`, and the Build, Test & Run section of `README.md`

## Context

The [specification](../SPECIFICATION.md#constraints) fixes Rust, edition 2024, on the latest
stable toolchain, with the command line on Linux, macOS and Windows, a BSD-3-Clause licence that
every dependency and everything bundled must match, and one maintainer with one coding agent.
Q-1 keeps the core and the renderers free of file system, process, thread and async runtime APIs.

Today `.github/workflows/nectar.yml` runs `cargo fmt --check`, `cargo clippy -D warnings` and
`cargo test` on the three systems and builds `examples/demo`, with its commands inline: the pre-push
hook never runs them, and check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails
on such a job in `checks.yml`.

The catalogue's *The linter's own rules first* fits a one-person project. Of the Rust row of its
table, cargo-deny earns its place from the start: its `licenses` check decides the licence
constraint for every dependency, and its `bans` refuse crates such as an async runtime where Q-1
forbids them.

## Decision

We will build, test and lint Nectar with Cargo through one script, `scripts/check-build.sh`, which
runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, a build of `examples/demo`, and `cargo deny check licenses bans sources`
against a `deny.toml` that allows only licences compatible with BSD-3-Clause, with the exceptions
the specification names, and bans the crates Q-1 keeps out of the core and the renderers.

- `scripts/check-all.sh` runs it with a `run` line, so the pre-push hook refuses a red change.
- The `build` job of `checks.yml` sets up the stable toolchain and runs it on Linux, macOS and
  Windows; `nectar.yml` goes, and the README lists the job among the required checks.
- The template's `scripts/sensor-tests-kept.sh` reports deleted tests and added `#[ignore]`
  markers, and rustc's `dead_code` lint, denied through Clippy's `-D warnings`, reports code nothing
  calls.

**Out of scope:** cargo-deny's advisories, which report vulnerable and unmaintained crates that
change without a commit and would turn the gate red without a change to decide; the build for
WebAssembly, which joins the script when the work on G-8 begins;
mutation testing, duplication and complexity sensors, which a design revision adds when the
mistakes call for them; the release workflow `nectar-release.yml`.

## Alternatives considered

- **The linter's own rules only** — rustfmt, Clippy and the tests as today. Not chosen: the licence
  constraint and Q-1 would be held by review alone.
- **One tool per kind** — Clippy's `too_many_lines` and `cognitive_complexity`, jscpd for
  duplication, cargo-mutants on a schedule. Not chosen now: nothing yet shows the mistakes they
  would catch; a design revision adds them.
- **`nectar.yml` next to `checks.yml`** — the matrix kept in its own workflow. Not chosen: two
  places for the same commands, and the pre-push hook would still not run them.

## Sources / Prior art

- Birgitta Böckeler, *Sensors for coding agents* —
  <https://martinfowler.com/articles/sensors-for-coding-agents.html>: computational sensors next
  to the inferential review.
- cargo-deny — <https://github.com/EmbarkStudios/cargo-deny>: the `licenses` and `bans` checks.
- Clippy lints — <https://rust-lang.github.io/rust-clippy/master/>.

## Consequences

- Positive: the pre-push hook and CI run the same commands; the licence constraint and the crates
  Q-1 forbids are decided by a check, not remembered by the reviewer.
- Negative / trade-offs: cargo-deny is one more tool to install in the Dev Container and CI; a new
  dependency with an unlisted licence fails until `deny.toml` takes it up, which is the point.
- Follow-ups: the WebAssembly build of G-8 in the script; the structural test of Q-1, which the
  shape of the code decides.

## Enforcement

The golden path's tests cite this ADR with a `Verifies: ADR-0008` marker, and it flips to accepted
with them. Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `build` job,
the `run` line of `scripts/check-all.sh`, and the required checks of the README together.
