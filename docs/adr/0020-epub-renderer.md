# ADR-0020: An `EpubRenderer` in `nectar-html` writes an EPUB 3.3 publication for every document that enables it

- **Status:** 🟢 accepted
- **Date:** 2026-11-08
- **Deciders:** Markus Brigl
- **Applies to:** `EpubRenderer` in `crates/html/`, the table `[document.epub]` of `nectar.toml`, the shell that composes the renderer in `crates/cli/`, `crates/wasm-link/`, and the tests and the check that verify G-13

## Context

The specification keeps EPUB out today. The maintainer has decided to make it a full third output,
and a specification change drafted with this ADR lands that decision. It adds G-13, drops EPUB from
the Non-Goals, and extends G-4, G-9 and Q-2 to the new output. This ADR rests on that change and
binds nothing until it has landed ([`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 7).

The forces, against what the code and EPUB 3.3 say:

- **The port fits.** `Renderer` takes the enriched documents and returns files as bytes, so the
  core learns nothing of the format beyond one table of settings.
- **The writer exists.** An EPUB holds XHTML, CSS and images. `PageWriter` in `nectar-html` writes
  blocks as HTML, is `pub(crate)`, and already serves `PrintRenderer`. It writes void elements
  unclosed, such as `<br>`, and EPUB 3.3 requires the XML syntax of HTML (section 6.1.2).
- **Language.** EPUB 3.3 requires `dc:language` (section 5.5.3.1). Nectar has no setting for it, and
  `print.html` hard-codes `en`.
- **Time.** EPUB 3.3 requires exactly one `dcterms:modified` in UTC (section 5.5.5). The clock of
  the machine would break Q-3, and ADR-0009 lets the host's environment reach a build only through
  the shell.
- **Raw HTML.** With `html = true`, a block of raw HTML is any text the writer wrote, usually not
  well-formed XML. One such block makes the whole content document invalid.
- **Cost of a rebuild.** `nectar build` and `nectar serve` share their renderers, so an EPUB that is
  on costs time in every rebuild (Q-4). Most projects publish no EPUB.

The catalogue of `propose-adr` has no entry for an output format; this decision works within
[ADR-0009](0009-ports-and-adapters-technology.md).

## Decision

We will write an EPUB 3.3 publication per enabling document with an `EpubRenderer` in `nectar-html`,
on the block writer of the website.

- **Enabling.** A writer enables the output with `enabled = true` under `[document.epub]`. The
  default is `false`. A document that does not enable it gives no EPUB.
- **Location.** The renderer must write `<document>/epub/<stem>.epub`, where `<stem>` is the stem of
  the start document. The website must link to it when the document enables the output.
- **Settings.** `[document.epub]` holds these keys besides `enabled`:
  - `language` is a BCP 47 tag. An enabled document without it must give an error diagnostic.
  - `identifier` is the `dc:identifier`. Without it, the renderer must derive one from the title of
    the site and the path of the document.
  - `cover_image` is an optional image, relative to the project root.
- **Content.**
  - The renderer must write each top-level chapter as one XHTML content document of well-formed XML.
  - The renderer must always write a navigation document of the chapters and their headings.
  - The renderer must not write the legacy NCX, scripts, or fonts.
- **Time.**
  - The shell must hand the renderer the value of `dcterms:modified` as it composes the renderer.
  - The shell must take that value from `SOURCE_DATE_EPOCH` when that variable is set.
  - Without the variable, the value must be `1980-01-01T00:00:00Z`.
  - The shell must hand over a malformed value as what is wrong with it.
  - A document that enables the output must then give an error diagnostic.
  - A project without such a document must build as if the variable were not set.
- **What the output cannot carry.** The renderer must report each of these as a warning with file
  and line, as G-4 and G-9 ask:
  - raw HTML, which the renderer must not pass into a content document;
  - an image whose media type is no core media type of EPUB 3.3, which keeps only its alt text;
  - an image from the web without a download, which keeps only its alt text;
  - an image whose first bytes do not fit its extension, which keeps only its alt text;
  - an SVG that is not well-formed, holds a script, or refers to anything outside itself, which
    keeps only its alt text, or its code for a graphic;
  - a link that is no URL of `http`, `https` or `mailto` and no heading or chapter of the document,
    which keeps only its text;
  - a repeated id of a heading, which the later heading loses;
  - a character that XML 1.0 does not allow, which the renderer drops;
  - a reference to a footnote from inside a footnote, which the core leaves without a number;
  - a symbol, which needs a font the output does not carry.
- **Dependency.** The renderer may read an SVG with `roxmltree`, a pure library the WebAssembly
  module already links.
- **Files of the project.** The renderer judges an image only by its first bytes, and an SVG only
  by the rules above. What else such a file holds is the project's, as G-13 says.
- **Containment.** Every image must lie inside the container. An image from the web must come from
  the downloads of the document.
- **Empty values.** An empty `identifier` must give an error diagnostic. An empty title must fall
  back to the stem of the start document.
- **Existing outputs.** The website and the print view of a document must stay byte-identical while
  no document of the project enables the output.
- **Hosts.** `nectar build`, `nectar serve` and `crates/wasm-link/` must compose the renderer.
  `nectar build <file>` must not compose it.

**Out of scope:**

- how the block writer comes to write well-formed XML;
- the style sheet of the output;
- how the container is packed, which a decision of its own takes;
- EPUB from `nectar build <file>`, which needs a change to G-10 and to
  [ADR-0016](0016-single-file-build.md).

## Alternatives considered

- **A crate `nectar-epub`, as `nectar-pdf` is one** — one crate per format. Not chosen:
  `nectar-html` would have to publish its writer and its anchors, and one renderer would depend on
  another for the first time. `PrintRenderer` already shows a second renderer in that crate.
- **On by default, as the print view and the PDF are** — equal rank by configuration. Not chosen:
  every existing project would gain a file and pay for it in each rebuild (Q-4), for an output most
  of them do not publish. The rank of the format comes from G-13 and Q-2, not from a default.
- **`dcterms:modified` from a required key in `nectar.toml`** — always a real date. Not chosen: a
  value to maintain by hand goes stale and is easily forgotten. CI already knows the convention
  of `SOURCE_DATE_EPOCH`.
- **A fixed `dcterms:modified` without an override** — the simplest form. Not chosen: a reading
  system could not tell two editions of a publication apart.
- **The clock of the machine** — the usual answer. Not chosen: it breaks Q-3, and
  `wasm32-unknown-unknown` has no clock of its own.
- **Raw HTML repaired into XML by an HTML parser** — keeps the block. Not chosen: it adds a
  dependency for a case the PDF already reports, and a repaired block may not mean what its writer
  wrote.
- **One content document for the whole document** — the shape of the print view. Not chosen:
  reading systems load and paginate per content document, so a book of a few hundred pages would
  open and turn slowly.
- **EPUB 2, or an NCX for its reading systems** — wider reach on old devices. Not chosen: EPUB 3.3
  makes the NCX optional and replaces it with the navigation document, and two tables of contents
  are a second thing to keep right.
- **Bundled fonts in the output** — the look of the website. Not chosen: reading systems let their
  readers choose fonts, and fonts make the file several times larger.
- **Conversion by Pandoc or Calibre** — no new renderer. Not chosen: it needs a program on the
  host, so it cannot run in WebAssembly (Q-1, G-8).
- **EPUB from `nectar build <file>` in the same change** — equal rank from the first day. Not
  chosen: it is a second decision, which changes G-10 and supersedes ADR-0016.

## Sources / Prior art

- W3C, *EPUB 3.3*, Recommendation of 13 January 2026 — <https://www.w3.org/TR/epub-33/>: sections
  5.5.3.1 and 5.5.5 for the required metadata, section 5.9 for the legacy NCX, section 6.1.2 for the
  XML syntax, and the core media types.
- W3C, *EPUBCheck* — <https://w3c.github.io/epubcheck/releases>: 5.3.0, of 2025-09-01, checks
  conformance to EPUB 3.3.
- Reproducible Builds, *SOURCE_DATE_EPOCH*:
  <https://reproducible-builds.org/specs/source-date-epoch/>
  gives seconds since the epoch in UTC, and a build that exits non-zero on a malformed value.
- `crates/core/src/render.rs`, `crates/html/src/print.rs` and `crates/html/src/write.rs`: the port,
  the precedent, and the writer as they stand.

## Consequences

- Positive: one writer serves all HTML outputs, so content means the same in them (Q-2). The
  renderer stays portable, since it is one more module of a crate that already runs in WebAssembly
  (Q-1, G-8). Projects without the table build byte-identically to today (Q-3). EPUBCheck gives
  an external verdict on conformance.
- Negative / trade-offs: `nectar-html` grows by a third renderer. The block writer carries a mode
  for well-formed XML beside its HTML. Without `SOURCE_DATE_EPOCH`, `dcterms:modified` is not a
  real date. A project that enables the output pays for it in each rebuild under `nectar serve`, and
  the user guide enabling it makes Q-4 measure that cost. The checks gain a Java program, as the
  environment already has for PlantUML; this is a dev-only tool, not the JVM tool chain the
  Non-Goals rule out.
- Follow-ups: a setting for the language of the website and the print view, which still hard-code
  `en`; EPUB from `nectar build <file>`; whether Q-4 still holds with the output on, measured by the
  first ticket.

## Enforcement

- `crates/html/tests/epub.rs` must cite this ADR and G-13 with a `Verifies:` marker. It builds
  projects from `MemoryFiles` and fails in these cases:
  - a document without the table gives an EPUB;
  - an enabled document without `language` gives no error;
  - raw HTML reaches a content document or stays unreported;
  - an image of another media type, an image from the web without a download, a link without a
    target, an SVG the renderer must refuse, a dropped character, or a symbol stays unreported;
  - the bytes of an image, a cover, or a graphic in the container differ from their source;
  - a content document is not well-formed XML;
  - `dcterms:modified` differs from the value the renderer was handed;
  - a value handed over as malformed leaves an enabled document without an error.
- `crates/cli/tests/build.rs` must fail when `SOURCE_DATE_EPOCH` does not set the value, when a
  malformed value exits zero for a project that enables the output, when it exits non-zero for a
  project that does not, and when `nectar build <file>` writes an EPUB.
- A new script `scripts/check-epub.sh` must build the user guide and a fixture and run EPUBCheck
  on their EPUBs. The fixture must hold every case a finding of EPUBCheck has shown so far.
  EPUBCheck is pinned by version and checksum, and any error fails the script.
  `scripts/check-all.sh` and `.github/workflows/checks.yml` must run it.
- The tests that measure Q-2 and Q-3 must cover the EPUB of the user guide.
- `crates/wasm-link/` must compose `EpubRenderer`, so that a link error fails the WebAssembly step
  of `scripts/check-build.sh`. `crates/cli/tests/structure.rs` already holds `nectar-html` to
  ADR-0009.
