# ADR-0008: Cargo builds and tests Nectar, rustfmt, Clippy and cargo-deny are its first sensors, run by one script, and the user guide is the project the checks build

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** `Cargo.toml`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-all.sh`, the `build` job of `.github/workflows/checks.yml`, the Build, Test & Run section of `README.md`, `docs/guide/`, and the tests that measure Q-2, Q-3 and Q-4

## Context

The [specification](../SPECIFICATION.md#constraints) fixes Rust, edition 2024, on the latest
stable toolchain, with the command line on Linux, macOS and Windows, a BSD-3-Clause licence that
every dependency and everything bundled must match, with the exceptions it names, and one
maintainer with one coding agent. Q-1 keeps the core and the renderers free of file system,
process, thread and async runtime APIs, and G-8 has the crates that must run in WebAssembly build
and link for `wasm32-unknown-unknown`.

The pre-push hook and CI have to run the same commands, or a change that passes one fails the
other. Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails on a job in
`checks.yml` whose commands are inline rather than a script that `scripts/check-all.sh` runs too,
so a build that only CI knows would never reach the pre-push hook. Nectar's first workflow,
`nectar.yml`, ran `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` on the three
systems in exactly that way, and is gone.

The catalogue's *The linter's own rules first* fits a one-person project. Of the Rust row of its
table, cargo-deny earns its place from the start: its `licenses` check decides the licence
constraint for every dependency, and its `bans` refuse crates such as an async runtime where Q-1
forbids them.

The checks need a project to build, and the quality goals Q-2, Q-3 and Q-4 need one to be
measured on. Nectar's own documentation, the user guide, is what writers read. A second sample
next to it drifts apart from it: a feature can work in the sample and look broken in the guide,
and nobody looks at the sample once the guide exists. The guide can stand in for every feature
only if it uses each feature it explains itself, not only as a code sample: chapters included
from files of their own (G-2), front matter, embedded files, a raster image with attributes, and
a diagram of each engine.

The guide's project file decides what `nectar serve` watches and what a test that copies the
project copies. Next to the ADRs and the specification in `docs/`, it would watch and copy them
too.

## Decision

We will build, test and lint Nectar with Cargo through one script, `scripts/check-build.sh`, which
runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, a build of the user guide as the project in `docs/guide/`, a build that
links the crates that must run in WebAssembly for `wasm32-unknown-unknown`, and
`cargo deny check licenses bans sources` against a `deny.toml` that allows only licences
compatible with BSD-3-Clause, with the exceptions the specification names, and bans the crates
Q-1 keeps out of the core and the renderers.

- `docs/guide/` holds the guide's `nectar.toml`, its start document `GUIDE.md`, the chapters it
  includes and the files it embeds. The guide uses every feature it explains at least once
  itself: chapters included from files of their own, front matter, embedded source and grammar
  files, a raster image with attributes, diagrams of every engine, alerts, footnotes, symbols,
  variables, task lists and tables. There is no second sample project.
- `scripts/check-all.sh` runs `scripts/check-build.sh` with a `run` line, so the pre-push hook
  refuses a red change.
- The `build` job of `checks.yml` sets up the stable toolchain with the WebAssembly target and
  runs it on Linux, macOS and Windows; it is the one workflow job that builds Nectar, and
  `CONTRIBUTING.md` lists its matrix runs among the required checks.
- Which crates the WebAssembly step links is
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

- **`examples/demo` as the project the checks build** — the first form of this decision: a demo
  that used each feature once, in a few lines, next to the guide, and on which the specification
  measured Q-2, Q-3 and Q-4. It did not stand: two samples to keep in step, and the one the
  checks built was not the one the readers saw.
- **Build the guide as one page in `docs/`** — the guide as it was before it became the project
  the checks build. Not chosen: G-2, embedded files, front matter, raster images and the pages of
  a split document would leave the build the checks run, held only by the tests of single
  features.
- **Keep a small fixture under `crates/cli/tests/`** — a project kept as test material for what
  the guide does not use. Not chosen: it is the second sample again, only out of sight.
- **The guide's project in `docs/`** — no directory of its own. Not chosen: `nectar serve` would
  rebuild on every edit of an ADR or the specification, and a test that copies the project would
  copy all of `docs/`.
- **The linter's own rules only** — rustfmt, Clippy and the tests. Not chosen: the licence
  constraint and Q-1 would be held by review alone.
- **One tool per kind** — Clippy's `too_many_lines` and `cognitive_complexity`, jscpd for
  duplication, cargo-mutants on a schedule. Not chosen now: nothing yet shows the mistakes they
  would catch; a design revision adds them.
- **`nectar.yml` next to `checks.yml`** — the matrix kept in its own workflow, as Nectar started.
  Not chosen: two places for the same commands, and the pre-push hook would still not run them.
- **`cargo check` for the WebAssembly step** — what the step ran until 2026-11. Not chosen: it
  never links, so a link error, such as a second `getrandom` backend, would reach a WebAssembly
  host unnoticed.

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

- Positive: one sample; a feature that breaks breaks in the documentation readers use, and the
  checks see it there first. The pre-push hook and CI run the same commands; the licence
  constraint and the crates Q-1 forbids are decided by a check; a dependency that cannot build or
  link for WebAssembly fails before it reaches the core.
- Negative / trade-offs: the guide carries files of its own, chapters and samples to embed, and a
  change to its wording can turn a check red. Q-4's 200 ms is measured on the guide, a larger
  project than a demo, and a rebuild may need work to stay under it. cargo-deny and the
  WebAssembly target are more to install in the Dev Container and CI; a new dependency with an
  unlisted licence fails until `deny.toml` takes it up, which is the point.
- Follow-ups: none.

## Enforcement

The tests of the golden path, in `crates/fetch/`, cite this ADR with a `Verifies:` marker. The
tests that cite Q-2, Q-3 and Q-4 build `docs/guide/`, and Q-2's asserts that the guide has
headings, tables, pictures and code. A test of the guide's project fails when the guide stops
using a feature it explains: an included chapter, front matter, an embedded file, a raster image
with attributes, a diagram of each engine. Check 9 of
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `build` job, the `run` line of
`scripts/check-all.sh`, and the required checks of `CONTRIBUTING.md` together.
