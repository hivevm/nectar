# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for this project. Accepted ADRs are
**binding** for humans and coding agents alike (see [`AGENTS.md`](../../AGENTS.md) in the repository
root). ADRs derive from the specification in [`docs/SPECIFICATION.md`](../SPECIFICATION.md).

## Mechanics

The rules live in three `NUC-` ADRs.
[NUC-decisions-are-proposed-then-accepted](NUC-decisions-are-proposed-then-accepted.md) decides
when an ADR is written, how it is developed, who flips its status, and that an accepted one is
immutable. [NUC-adrs-are-named-by-category](NUC-adrs-are-named-by-category.md) decides how it is
named, cited, superseded, and indexed. [NUC-base-is-fixed](NUC-base-is-fixed.md) decides the fixed
base. This section carries only the mechanics of the record;
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) names in its header which of them it
verifies.

- **File.** Copy [`template.md`](template.md) to `<CAT>-<slug>.md` and head it
  `# <CAT>-<slug>: <title>`, the category from the [list](#categories). The author picks the
  identifier when creating the file, and it never changes or returns
  ([NUC-adrs-are-named-by-category §1](NUC-adrs-are-named-by-category.md)).
- **Wording.** The *Decision* and the *Enforcement* follow the writing rule of
  [`AGENTS.md` §4](../../AGENTS.md#4-working-style).
- **Index.** The tables below list the project's ADRs, grouped by category, and no `NUC-` ADR
  ([NUC-adrs-are-named-by-category §5](NUC-adrs-are-named-by-category.md)). They change in the same
  commit as the ADR they describe, for additions, supersessions, and status flips alike. The status
  is shown via the legend's bullet and the ADR's `Applies to` header is mirrored verbatim in its own
  column. The index routes a reader from a change to the decisions that bind it, so a row says what
  its ADR governs without the file being opened. Two tables keep what a session reads apart from
  what it looks up: an accepted or proposed ADR sits under **Binding**, a superseded or rejected one
  under **Superseded and rejected**, and a status flip moves the row.
- **Identifiers are permanent once on `main`.** A normalization keeps them, and only a merged or
  split ADR takes a new one
  ([NUC-decisions-are-proposed-then-accepted §8](NUC-decisions-are-proposed-then-accepted.md)).
  Between two normalizations, never rename, delete, or merge an ADR there: other ADRs, commits
  (`Implements <CAT>-<slug>`), and code cite it. A superseded ADR keeps its file and its body. The
  superseding ADR's `Supersedes` field names it from the proposal on; its `Status` line flips to
  `⚪ superseded by <CAT>-<slug>` in the change where a human accepts the successor
  ([NUC-adrs-are-named-by-category §4](NUC-adrs-are-named-by-category.md)), its index row moving
  with it.
- **Between normalizations, sprawl is curbed by supersession, never by editing.** One ADR may
  supersede several whose decisions have grown into one. Its `Supersedes` field names each, and
  each flips its status when the superseding ADR is accepted, so the set an agent reads shrinks
  while every identifier and every body stays.
- **A normalization restates the record**
  ([NUC-decisions-are-proposed-then-accepted §8](NUC-decisions-are-proposed-then-accepted.md)).
  - **Start.** The tree has no proposed ADR, no feature spec or ticket under `docs/plan/`, and
    green checks.
  - **Steps.** Three, in this order on one branch, each handed to the human and confirmed
    before the next begins: the restated specification, whose `G-n` and `Q-n` run from 1; the
    supersessions it calls for; and the restated record. The record begins once the human has
    decided each supersession. One pull request lands them, merged by a human.
  - **Identifiers.** An ADR restated alone keeps its identifier; a merged or split one takes a
    new one. The `NUC-` files stay as the release brought them.
  - **Clauses.** Every clause of an old ADR lives on in one new ADR. The hand-off of the
    record lists each clause with the new ADR that carries it.
  - **Header.** A new ADR keeps the status of the ADRs it restates. Its `Deciders` name all of
    theirs, its `Date` is the latest of theirs, and it has no `Supersedes` field. In an
    adoption, an ADR the project had accepted stays accepted. A decision found outside any
    record enters as proposed.
  - **Dead ADRs.** Superseded and rejected ADRs are deleted. What one of them chose, and why it
    did not stand, becomes an entry under *Alternatives considered* in the new ADR that governs
    its topic.
  - **References.** Each step moves its references with a map, old to new, applied by the map
    script of the `normalize` skill. The map stands in the hand-off and the
    commit body, and in no document. A commit older than the normalization cites the identifiers
    it had then.
- **Cite only what exists.** Every citation, `<CAT>-<slug>` or a clause `<CAT>-<slug> §n`, names a
  file already in this directory. An anticipated follow-up is described by topic ("a follow-up ADR
  on session storage"). What counts as a citation, and its form in Markdown, is
  [NUC-adrs-are-named-by-category §6](NUC-adrs-are-named-by-category.md).

## Index

**The base:** the template's `NUC-` ADRs in this directory, each named with its blob in
[`NUC.blobs`](NUC.blobs), as `AGENTS.md` is. They bind like any ADR in force, and a project
extends or narrows a clause in an ADR of its own, never edits one
([NUC-base-is-fixed §1](NUC-base-is-fixed.md)).

### Categories

The category of an identifier says what its decision is about
([NUC-adrs-are-named-by-category §2](NUC-adrs-are-named-by-category.md)). The fixed ones:

- **NUC** — the template's own ADRs, reserved to the template repository.
- **SEC** — security.
- **OPS** — build, release, deployment, and operation.
- **API** — interfaces the system offers and consumes, adapters among them.

The project's own follow here, each in the same form with one line on what it holds:

- **ARCH** — the shape of the code: the core, its ports, and how the crates divide.
- **DEV** — how the work is done: the harness, the reviews, and the development environment.
- **ENG** — the code engines: what each draws or highlights, on what, and on which hosts.
- **OUT** — the output formats: what a renderer writes, and the libraries it writes with.

**Status legend:** 🟢 accepted · 🟡 proposed · 🔴 rejected · ⚪ superseded

### Binding

The decisions in force: accepted, and proposed ones that bind the work implementing them. This
is the table a session reads.

| ADR | Title | Applies to | Status |
|-----|-------|------------|--------|
| [SEC-actions-pinned-by-sha](SEC-actions-pinned-by-sha.md) | The workflows reference every action by its full commit SHA, kept current by Dependabot | every `uses:` line under `.github/workflows/`, `.github/dependabot.yml`, and `scripts/check-all.sh` | 🟢 accepted |
| [OPS-cargo-clippy-deny-toolchain](OPS-cargo-clippy-deny-toolchain.md) | Cargo builds and tests Nectar, rustfmt, Clippy and cargo-deny are its first sensors, run by one script, and the user guide is the project the checks build | `Cargo.toml`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-all.sh`, the `build` job of `.github/workflows/checks.yml`, the Build, Test & Run section of `README.md`, `docs/guide/`, and the tests that measure Q-2, Q-3 and Q-4 | 🟢 accepted |
| [OPS-release-archives-attested](OPS-release-archives-attested.md) | A Nectar release is archives built against glibc 2.35, their checksums and a provenance attestation for each, and the repository carries the installer that verifies them | `.github/workflows/nectar-release.yml`, `scripts/release.sh`, `release/install-nectar.sh` and the check that tests it, and every installer outside this repository that takes a Nectar release, such as the Dev Container Feature of `hivevm/features` | 🟢 accepted |
| [API-render-runs-one-engine](API-render-runs-one-engine.md) | `nectar render` runs one code engine on one text | the command line in `crates/cli/`, its `render` command and its arguments, and the tests in `crates/cli/tests/render.rs` | 🟢 accepted |
| [API-rough-defaults-in-config](API-rough-defaults-in-config.md) | `[engine.rough]` in `nectar.toml` sets the defaults of the rough engine | the keys of `[engine.rough]` in `nectar.toml`, how the command line hands them to the rough engine in `crates/cli/` and `crates/rough/`, and the tests that verify them | 🟢 accepted |
| [API-single-file-build](API-single-file-build.md) | `nectar build` given a Markdown file writes its PDF next to it, without a `nectar.toml` | the command line in `crates/cli/`, its `build` command and its arguments, and the tests that verify G-10 | 🟢 accepted |
| [ARCH-core-owns-its-ports](ARCH-core-owns-its-ports.md) | A core without technology, the host's environment and the tests' dependencies included, every host-dependent effect behind a port it owns | every crate under `crates/`, the dependencies and dev-dependencies in their `Cargo.toml`, and the structural test that holds the split | 🟢 accepted |
| [DEV-image-with-features](DEV-image-with-features.md) | The Dev Container is a base image with Features and a setup script, and the work needs no containers | `.devcontainer/`, and every change that would have the work start or reach a container | 🟢 accepted |
| [DEV-one-agent-human-reviews](DEV-one-agent-human-reviews.md) | One coding agent implements, the maintainer reviews every change, nothing runs unattended | every change, every agent session, the specification, the ADR record, and the release workflows under `.github/workflows/` | 🟢 accepted |
| [ENG-rough-on-roughr-usvg](ENG-rough-on-roughr-usvg.md) | The rough engine sketches with `roughr` and reads SVG with `usvg` | the dependencies of `crates/rough/Cargo.toml`, the `PORTABLE` list of `crates/cli/tests/structure.rs`, the WebAssembly step of `scripts/check-build.sh`, and the fonts the rough engine draws text with | 🟢 accepted |
| [ENG-rough-redraws-svg](ENG-rough-redraws-svg.md) | A code engine `rough` redraws an SVG in a hand-drawn style | the code engine for the language `rough` in `crates/rough/`, the input it accepts and its `data-rough-*` attributes, the shell that composes it in `crates/cli/`, and the tests that verify it | 🟢 accepted |
| [ENG-textmate-native-only](ENG-textmate-native-only.md) | The TextMate engine runs on native hosts only, and a WebAssembly host highlights through the code engine port | `crates/textmate/`, the WebAssembly step of `scripts/check-build.sh`, and the code engines a WebAssembly shell composes | 🟢 accepted |
| [OUT-epub-renderer](OUT-epub-renderer.md) | An `EpubRenderer` in `nectar-html` writes an EPUB 3.3 publication for every document that enables it | `EpubRenderer` in `crates/html/`, the table `[document.epub]` of `nectar.toml`, the shell that composes the renderer in `crates/cli/`, `crates/wasm-link/`, and the tests and the check that verify G-13 | 🟢 accepted |
| [OUT-own-zip-writer](OUT-own-zip-writer.md) | A ZIP writer of our own packs the EPUB container, on `miniz_oxide` and `crc32fast` | the ZIP writer in `crates/html/`, the dependencies on `miniz_oxide` and `crc32fast` in `crates/html/Cargo.toml`, and the tests that verify the container | 🟢 accepted |
| [OUT-png-via-resvg](OUT-png-via-resvg.md) | A graphic becomes a PNG through resvg, with the bundled fonts only | the `png` format of `nectar render` in `crates/cli/`, the dependency on `resvg` in `crates/cli/Cargo.toml`, and the fonts it renders text with | 🟢 accepted |

### Superseded and rejected

The record of what was decided against or replaced: read one when a change touches what it
governed, to learn why the current decision stands. A row moves here in the change that
flips its status.

None yet.
