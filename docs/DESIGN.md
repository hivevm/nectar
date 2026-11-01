# Nectar design

Nectar turns Markdown sources into HTML. It runs in two modes that share one core:

- `nectar build` writes a complete static site to an output directory.
- `nectar serve` serves the site over local HTTP, watches the sources and reloads the browser as
  soon as a source changes.

## Constraints

1. A start document includes further Markdown files through links. The result is one complete
   document.
2. The source format is CommonMark with the extensions of the
   [guide](GUIDE.md#extended-syntax); see [Markdown dialect](#markdown-dialect).
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

Which links include a file, and what happens to the included file, is described in the
[guide](GUIDE.md#documents-and-chapters).

Every Markdown file is parsed on its own. Merging happens on the document model: an include
becomes a chapter node that carries the included file's document, with its heading levels shifted
below the level at which it is included. Every node records its origin (file and line), so
diagnostics point at the source file and not at a merged text.

### Front matter

The fields are described in the [guide](GUIDE.md#front-matter). The parser splits the TOML
block off before parsing and blanks its lines, so line numbers in diagnostics stay those of the
file; the chapter node carries the front matter.

## Markdown dialect

The syntax is described once, in the [guide](GUIDE.md#common-mark). The parser is
[comrak](https://docs.rs/comrak) with its extensions for strikethrough, tables, autolinks, task
lists, footnotes, GitHub alerts, heading ids and link attributes. Before it runs, variables are
replaced in the source text and the line-start alerts (`!w`, `!!w` … `!!`) are rewritten into
block quotes, one line for one line; the conversion into
the document model recognises highlight, underline, symbols, image attributes and the column
widths of tables. Raw HTML becomes text in the first [transform pass](#transform-passes) unless
`html = true`.

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
download pass fetches images with `http(s)` URLs through the `Fetcher` port and decodes `data:`
URIs. The core makes no request itself: the command line plugs in `nectar-fetch`, which allows 30
seconds, 20 MB and five redirects, and a project without a fetcher keeps the URLs. An image must
arrive with an image media type. It is
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

With `numbered` the numbering pass numbers the headings of all chapters in reading order, as the
[guide](GUIDE.md#headings) describes, and stores the number in the heading node. Renderers show
it in front of the heading and of chapter titles. With `contents` the renderers list the
headings of the whole document on its first page, linked across pages.

## Transform passes

A pass is a function over the whole document:

```rust
pub trait Pass {
    fn apply(&self, doc: &mut Document, ctx: &Context) -> Vec<Diagnostic>;
}
```

The core runs its passes in a fixed order, once per build, independent of the output format. The
first turns raw HTML into text unless `html = true`, so the renderers only see raw HTML the project
allowed.

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
the block's origin. Distinct blocks are rendered once each, side by side when the host's
`Workers` adapter allows it; without one they are rendered in turn. Results are cached by engine,
language and a hash of the code for the lifetime of the project, so `serve` does not render an
unchanged diagram twice.

The core defines the port and knows no engine. Engines are adapters in crates of their own:

| Adapter | Languages | Works through |
|---|---|---|
| `nectar-textmate` | every language with a bundled TextMate grammar | TextMate grammars and themes in-process |
| `nectar-railroad` | `railroad` | (E)BNF parser and SVG layout in-process |
| `nectar-process` | as configured, such as `dot`, `uml`, `ebnf`, `regex`, `jsongraph` | a local command (`dot`, `plantuml`) |

The railroad engine stacks the drawings of a block's rules into one SVG. Text boxes are sized
with Verdana metrics, and texts are stretched to their box, so the drawing does not depend on the
fonts of the viewer.

`nectar.toml` selects the adapters and configures them; the order in which they are asked, and
when a process engine fails, are described in the [guide](GUIDE.md#engines). A process engine
that runs longer than its `timeout` is stopped.

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

The website follows the system's colour scheme: every colour of the default theme is a pair for
the light and the dark scheme (`light-dark()`), and highlighted code carries the colours of both
TextMate themes (`theme`, `dark_theme`, by default `github-light` and `github-dark`) as custom
properties, from which the stylesheet picks. A switch in the top bar steps through system, light
and dark and keeps the choice for the site (`localStorage`); it shows only with scripts. The
print view and the PDF stay light.

The HTML renderer lives in `nectar-html`. Per start document it either writes one page for the
whole document or one page per chapter (`split`, default `"chapter"`). With chapters on pages of
their own, the menu, table of contents and previous/next links come from the include tree, and
internal links resolve across pages. The menu is a tree of the chapters; each branch opens and
closes without scripts, and the branches along the current page are open when it loads. A small
script remembers the branches opened in a browser tab (`sessionStorage`); without it the menu
works the same and just forgets. A
document on a single page shows its headings of levels 2 and 3 in the tree instead.

The renderers write the layout of files described in the [guide](GUIDE.md#output), with
relative URLs between them.

### Print renderer

`PrintRenderer`, also in `nectar-html`, writes every document as one file at
`<document>/print/index.html`. It uses the same block writer as the HTML renderer, so content looks
alike, and adds a cover, the contents and running headers and footers through CSS paged media
(`layouts/print.html`, `static/print.css`). Footnotes stand where they are referenced and float to
the foot of the page. Paged.js, bundled with the theme, lays the file out into pages in the browser;
the same file is the preview on screen and the source for printing or saving as PDF. A thematic
break starts a new page, as in the PDF. A small handler limits how long Paged.js waits for an image
to report that it has loaded, since a page that never reports would stop the layout. The web pages
link to the print view when the HTML renderer runs with `print_view`.

Settings come from `[document.print]` (`enabled`, `size`, `margin`, `cover`, `subtitle`,
`cover_image`, `chapter_pages`, `watermark`), which the core passes on as a table without
reading it.

A build runs every renderer over the same documents. Renderers may write the same file, such as
a shared theme file, only with the same content; anything else is an error.

### PDF renderer

`PdfRenderer` in `nectar-pdf` typesets every document with Typst, used as a library, into
`<document>/print/<start document>.pdf`. It translates the document model into Typst source: text
always as Typst strings, so nothing in a document is read as Typst markup; headings with their
section numbers and labels for internal links; footnotes as Typst footnotes; highlighted code with
the colours of the code engine; graphics as SVG files. Typst compiles the source in a world held in
memory, with the bundled fonts (Ubuntu Sans, Victor Mono, Material Icons) and the document's images.
A thematic break starts a new page, as documents written for the former Gradle plugin expect.
Errors name the line of generated source they point to. The PDF reports what it leaves out: raw HTML
and images that are no assets; links to local files point to the copied file, relative to the PDF.
The settings of `[document.print]` apply as for the print view; `pdf = false` turns the PDF off.
Typst's caches keep rebuilds under `nectar serve` short.

### Themes

The [guide](GUIDE.md#themes) describes what a theme holds. The default theme lives in
`crates/html/theme/` and is built into the binary with `include_str!` and `include_bytes!`; a
project theme overrides it file by file.

## Configuration

Every key of `nectar.toml` is described in the [guide](GUIDE.md#configuration). The core reads
the file into its configuration when a project is opened; `[engine.…]` goes to the command line,
which builds the adapters from it, and `[document.print]` is passed on to the renderers as a
table.

## Crates

The crates, what each holds and how their dependencies point are described once, in the
[building blocks of the architecture overview](ARCHITECTURE.md#building-blocks).

## Core interface

```rust
let files = Box::new(Directory::new(root));  // or any other adapter of `Files`
let mut project = Project::load(files)?;       // nectar.toml
project.set_workers(Box::new(Threads));        // without: engines run in turn
project.set_environment(std::env::vars());     // NECTAR_<name> overrides a variable
project.add_engine(engine);                    // once per adapter, in order
let build = project.build(&[&web, &print]);   // files in memory, sources, diagnostics
```

`nectar build` writes the files of the build to the output directory; the core writes nothing.

A build holds the output files as a map from path to bytes, every project file it read (Markdown
files, images, theme files, configuration) and the diagnostics. The core reads files only through
`Files`, by paths relative to the project root: links and includes that leave it are reported and
not followed, and `Directory` refuses a path that a symbolic link leads outside.

A project keeps its engines and their result cache between builds, so building again after an
edit re-parses and re-renders in memory but runs no engine for unchanged code. A rebuild of a
site with dozens of diagrams takes milliseconds.

## Modes

The options of both commands are listed in the [guide](GUIDE.md#quick-start).

**`nectar build`** loads the project, builds it and writes the output. It reports all diagnostics
and exits non-zero when any is an error.

**`nectar serve`** builds once and keeps the build in memory.

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

## Open points

- Symbols beyond the default theme: `@name@` is a ligature name of the theme's symbol font; the
  default theme ships Material Icons.
