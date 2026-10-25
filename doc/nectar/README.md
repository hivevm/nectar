# Nectar

Nectar turns Markdown into a website. `nectar build` writes the site to a directory;
`nectar serve` serves it on your machine and reloads the browser whenever a source file
changes.

## Install

Download the archive for your platform from the
[releases](https://github.com/hivevm/doc/releases) (tags `nectar-v…`), unpack it and put
`nectar` on your `PATH`. Each release lists the SHA-256 checksums of its archives in
`SHA256SUMS`.

To build from source, install Rust and run:

```sh
cargo install --path crates/cli
```

## Quick start

A project is a directory with a `nectar.toml`:

```toml
title = "My Manual"

[[document]]
source = "content/manual.md"
```

```sh
nectar serve            # http://127.0.0.1:1313/, reloads on every change
nectar build            # writes the site to public/
```

Run either command in the project directory, or name the directory with `-C <dir>`.
`examples/demo` in this repository is a complete project to start from.

## Documents and chapters

Every `[[document]]` names a start document. A link that stands alone on its line includes
another Markdown file as a chapter; consecutive lines of such links include one file each:

```md
# Manual

[Installation](parts/install.md)

## [Appendix](parts/appendix.md)
```

- Headings of the included file move below the heading the include follows, or below the
  heading marker written in front of the link.
- The link text replaces the first top-level heading of the included file.
- Relative links and images in the included file resolve against its own directory.

With `split = "chapter"` (the default) every chapter becomes a page of its own, with breadcrumbs
and previous/next links. The menu shows the chapters as a tree whose branches open and close;
the branches along the current page are open. With `split = "none"` the whole document is one
page and the menu tree lists its headings.

### Front matter

A file may start with TOML between `+++` lines:

```md
+++
title = "Installation"    # page title
slug  = "install"         # path segment of the page
draft = true              # left out by `nectar build`, shown by `nectar serve`
[vars]
version = "1.2.0"
+++
```

## Markdown

Nectar reads CommonMark with these additions:

| Feature | Write |
|---|---|
| Tables | GFM tables; the number of `-` in a column's delimiter sets its relative width |
| Alerts | `!n`, `!s`, `!w`, `!e` at the start of a line; several paragraphs as `!!w` … `!!` |
| Highlight | `//text//` |
| Underline | `____text____` |
| Strikethrough | `~~text~~` |
| Task lists | `- [ ]`, `- [x]` |
| Footnotes | `text[^id]` and `[^id]: The footnote.` |
| Links as footnotes | `[text][id]` or `[][id]` with `[id]: https://… "Title"`: the link gets a footnote that shows the title and the URL |
| Image size | `![logo](logo.png){width=120 height=4cm align=center}` |
| Heading ids | `## Heading {#id}` |
| Internal links | `[text](#id)`, `[text](#Heading text)`, `[text](other.md#heading)` |
| Symbols | `@name@`, a [Material Icons](https://fonts.google.com/icons) name such as `@web@` |
| Variables | `{{name}}` or `{{name;format}}` (the format is ignored), from `[vars]` in `nectar.toml`, environment variables `NECTAR_<name>` or the front matter, each overriding the one before; unknown names stay as written and are reported unless they stand in code |
| Comments | `[comment]: # (not shown)` |

Headings are numbered (`1.`, `1.2.`, …) below the top level, and the first page of a document
lists the contents of the whole document. `numbered = false` and `contents = false` in the
`[[document]]` table turn either off.

## Code and diagrams

The info string of a fenced code block picks an engine:

- Every language with a TextMate grammar is highlighted, from `rust` and `java` to `yaml` and
  `sql`.
- `railroad` draws railroad diagrams from (E)BNF; `\` continues a rule on a new line.
- `dot`, `uml`, `ebnf`, `regex` and `jsongraph` are drawn by Graphviz and PlantUML, once they
  are configured (see below).

An image that stands alone in its paragraph and points to a file that is not a picture embeds
that file instead, with the image text as caption:

```md
![The entry point](../src/Main.java)

![Expression grammar](grammar.railroad)
```

The file extension picks the engine: `.java` is highlighted as Java, `.rs` as Rust,
`.railroad` is drawn, `.dot` and `.puml` go to Graphviz and PlantUML.

### Engines

```toml
[engine.textmate]
theme = "github-light"            # any of the ~60 bundled themes

[engine.railroad]
enabled = true                    # on by default

[engine.process.dot]
command   = "dot"
args      = ["-Tsvg"]
languages = ["dot"]

[engine.process.plantuml]
command   = "plantuml"
args      = ["-tsvg", "-pipe"]
languages = { uml = "uml", ebnf = "ebnf", regex = "regex", jsongraph = "json" }
input     = "@start{lang}\n!pragma layout smetana\n{code}\n@end{lang}"
timeout   = 30
```

A process engine passes the code on standard input and expects SVG on standard output. When a
command is missing or fails, the block stays plain code and Nectar reports a warning.

## Configuration

| Key | Meaning |
|---|---|
| `title` | Site title |
| `output` | Output directory of `nectar build`, default `public` |
| `theme` | Theme directory below `themes/`; the built-in theme otherwise |
| `[[document]] source` | Start document |
| `[[document]] path` | Output directory of the document; the root for a single document |
| `[[document]] split` | `"chapter"` or `"none"` |
| `[[document]] numbered` | Section numbers, default `true` |
| `[[document]] contents` | Contents of the whole document on its first page, default `true` |
| `[vars]` | Values for `{{name}}` |
| `[engine.…]` | Code engines, see above |
| `[serve] port`, `bind` | Address of `nectar serve`, default `127.0.0.1:1313` |
| `[document.print]` | Print view, see below |

## Print view and PDF

Besides the website, every document gets two outputs for print, both next to each other in
`<document>/print/`:

- the print view `index.html`: the whole document in one file, laid out into pages in the
  browser; the print icon at the top of every web page opens it;
- the PDF file `<start document>.pdf`, typeset with [Typst](https://typst.app); the PDF icon
  opens it.

The print view It shows the pages on screen as they will print; printing it, or saving it as
PDF from the browser's print dialog, gives exactly these pages.

Both have a cover, the contents with page numbers, running headers with the document and
chapter title, page numbers and footnotes at the foot of the page. They share their settings
per document:

```toml
[document.print]
enabled       = true          # false: no print view for this document
pdf           = true          # false: no PDF for this document
size          = "A4"          # A3–A6, B4, B5, letter, legal, "A5 landscape" or two lengths
margin        = "20mm 18mm"   # one to four lengths in mm, cm, in or pt, as in CSS
cover         = true          # cover page with site and document title
subtitle      = "Version 1.2" # on the cover
cover_image   = "img/logo.png"
chapter_pages = false         # each chapter starts with a page showing its number
watermark     = "draft"       # "draft", "confidential" or an image of the project
```

The pages are laid out by [Paged.js](https://pagedjs.org), which the built-in theme bundles.

## Themes

A theme is a directory `themes/<name>/` with `layouts/page.html`, a
[minijinja](https://docs.rs/minijinja) template, and `static/`, whose files are copied to
`_theme/` in the output. A project theme overrides the built-in theme file by file.

## Diagnostics

Problems are reported with file and line. `nectar build` fails on errors and lists warnings;
`nectar serve` also shows them in a panel on every page and keeps watching.

## Editor

`.vscode/launch.json` starts `nectar serve` or `nectar build` under the debugger and opens the
site in the browser.
