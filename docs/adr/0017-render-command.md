# ADR-0017: `nectar render` runs one code engine on one text

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the command line in `crates/cli/`, its `render` command and its arguments, and the tests in `crates/cli/tests/render.rs`

## Context

A fenced code block names a language, and a code engine turns its text into highlighted code or
a graphic (G-5). Today that only happens inside a document: to see what `railroad`, a TextMate
grammar or a `[engine.process.*]` program makes of a snippet, the writer has to put it into a
Markdown file and build. The writer wants to call an engine directly from the command line: text
in, the engine's output out, for a quick look, for a script, or to place a diagram somewhere
Nectar does not build.

Forces:

- The command line is a public interface ([ADR-0016](0016-single-file-build.md)): what it accepts
  binds scripts and users.
- A snippet should look the same from the command line as in a built document, so the engines are
  the same set, configured the same way.
- The port `CodeEngine` returns either `Highlighted` tokens or a `Graphic` that is SVG. A second
  output format is a conversion of that, not a concern of each engine.
- Confinement (Q-5): Nectar touches nothing the writer did not point it at. A process engine runs
  a program; it may only run when the writer named the configuration that sets it up.
- [ADR-0016](0016-single-file-build.md) rejected the result depending on the directory the
  command runs in. The same holds here.

G-11 is the criterion for `render`: what it writes, where, and when it writes nothing.

## Decision

We will add a command `nectar render <LANGUAGE> [FILE] [-o <PATH>] [-f <FORMAT>]` that reads one
text from `FILE`, or from standard input when `FILE` is missing or `-`, runs the code engine that
supports `LANGUAGE` as a code block's info string would select it, and writes the result to the
file `-o` names, or to standard output.

- **Engines.** The same set `build` and `serve` use. The built-in engines, `railroad`, `rough` and
  TextMate highlighting, run in every command with their defaults, each unless
  `[engine.<name>] enabled = false`; without a `nectar.toml` they are the only ones. Without
  `-C`, `render` reads none. With `-C <dir>`, the `[engine]` table of that directory's
  `nectar.toml` configures them as for a build, process engines included.
- **Formats.** `-f` chooses among the formats the engine's output allows:

  | Output of the engine | Formats       | Default |
  |----------------------|---------------|---------|
  | a graphic            | `svg`, `png`  | `svg`   |
  | highlighted code     | `html`, `ansi`| `html`  |

  Without `-f`, the extension of `-o` chooses: an allowed format is used; a format of the other
  kind of output, such as `.png` for highlighted code, is an error as the same `-f` would be; any
  other extension, or none, gives the default. `ansi` is 24-bit colour escape sequences for a
  terminal; `html` is a `<pre>` with inline styles, standalone, without the site's stylesheet.
- **Errors.** A language no engine supports, a format the output does not allow, given by `-f` or
  by the extension of `-o`, or an engine that fails is an error on standard error with a non-zero
  exit, and nothing is written. The message for a format names the allowed ones. Unlike a build
  (G-5), there is no plain-code fallback: the call asked for the engine's output and nothing else.
- `--offline` has no effect on `render`; no engine fetches.

**Out of scope:** several snippets in one call; a dark-theme variant chosen on the command line;
options of an engine, such as a theme, on the command line instead of in `nectar.toml`; how a
graphic becomes a PNG, which a separate ADR decides.

## Alternatives considered

- **An option on `build`, such as `nectar build --engine <lang>`.** Not chosen: `build` builds
  documents into an output directory; `render` has no document, no renderer and no output
  directory, and would share nothing with it but the name.
- **The language taken from the file extension (`nectar render grammar.railroad`).** Not chosen:
  standard input has no extension, and a code block selects its engine by name, not by file type.
  The name keeps both ways of selecting the same.
- **Read `nectar.toml` of the current directory when present.** Not chosen: the same call would
  run different engines, and possibly a program, depending on where it runs, which
  [ADR-0016](0016-single-file-build.md) rejected for the same reason.
- **`ansi` as the default for highlighted code when standard output is a terminal.** Not chosen:
  a default that changes with where the output goes is harder to predict and to test; `-f ansi`
  is three words.
- **The engine chooses the format, so a process engine could emit PNG itself.** Not chosen: the
  port would have to negotiate formats with every engine; converting the one SVG keeps the port as
  it is and every engine gets every format.
- **Any extension the output does not allow falls back to the default.** Not chosen:
  `nectar render rust main.rs -o main.png` would succeed and write HTML into `main.png`; a file
  named for one format and holding another is found only when it fails to open.
- **No command: put the snippet into a Markdown file and `nectar build` it.** Not chosen: that is
  the status quo the writer wants to leave, and a single-file build writes a PDF, not the graphic.

## Sources / Prior art

Recalled, not re-checked in this session:

- Graphviz — <https://graphviz.org/doc/info/command.html>: `dot -Tsvg in.gv -o out.svg`, standard
  input and output by default, the format named by `-T`.
- Pygments — <https://pygments.org/docs/cmdline/>: `pygmentize -l <lexer> -f html|terminal256
  file`, the language by name, the format by `-f`, standard output by default.
- Mermaid CLI — <https://github.com/mermaid-js/mermaid-cli>: `mmdc -i in.mmd -o out.png`, the
  format taken from the extension of `-o`.

## Consequences

- Positive: every engine Nectar has is usable outside a document, in scripts and pipes, with the
  output a build would embed.
- Positive: the port `CodeEngine` stays as it is; formats are serializations in the command line.
- Negative / trade-offs: a third command and a fourth output format (`ansi`) to keep stable as a
  public interface.
- Negative / trade-offs: without `-C`, a process engine is not available, even when the current
  directory has a `nectar.toml` that sets it up; the error message has to say so.
- Follow-ups: none. The website and `render` write highlighted code as HTML through one token
  writer in `nectar-html`.

## Enforcement

Tests in `crates/cli/tests/render.rs` cite this ADR (`Verifies: ADR-0017`). They
run the built binary and check: `railroad` from standard input gives SVG on standard output; `-o
x.png` gives a PNG; `-f ansi` and `-f html` for a TextMate language; a disallowed format, by `-f`
or by the extension of `-o`, and an unknown language fail with nothing written, and an extension
that names no format gives the default; a `nectar.toml` with a process engine in the current
directory is ignored without `-C` and used with it.
