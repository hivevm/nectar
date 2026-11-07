+++
slug = "syntax"
+++
# Extended syntax

Nectar builds on CommonMark with the features of this chapter. Some of them read plain CommonMark
differently; [Differs from CommonMark](#differs-from-commonmark) lists those.

## Documents and chapters

Documentation can be spread over many files. Every `[[document]]` in `nectar.toml` names a start
document. A link that stands alone on its line includes another Markdown file as a chapter;
consecutive lines of such links include one file each:

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

## Front matter

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

This chapter starts with `slug = "syntax"`, so its page is `syntax/` rather than
`extended-syntax/`.

## Comments

A link reference definition is never shown itself; one with the target `#` and the text in
parentheses serves as a comment.

```md
[comment]: # (This line is not shown.)
```

[comment]: # (This line is not shown.)

## Internal links

A link to a heading of the documentation takes its id as target, `#` in front:

- the id given in braces after the heading, `## Installation {#install}`;
- the slug of the heading, its text in lower case with spaces turned into `-`;
- the heading text itself, spaces included, which Nectar turns into the slug;
- `other.md#heading` for a heading of another file.

```md
[Tables](#tables), [Raw HTML](#raw-html), [Footnotes](#footnotes)
```

[Tables](#tables), [Raw HTML](#raw-html), [Footnotes](#footnotes)

The target `#Raw HTML`, the heading text, links to the same heading as `#raw-html`.
Headings without special characters keep their links simple.

## Tables

A table is a header row, a delimiter row of three or more `-` for each column, and the body rows,
the cells separated by `|`; the pipes at both ends are optional. A colon in the delimiter row
aligns the column: `:---` left, `:---:` centred, `---:` right.

```md
| Auto | Centre | Left  | Right       |
|------|:------:|:------|------------:|
| Auto | Centre | Left  | Right       |
```

| Auto | Centre | Left  | Right       |
|------|:------:|:------|------------:|
| Auto | Centre | Left  | Right       |

- The number of `-` in each cell of the delimiter row sets the column's relative width, here
  6 : 8 : 7 : 13. The PDF keeps the ratio exactly; on the website it is a preference, no column
  gets narrower than its longest word, and a table too wide for the page scrolls sideways. When
  the delimiter row has fewer or more cells than the table has columns, all columns are equal.
- In the print view the header row repeats on every page the table runs across.
- A cell holds inline content: emphasis, code, links, images, footnotes, symbols, variables. It
  holds no paragraphs, lists or code blocks, and cells cannot span rows or columns.
- A table may stand in a block quote or an alert.

## Alerts

An alert is a block quote coloured by its kind. A line that starts with `!` and
a letter for the kind opens one: `n` note, `s` success, `w` warning, `e` error.

```md
!n Note
```

!n Note

```md
!s Success
```

!s Success

```md
!w Warning
```

!w Warning

```md
!e Error
```

!e Error

An alert of several paragraphs starts with `!!` and the letter, and ends with a line of `!!`.

```md
!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr.

Sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat.
!!
```

!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr.

Sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat.
!!

GitHub's notation `> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]` and `> [!CAUTION]`
gives alerts as well.

## Footnotes

A footnote is written as `text[^id]` with a definition `[^id]: The footnote.` anywhere in the
same file.

```md
Nectar reads CommonMark.[^dialect]

[^dialect]: With the additions of this chapter.
```

Nectar reads CommonMark.[^dialect]

[^dialect]: With the additions of this chapter.

Labels belong to their file, so two chapters may both use `[^1]`; the numbers run through the
whole document in reading order. The website lists the footnotes of a page at its end, the
reference linking to them. The print view and the PDF put each footnote at the foot of the page
it is referenced on.

## Symbols

A name between two `@` is a symbol of the [Material Icons](https://fonts.google.com/icons) font.

```md
@web@ @smartphone@ @settings@ @link@
```

@web@ @smartphone@ @settings@ @link@

## Variables

`{{name}}` is replaced by the value of the variable `name`; this guide is built with
`version` set to {{version}}. Values come from `[vars]` in `nectar.toml`, environment variables
`NECTAR_<name>` or the front matter, each overriding the one before. `{{name;format}}` is
accepted, the format ignored. A known name is replaced in code as well. An unknown name stays as
written and is reported, unless it stands in code.

## Raw HTML

Raw HTML, such as `<file>` written in a sentence, is shown as text and reported, so nothing in
a source disappears into the page as markup. `html = true` in `nectar.toml` passes it to the
website instead; the PDF still leaves it out and reports it.

## Differs from CommonMark

Some plain CommonMark reads differently in Nectar:

- A link that stands alone on its line and points to a `.md` file includes that file as a
  chapter ([Documents and chapters](#documents-and-chapters)).
- An image that stands alone in its paragraph and points to a file that is not a picture embeds
  that file as code ([Embedded files](code-and-diagrams.md#embedded-files)).
- A thematic break (`---`, `___`, `***`) starts a new page in the print view and the PDF.
- Raw HTML is shown as text unless `html = true`.
- A reference link to an external target gets a footnote with its title and URL.
- `____text____` underlines instead of nesting strong emphasis.
- A line that starts with `!n`, `!s`, `!w`, `!e` or `!!` opens an alert instead of a paragraph.
- `//text//`, `@name@` and `{{name}}` are read as highlight, symbol and variable.
- A link target `#Heading text` with spaces is a link to that heading.
