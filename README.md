# Nectar

[![Checks](https://github.com/hivevm/nectar/actions/workflows/checks.yml/badge.svg)](https://github.com/hivevm/nectar/actions/workflows/checks.yml)

**Nectar** turns Markdown into a website and a PDF: `nectar build` writes the site, `nectar serve`
serves it on your machine and reloads the browser on every change. The work is driven by a written
**specification** ([`docs/SPECIFICATION.md`](docs/SPECIFICATION.md)) and **Architecture Decision
Records** ([`docs/adr/`](docs/adr/)), so intent and the reasoning behind every structural choice
stay explicit and reviewable.

> For agent instructions, see [`AGENTS.md`](AGENTS.md), the single source of truth for all coding
> agents.

## Overview

Nectar is the command line `nectar`, written in Rust under [`crates/`](crates/). It turns a
project of Markdown files into a website with a print view and a PDF, highlights code with
TextMate grammars and draws railroad, Graphviz and PlantUML diagrams. The
[Nectar guide](docs/GUIDE.md) describes it in full, its design is
[`docs/DESIGN.md`](docs/DESIGN.md), and [`examples/demo/`](examples/demo/) is a
complete project to start from.

The full problem statement and goals live in [`docs/SPECIFICATION.md`](docs/SPECIFICATION.md), the
vocabulary in [`docs/GLOSSARY.md`](docs/GLOSSARY.md).

## Prerequisites

- [VS Code](https://code.visualstudio.com/) with the
  [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
  extension, or any DevContainer-compatible IDE
- Docker / Podman (rootless) available on the host
- Outside the Dev Container: bash 4.4 or later for the checks, 3.2 for the git hooks
  ([ADR-0002](docs/adr/0002-decisions-verified-by-tests.md)); macOS ships 3.2

## Getting Started

1. Open the repository in VS Code and choose **Reopen in Container**. The Dev Container and the
   preconfigured agent extensions build automatically, and the container enables the repository's
   git hooks with the merge settings they rely on ([Git hooks](CONTRIBUTING.md#git-hooks)). Working
   outside the container? Run the three commands listed there yourself, once per clone.
2. Authenticate your coding agent inside the container (for Claude Code: `claude login`).
3. Start working with the agent. Drive the work from the specification and the ADRs.

## Build, Test & Run

<!-- This section is the single source for build/test/lint/run commands; both humans and
     agents rely on it, and the rule file names it. -->

The Dev Container brings Rust with the `wasm32-unknown-unknown` target, cargo-deny, Graphviz and
PlantUML, with Java 21 as PlantUML's runtime, and strace, under which a test traces a build on
Linux. The commands run from the repository root; `scripts/check-build.sh` runs build, test and
lint in one go, as the pre-push hook and the `build` job of CI do
([ADR-0008](docs/adr/0008-toolchain-cargo-clippy-deny.md)):

- **Build:** `cargo build --workspace`
- **Test:** `cargo test --workspace`, then `cargo run -p nectar -- -C examples/demo build`,
  which has to succeed even where Graphviz and PlantUML are missing, and
  `cargo check --target wasm32-unknown-unknown -p nectar-core -p nectar-html -p nectar-pdf
  -p nectar-railroad`
- **Lint:** `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo deny --all-features check licenses bans sources`; the repository checks with
  `scripts/check-all.sh`
- **Run:** `cargo run -p nectar -- -C examples/demo serve`, then open http://127.0.0.1:1313/

The `build` job of [CI](.github/workflows/checks.yml) runs `scripts/check-build.sh` on Linux,
macOS and Windows. A release of Nectar is a tag `nectar-vX.Y.Z`: the
[release workflow](.github/workflows/nectar-release.yml) builds `nectar` for Linux (x86-64,
ARM), macOS (x86-64, ARM) and Windows (x86-64) and attaches the archives, with the notices of
the crates and their `SHA256SUMS`, to a GitHub release of that tag.

## Usage

Download the archive for your platform from the
[releases](https://github.com/hivevm/nectar/releases), unpack it and put `nectar` on your
`PATH`, or build it from source with `cargo install --path crates/cli`.

A project is a directory with a `nectar.toml`:

```toml
title = "My Manual"

[[document]]
source = "content/manual.md"
```

```sh
nectar serve            # http://127.0.0.1:1313/, reloads on every change
nectar build            # writes the site, the print view and the PDF to public/
```

The options of the command line, chapters, front matter, the Markdown dialect, code engines,
the print view, themes and every key of `nectar.toml` are described in the
[Nectar guide](docs/GUIDE.md).

## Template

- **Template release:** v0.7.0 — the release of [NUC](https://github.com/hivevm/nuc) this
  repository carries; a project moves the line when it takes up a later one
  ([ADR-0005](docs/adr/0005-template-releases.md)).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow (specification- and ADR-driven, small
reviewable changes), the project layout, the Dev Container, the coding agents, the git hooks, the
integration branches, and the repository settings, and [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
for the community standards we expect of everyone taking part. Security issues: please follow
[`SECURITY.md`](SECURITY.md) instead of opening a public issue.

## License

Released under the BSD 3-Clause License. See [`LICENSE`](LICENSE).
