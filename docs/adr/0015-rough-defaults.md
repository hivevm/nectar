# ADR-0015: `[engine.rough]` in `nectar.toml` sets the defaults of the rough engine

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the keys of `[engine.rough]` in `nectar.toml`, how the command line hands them to the rough engine in `crates/cli/` and `crates/rough/`, and the tests that verify them

## Context

[ADR-0013](0013-rough-engine.md) styles each shape of a `rough` block by `data-rough-*`
attributes in the SVG, and leaves settings for the whole engine under `[engine.rough]` out of
scope beyond `enabled`. So a sketch takes Rough.js's defaults unless its SVG carries attributes.
That fails where the writer does not write the SVG: `nectar render railroad g.txt | nectar
render rough` passes railroad's SVG, which has no `data-rough-*` attributes, and so does any SVG
another tool exports. The writer asked whether the engine takes parameters; today only the SVG
can give them.

Forces:

- [ADR-0017](0017-render-command.md) keeps engine options off the command line and configures
  engines through `[engine]` of the `nectar.toml` that `-C` names, as for a build. A setting in
  `nectar.toml` reaches `render` with `-C`, `build` and `serve` alike.
- The TextMate engine already takes settings there (`theme`, `dark_theme`); the keys of
  `nectar.toml` are snake case.
- The attributes of ADR-0013 are the interface a shape is styled by; defaults must not take that
  away. The nearest setting should win, as it does between an SVG's elements.
- Determinism (Q-3): a default seed in the configuration keeps the same text and configuration
  giving the same graphic.
- G-12 states the style rule, the defaults of `[engine.rough]` in the project's `nectar.toml`
  among its sources.

## Decision

We will read every option of ADR-0013's attributes, in snake case, from `[engine.rough]` of the
project's `nectar.toml` as the rough engine's defaults, below every `data-rough-*` attribute in
the SVG and above Rough.js's defaults, and refuse a key or a value the engine does not know when
the configuration is loaded.

- **Keys:** `fill_style`, `roughness`, `bowing`, `hachure_angle`, `hachure_gap`, `fill_weight`,
  `dash_offset`, `dash_gap`, `zigzag_offset`, `disable_multi_stroke`, `preserve_vertices` and
  `seed`, beside `enabled`. Their values are those of the attributes, as TOML strings, numbers
  and booleans.
- **Precedence:** an attribute on a shape, an ancestor or the `<use>` that draws it, as
  ADR-0013 has it, over the key, over Rough.js's default.
- **Errors:** an unknown key or an unreadable value is an error of the configuration, reported
  as an unknown `[engine.*]` table is today, before anything is built or rendered.

**Out of scope:** options on the command line; defaults per document or per code block, such as
an info string `rough fill_style=dots`; settings that Rough.js does not have.

## Alternatives considered

- **Options on the command line (`nectar render rough --roughness 2`).** Not chosen: ADR-0017
  keeps engine options off the command line, a build could not use them, and superseding it for
  one engine would make `render` the only place an engine is configured differently.
- **Options in the info string of a code block (```` ```rough roughness=2 ````).** Not chosen
  for now: it serves documents but not `nectar render`, and no other engine reads its info string
  beyond the language; a later ADR can add it.
- **Keys in kebab case, the same as the attributes.** Not chosen: every other key of
  `nectar.toml` is snake case; the attribute names stay kebab case, as HTML's `data-*` are.
- **A wrapper the writer adds to the SVG (`<g data-rough-…>` around the content).** Not chosen:
  that is the workaround today, and it does not work in a pipe without a further tool.

## Sources / Prior art

- ADR-0017 and ADR-0013 in this repository; `crates/cli/src/main.rs` `engines()`, which reads
  `[engine.textmate] theme` and `dark_theme` and refuses an unknown `[engine.*]` table.
- Rough.js options — <https://github.com/rough-stuff/rough/wiki>: the defaults the keys override.

## Consequences

- Positive: a project sketches every graphic, a piped railroad diagram included, in its own
  style without touching the SVG; the same keys work in a build and in `render -C`.
- Negative / trade-offs: a second place a style comes from, and twelve more keys of
  `nectar.toml` to keep stable as a public interface. Without `-C`, `nectar render rough` still
  takes Rough.js's defaults, as ADR-0017 has it.
- Negative / trade-offs: `nectar-rough` takes its defaults from the shell at construction, so
  its interface grows by a defaults value.
- Follow-ups: options per code block.

## Enforcement

Tests cite this ADR (`Verifies: ADR-0015`): in `crates/rough/`, defaults given
to the engine style a shape without attributes, and an attribute overrides them; in
`crates/cli/tests/render.rs`, `nectar -C <dir> render rough` with `[engine.rough] fill_style =
"solid"` draws a solid fill for an SVG without attributes, and an unknown key or an unreadable
value fails before anything is written.
