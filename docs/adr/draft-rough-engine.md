# ADR-draft-rough-engine: A code engine `rough` redraws an SVG in a hand-drawn style

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the code engine for the language `rough` in `crates/rough/`, the input it accepts and its `data-rough-*` attributes, the shell that composes it in `crates/cli/`, and the tests that verify it

## Context

The writer wants to turn an SVG into another SVG, or a PNG, drawn as if sketched by hand, in the
style of Rough.js: wobbly double strokes and fills of hachure lines instead of solid colour. The
first use is `nectar render` ([ADR-0018](0018-render-command.md)): `nectar render rough in.svg -o
out.png`.

Forces:

- [ADR-0018](0018-render-command.md) runs one code engine on one text and converts the engine's
  `Graphic` to `svg` or `png` itself. A code engine whose text is an SVG gets both formats, and the
  port `CodeEngine` stays as it is.
- An engine is the same set in `render`, `build` and `serve` (ADR-0018). A `rough` engine is also
  a fenced code block ```` ```rough ```` in a document. G-5 names the languages that are drawn
  today; a criterion for `rough` is drafted with this ADR and lands as a specification change by
  the human ([`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 7).
- Rough.js takes its style per shape: `fillStyle`, `hachureAngle`, `hachureGap`, `fillWeight`,
  `roughness` and more. An SVG has no place for them but attributes; the colours it already has,
  `stroke`, `fill` and `stroke-width`.
- Determinism (Q-3): Rough.js draws with a random generator. The same text must give the same
  graphic.
- The writer considered [hivevm/rough](https://github.com/hivevm/rough), an unfinished Java port
  of Rough.js, and chose not to migrate it; its examples become the engine's tests.

## Decision

We will add a code engine in a crate of its own, `nectar-rough` under `crates/rough/`, that
supports the language `rough`, takes the text of its code block as one SVG document and returns
a `Graphic`: the same document with every shape (path, rectangle, circle, ellipse, line,
polyline, polygon) redrawn in Rough.js's sketch style in its own stroke and fill colours and stroke
width, faded by the opacity of the groups around it, its text drawn as plain outlines, and the
style of each shape taken from `data-rough-*` attributes on it or an ancestor.

- **Attributes.** One per option of Rough.js that applies to a shape, named in kebab case:
  `data-rough-fill-style` (`hachure`, `solid`, `zigzag`, `cross-hatch`, `dots`, `dashed`,
  `zigzag-line`), `data-rough-roughness`, `data-rough-bowing`, `data-rough-hachure-angle`,
  `data-rough-hachure-gap`, `data-rough-fill-weight`, `data-rough-dash-offset`,
  `data-rough-dash-gap`, `data-rough-zigzag-offset`, `data-rough-disable-multi-stroke`,
  `data-rough-preserve-vertices` and `data-rough-seed`. The nearest element that sets one wins;
  without any, Rough.js's defaults apply. A `data-rough-*` attribute the engine does not know, or
  a value it cannot read, is an `EngineError` that names the attribute.
- **`<use>`.** A shape drawn through a `<use>` takes the attributes on it and its ancestors
  within the referenced content, over those on the `<use>` and its ancestors, as SVG inherits
  properties into a `<use>`. A shape inside a marker, a shape expanded from an entity of the
  DTD, and a shape drawn through a `<use>` the engine cannot resolve, such as one that nests
  deeper than eight, points outside the document, is placed in units other than pixels, sizes a
  referenced `<svg>`, or lies beyond the size the resolution may grow to, take the defaults. A
  `<use>` that references itself, directly or through others, draws nothing, as SVG has it,
  unless it lies beyond those limits and is left to usvg.
- **Paint.** A gradient fills or strokes with its first stop; a pattern does not paint.
- **Seed.** Without `data-rough-seed`, a fixed seed, so the same text gives the same graphic.
- **Errors.** Text that is no SVG is an `EngineError`, as a failing engine is under G-5 and
  ADR-0018.

**Out of scope:** settings for the whole engine under `[engine.rough]` in `nectar.toml` beyond
the `enabled` every built-in engine has; a sketch
style for the graphics of other engines, such as railroad, without passing their SVG through
`rough`; gradients beyond their first stop, patterns, filters, masks and clip paths, which the
engine does not draw; styles for the shapes of markers; raster images inside the SVG; the
technology the engine draws and parses with, which a separate ADR decides.

## Alternatives considered

- **A switch `--sketch` on `nectar render` for every graphic, with a pass-through language
  `svg`.** Not chosen by the writer: it would sketch railroad and process output directly, but
  only on the command line, and a document could not ask for it.
- **Style options on the command line, such as `--fill-style zigzag`.** Not chosen: one style
  for the whole graphic, and ADR-0018 leaves engine options on the command line out of scope.
- **Style from `[engine.rough]` in `nectar.toml` only.** Not chosen: one style per project, while
  Rough.js styles shapes one by one, as the examples of hivevm/rough do; a later ADR may add it as
  the defaults the attributes override.
- **A language of its own for shapes, like Rough.js's JavaScript calls (`rectangle 10 10 200
  200`).** Not chosen: a new format to define and keep stable, when SVG already describes the
  shapes and every engine already emits it.
- **A random seed per call.** Not chosen: the output would change on every build (Q-3).
- **Migrate hivevm/rough.** Not chosen by the writer: an unfinished port whose SVG and canvas
  back ends are empty, in Java.

## Sources / Prior art

- Rough.js, API and options — <https://github.com/rough-stuff/rough/wiki>: the option names this
  ADR turns into attributes, and their defaults.
- hivevm/rough at `main`, read in this session: `fillers/Filler.java` maps the fill styles
  `hachure`, `solid`, `zigzag`, `cross-hatch`, `dots`, `dashed`, `zigzag-line`;
  `src/test/java/rough/RoughTest.java` holds the examples.
- `crates/core/src/engine.rs`: the port `CodeEngine` and its `Graphic` output.

## Consequences

- Positive: any SVG becomes a sketch as SVG or PNG through `nectar render`, and the same in a
  document, without a change to the port or to the command.
- Positive: the style lives in the SVG, next to the shape it styles, and survives a round trip
  through other tools that keep unknown attributes.
- Negative / trade-offs: a set of `data-rough-*` attributes to keep stable as a public input
  format. Text is no longer text in the output, so it cannot be selected or searched.
- Negative / trade-offs: a railroad diagram is sketched only by rendering it to SVG first and
  passing that through `rough`; two calls in a pipe.
- Follow-ups: a criterion for `rough` in G-5; defaults under `[engine.rough]`; a sketch style
  for the graphics of other engines.

## Enforcement

Tests in `crates/rough/` cite this ADR (`Verifies: ADR-draft-rough-engine`). They draw the
examples of hivevm/rough as SVG through the engine and check: every shape comes back as paths in
its own colours; each of the seven fill styles gives its pattern; an attribute on a group reaches
its shapes and one on a shape overrides it; the same text gives the same bytes; text that is no
SVG, an unknown attribute and an unreadable value are errors. A test in
`crates/cli/tests/render.rs` cites it too: `nectar render rough` writes SVG and PNG.
