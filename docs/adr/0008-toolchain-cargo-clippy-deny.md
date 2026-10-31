# ADR-0008: Cargo builds and tests Nectar, rustfmt, Clippy and cargo-deny are its first sensors, run by one script

- **Status:** 🟢 accepted
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** `Cargo.toml`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-all.sh`, the `build` job of `.github/workflows/checks.yml`, and the Build, Test & Run section of `README.md`

## Context

The [specification](../SPECIFICATION.md#constraints) fixes Rust, edition 2024, on the latest
stable toolchain, with the command line on Linux, macOS and Windows, a BSD-3-Clause licence that
every dependency and everything bundled must match, and one maintainer with one coding agent.
Q-1 keeps the core and the renderers free of file system, process, thread and async runtime APIs,
and G-8 has them build for `wasm32-unknown-unknown`.

Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails on a job in `checks.yml`
whose commands are inline rather than a script that `scripts/check-all.sh` runs too, so a build
that only CI knows would never reach the pre-push hook. Nectar's first workflow, `nectar.yml`, ran
`cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` on the three systems in exactly
that way, and is gone.

The catalogue's *The linter's own rules first* fits a one-person project. Of the Rust row of its
table, cargo-deny earns its place from the start: its `licenses` check decides the licence
constraint for every dependency, and its `bans` refuse crates such as an async runtime where Q-1
forbids them.

## Decision

We will build, test and lint Nectar with Cargo through one script, `scripts/check-build.sh`, which
runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, a build of `examples/demo`, a check of the crates that must run in
WebAssembly for `wasm32-unknown-unknown`, and `cargo deny check licenses bans sources` against a
`deny.toml` that allows only licences compatible with BSD-3-Clause, with the exceptions the
specification names, and bans the crates Q-1 keeps out of the core and the renderers.

- `scripts/check-all.sh` runs it with a `run` line, so the pre-push hook refuses a red change.
- The `build` job of `checks.yml` sets up the stable toolchain with the WebAssembly target and
  runs it on Linux, macOS and Windows; it is the one workflow job that builds Nectar, and the
  README lists its matrix runs among the required checks.
- Which crates the WebAssembly step checks is
  [ADR-0012](0012-textmate-engine-native-only.md)'s to say.
- The template's `scripts/sensor-tests-kept.sh` reports deleted tests and added `#[ignore]`
  markers, and rustc's `dead_code` lint, denied through Clippy's `-D warnings`, reports code nothing
  calls.

**Out of scope:** cargo-deny's advisories, which report vulnerable and unmaintained crates that
change without a commit and would turn the gate red without a change to decide; mutation testing,
duplication and complexity sensors, which a design revision adds when the mistakes call for them;
the release workflow `nectar-release.yml` and the packaging of `scripts/release.sh`.

## Alternatives considered

- **The linter's own rules only** — rustfmt, Clippy and the tests. Not chosen: the licence
  constraint and Q-1 would be held by review alone.
- **One tool per kind** — Clippy's `too_many_lines` and `cognitive_complexity`, jscpd for
  duplication, cargo-mutants on a schedule. Not chosen now: nothing yet shows the mistakes they
  would catch; a design revision adds them.
- **`nectar.yml` next to `checks.yml`** — the matrix kept in its own workflow, as Nectar started.
  Not chosen: two places for the same commands, and the pre-push hook would still not run them.

## Sources / Prior art

- Birgitta Böckeler, *Sensors for coding agents* —
  <https://martinfowler.com/articles/sensors-for-coding-agents.html>: computational sensors next
  to the inferential review.
- cargo-deny — <https://github.com/EmbarkStudios/cargo-deny>: the `licenses` and `bans` checks.
- Clippy lints — <https://rust-lang.github.io/rust-clippy/master/>.

## Consequences

- Positive: the pre-push hook and CI run the same commands; the licence constraint and the crates
  Q-1 forbids are decided by a check, not remembered by the reviewer; a dependency that cannot
  build for WebAssembly fails before it reaches the core.
- Negative / trade-offs: cargo-deny and the WebAssembly target are more to install in the Dev
  Container and CI; a new dependency with an unlisted licence fails until `deny.toml` takes it up,
  which is the point.
- Follow-ups: none beyond what is out of scope.

## Enforcement

The tests of the golden path, in `crates/fetch/`, cite this ADR with a `Verifies: ADR-0008`
marker. Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `build` job,
the `run` line of `scripts/check-all.sh`, and the required checks of the README together.
