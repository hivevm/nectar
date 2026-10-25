# Nectar design

Nectar turns Markdown sources into HTML. It runs in two modes that share one core:

- `nectar build` writes a complete static site to an output directory.
- `nectar serve` serves the site over local HTTP, watches the sources and reloads the browser as
  soon as a source changes.

## Constraints

1. A start document includes further Markdown files through links. The result is one complete
   document.
2. The source format is CommonMark with the extensions listed under [Markdown dialect](#markdown-dialect).
3. A document model of its own, independent of Markdown, describes the document.
4. Syntax highlighting and diagrams are produced once, by engines behind a single port, in a
   transform pass. Their results are stored in the document model as format-neutral data.
5. Rendering works on the enriched document model, behind a renderer interface. HTML is the first
   renderer.
6. The core is shared. The CLI (`build`) and the HTTP server (`serve`) are thin shells around it.

## Pipeline

```
sources ─▶ parse (per file) ─▶ merge ─▶ document ─▶ transform passes ─▶ enriched document ─▶ renderer
                                                     ├─ ids
                                                     ├─ section numbers
                                                     ├─ footnotes
                                                     ├─ internal links
                                                     ├─ assets
                                                     └─ code engines (highlighting, diagrams)
```

Variables are replaced in the source text before parsing; symbols and the other syntax extensions
are recognised while parsing. Loading a file also embeds files referenced by images (see
[Embedded files](#embedded-files)).

Every stage is synchronous and works in memory. The core reads sources through the project root
and returns its output as data; it never writes files itself.

## Sources

### Start documents and includes

A project names one or more start documents. Each start document produces one complete document.

Every Markdown file is parsed on its own. Merging happens on the document model: an include
becomes a chapter node that carries the included file's document. A link is an include when

- it stands alone on its line, optionally after a heading marker (`## [Title](path.md)`);
  consecutive lines of such links include one file each,
- its target ends in `.md`, and
- the target file exists.

For an included file:

- its heading levels are shifted below the level at which it is included,
- the link text, if present, replaces the file's first top-level heading,
- relative paths (images, further includes) resolve against the included file's directory,
- footnote definitions are scoped to the file and numbered across the whole document.

An include cycle is an error. Every node records its origin (file and line), so diagnostics point
at the source file and not at a merged text.

### Front matter

A file may start with TOML front matter between `+++` lines. All fields are optional; a file
without front matter behaves exactly as one with an empty block.

```toml
+++
title = "Installation"   # overrides the first heading as page title
slug  = "install"        # path segment when the document is split into pages
draft = false            # drafts are left out by `build` and shown by `serve`
[vars]
version = "1.2.0"
+++
```

## Markdown dialect

On top of CommonMark:

| Feature | Syntax |
|---|---|
| Tables | GFM pipe tables; the number of `-` in a delimiter cell sets the column's relative width (in HTML a preference: no column gets narrower than its longest word) |
| Image attributes | `![alt](img.png){width=50% height=4cm align=center}` |
| Alerts | `!n`, `!s`, `!w`, `!e` at line start; multi-line as `!!w` … `!!`; also `> [!NOTE]` |
| Highlight | `//text//` |
| Underline | `____text____` |
| Strikethrough | `~~text~~` |
| Task lists | `- [ ]` and `- [x]` |
| Footnotes | `text[^id]` with a definition `[^id]: footnote text` |
| Links as footnotes | `[text][id]` or `[][id]` with `[id]: url "title"`, scoped to the file; to an external target the link gets a footnote with title and URL |
| Autolinks | bare URLs such as `https://example.org` |
| Comments | `[label]: # (comment)`, dropped from the output |
| Heading ids | `# Heading {#id}` |
| Internal links | `[text](#id)`, `[text](#heading-slug)` or `[text](#Heading Text)` |
| Symbols | `@name@`, a named glyph of the theme's symbol font |
| Variables | `{{name}}` or `{{name;format}}` (format ignored), from `nectar.toml`, environment variables `NECTAR_<name>` and front matter, each overriding the one before; unknown names stay as written and are reported outside code |
| Code | fenced with ` ``` ` or `~~~`; the info string selects the code engine |

## Document model

The document model is a tree that knows nothing of Markdown or of any output format.

- **Document**: title, metadata, and a sequence of blocks.
- **Chapter**: produced by an include; carries its source file, front matter and blocks.
- **Blocks**: heading (with its section number), paragraph, quote, alert, list and list item,
  table, thematic break, code, highlighted code, graphic, figure (a block with a caption).
- **Inlines**: text, emphasis, strong, highlight, strikethrough, underline, code, link, image,
  footnote reference, symbol, line break. Inlines nest; styles are not encoded as start and end
  markers. A link written as a reference link carries the number of its footnote.

Highlighted code holds lines of tokens, each with a resolved style (colour, background, bold,
italic, underline, strikethrough), and a second style when the engine highlighted with a dark
theme as well. A graphic holds SVG, an alternative text and the source it was
generated from.

## Images from the web

Images belong to the documentation wherever they come from. After the assets are collected, the
download pass fetches images with `http(s)` URLs and decodes `data:` URIs. An image must arrive
with an image media type, within 30 seconds and 20 MB and after at most five redirects. It is
stored with the document as an asset at `<document>/_remote/<hash>.<extension>`, its bytes kept
in the document, so every renderer includes it like a local image: the website serves it from
its own output, the PDF embeds it. The project fetches each URL once; a rebuild under
`nectar serve` reuses it. A failed download is reported and the image keeps its URL. `--offline`
fetches nothing.

## Embedded files

An image that stands alone in its paragraph and points to a local file that is not a picture
embeds that file: the loader replaces the paragraph with a code block holding the file's
content, its language taken from the file extension (`.java` → `java`, `.rs` → `rust`,
`.puml` → `uml`, otherwise the extension itself). A non-empty image text becomes the caption of
a figure around the block. The code engines then treat the block like any fenced block, so a
`.java` file is highlighted and a `.railroad` file drawn. The embedded file counts as a source
of the build.

## Section numbers and contents

With `numbered` (default on) the numbering pass numbers the headings of all chapters in reading
order. The top level stays unnumbered; below it numbers read `1.`, `1.2.`, `1.2.3.`. Below a
skipped level, as with `####` directly under `##`, numbering continues in Roman numerals.
Renderers show the number in front of the heading and of chapter titles.

With `contents` (default on) the first page of a document lists the headings of levels 2 and 3
of the whole document, linked across pages.

## Transform passes

A pass is a function over the whole document:

```rust
pub trait Pass {
    fn apply(&self, doc: &mut Document, ctx: &Context) -> Vec<Diagnostic>;
}
```

The core runs its passes in a fixed order, once per build, independent of the output format.

### Code engines

Highlighting and diagrams use the same port. An engine tells whether it handles an info-string
language and turns the code of a block into highlighted code or a graphic:

```rust
pub trait CodeEngine: Send + Sync {
    fn name(&self) -> &str;
    fn supports(&self, language: &str) -> bool;
    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError>;
}

pub enum EngineOutput {
    Highlighted(HighlightedCode),
    Graphic(Graphic),
}
```

The code engine pass asks the engines in the order they were added and gives the block to the
first one that supports its language. When no engine handles the language, the block stays as
plain code. When an engine fails, the block stays as plain code and the pass reports a warning at
the block's origin. Distinct blocks are rendered in parallel. Results are cached by engine,
language and a hash of the code for the lifetime of the project, so `serve` does not render an
unchanged diagram twice.

The core defines the port and knows no engine. Engines are adapters in crates of their own:

| Adapter | Languages | Works through |
|---|---|---|
| `nectar-textmate` | every language with a bundled TextMate grammar | TextMate grammars and themes in-process |
| `nectar-railroad` | `railroad` | (E)BNF parser and SVG layout in-process |
| `nectar-process` | as configured, such as `dot`, `uml`, `ebnf`, `regex`, `jsongraph` | a local command (`dot`, `plantuml`) |

The railroad engine draws every rule of a block under its name and stacks the drawings into one
SVG. A rule reference written as `\` continues a rule on a new line. A loop is labelled with
the number of passes through it (`3×`, `1..3×`, `≥2×`); "once or more" has no label. Text boxes are sized with
Verdana metrics, and texts are stretched to their box, so the drawing does not depend on the
fonts of the viewer.

`nectar.toml` selects the adapters and configures them. Process engines are asked first, then
railroad, then TextMate; railroad and TextMate are on unless their table says `enabled = false`.
A process engine writes the code, or the
code filled into its `input` template, to the command's standard input and takes SVG from its
standard output. A non-zero exit status or a missing `<svg` element is a failure; a command that
runs longer than its `timeout` is stopped.

## Rendering

```rust
pub trait Renderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext) -> Output;
}
```

A renderer gets all documents at once, so it can write shared files and a site index. Its output
holds the files, the project files it read and its diagnostics.

A renderer maps nodes to its format and does not repeat work done by passes: it writes tokens of
highlighted code as styled spans and embeds graphics as they are.

The website follows the system's colour scheme: the default theme has a dark variant, and
highlighted code carries the colours of both TextMate themes (`theme`, `dark_theme`, by default
`github-light` and `github-dark`) as custom properties, from which the stylesheet picks. The
print view and the PDF stay light.

The HTML renderer lives in `nectar-html`. Per start document it either writes one page for the
whole document or one page per chapter (`split`, default `"chapter"`). With chapters on pages of
their own, the menu, table of contents and previous/next links come from the include tree, and
internal links resolve across pages. The menu is a tree of the chapters; each branch opens and
closes without scripts, and the branches along the current page are open when it loads. A small
script remembers the branches opened in a browser tab (`sessionStorage`); without it the menu
works the same and just forgets. A
document on a single page shows its headings of levels 2 and 3 in the tree instead.

The output is laid out as follows:

```
public/
  index.html                 site index, when every document has a directory of its own
  <document>/index.html      root chapter
  <document>/<slug>/index.html
  <document>/<asset path>    local images and files, relative to the start document
  _graphics/<hash>.svg       graphics, named by their content
  _theme/                    the theme's static files
```

URLs between output files are relative, so the output also works when opened from disk.

### Print renderer

`PrintRenderer`, also in `nectar-html`, writes every document as one file at
`<document>/print/index.html`. It uses the same block writer as the HTML renderer, so content
looks alike, and adds a cover, the contents and running headers and footers through CSS paged
media (`layouts/print.html`, `static/print.css`). Footnotes stand where they are referenced and
float to the foot of the page. Paged.js, bundled with the theme, lays the file out into pages in
the browser; the same file is the preview on screen and the source for printing or saving as
PDF. A small handler limits how long Paged.js waits for an image to report that it has loaded,
since a page that never reports would stop the layout. The web pages link to the print view
when the HTML renderer runs with `print_view`.

Settings come from `[document.print]` (`enabled`, `size`, `margin`, `cover`, `subtitle`,
`cover_image`, `chapter_pages`, `watermark`), which the core passes on as a table without
reading it.

A build runs every renderer over the same documents. Renderers may write the same file, such as
a shared theme file, only with the same content; anything else is an error.

### PDF renderer

`PdfRenderer` in `nectar-pdf` typesets every document with Typst, used as a library, into
`<document>/print/<start document>.pdf`. It translates the document model into Typst source:
text always as Typst strings, so nothing in a document is read as Typst markup; headings with
their section numbers and labels for internal links; footnotes as Typst footnotes; highlighted
code with the colours of the code engine; graphics as SVG files. Typst compiles the source in a
world held in memory, with the bundled fonts (Ubuntu Sans, Victor Mono, Material Icons) and the
document's images. Errors name the line of generated source they point to. The PDF reports what it leaves out:
raw HTML and images that are no assets; links to local files point to the copied file, relative
to the PDF. The settings of
`[document.print]` apply as for the print view; `pdf = false` turns the PDF off. Typst's caches
keep rebuilds under `nectar serve` short.

### Themes

A theme is a directory:

```
themes/<name>/
  layouts/     minijinja templates (page, chapter, partials)
  static/      CSS, fonts, images; copied to the output as they are
```

A default theme is built into the binary. A project theme overrides it file by file.

## Configuration

```toml
# nectar.toml
title  = "HiveVM Manual"
theme  = "hive"
output = "public"

[[document]]
source = "content/manual.md"
split  = "chapter"              # "chapter" or "none"
numbered = true                 # section numbers
contents = true                 # contents of the whole document on the first page

[vars]
version = "1.2.0"

[engine.textmate]
theme = "github-light"             # on by default; `enabled = false` turns it off

[engine.process.dot]
command   = "dot"
args      = ["-Tsvg"]
languages = ["dot"]

[engine.process.plantuml]
command   = "plantuml"
args      = ["-tsvg", "-pipe"]
languages = { uml = "uml", ebnf = "ebnf", regex = "regex", jsongraph = "json" }
input     = "@start{lang}\n!pragma layout smetana\n{code}\n@end{lang}"
timeout   = 30                     # seconds

[serve]
port = 1313
bind = "127.0.0.1"
```

## Crates

| Crate | Content |
|---|---|
| `nectar-core` | project loading, sources, parser, merge, document model, passes, `CodeEngine` and `Renderer` ports, diagnostics |
| `nectar-html` | HTML renderer and the default theme |
| `nectar-textmate` | TextMate code engine |
| `nectar-railroad` | railroad code engine |
| `nectar-process` | code engine that runs local commands |
| `nectar-server` | HTTP server, file watcher, live reload |
| `nectar` | the CLI binary; composes core, renderer and engines for both `build` and `serve` |

`nectar-core` depends on no adapter, no renderer and no async runtime. Only the `nectar` binary
knows the complete set of adapters, so `build` and `serve` always use the same engines.

## Core interface

```rust
let mut project = Project::load(root)?;        // nectar.toml
project.add_engine(engine);                    // once per adapter, in order
let build = project.build(&[&web, &print]);   // files in memory, sources, diagnostics
build.write_to(&out_dir)?;                     // `nectar build` only
```

A build holds the output files as a map from path to bytes, every project file it read (Markdown
files, images, theme files, configuration) and the diagnostics. The core reads files only below
the project root; links and includes that leave it are reported and not followed.

A project keeps its engines and their result cache between builds, so building again after an
edit re-parses and re-renders in memory but runs no engine for unchanged code. A rebuild of a
site with dozens of diagrams takes milliseconds.

## Modes

**`nectar build`** loads the project, builds it and writes the output. It reports all diagnostics
and exits non-zero when any is an error. `--drafts` includes files marked as draft, `--output`
overrides the output directory and `-C` names the project root.

**`nectar serve`** builds once, with drafts unless `--no-drafts`, and keeps the build in memory.
It listens on `127.0.0.1:1313`; `[serve] port` and `bind` in `nectar.toml` or `--port` and
`--bind` change that.

A debounced file watcher reports changes below the project root. Only events that change what a
file holds count; reading files, as every build does, does not. Hidden files, editor backups and
the output directory of `nectar build` are ignored. The server rebuilds, swaps the build
atomically and announces its version to connected browsers over server-sent events
(`/_nectar/reload`); a page reloads when the version differs from the one it was loaded with. A
change to `nectar.toml` opens the project again, with its engines.

The server injects the reload script into HTML responses; the core does not know about live
reload. Diagnostics appear in a panel on every page: errors open, warnings folded. When the
project cannot be opened or a page is missing because of errors, the server answers with a page
that shows the diagnostics, and keeps watching. Requests are answered from the build in memory
only; a directory without its trailing slash is redirected, so relative links keep working.

## Diagnostics

A diagnostic has a severity (error, warning), a message and an origin (file and line).
Passes, engines and the renderer return diagnostics as data and never print them. The shells
decide how to show them: the CLI on the terminal, the server on the terminal and in the browser.

## Distribution

The workflow `.github/workflows/nectar.yml` checks formatting and Clippy warnings and runs the
tests and a build of `examples/demo` on Linux, macOS and Windows. A tag `nectar-v<version>`
starts `.github/workflows/nectar-release.yml`, which builds the CLI for Linux (x86-64, ARM),
macOS (x86-64, ARM) and Windows (x86-64) and publishes the archives with their SHA-256 checksums
as a GitHub release. The user guide is `doc/nectar/README.md`.

## Open points

- Symbols beyond the default theme: `@name@` is a ligature name of the theme's symbol font; the
  default theme ships Material Icons.
