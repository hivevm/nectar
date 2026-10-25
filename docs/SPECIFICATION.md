# Specification — Nectar

> The **specification** of this project: the problem it solves, where it is going, and what has to
> be true for it to have succeeded, written in the vocabulary of [`GLOSSARY.md`](GLOSSARY.md). It
> is the **constitution**: every Architecture Decision Record in [`adr/`](adr/) derives from it and
> must not contradict it ([ADR-0001](adr/0001-agent-governance-model.md)).

## Problem

Projects keep their documentation as Markdown in the repository, next to the code: user manuals,
developer manuals, specifications. HiveVM's projects do, and so do others outside it. They need it
as a website to read and link, and as a PDF to hand out, print or archive.

Today the website comes from Hugo with a theme. Hugo is a Go binary built for one job, a static
site: it gives no PDF, and it runs only as a program on a machine. It cannot be embedded in
another tool, and it cannot run in a browser or in a WebAssembly host to preview or render
documentation where it is edited. HiveVM's PDFs came from its own Gradle plugin `org.hivevm.doc`
through Apache FOP. That is a second tool chain with a Markdown dialect of its own, tied to the JVM
and to Gradle, so the two outputs drift apart in diagrams, highlighting and includes.

What is missing is one engine, written in Rust, that turns the same Markdown into both outputs and
runs anywhere: as a command line, as a library, and compiled to WebAssembly. With it, the
documentation no longer has to be maintained twice.

## Mission

Nectar lets a project write its documentation once in Markdown and publish it as a website and a
PDF from the same engine, wherever that engine runs.

## Vision

Nectar replaces both Hugo and the Gradle plugin for HiveVM's projects and becomes a tool other
projects pick for the same reason: one engine, one dialect, everywhere.

## Strategy

A core in Rust that parses the sources into a document model of its own and enriches it in passes,
with renderers for HTML and PDF behind one interface. The command line, the local server, and later
a WebAssembly build are thin shells around it. The core stays free of file system and process
access, so that it compiles to WebAssembly. Engines that need external programs, such as Graphviz
or PlantUML, sit behind a port and can be swapped per host.

## Goals / Success Criteria

Acceptance criteria a test can decide, where it helps in the form *When <trigger>, the system
shall <response>*. Each starts with a bold `G-n`, which a test cites with `Verifies: G-n`. Until
that test exists the criterion is pending work, and the traceability check lists it
([ADR-0002](adr/0002-decisions-verified-by-tests.md)). Identifiers are never reused.
Only a normalization renumbers them ([`AGENTS.md` §3](../AGENTS.md#3-adr-rules), rule 5).

1. **G-1** — When `nectar build` runs on a project, the system shall write a website for every
   `[[document]]` to the output directory, and exit non-zero exactly when a diagnostic is an
   error.
2. **G-2** — When a link to a Markdown file stands alone on its line, the system shall include
   that file as a chapter: its headings move below the heading the link follows, and its relative
   links and images resolve against its own directory.
3. **G-3** — When a document is built, the system shall also write a print view and a PDF from the
   same document model as its website, with cover, contents with page numbers, running headers and
   footnotes.
4. **G-4** — When the PDF cannot show an element of the source, such as raw HTML or an image it
   cannot embed, the system shall report it as a warning with file and line instead of dropping it
   silently.
5. **G-5** — When a fenced code block names a language with a TextMate grammar, or `railroad`, the
   system shall highlight or draw it in every output. When a block needs an external program that
   is missing or fails, it shall keep the block as plain code and report a warning.
6. **G-6** — When a file below the project root changes while `nectar serve` runs, the system shall
   rebuild and reload every connected browser. Errors appear in a panel on the page, and the server
   keeps running.
7. **G-7** — When a link, include or image points outside the project root, the system shall not
   follow it and shall report it.
8. **G-8** — When `nectar-core` and the renderers are compiled for `wasm32-unknown-unknown`, the
   build shall succeed, and a project given as files in memory shall produce the same website as
   the command line.
9. **G-9** — When a source contains raw HTML and `nectar.toml` does not set `html = true`, the
   system shall show it as text and report a warning with file and line; with `html = true` it
   shall pass it to the website, while the PDF still reports it (G-4).

## Quality Goals

The few properties that drive decisions, in priority order. Every ADR weighs its *Consequences*
against them. Each has a measure concrete enough to settle a disagreement and starts with a bold
`Q-n`, cited like a goal. Security goals, what the system must never do, belong here, derived
from the threats below.

- **Q-1** — Portability: the core and the renderers run wherever Rust compiles, WebAssembly
  included, because that is what Hugo cannot do. Measured by a structural test that fails on any
  use of a file system, process, thread or async runtime API in `nectar-core` and the renderer
  crates; only the shells (`cli`, `server`, `process`) use them.
- **Q-2** — Fidelity between outputs: a source means the same thing on the website and in the PDF,
  so nobody maintains the documentation twice. Measured on the documents in `examples/demo`: every
  heading, paragraph, table, figure and code block of the website appears in the PDF, or a
  diagnostic says it does not (G-4).
- **Q-3** — Determinism: the same sources and configuration give byte-identical output, so a build
  can be diffed and cached. Measured by two builds of `examples/demo` in a row, on the same machine
  and with images from the web offline, giving identical files.
- **Q-4** — Responsiveness while writing: an edit shows up in the browser before the writer looks
  over. Measured as a rebuild of `examples/demo` after a one-line edit, with the engine cache warm,
  in under 200 ms on a developer machine.
- **Q-5** — Confinement: Nectar touches nothing the writer did not point it at. Measured by tests
  that build fixture projects whose links, includes and images point outside the root and whose
  code blocks try shell injection, run with `--offline`, and fail on any file access outside the
  root and the output directory, any extra process, and any network request.

## Non-Goals

What this project explicitly does not try to do.

- No JVM tool chain. The former Gradle plugin `org.hivevm.doc` leaves the repository, and Nectar
  ships no Gradle integration of its own.
- No general static site generator. Nectar renders documentation: no blog, no taxonomies, no
  pagination of lists, no arbitrary content types as in Hugo.
- No other source formats. Markdown in Nectar's dialect only: no AsciiDoc, reStructuredText or Org.
- No Hugo or Pandoc compatibility. Nectar does not read Hugo shortcodes or themes, and does not aim
  at Pandoc's Markdown extensions.
- No editor. Nectar renders and previews; writing happens in the editor of the writer's choice.
- No hosting. `nectar build` writes files; publishing them, to GitHub Pages or elsewhere, is the
  project's business.
- No output formats beyond the website and the PDF, such as EPUB, DOCX or man pages.

## Constraints

What the solution has to live with regardless of the better choice: platforms, standards,
systems to interoperate with, limits of time, budget, and team. A constraint not written here is
rediscovered as a rejected alternative in an ADR.

- Rust, edition 2024, on the latest stable toolchain. Nectar is shipped as binaries, not as crates
  for others to build, so it declares no minimum supported Rust version.
- The command line runs on Linux (x86-64, ARM), macOS (x86-64, ARM) and Windows (x86-64);
  WebAssembly means `wasm32-unknown-unknown`, in the browser and in other hosts.
- CommonMark is the base of Nectar's Markdown dialect.
- The PDF is typeset with Typst as a library; the print view is laid out by Paged.js in the
  browser.
- Graphviz and PlantUML are always reached through the code engine port, by an adapter the host
  provides: a local process on the command line, whatever the host offers in WebAssembly. Nectar
  works without them and degrades as G-5 says.
- The licence is BSD-3-Clause; every dependency and everything bundled, themes, fonts, TextMate
  grammars and Paged.js among them, carries a compatible licence.
- One maintainer works with one coding agent.
- Nectar is distributed as binaries in GitHub releases, with SHA-256 checksums, and possibly as a
  Dev Container Feature; not on crates.io and not on npm.

## Threats & Forbidden Actions

The assets, the actors who threaten them, and the trust boundaries between the parts. Then what
the system must never do, regardless of what a task asks. A prohibition not written here does not
exist for an agent. One a test can decide is restated as a `Q-n`, so that it is verified.

- **Assets:** the files of the writer's machine outside the project; the browsers of the readers
  of a built site; the integrity of the released binaries; the writer's local network while
  `nectar serve` runs.
- **Actors and trust boundaries:**
  - The project root is a boundary. Sources come from the project's own repository and are trusted
    like its code, but a source from a pull request or a third party may try to reach beyond the
    root, or put markup into the readers' browsers.
  - Hosts of images from the web are untrusted: they may answer slowly, with huge bodies, or with
    something that is not an image.
  - The commands of process engines are named in `nectar.toml` and trusted like a build script;
    the code blocks fed to them are not.
  - Other machines on the network must not reach `nectar serve` unless the writer asks for it.
  - In WebAssembly, the host decides what Nectar sees; Nectar has no access of its own.
- The system shall never read or write a file outside the project root and the output directory.
- The system shall never run a program not named in `nectar.toml`, and shall pass code to it only
  on standard input, never as arguments or through a shell.
- The system shall never make a network request with `--offline`, nor any request other than
  fetching an image a source names, within the limits of time and size.
- The system shall never listen on any address but loopback unless `bind` or `--bind` says so.
- The system shall never pass raw HTML from a source into an output unless `html = true` in
  `nectar.toml` allows it; without it, raw HTML is shown as text and reported (G-9).

## Assumptions

What is believed true today and would invalidate the decisions above if it turned out false:
expected scale, the shape of the data, who operates the result, which dependency stays
maintained.

- A document has up to a few hundred pages and a few dozen diagrams, and a project a handful of
  documents; Q-4 and the build in memory rest on that.
- Typst stays maintained, keeps its library usable from Rust, and compiles to WebAssembly, so the
  PDF renderer can follow the core there (G-8, Q-1).
- Paged.js stays usable for the print view, and the readers' browsers run it.
- The sources are written by the project's own people and reviewed like code; raw HTML is the one
  way a source reaches the readers' browsers, and G-9 gates it.
- Writers who come from Hugo accept Nectar's dialect, since there is no compatibility.
- Where diagrams of Graphviz or PlantUML are wanted, the host provides an adapter for them: on the
  command line the programs are installed, in WebAssembly the host offers an engine, or the
  diagrams stay code.
- The bundled TextMate grammars and themes keep licences compatible with BSD-3-Clause.
