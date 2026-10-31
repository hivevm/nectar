# ADR-0009: A core without technology, every host-dependent effect behind a port it owns, adapters and shells at the edge

- **Status:** 🟢 accepted
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** every crate under `crates/`, the dependencies in their `Cargo.toml`, and the structural test that holds the split

## Context

Q-1 asks the core and the renderers to run wherever Rust compiles, WebAssembly included, and the
[strategy](../SPECIFICATION.md#strategy) gives the core a port for every effect that depends on
the host, with the code engines not every host can offer behind a port too, chosen per host.
`nectar-core` owns the ports `Files`, `Workers`, `CodeEngine`, `Renderer` and `Fetcher`; the
adapters `nectar-native` (a directory on disk, threads), `nectar-fetch` (images over HTTP) and
`nectar-process` (programs named in `nectar.toml`) fill them on a machine, and the shells
`nectar` and `nectar-server` compose them. A WebAssembly host is to fill the same ports with its
own adapters (G-8).

The catalogue's *Ports and adapters* fits when the core is expected to outlive its technologies
and the same behaviour is filled by different technologies per host; that is Nectar's case. Its
*Does not fit* warns of a small CLI where the indirection costs more than the seam, which does not
hold once WebAssembly is a goal (G-8), and of a team that will not keep the split, which the
structural test answers.

## Decision

We will keep `nectar-core`, the renderers (`nectar-html`, `nectar-pdf`) and the engines that need
nothing of the host (`nectar-textmate`, `nectar-railroad`) free of every technology a host decides,
put each effect such as reading the project's files, running work on threads, fetching images from
the web, running programs, serving HTTP and watching files behind a port that `nectar-core` owns,
and implement those ports in adapter crates and shells (`nectar-native`, `nectar-fetch`,
`nectar-process`, `nectar-server`, `nectar`, and later a WebAssembly shell), every dependency
pointing toward the core.

- **Technology** means `std::fs`, `std::process`, `std::thread`, `std::net`, an async runtime, an
  HTTP client or server, and a file watcher; the standard library otherwise and pure libraries
  such as `comrak`, `serde`, `toml`, `regex`, `minijinja`, `typst` and `giallo` are not.
- **No crate depends on an adapter crate** except the shell that composes them.

**Out of scope:** the shape inside a crate, such as how the passes compose; the API of the
WebAssembly shell; which adapters a WebAssembly host offers for Graphviz and PlantUML.

## Alternatives considered

- **Functional core, imperative shell** — decisions in pure functions, effects in the shell. Not
  chosen: Nectar swaps technologies per host behind the same behaviour, which that seam does not
  name and ports do.
- **No prescribed shape** — as the template ships. Not chosen: Q-1 would rest on review and on
  cargo-deny alone; before this decision the core read the project's files itself and fetched
  images with `ureq`, which shows how it erodes.
- **Clean or onion architecture** — the same dependency rule with more prescribed rings; heavier
  vocabulary for the same seam.

## Sources / Prior art

- Alistair Cockburn, *Hexagonal Architecture* —
  <https://alistair.cockburn.us/hexagonal-architecture/>: the core, the ports it owns, and the
  adapters that plug into them.
- Robert C. Martin, *The Clean Architecture* —
  <https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html>: source code
  dependencies point inward only.

## Consequences

- Positive: Q-1 and G-8 are reachable by construction; the core is tested without disk,
  network or programs; a host chooses its adapters, as the strategy says.
- Negative / trade-offs: every new effect on the host costs a port in the core and an adapter at
  the edge before it can be used; the shell is the one place that knows every adapter.
- Follow-ups: the WebAssembly shell of G-8 and the adapters it composes.

## Enforcement

A structural test, `crates/cli/tests/structure.rs`, citing this ADR and Q-1 with
`Verifies: ADR-0009, Q-1`, reads the manifests and the sources of the crates named above, and fails
on a dependency on an adapter crate or a technology crate, and on a use of the standard library's
technology modules. The bans of `deny.toml` ([ADR-0008](0008-toolchain-cargo-clippy-deny.md))
refuse the technology crates they list for the same crates. That a port is no wider than its
callers need is review.
