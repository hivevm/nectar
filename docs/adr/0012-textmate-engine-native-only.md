# ADR-0012: The TextMate engine runs on native hosts only, and a WebAssembly host highlights through the code engine port

- **Status:** 🟢 accepted
- **Date:** 2026-10-26
- **Deciders:** Markus Brigl
- **Applies to:** `crates/textmate/`, the WebAssembly step of `scripts/check-build.sh`, and the code engines a WebAssembly shell composes

## Context

Q-1 asks the core and the renderers to run wherever Rust compiles, WebAssembly included, and G-8
asks a project given as files in memory to produce the same website as the command line with the
same code engines, once `nectar-core` and the renderers build for `wasm32-unknown-unknown`.
[ADR-0009](0009-ports-and-adapters.md) counts `nectar-textmate` among the crates that need nothing
of the host and keeps it free of every technology a host decides; it does not say that the crate
builds for WebAssembly, and its *Out of scope* leaves the API of the WebAssembly shell open.

`nectar-textmate` highlights through `giallo` 0.5.2, which depends on `onig` (as `onig-regset`)
without a feature to turn it off, and, through the `dump` feature that `Registry::builtin()` needs
for the bundled grammars and themes, on `zstd`. Both build C code: `cargo check --target
wasm32-unknown-unknown -p nectar-textmate` fails in the build scripts of `onig_sys` and
`zstd-sys`, a target with no C library. The test that verifies G-8 builds from files in memory
on the native host, where the TextMate engine is available.

Without a decision, how code is highlighted where the TextMate engine cannot run is settled
silently: by dropping `nectar-textmate` from the structural test's portable crates, or by pulling
a C tool chain into the WebAssembly build. G-8 asks for the same website only with the same code
engines, so a WebAssembly host that brings another engine, or none, does not break it; the
[strategy](../SPECIFICATION.md#strategy) names the TextMate highlighter among the engines a host
chooses.

## Decision

We will build `nectar-textmate` for native hosts only and leave highlighting in a WebAssembly host
to a code engine that host provides through the `CodeEngine` port, as it does for Graphviz and
PlantUML.

- `nectar-textmate` stays free of host technology under [ADR-0009](0009-ports-and-adapters.md)
  and stays among the crates the structural test reads; the WebAssembly step of
  `scripts/check-build.sh` checks the core, the renderers and the railroad engine.
- In a WebAssembly host without a highlighting engine, code blocks stay plain code, as G-5 has
  them for a missing engine.

**Out of scope:** which engine a WebAssembly host brings, such as Shiki in a browser over the same
shikijs grammars that `giallo` bundles; the API of the WebAssembly shell; a later
highlighter that builds for `wasm32-unknown-unknown`, which a superseding ADR takes up.

## Alternatives considered

- **Build `giallo`'s C code for WebAssembly** — a clang with the `wasm32` target and a C library
  shim for Oniguruma and zstd in the Dev Container and CI. Not chosen: a second tool chain for one
  engine, and whether `onig_sys` builds against such a shim was not verified.
- **Replace `giallo` with a highlighter in pure Rust** — such as `syntect` with its `fancy-regex`
  backend. Not chosen now: the bundled grammars are TextMate JSON from shikijs, which `syntect`
  reads only after conversion, and `fancy-regex` does not match Oniguruma in every grammar; it
  would also end the one licence exception of the specification. The strongest alternative should
  a WebAssembly host need the same colours as the command line.
- **Target `wasm32-wasip1` instead** — a C library exists there. Not chosen: the specification
  fixes `wasm32-unknown-unknown`, for the browser.

## Sources / Prior art

- `giallo` 0.5.2 manifest: `onig` is unconditional, `zstd` comes with the `dump` feature (read from
  the crate in the local registry).
- `cargo check --target wasm32-unknown-unknown -p nectar-textmate`, run on 2026-10-26: fails in the
  build scripts of `onig_sys` 69.9.3 and `zstd-sys` 2.1.2.

## Consequences

- Positive: no C tool chain and no new dependency; the core and the renderers stay checked for
  WebAssembly as they are; a host decides what it can offer, as the
  [strategy](../SPECIFICATION.md#strategy) says.
- Negative / trade-offs: a WebAssembly host highlights differently from the command line, or not
  at all, so a website built there matches the command line's only where both use the same code
  engines, as G-8 says.
- Follow-ups: the engine of the first WebAssembly host.

## Enforcement

The WebAssembly step of `scripts/check-build.sh` cites this ADR beside G-8: it checks the core,
the renderers and the railroad engine for `wasm32-unknown-unknown`, and fails when one of them
comes to depend on `nectar-textmate`. `crates/cli/tests/structure.rs`, citing
ADR-0009 and Q-1, keeps `nectar-textmate` free of host technology.
