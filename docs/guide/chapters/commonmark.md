# Common Mark

[Markdown][markdown] is a plain text format for writing structured documents, based on the
conventions of email and Usenet. It does nothing fancy like changing the font size, colour or
type, just the essentials, written with keyboard symbols you already know. A Markdown document
should be readable as it is, as plain text, without looking marked up: its syntax is made of
punctuation chosen to look like what it means. Asterisks around a word look like *emphasis*,
lists look like lists, and block quotes look like quoted passages of an email.

There are many dialects of Markdown. Nectar reads [CommonMark][commonmark], a strongly defined,
highly compatible specification of Markdown; the [CommonMark help](https://commonmark.org/help/)
is a short tour of it. This chapter shows the basic syntax, the next one what Nectar adds. Each
example shows the source first, then how it renders.

[markdown]: https://en.wikipedia.org/wiki/Markdown "Markdown on Wikipedia"
[commonmark]: https://spec.commonmark.org/ "CommonMark Spec"

## Headings

A line that starts with `#` and a space is a heading. The more `#`, the lower the level.

```md
# Manual title
## Chapter
### Section
```

Headings are numbered (`1.`, `1.2.`, …) below the top level; below a skipped level, as with
`####` directly under `##`, numbering continues in Roman numerals. The first page of a document
lists the headings of levels 2 and 3 of the whole document. `numbered = false` and
`contents = false` in the `[[document]]` table turn either off.

!w Write every Markdown file as a document of its own, starting with a top-level heading. When
another file includes it, it becomes a chapter, and its headings move below the heading of the
include ([Documents and chapters](extended-syntax.md#documents-and-chapters)).

## Paragraphs and line breaks

A paragraph is a run of consecutive lines; one or more blank lines separate paragraphs. A single
line break inside a paragraph is a soft break: the lines run on as one, so a source can be
wrapped for reading without changing the text.

```md
Lorem ipsum dolor sit amet, consetetur sadipscing elitr,
sed diam nonumy eirmod tempor.
```

Lorem ipsum dolor sit amet, consetetur sadipscing elitr,
sed diam nonumy eirmod tempor.

A backslash `\` or two spaces at the end of a line make a hard break: the text continues on a
new line.

```md
Lorem ipsum dolor sit amet, consetetur sadipscing elitr,\
sed diam nonumy eirmod tempor.
```

Lorem ipsum dolor sit amet, consetetur sadipscing elitr,\
sed diam nonumy eirmod tempor.

## Page breaks

Three or more `-`, `_` or `*` on a line of their own form a thematic break. On the website it is
a horizontal rule; in the print view and the PDF it starts a new page.

```md
End of the part.

___

Start of the next part.
```

This guide breaks the page here, so in print the next section starts on a page of its own:

___

## Blockquotes

A line that starts with `>` and an optional space is a block quote. Block quotes nest and may
hold any other formatting.

```md
> First line
> Another line
>
> > Nested line
>
> Last line
```

> First line
> Another line
>
> > Nested line
>
> Last line

## Emphasis

One `*` or `_` on each side makes *italic*, two make **bold**, three ***bold and italic***. A
backslash in front, `\*` or `\_`, writes the character itself. Nectar adds ____underline____
with four underscores, ~~strikethrough~~ with two tildes and //highlight// with two slashes.

```md
*italic* _italic_
**bold** __bold__
***bold and italic***
____underline____
~~strikethrough~~
//highlight//
```

## Lists

An unordered list uses `-`, `*` or `+` as marker; an ordered list uses numbers followed by `.`
or `)`. Lists nest when an item is indented below the text of its parent, and ordered and
unordered lists mix.

```md
1. Fruit
   - Apple
   - Pear
2. Vegetable
   - Potato
   - Carrot
```

1. Fruit
   - Apple
   - Pear
2. Vegetable
   - Potato
   - Carrot

An item that starts with `[ ]` or `[x]` is a task.

```md
- [x] Release candidate
- [ ] Product release
```

- [x] Release candidate
- [ ] Product release

## Links

A link is its text in square brackets followed by the target in parentheses. A bare URL such as
https://commonmark.org becomes a link by itself.

```md
[CommonMark](https://commonmark.org)
```

A reference link names its target by an identifier, defined anywhere in the same file. Without
a text, `[][id]` shows only the footnote of the link.

```md
See [the specification][spec] or [][spec].

[spec]: https://spec.commonmark.org/ "CommonMark Spec"
```

See [the specification][spec] or [][spec].

[spec]: https://spec.commonmark.org/ "CommonMark Spec"

A reference link to an external target gets a footnote that shows the title and the URL, so the
target survives on paper; links to the same URL in one file share it. Links within the
documentation are described in [Internal links](extended-syntax.md#internal-links).

## Images

An image is written like a link with a `!` in front; the text in brackets describes the image.

```md
![](img/logo.png)
![The logo](img/logo.png)
![The logo](img/logo.png){width=4cm align=center}
```

Attributes in braces right after the image set its size and alignment. `width` and `height` take
a bare number in pixels or a length such as `50%` or `4cm`; `align` takes `left`, `center` or
`right`, and `left` and `right` let the text flow around the image.

```md
![The Nectar logo](../img/logo.png){width=96 align=center}
```

![The Nectar logo](../img/logo.png){width=96 align=center}

Images are part of the documentation wherever they come from. Local images are copied into
the output; images from the web (`https://…`) and `data:` URIs are downloaded or decoded during
the build, stored in `<document>/_remote/` and served from there, and the PDF embeds them all.
A download takes at most 30 seconds, 20 MB and five redirects, must be an image, and happens
once per `nectar serve` session. An image that cannot be fetched keeps its link and is reported.
`--offline` keeps images from the web as links.

## Code

Backticks around text make `inline code`. A code block is indented by four spaces, or fenced
by three backticks or three tildes on a line above and below it. The word after the opening
fence picks the language ([Code and diagrams](code-and-diagrams.md)).

````md
```ini
[owner]
name = John Doe
```
````

```ini
; last modified 1 April 2001 by John Doe
[owner]
name = John Doe
organization = Acme Widgets Inc.
```
