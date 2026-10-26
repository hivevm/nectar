# Architecture — Nectar

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([ADR-0001](adr/0001-agent-governance-model.md)).
>
> **Last design revision:** 2026-10-25, due after 20 changes. The revision that ran moves the
> date; the number is this project's to set; a sensor counts the changes outside `docs/` since
> the commit that set the date and says when the next is due
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
(`nectar build`) or answers from memory (`nectar serve`). Everything that crosses the boundary
goes through an adapter the host plugs in: the project's files, images from the web, external
programs for diagrams, and the browser of `nectar serve`; so do the threads that run the code
engines side by side ([ADR-0009](adr/0009-ports-and-adapters.md)).

## Building blocks

The crates under `crates/`, every dependency pointing toward the core
([ADR-0009](adr/0009-ports-and-adapters.md)):

- **`nectar-core`** — loads the project, parses the dialect into the document model, merges the
  chapters and runs the passes; owns the ports `Files`, `Workers`, `CodeEngine`, `Renderer` and
  `Fetcher`. Knows no adapter, no output format, no file system and no threads.
- **`nectar-html`** — the renderer for the website and the print view, with the built-in theme.
- **`nectar-pdf`** — the renderer for the PDF, typeset with Typst in memory.
- **`nectar-textmate`**, **`nectar-railroad`** — code engines that need nothing of the host.
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
engines and the fetcher, and asks the core for a build; the core loads every start document with
its chapters, runs the passes (ids, numbers, footnotes, links, assets, images from the web, code
engines) and hands the enriched documents to each renderer. The build is a map of output files in
memory, which the command line writes. `nectar serve` keeps the project and its engine cache
between builds, rebuilds on every change below the root and swaps the build the server answers
from. Details are in
[`doc/nectar/design.md`](../doc/nectar/design.md).
