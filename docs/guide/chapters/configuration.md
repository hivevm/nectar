# Configuration

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
| `[engine.…]` | Code engines, see [Engines](code-and-diagrams.md#engines) |
| `[serve] port`, `bind` | Address of `nectar serve`, default `127.0.0.1:1313` |
| `[document.print]` | Print view and PDF, see [Print view and PDF](print-view-and-pdf.md) |
| `[document.epub]` | The EPUB, see [EPUB](epub.md) |
