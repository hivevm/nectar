# EPUB

A document can also become an EPUB 3 publication for an e-reader, written to
`<document>/epub/<start document>.epub`. It holds the same content as the website, and an icon at
the top of every web page opens it. Unlike the print view and the PDF, it is off until a document
asks for it:

```toml
[document.epub]
enabled     = true                     # false, the default: no EPUB for this document
language    = "en"                     # required: the language of the text, as a BCP 47 tag
identifier  = "urn:isbn:9780000000000" # the default comes from the site's title and the path
cover_image = "img/cover.png"          # optional: an image of the project, as a page of its own
```

Each top-level chapter is one file in the publication, and the contents list every heading. With
`contents = false`, reading starts at the first chapter, and the contents stay available in the
reader's menu.

A reader chooses its own fonts and colours, so the publication carries no fonts and a small style
sheet. Highlighted code keeps the colours of the light theme.

What an EPUB cannot carry is left out, and a warning says so with file and line:

- raw HTML, even with `html = true`;
- an image other than PNG, JPEG, GIF, WebP or SVG, one whose first bytes do not fit its extension,
  and an image from the web that was not downloaded, as with `--offline`: its alt text stays;
- an SVG that is not well-formed, holds a script, or refers to anything outside itself: its alt
  text stays, or its code for a diagram;
- a link that is no URL of `http`, `https` or `mailto` and no heading or chapter of the document,
  such as a link to a local file: its text stays;
- an id given twice to a heading: the later heading loses it;
- a character XML does not allow, such as the escape codes of text pasted from a terminal;
- a footnote referenced from inside a footnote;
- a symbol of the Material Icons font.

A build gives the same bytes from the same sources, so the publication's time of modification
is fixed to `1980-01-01T00:00:00Z`. Set the variable `SOURCE_DATE_EPOCH` to a number of seconds
since 1970 to give a real one, for example in a release:

```sh
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) nectar build
```

A value that is no such number stops the build of a project that enables an EPUB.
`nectar build notes.md` writes only a PDF.

Nectar judges an image you supply only by its first bytes, and an SVG by the rules above. What else
such a file holds is yours to get right, and [EPUBCheck](https://www.w3.org/publishing/epubcheck/)
can check the finished publication.
