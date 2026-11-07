# ADR-draft-guide-as-reference-project: Cargo builds and tests Nectar, rustfmt, Clippy and cargo-deny are its first sensors, run by one script, and the user guide is the project the checks build

- **Status:** 🟡 proposed
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** `Cargo.toml`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-all.sh`, the `build` job of `.github/workflows/checks.yml`, the Build, Test & Run section of `README.md`, `docs/guide/`, and the tests that measure Q-2, Q-3 and Q-4
- **Supersedes:** [ADR-0008](0008-toolchain-cargo-clippy-deny.md)

## Context

[ADR-0008](0008-toolchain-cargo-clippy-deny.md) has one script, `scripts/check-build.sh`, build,
test and lint Nectar, and among its steps build `examples/demo`. The specification measures Q-2,
Q-3 and Q-4 on that project too. Everything else ADR-0008 decided still holds and is restated
below; only the project the checks build changes.

`examples/demo` is a second sample next to the user guide, `docs/GUIDE.md`, which explains every
feature the demo shows, and more. The demo uses each feature once, in a few lines; the guide is
what writers read. Two samples drift apart: a feature can work in the demo and look broken in
the guide, and nobody looks at the demo once the guide exists.

The maintainer wants Nectar's own documentation to be the project the checks build and the
quality goals are measured on, so that a fault shows where the readers see it. Today the guide
does not use every feature itself: it is one page, it includes no chapter (G-2), embeds no file,
and has no front matter and no raster image with attributes. Those features appear in it only as
code samples. Built as it stands, the guide would take G-2, chapter pages, breadcrumbs and the
pager out of the build the checks run.

Its project file, `docs/nectar.toml`, sits in `docs/`, so `nectar serve` watches the ADRs and the
specification as well, and a test that copies the project copies all of `docs/`.

## Decision

We will build, test and lint Nectar with Cargo through one script, `scripts/check-build.sh`, which
runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, a build of the user guide as the project in `docs/guide/`, a check of
the crates that must run in WebAssembly for `wasm32-unknown-unknown`, and
`cargo deny check licenses bans sources` against a `deny.toml` that allows only licences
compatible with BSD-3-Clause, with the exceptions the specification names, and bans the crates
Q-1 keeps out of the core and the renderers.

- `docs/guide/` holds the guide's `nectar.toml`, its start document `GUIDE.md`, the chapters it
  includes and the files it embeds. The guide uses every feature it explains at least once
  itself: chapters included from files of their own, front matter, embedded source and grammar
  files, a raster image with attributes, diagrams of every engine, alerts, footnotes, symbols,
  variables, task lists and tables. `examples/demo` is removed.
- `scripts/check-all.sh` runs `scripts/check-build.sh` with a `run` line, so the pre-push hook
  refuses a red change.
- The `build` job of `checks.yml` sets up the stable toolchain with the WebAssembly target and
  runs it on Linux, macOS and Windows; it is the one workflow job that builds Nectar, and
  `CONTRIBUTING.md` lists its matrix runs among the required checks.
- Which crates the WebAssembly step checks is
  [ADR-0012](0012-textmate-engine-native-only.md)'s to say.
- The template's `scripts/sensor-tests-kept.sh` reports deleted tests and added `#[ignore]`
  markers, and rustc's `dead_code` lint, denied through Clippy's `-D warnings`, reports code nothing
  calls.

**Out of scope:** cargo-deny's advisories, which report vulnerable and unmaintained crates that
change without a commit and would turn the gate red without a change to decide; mutation testing,
duplication and complexity sensors, which a design revision adds when the mistakes call for them;
the release workflow `nectar-release.yml` and the packaging of `scripts/release.sh`; publishing
the built guide anywhere.

## Alternatives considered

- **Keep `examples/demo`** — ADR-0008 as it stands. Not chosen: two samples to keep in step, and
  the one the checks build is not the one the readers see.
- **Build the guide as it stands** — one page in `docs/`. Not chosen: G-2, embedded files, front
  matter, raster images and the pages of a split document would leave the build the checks run,
  held only by the tests of single features.
- **Keep a small fixture under `crates/cli/tests/`** — the demo gone for readers, a project kept
  as test material for what the guide does not use. Not chosen: it is the second sample again,
  only out of sight.
- **Leave the guide's project in `docs/`** — no move. Not chosen: `nectar serve` would rebuild on
  every edit of an ADR or the specification, and a test that copies the project would copy all
  of `docs/`.
- **The linter's own rules only** — rustfmt, Clippy and the tests, carried over from ADR-0008.
  Not chosen: the licence constraint and Q-1 would be held by review alone.
- **One tool per kind** — Clippy's `too_many_lines` and `cognitive_complexity`, jscpd for
  duplication, cargo-mutants on a schedule, carried over from ADR-0008. Not chosen now: nothing
  yet shows the mistakes they would catch; a design revision adds them.
- **`nectar.yml` next to `checks.yml`** — the matrix kept in its own workflow, as Nectar started,
  carried over from ADR-0008. Not chosen: two places for the same commands, and the pre-push hook
  would still not run them.

## Sources / Prior art

- Rust's own documentation practice, *rustdoc* doctests —
  <https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html>: the examples
  readers see are the ones the tests run.
- Birgitta Böckeler, *Sensors for coding agents* —
  <https://martinfowler.com/articles/sensors-for-coding-agents.html>: computational sensors next
  to the inferential review.
- cargo-deny — <https://github.com/EmbarkStudios/cargo-deny>: the `licenses` and `bans` checks.
- Clippy lints — <https://rust-lang.github.io/rust-clippy/master/>.

## Consequences

- Positive: one sample instead of two; a feature that breaks breaks in the documentation readers
  use, and the checks see it there first. The pre-push hook and CI run the same commands; the
  licence constraint and the crates Q-1 forbids are decided by a check; a dependency that cannot
  build for WebAssembly fails before it reaches the core.
- Negative / trade-offs: the guide grows files of its own, chapters and samples to embed, and a
  change to its wording can now turn a check red. It is larger than the demo: Q-4's 200 ms is
  measured on a bigger project, and a rebuild of the guide may need work to stay under it. Every
  link to `docs/GUIDE.md` and `examples/demo` moves. cargo-deny and the WebAssembly target are
  more to install in the Dev Container and CI.
- Follow-ups: the specification measures Q-2, Q-3 and Q-4 on `examples/demo`; that change of the
  specification is the maintainer's, drafted with this ADR. The tests that cite Q-2, Q-3 and Q-4
  build `docs/guide/`.

## Enforcement

The tests of the golden path, in `crates/fetch/`, cite this ADR with a `Verifies:` marker; the
marker moves from ADR-0008 when this ADR is accepted. The tests that cite Q-2, Q-3 and Q-4 build
`docs/guide/`, and Q-2's asserts that the guide has headings, tables, pictures and code. A test of
the guide's project fails when the guide stops using a feature it explains: an included chapter,
front matter, an embedded file, a raster image with attributes, a diagram of each engine.
Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `build` job, the
`run` line of `scripts/check-all.sh`, and the required checks of `CONTRIBUTING.md` together.
