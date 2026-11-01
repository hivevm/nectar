# Nectar

Nectar turns Markdown into a website. `nectar build` writes the site to a directory;
`nectar serve` serves it on your machine and reloads the browser whenever a source file
changes.

## Install

Download the archive for your platform from the
[releases](https://github.com/hivevm/nectar/releases) (tags `nectar-v…`), unpack it and put
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

| Option | Meaning |
|---|---|
| `-C <dir>` | Project directory, the one that holds `nectar.toml`; default the current one |
| `--offline` | Keep images from the web as links instead of downloading them |
| `build -o <dir>` | Output directory; overrides `output` in `nectar.toml` |
| `build --drafts` | Include files whose front matter says `draft = true` |
| `serve -p <port>`, `serve --bind <addr>` | Address to serve on; default `127.0.0.1:1313` |
| `serve --no-drafts` | Leave out drafts, which `serve` shows by default |

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
- Relative links, images and further includes in the included file resolve against its own
  directory.
- Footnote definitions belong to their file; footnotes are numbered across the whole document.

A link is an include only when its target ends in `.md` and the file exists. An include cycle is
an error.

With `split = "chapter"` (the default) every chapter becomes a page of its own, with breadcrumbs
and previous/next links. The menu shows the chapters as a tree whose branches open and close;
the branches along the current page are open, and the menu remembers the branches you opened
while the browser tab stays open. With `split = "none"` the whole document is one
page and the menu tree lists its headings.

### Front matter

A file may start with TOML between `+++` lines. All fields are optional; a file without front
matter behaves exactly as one with an empty block.

```md
+++
title = "Installation"    # page title, instead of the first heading
slug  = "install"         # path segment of the page when the document is split
draft = true              # left out by `nectar build`, shown by `nectar serve`
[vars]
version = "1.2.0"
+++
```

## Markdown

Nectar reads [CommonMark](https://spec.commonmark.org/), whose specification describes the
syntax this guide builds on, with these additions:

| Feature | Write |
|---|---|
| Tables | GFM tables; the number of `-` in a column's delimiter sets its relative width; on the website no column gets narrower than its longest word |
| Alerts | `!n`, `!s`, `!w`, `!e` at the start of a line; several paragraphs as `!!w` … `!!`; also GitHub's `> [!NOTE]` |
| Highlight | `//text//` |
| Underline | `____text____` |
| Strikethrough | `~~text~~` |
| Task lists | `- [ ]`, `- [x]` |
| Footnotes | `text[^id]` and `[^id]: The footnote.` |
| Links as footnotes | `[text][id]` or `[][id]` with `[id]: https://… "Title"`, defined in the same file: a link to an external target gets a footnote that shows the title and the URL |
| Autolinks | bare URLs such as `https://example.org` |
| Image size | `{width=120 height=4cm align=center}` right after an image; a bare number is in pixels, otherwise a length such as `50%` or `4cm` |
| Heading ids | `## Heading {#id}` |
| Internal links | a link target `#id`, `#heading-slug`, `#Heading text` or `other.md#heading` |
| Symbols | `@name@`, a [Material Icons](https://fonts.google.com/icons) name such as `@web@` |
| Variables | `{{name}}` or `{{name;format}}` (the format is ignored), from `[vars]` in `nectar.toml`, environment variables `NECTAR_<name>` or the front matter, each overriding the one before; unknown names stay as written and are reported unless they stand in code |
| Comments | `[comment]: # (not shown)` |
| Code | fenced with ` ``` ` or `~~~`; the info string picks the [engine](#code-and-diagrams) |
| Page break | `---`, `___` or `***` on a line of its own: a new page in the print view and the PDF, a rule on the website |

Raw HTML, such as `<file>` written in a sentence, is shown as text and reported, so nothing in
a source disappears into the page as markup. `html = true` in `nectar.toml` passes it to the
website instead; the PDF still leaves it out and reports it.

Headings are numbered (`1.`, `1.2.`, …) below the top level; below a skipped level, as with
`####` directly under `##`, numbering continues in Roman numerals. The first page of a document
lists the headings of levels 2 and 3 of the whole document. `numbered = false` and
`contents = false` in the `[[document]]` table turn either off.

### Differs from CommonMark

Some plain CommonMark reads differently in Nectar:

- A link that stands alone on its line and points to a `.md` file includes that file as a
  chapter ([Documents and chapters](#documents-and-chapters)).
- An image that stands alone in its paragraph and points to a file that is not a picture embeds
  that file as code ([Code and diagrams](#code-and-diagrams)).
- A thematic break (`---`, `___`, `***`) starts a new page in the print view and the PDF.
- Raw HTML is shown as text unless `html = true`.
- A reference link to an external target gets a footnote with its title and URL.
- `____text____` underlines instead of nesting strong emphasis.
- A line that starts with `!n`, `!s`, `!w`, `!e` or `!!` opens an alert instead of a paragraph.
- `//text//`, `@name@` and `{{name}}` are read as highlight, symbol and variable.
- A link target `#Heading text` with spaces is a link to that heading.

### Tables

```md
| Key   | Price                     |
|:------|--------------------------:|
| `a`   | 12.50                     |
```

- The number of `-` in each cell of the delimiter row sets the column's relative width, here
  6 : 26. The PDF keeps the ratio exactly; on the website it is a preference, no column gets
  narrower than its longest word, and a table too wide for the page scrolls sideways. When the
  delimiter row has fewer or more cells than the table has columns, all columns are equal.
- `:---` aligns a column left, `:---:` centres it, `---:` aligns it right.
- In the print view the header row repeats on every page the table runs across.
- A cell holds inline content: emphasis, code, links, images, footnotes, symbols, variables. It
  holds no paragraphs, lists or code blocks, and cells cannot span rows or columns.
- A table may stand in a block quote or an alert.

### Footnotes

A footnote is written as `text[^id]` with a definition `[^id]: The footnote.` anywhere in the
same file; a reference link to an external target gets one as well, and links to the same URL in
one file share it. Labels belong to their file, so two chapters may both use `[^1]`; the numbers
run through the whole document in reading order.

The website lists the footnotes of a page at its end, the reference linking to them. The print
view and the PDF put each footnote at the foot of the page it is referenced on.

## Images

Images are part of the documentation wherever they come from. Local images are copied into
the output; images from the web (`https://…`) and `data:` URIs are downloaded or decoded during
the build, stored in `<document>/_remote/` and served from there, and the PDF embeds them all.
A download takes at most 30 seconds, 20 MB and five redirects, must be an image, and happens
once per `nectar serve` session. An image that cannot be fetched keeps its link and is reported.
`--offline` keeps images from the web as links.

## Code and diagrams

The info string of a fenced code block picks an engine:

- Every language with a TextMate grammar is highlighted, from `rust` and `java` to `yaml` and
  `sql`.
- `railroad` draws railroad diagrams from (E)BNF, every rule of a block under its name; `\`
  continues a rule on a new line, and a loop is labelled with the number of passes, such as `3×`
  for `3 * a`, `1..3×` or `≥2×`; "once or more" has no label.
- `dot`, `uml`, `ebnf`, `regex` and `jsongraph` are drawn by Graphviz and PlantUML, once they
  are configured (see below).

An image that stands alone in its paragraph and points to a file that is not a picture embeds
that file instead, with the image text as caption:

```md
![The entry point](../src/Main.java)

![Expression grammar](grammar.railroad)
```

The file extension picks the engine: `.java` is highlighted as Java, `.rs` as Rust,
`.railroad` is drawn, `.dot` and `.puml` go to Graphviz and PlantUML; any other extension is
taken as the language. Without an image text the file is embedded without a caption. An
embedded file is a source of the build, so `nectar serve` rebuilds when it changes.

### Engines

```toml
[engine.textmate]
enabled    = true                 # on by default
theme      = "github-light"       # any of the ~60 bundled themes
dark_theme = "github-dark"        # for a dark colour scheme; "" turns it off

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

Process engines are asked first, then railroad, then TextMate; a language no engine takes stays
plain code. A process engine passes the code, or the code filled into `input`, on standard input
and expects SVG on standard output. When a command is missing, exits non-zero, writes no `<svg`
element or runs longer than `timeout` seconds, the block stays plain code and Nectar reports a
warning.

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
| `html` | `true` passes raw HTML to the website; default `false`, which shows it as text |
| `[engine.…]` | Code engines, see above |
| `[serve] port`, `bind` | Address of `nectar serve`, default `127.0.0.1:1313` |
| `[document.print]` | Print view, see below |

## Output

`nectar build` writes, and `nectar serve` serves:

```
public/
  index.html                 site index, when every document has a directory of its own
  <document>/index.html      root chapter
  <document>/<slug>/index.html
  <document>/<asset path>    local images and files, relative to the start document
  <document>/_remote/        images from the web
  <document>/print/          print view and PDF
  _graphics/<hash>.svg       diagrams, named by their content
  _theme/                    the theme's static files
```

Links between the files are relative, so the output also works when opened from disk.

## Print view and PDF

Besides the website, every document gets two outputs for print, both next to each other in
`<document>/print/`:

- the print view `index.html`: the whole document in one file, laid out into pages in the
  browser; the print icon at the top of every web page opens it;
- the PDF file `<start document>.pdf`, typeset with [Typst](https://typst.app); the PDF icon
  opens it.

The print view shows the pages on screen as they will print; printing it, or saving it as
PDF from the browser's print dialog, gives exactly these pages.

Both have a cover, the contents with page numbers, running headers with the document and
chapter title, page numbers and footnotes at the foot of the page. Both stay light when the
website is dark. They share their settings
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

## Colour scheme

The website follows the system's light or dark colour scheme, highlighted code included
(`theme` and `dark_theme` of `[engine.textmate]`). A switch in the top bar steps through system,
light and dark and keeps the choice for the site; it shows only where scripts run.

## Themes

A theme is a directory `themes/<name>/` with the [minijinja](https://docs.rs/minijinja)
templates `layouts/page.html` for the website and `layouts/print.html` for the print view, and
`static/`, whose files are copied to `_theme/` in the output. A project theme overrides the
built-in theme file by file.

## Diagnostics

Problems are reported with file and line. `nectar build` fails on errors and lists warnings;
`nectar serve` also shows them in a panel on every page and keeps watching. Raw HTML shown as text
is reported, and the PDF reports what it leaves out: raw HTML where `html = true`, and images it
could not embed.

## Editor

`.vscode/launch.json` starts `nectar serve` or `nectar build` under the debugger and opens the
site in the browser.
