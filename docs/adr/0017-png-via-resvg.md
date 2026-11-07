# ADR-0017: A graphic becomes a PNG through resvg, with the bundled fonts only

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the `png` format of `nectar render` in `crates/cli/`, the dependency on `resvg` in `crates/cli/Cargo.toml`, and the fonts it renders text with

## Context

[ADR-0018](0018-render-command.md) lets `nectar render` write a graphic as `svg`
or `png`. Every code engine returns its graphic as SVG, so a PNG is SVG rasterized. Nothing in
Nectar rasterizes today.

Forces:

- Simplicity ([`AGENTS.md` §1](../../AGENTS.md#1-principles)): a dependency only for a present
  need. `resvg` 0.47 with `tiny-skia` and its PNG encoder is already in `Cargo.lock`, pulled in by
  `typst-pdf` through `krilla-svg` for the PDF. A direct dependency on it adds no crate to the
  build and no licence to `deny.toml`.
- Determinism (Q-3): the same text should give the same PNG, as Q-3 measures it, on the same
  machine. `resvg` renders SVG text with the fonts its font database holds; with system fonts the
  picture would also depend on the fonts a host has installed.
- Confinement (Q-5): reading the host's font directories touches files the writer did not point
  Nectar at.
- The `nectar-pdf` crate bundles fonts (Ubuntu Sans, Victor Mono) in memory for the PDF.
- Railroad diagrams size their boxes from Verdana's advance widths and stretch the text to fit,
  so any font fits their boxes.

## Decision

We will rasterize a graphic to PNG with the `resvg` crate in the version `Cargo.lock` already
carries, with a font database that holds only the fonts `nectar-pdf` bundles and no system fonts,
at the SVG's own size and a scale of 1.

**Out of scope:** a scale or DPI option; PNG anywhere but `nectar render`, such as images in the
website; rasterizing in the WebAssembly build; other raster formats.

## Alternatives considered

- **SVG only, no PNG.** Not chosen: the writer asked for PNG for places that take no SVG; the cost
  is one direct dependency on a crate already built.
- **An external program, such as `rsvg-convert` or Inkscape.** Not chosen: a program the host may
  lack, run as a process; the in-tree crate needs neither.
- **`resvg` with system fonts.** Not chosen: the PNG would differ between hosts (Q-3) and Nectar
  would read the host's font directories (Q-5).
- **Fonts of its own for the PNG.** Not chosen: a second set to ship and license when the PDF's
  set already covers text and code.

## Sources / Prior art

- `resvg` 0.47 and `tiny-skia` 0.12 as vendored in the local Cargo registry: `resvg::render`
  draws a `usvg::Tree` into a `tiny_skia::Pixmap`, and `Pixmap::encode_png` (feature
  `png-format`, on by default) writes the PNG. `usvg::Options` carries the font database.
- `cargo tree -i resvg`: `resvg` 0.47.0 reaches `nectar` through `nectar-pdf`, `typst-pdf` and
  `krilla-svg`.

## Consequences

- Positive: PNG without a new crate, host program or font set; the same text gives the same bytes
  on the same machine, and no host's installed fonts change the picture.
- Negative / trade-offs: whether hosts of other CPU architectures give the same pixels is not
  verified; `tiny-skia` rasterizes in floating point, with SIMD paths that may differ.
- Negative / trade-offs: the version of `resvg` follows Typst's; an upgrade of Typst can move it,
  and a mismatch would build a second copy. Text in a font the bundled set lacks falls back to one
  of the bundled fonts.
- Negative / trade-offs: `nectar-pdf` has to make its bundled fonts available to the command line.
- Follow-ups: a scale option, if a PNG at the SVG's size proves too small.

## Enforcement

A test in `crates/cli/tests/render.rs` cites this ADR (`Verifies: ADR-0017`): it
renders the same railroad text to PNG twice and checks a PNG signature and identical bytes.
**Not mechanically decidable:** that no system font is loaded on every host; the test checks
determinism on the host it runs on, and the review checks that the font database is built from
the bundled fonts alone.
