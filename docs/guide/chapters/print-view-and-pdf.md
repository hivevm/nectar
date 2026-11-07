# Print view and PDF

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
website is dark. They share their settings per document:

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
