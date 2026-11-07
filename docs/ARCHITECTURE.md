# Architecture — Nectar

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([ADR-0001](adr/0001-agent-governance-model.md)).
>
> **Last design revision:** 2026-11-07, due after 20 changes. The revision that took the count
> moves the date; the number is this project's to set; a sensor counts the changes outside
> `docs/` since the commit that set the date and says when the next is due
> ([ADR-0006](adr/0006-feature-layer.md)).

## Context

```
 writer ──edits──▶ project (nectar.toml, Markdown, images) ◀──reads── nectar ──writes──▶ output
                                                              │  │  │            (website, print
 browser ◀──HTTP, live reload── nectar serve ◀────────────────┘  │  │             view, PDF)
 web hosts ◀──images over HTTP── fetcher ◀───────────────────────┘  │
 Graphviz, PlantUML ◀──stdin/stdout── process engine ◀──────────────┘
```

Nectar reads a project below its root and nothing outside it; the output is files on disk
(`nectar build`) or answers from memory (`nectar serve`). `nectar render` takes one text from a
file or standard input and writes one engine's output to a file or standard output; of a project
it reads only the `[engine]` table of the `nectar.toml` that `-C` names, and nothing without
`-C`. Everything that crosses the boundary goes through an adapter the host plugs in: the
project's files, images from the web, external programs for diagrams, and the browser of
`nectar serve`; so do the threads that run the code engines side by side
([ADR-0009](adr/0009-ports-and-adapters-technology.md)).

## Building blocks

The crates under `crates/`, every dependency pointing toward the core
([ADR-0009](adr/0009-ports-and-adapters-technology.md)):

- **`nectar-core`** — loads the project, parses the dialect into the document model, merges the
  chapters and runs the passes; owns the ports `Files`, `Workers`, `CodeEngine`, `Renderer` and
  `Fetcher`. Knows no adapter, no output format, no file system, no threads and no environment
  variables but those the shell hands over. `MemoryFiles` holds a project's files in memory,
  for a host without a file system and for the tests of every crate.
- **`nectar-html`** — the renderer for the website and the print view, with the built-in theme.
- **`nectar-pdf`** — the renderer for the PDF, typeset with Typst in memory.
- **`nectar-textmate`**, **`nectar-railroad`**, **`nectar-rough`** — code engines that need
  nothing of the host. The rough engine redraws an SVG as a sketch with `roughr`, reading it with
  `usvg` and drawing text with the fonts the shell hands it. The railroad and rough engines build
  for WebAssembly; the TextMate engine does not, since `giallo` needs the C libraries Oniguruma
  and zstd.
- **`nectar-process`** — the code engine that runs a program named in `nectar.toml`, such as
  Graphviz or PlantUML.
- **`nectar-fetch`** — the fetcher that downloads images from the web over HTTP, within limits
  of time and size.
- **`nectar-native`** — the adapters for a machine: the project's files in a directory on disk,
  kept inside the root, and threads for the code engines.
- **`nectar-server`** — the shell of `nectar serve`: HTTP server, file watcher, live reload.
- **`nectar`** — the command line, the one place that composes the core with the renderers, the
  engines and the adapters, and writes the build to disk.

**Golden path:** the `Fetcher` port — the trait in `crates/core/src/fetch.rs`, the decisions that
stay in the core in `crates/core/src/download.rs`, the adapter in `crates/fetch/`, and the
composition in `crates/cli/src/main.rs`. **Test pattern:** the core's side tested against a fake
behind the port (`download.rs`), the adapter against a real counterpart on loopback
(`crates/fetch/`), and the shape held by `crates/cli/tests/structure.rs`.

## How it runs

`nectar build`: the command line opens the project on its directory, plugs in the workers, the
engines and the fetcher, and asks the core for a build; the core loads every start document with its
chapters, runs the passes (ids, numbers, footnotes, links, assets, images from the web, code
engines) and hands the enriched documents to each renderer. The build is a map of output files in
memory, which the command line writes. `nectar build <file>` opens a project of that file's
directory with `Project::new` and a configuration made in memory, and writes only its PDF. `nectar
render` builds no project: it composes the same engines, runs the one that takes the language on
one text, and serializes its output in the command line (`crates/cli/src/render.rs`), except
highlighted code as HTML, which `nectar-html` writes with the token writer of the site. `nectar
serve` keeps the project and its engine cache between builds, rebuilds on every change below the
root and swaps the build the server answers from. Details are in [`docs/DESIGN.md`](DESIGN.md).

## Delivery

A tag `nectar-vX.Y.Z` starts `.github/workflows/nectar-release.yml`, which runs
`scripts/release.sh`: it builds the command line per target, Linux on `ubuntu-22.04` against
glibc 2.35, packs each archive with `LICENSE` and the crates' `NOTICE` files, writes `SHA256SUMS`,
attests every archive and publishes the release. `release/install-nectar.sh` fetches one
version's archive for the host and installs it only when it matches the checksums its caller
carries; `scripts/check-all.sh` runs `scripts/release.sh` without a command to test the packing
and the installer before a tag is pushed
([ADR-0016](adr/0016-release-glibc-floor.md)). The user guide in `docs/guide/` is the project the
checks build ([ADR-0014](adr/0014-guide-as-reference-project.md)).
