# ADR-0013: A core without technology, the host's environment and the tests' dependencies included, every host-dependent effect behind a port it owns

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every crate under `crates/`, the dependencies and dev-dependencies in their `Cargo.toml`, and the structural test that holds the split
- **Supersedes:** [ADR-0009](0009-ports-and-adapters.md)

## Context

Q-1 asks the core and the renderers to run wherever Rust compiles, WebAssembly included, and the
[strategy](../SPECIFICATION.md#strategy) gives the core a port for every effect that depends on
the host, with the code engines not every host can offer behind a port too, chosen per host.
`nectar-core` owns the ports `Files`, `Workers`, `CodeEngine`, `Renderer` and `Fetcher`; the
adapters `nectar-native` (a directory on disk, threads), `nectar-fetch` (images over HTTP) and
`nectar-process` (programs named in `nectar.toml`) fill them on a machine, and the shells
`nectar` and `nectar-server` compose them. A WebAssembly host is to fill the same ports with its
own adapters (G-8).

[ADR-0009](0009-ports-and-adapters.md) decided this shape, and its definition of technology —
`std::fs`, `std::process`, `std::thread`, `std::net`, an async runtime, an HTTP client or server
and a file watcher, "the standard library otherwise" not — is narrower than what
`crates/cli/tests/structure.rs` holds today:

- **The host's environment.** `Project::documents` read `std::env::vars()` itself until the
  command line handed the variables over through `Project::set_environment` (0aec603); since then
  the structural test counts `std::env` as technology, as the maintainer read
  [ADR-0009](0009-ports-and-adapters.md). A variable of the host that reaches a build unseen breaks
  Q-3, the same sources giving the same output, and `wasm32-unknown-unknown` has no environment of
  its own (G-8). [ADR-0009](0009-ports-and-adapters.md) leaves `std::env` under "the standard
  library otherwise", so a session that follows its text would loosen the test.
- **The tests' dependencies.** The structural test reads the `[dev-dependencies]` of the portable
  crates as well, so their tests run against fakes behind the ports, such as the in-memory `Files`
  adapter `MemoryFiles`, and pull in no crate that creates files, such as `tempfile`. Their test
  sources are not read: `crates/core/tests/environment.rs` sets a variable with
  `std::env::set_var` to show that the core ignores it.
- **Technology crates.** The test names its crates in one list (`axum`, `hyper`, `notify`,
  `notify-debouncer-full`, `reqwest`, `tempfile`, `tokio`, `ureq`); the bans of `deny.toml` name
  fewer, the async runtime, server, watcher and HTTP client crates the dependency graph would
  otherwise let in.

The catalogue's *Ports and adapters* fits when the core is expected to outlive its technologies
and the same behaviour is filled by different technologies per host; that is Nectar's case. Its
*Does not fit* warns of a small CLI where the indirection costs more than the seam, which does not
hold once WebAssembly is a goal (G-8), and of a team that will not keep the split, which the
structural test answers.

## Decision

We will keep `nectar-core`, the renderers (`nectar-html`, `nectar-pdf`) and the engines that need
nothing of the host (`nectar-textmate`, `nectar-railroad`) free of every technology a host decides,
in their code and in the dependencies of their tests, put each effect such as reading the
project's files, running work on threads, reading the host's environment, fetching images from the
web, running programs, serving HTTP and watching files behind a port that `nectar-core` owns, and
implement those ports in adapter crates and shells (`nectar-native`, `nectar-fetch`,
`nectar-process`, `nectar-server`, `nectar`, and later a WebAssembly shell), every dependency
pointing toward the core.

- **Technology** means `std::env`, `std::fs`, `std::process`, `std::thread` and `std::net`, and a
  crate of one of these kinds: an async runtime, an HTTP client or server, a file watcher, and a
  crate that creates or reads files. The standard library otherwise and pure libraries such as
  `comrak`, `serde`, `toml`, `regex`, `minijinja`, `typst` and `giallo` are not. The structural
  test names the crates of each kind it refuses.
- **The host's environment** reaches the core only as the shell hands it over.
- **Tests:** the `[dev-dependencies]` of these crates are held to the same rule, so their tests
  run against fakes behind the ports. Their test code may use the standard library, as a test
  that shows the core ignoring the host's environment has to.
- **No crate depends on an adapter crate** except the shell that composes them.

**Out of scope:** the shape inside a crate, such as how the passes compose; the API of the
WebAssembly shell; which adapters a WebAssembly host offers for Graphviz and PlantUML; which
technology crates the bans of `deny.toml` repeat.

## Alternatives considered

- **ADR-0009 as it stands** — `std::env` as plain standard library, the tests unconstrained. Not
  chosen: the structural test already holds more, so the record and the check would disagree,
  and a variable of the host could change a build without the shell knowing (Q-3).
- **Test sources read as well** — `tests/` and the code from `#[cfg(test)]` on held to the same
  rule. Not chosen: a test that proves the core ignores the host's environment needs the
  environment, so the rule would need a list of exceptions to keep.
- **Every technology crate named in the ADR** — the list of the structural test written into the
  decision. Not chosen: every new crate of a known kind would need a supersession; the kinds are
  the decision, the list is how the test holds it.
- **The bans of `deny.toml` aligned with the test's list** — a second, complete guard. Not chosen:
  the structural test already decides every portable crate's manifest; the bans stay a guard for
  the crates the dependency graph would pull in through a dependency.
- **Functional core, imperative shell** — decisions in pure functions, effects in the shell. Not
  chosen: Nectar swaps technologies per host behind the same behaviour, which that seam does not
  name and ports do.
- **No prescribed shape** — as the template ships. Not chosen: Q-1 would rest on review and on
  cargo-deny alone; before ADR-0009 the core read the project's files itself and fetched images
  with `ureq`, which shows how it erodes.
- **Clean or onion architecture** — the same dependency rule with more prescribed rings; heavier
  vocabulary for the same seam.

## Sources / Prior art

- Alistair Cockburn, *Hexagonal Architecture* —
  <https://alistair.cockburn.us/hexagonal-architecture/>: the core, the ports it owns, and the
  adapters that plug into them.
- Robert C. Martin, *The Clean Architecture* —
  <https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html>: source code
  dependencies point inward only.
- `crates/cli/tests/structure.rs` and `crates/core/tests/environment.rs` at `e40db81`: the rule as
  the code holds it.

## Consequences

- Positive: the record says what the structural test holds, so neither is loosened by mistake;
  Q-1 and G-8 are reachable by construction, and no variable of the host reaches a build except
  through the shell (Q-3); the core is tested without disk, network or programs; a host chooses
  its adapters, as the strategy says.
- Negative / trade-offs: every new effect on the host costs a port in the core and an adapter at
  the edge before it can be used; a test of a portable crate that needs a real directory belongs
  to an adapter or the shell; the bans of `deny.toml` stay shorter than the test's list.
- Follow-ups: the WebAssembly shell of G-8 and the adapters it composes; the structural test's
  marker moves to this ADR once it is accepted.

## Enforcement

`crates/cli/tests/structure.rs`, which cites this ADR and Q-1 with a `Verifies:` marker once this
ADR is accepted, reads the manifests, dev-dependencies included, and the sources of the crates
named above. It fails on a dependency on an adapter crate or a technology crate, and on a use of
`std::env`, `std::fs`, `std::net`, `std::process` or `std::thread` outside their tests. The bans of
`deny.toml` ([ADR-0008](0008-toolchain-cargo-clippy-deny.md)) refuse the technology crates they
list for the same crates. That a port is no wider than its callers need is review.
