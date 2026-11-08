# ADR-0016: `nectar build` given a Markdown file writes its PDF next to it, without a `nectar.toml`

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the command line in `crates/cli/`, its `build` command and its arguments, and the tests that verify G-10

## Context

Every command of Nectar reads a `nectar.toml` with at least one `[[document]]`; without one it
stops with "cannot read the configuration". The smallest configuration is two lines, but writing
it is a step between a Markdown file and its PDF that a writer who wants one PDF does not need.
The maintainer wants `nectar build` to take a Markdown file and write its PDF, with no
configuration.

The forces:

- The command line is a public interface: what it accepts binds scripts and users
  ([`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 1).
- Q-5 and G-7 confine a build to its project root. Without a `nectar.toml` there is no project
  directory to name, so the root has to follow from the file.
- G-3 already writes a PDF for every document, from the same document model as its website; the
  PDF renderer needs no website to run.
- G-5 keeps a block plain code and reports it when its engine is not configured; Graphviz and
  PlantUML run only where `nectar.toml` sets them up under `[engine.process.*]`.
- The specification names projects only (G-1). This decision needs a goal of its own, G-10,
  which the maintainer lands.

## Decision

We will have `nectar build` take an optional Markdown file and, when one is given, build that
file alone without reading any `nectar.toml`: the file's directory is the project root, the
configuration is the defaults with the file as its one document, and only the PDF is written,
to the file's name with `.pdf` next to it, or to the file `-o` names.

- Without a file, `nectar build` builds the project as before; `serve` is unchanged.
- A `nectar.toml` beside the file is ignored, so the result does not depend on the directory.
- `-C` and a file together are an error; the file is resolved against the current directory.
- Code engines are the built-in ones, as [ADR-0017](0017-render-command.md) names them, on with
  their defaults; a block for an engine nothing configures stays plain code and is reported (G-5).
- Includes, images and embedded files outside the file's directory are refused and reported
  (G-7), and the build touches nothing else outside it (Q-5) but the PDF it writes.
- `--offline` and `--drafts` mean what they mean for a project.

**Out of scope:** `serve` for a single file; a website or print view for a single file; several
files in one call; choosing the output format; reading settings such as variables or print
options from anywhere but the file's own front matter.

## Alternatives considered

- **`nectar <file>`, without a command** — the shortest call. Not chosen: a second way to build
  next to `build`, and a file named like a command would be ambiguous.
- **A command of its own, `nectar pdf <file>`** — clear by its name. Not chosen: a third command
  for what `build` already does for a project.
- **The current directory as the root** — the file must lie below where the command runs. Not
  chosen: the result would depend on where the command is run, and `nectar build ../x.md` would
  fail for a reason the writer cannot see.
- **Use a `nectar.toml` beside the file** — its variables, engines and print settings for this one
  document. Not chosen: the same call would give different PDFs in different directories.
- **Default process engines** — Graphviz and PlantUML as the guide sets them up, when found on the
  host. Not chosen: a program would run that nothing configured, against what G-5 and Q-5 expect
  of a build.
- **PDF and website** — the PDF beside the file and the website in `public/`. Not chosen: the
  request is a PDF; a website for one file is what a project is for.

## Sources / Prior art

- Typst — <https://typst.app/docs/reference/>: `typst compile file.typ` writes `file.pdf` next to
  the source unless an output is named.
- Pandoc — <https://pandoc.org/MANUAL.html>: `pandoc in.md -o out.pdf`, one file in, one file out,
  no project file.
- mdBook — <https://rust-lang.github.io/mdBook/>: needs its `book.toml` for every build, as Nectar
  does today.

## Consequences

- Positive: one command from a Markdown file to a PDF; the PDF renderer runs alone, without the
  website and the print view.
- Negative / trade-offs: `build` has two modes, and `-o` names a directory for a project but a
  file for a single file. A file that relies on Graphviz or PlantUML gets plain code until it
  becomes a project. Diagrams from the built-in engines and highlighting work as in a project.
- Follow-ups: `serve` for a single file, if it is wanted, is a decision of its own.

## Enforcement

Tests in `crates/cli/tests/` cite this ADR and G-10: `nectar build <file>` in a directory without a
`nectar.toml` exits zero and writes exactly `<file stem>.pdf` next to the file; with `-o` it writes
that file instead; a `nectar.toml` beside the file changes nothing; an image outside the file's
directory is reported and not read; `-C` together with a file fails.
