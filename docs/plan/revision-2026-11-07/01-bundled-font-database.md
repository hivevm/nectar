# Ticket: One font database of the bundled fonts, for the PNG and the rough engine

## What to build

The fonts `nectar-pdf` bundles become a `usvg` font database in two places that each decide the
fallback for a family the fonts lack, and decide it differently: `png` in
`crates/cli/src/render.rs` names `Ubuntu Sans` for `serif` and `Victor Mono` for `monospace`,
while `Rough::new` in `crates/rough/src/lib.rs` takes the family of whichever face comes first in
`nectar_pdf::FONT_FILES`, so reordering that list silently changes the rough engine's text. The
rough engine's tests reach `nectar-pdf` as a dev-dependency only for those fonts. ADR-0017 and
ADR-0020 decide one thing here, a database of the bundled fonts and no system fonts; the code
should state it once, with one fallback rule, so a sketch's text and a PNG's text fall back to the
same family.

Where the one rule lives is the session's to propose to the human: the shell building the
database once and handing it to both, the rough engine taking the fallback family from its
caller, or the font list and its families named together where `FONT_FILES` is. ADR-0020 has the
shell hand the fonts in, and ADR-0009 keeps the rough engine free of the host's fonts.

## Acceptance criteria

- [ ] Given an SVG whose text names a family the bundled fonts lack, when `nectar render rough`
      draws it and when `nectar render` rasterizes it to PNG, then both fall back to the same
      bundled family, held by a test.
- [ ] Given `FONT_FILES` in another order, when the rough engine draws that SVG, then its bytes do
      not change.
- [ ] Given the tests that cite ADR-0017 and ADR-0020, when the change lands, then they pass
      unchanged.

## Read first

- Touches: `crates/cli/src/render.rs`, `crates/cli/src/main.rs`, `crates/rough/src/lib.rs`,
  `crates/rough/Cargo.toml`, `crates/pdf/src/world.rs`
- ADR-0017 and ADR-0020, *Decision* and *Enforcement*; ADR-0009 for where the fonts may come from.
- Conventions: *A port is as narrow as its one caller.*

## Blocked by

- None (can start immediately)

## Mode

with the human: ADR-0007 lets no agent work run unattended, and where the rule lives is a choice
the human weighs against the wording of ADR-0020.
