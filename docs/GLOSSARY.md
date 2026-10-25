# Glossary — Nectar

> The **vocabulary** of this project: the terms everyone, humans and agents alike, uses in code,
> comments, documentation, and ADRs. It defines words and decides nothing. Whoever resolves a term
> adds it here in the same change ([ADR-0001](adr/0001-agent-governance-model.md)).

One term per concept, only terms specific to this domain. Say what a term *is* in one or two
sentences, and list under _Avoid:_ the synonyms it displaces, where there are any. A term the
specification, an ADR, or the code uses without an entry here is a gap to close, not a convention
to invent.

- **Adapter** — an implementation of a port for one host, such as the process adapter that runs
  Graphviz.
- **Chapter** — a Markdown file included by a link that stands alone on its line. _Avoid:_ include
  file, part.
- **Code engine** — the port through which a fenced code block becomes highlighted code or a
  diagram. _Avoid:_ highlighter, diagram plugin.
- **Diagnostic** — a problem reported as data, with severity, message, file and line. _Avoid:_ log
  message.
- **Dialect** — CommonMark with Nectar's extensions, the one source format. _Avoid:_ flavour,
  syntax.
- **Document model** — Nectar's own tree of a document, independent of Markdown, that the passes
  enrich and the renderers read. _Avoid:_ AST.
- **Fetcher** — the port through which the core receives images from the web; the host's adapter
  makes the request. _Avoid:_ downloader, HTTP client.
- **Host** — what a shell runs on, which decides the adapters available: a machine, a browser,
  another program.
- **Pass** — one step that enriches the document model, such as section numbers, footnotes or code
  engines. _Avoid:_ filter, plugin.
- **Print view** — the whole document as one HTML page, laid out into pages in the browser.
  _Avoid:_ print page.
- **Project** — a directory with a `nectar.toml`; its root bounds everything Nectar reads.
- **Renderer** — the port that turns the enriched document model into output files: the website,
  the print view, the PDF.
- **Shell** — a thin program around the core: the command line, the server, a WebAssembly build.
  _Avoid:_ frontend, wrapper.
- **Start document** — the Markdown file a `[[document]]` names; it and its chapters make one
  document. _Avoid:_ main file, index.
  _Avoid:_ <the synonyms not used here>.
