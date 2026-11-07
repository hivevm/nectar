# ADR-draft-roughr-usvg: The rough engine sketches with `roughr` and reads SVG with `usvg`

- **Status:** 🟡 proposed
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the dependencies of `crates/rough/Cargo.toml`, the `PORTABLE` list of `crates/cli/tests/structure.rs`, the WebAssembly step of `scripts/check-build.sh`, and the fonts the rough engine draws text with

## Context

[ADR-draft-rough-engine](draft-rough-engine.md) adds an engine that redraws an SVG in Rough.js's
sketch style. It has to read an SVG with its transforms, styles and shapes, and to compute
Rough.js's strokes and fills.

Forces:

- Simplicity ([`AGENTS.md` §1](../../AGENTS.md#1-principles)): Rough.js's geometry (curve
  fitting, seven fill patterns, path normalization) is several thousand lines; a maintained port
  is cheaper than one of our own.
- [ADR-0009](0009-ports-and-adapters-technology.md): an engine that needs nothing of the host
  stays free of technology. Neither crate below reads files unless asked to: `usvg` resolves an
  `<image href>` through a resolver its options carry, and loads fonts only into the database it
  is given.
- Determinism (Q-3) and confinement (Q-5): text must be drawn with the fonts Nectar bundles, not
  the host's ([ADR-0017](0017-png-via-resvg.md)).
- G-8: the core, the renderers and the portable engines compile for `wasm32-unknown-unknown`.
  `roughr` depends on `rand` 0.8 with its default features, which pull `getrandom` 0.2 for the
  operating system's entropy. `wasm32-unknown-unknown` has no operating system, so `getrandom`
  refuses to compile there unless a backend is chosen: `js` (through `wasm-bindgen`, for a
  JavaScript host) or `custom` (a function the build registers).
- `roughr` asks `getrandom` only for options without a seed; with a seed it uses `StdRng`
  seeded from it.

## Decision

We will build the rough engine on `roughr` 0.14 for the sketch geometry and on `usvg` in the
version `Cargo.lock` already carries for reading the SVG, always seeding `roughr`, with a font
database that holds only the fonts `nectar-pdf` bundles, handed in by the shell, and for
`wasm32` with `getrandom`'s `custom` backend registered to a function that fails.

- `usvg` turns every shape into a path with its absolute transform and resolved paint; the engine
  hands each path to `roughr`'s `Generator::path` with options from the `data-rough-*`
  attributes, always with a seed, and writes the result with `Generator::to_paths` as SVG path
  data.
- Text: `usvg` flattens it to outlines with the given font database. `<image>` elements are
  dropped, with the image resolver set to resolve nothing, so no file is read.
- `nectar-rough` joins the `PORTABLE` crates of the structural test and the WebAssembly step of
  `scripts/check-build.sh`.

**Out of scope:** adapters of `roughr` such as `rough_tiny_skia` or `rough_piet`, which the engine
does not need because it writes SVG and PNG comes from ADR-0017; raster images inside the SVG.

## Alternatives considered

- **A port of Rough.js of our own, in Rust.** Not chosen: the code of `roughr` again, maintained
  by us.
- **Migrate hivevm/rough.** Not chosen by the writer: unfinished, in Java, and no further than
  `roughr`.
- **`getrandom`'s `js` backend.** Not chosen: it adds `wasm-bindgen` and binds the WebAssembly
  build to a JavaScript host, for entropy the engine never asks for.
- **Native hosts only, as [ADR-0012](0012-textmate-engine-native-only.md) decided for TextMate.**
  Not chosen: TextMate needs a native regex library; here only an unused entropy source is in the
  way, and G-8 asks for the same engines in WebAssembly.
- **Parse the SVG with `svgtypes` or `roxmltree` alone.** Not chosen: transforms, CSS, `<use>`,
  inherited paint and the basic shapes would be ours to resolve; `usvg` already does it and is in
  the build.
- **Keep text as `<text>`.** Not chosen: the PNG and every viewer would need the fonts the text
  names; outlines from the bundled fonts look the same everywhere, as ADR-0017 wants for the PNG.
- **Draw text with the host's fonts.** Not chosen: Q-3 and Q-5, as in ADR-0017.

## Sources / Prior art

- `roughr` on crates.io and <https://github.com/orhanbalci/rough-rs>, checked 2026-11-07: 0.14.0
  of 2026-09-24, last commit 2026-11-04, every issue closed, about 48 000 downloads, MIT. Its
  `FillStyle` has `Solid`, `Hachure`, `ZigZag`, `CrossHatch`, `Dots`, `Dashed`, `ZigZagLine`;
  `Options` carries every option of Rough.js but `fillShapeRoughnessGain`, and `seed`, which
  defaults to 345. Dependencies: `points_on_curve`, `svg_path_ops`, `euclid`, `rand` 0.8,
  `num-traits`, `derive_builder`, `svgtypes` 0.16, `palette`. One maintainer.
- A prototype in a scratch directory, this session: `roughr` 0.14.0 with `usvg` 0.47.0 passes
  `cargo check` natively. For `wasm32-unknown-unknown` it fails in `getrandom` 0.2 ("the
  wasm*-unknown-unknown targets are not supported by default, you may need to enable the "js"
  feature"), and passes with `getrandom` 0.2's `custom` feature and
  `register_custom_getrandom!`, with no `wasm-bindgen` in the tree.
- `usvg` 0.47.0 in the local Cargo registry: `Text::flattened` gives the outlines, `writer.rs`
  writes a tree back as SVG.
- `Cargo.lock`: `usvg`, `svgtypes` 0.16, `euclid` 0.22, `palette` 0.7 and `rand` 0.8 are already in
  the build through `typst` and `resvg`.

## Consequences

- Positive: Rough.js's full set of fill styles and options with three new crates in the build
  (`roughr`, `points_on_curve`, `svg_path_ops`) and `derive_builder`; no licence beyond MIT.
- Positive: the engine is portable; a WebAssembly host can compose it like railroad.
- Negative / trade-offs: `roughr` is 0.x with one maintainer; an upgrade may change its API or its
  output, and the tests that pin bytes will show it. The version of `usvg` follows Typst's, as
  `resvg`'s does in ADR-0017.
- Negative / trade-offs: the `custom` backend is registered in a library crate; a second crate in
  the same WebAssembly build that registers one would clash at link time. A call without a seed
  would fail in WebAssembly, which the always-seeded generator rules out.
- Negative / trade-offs: `nectar-pdf` has to hand its bundled fonts to a second consumer.
- Follow-ups: `getrandom` 0.3 changes how a custom backend is chosen, when `roughr` moves to
  `rand` 0.9; whether `fillShapeRoughnessGain` is missed.

## Enforcement

`crates/cli/tests/structure.rs` holds `nectar-rough` to ADR-0009 through the `PORTABLE` list, and
the WebAssembly step of `scripts/check-build.sh` compiles it for `wasm32-unknown-unknown`. A test
in `crates/rough/` cites this ADR (`Verifies: ADR-draft-roughr-usvg`): an SVG with text and an
`<image href>` to a file renders with the text as outlines and without the image, and the same
text gives the same bytes. **Not mechanically decidable:** that the font database holds only the
bundled fonts; the review checks how the shell builds it.
