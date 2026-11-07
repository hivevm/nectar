# Ticket: G-11 names a failing engine (drafted specification change)

## What to build

G-12 says text that is no SVG "shall fail as an engine does under G-5 and G-11", but G-11 says
nothing of a failing engine: it names only an unsupported language and a disallowed format.
ADR-0018 decides the failing engine ("an engine that fails is an error … and nothing is
written"), and `an_engine_that_fails_writes_nothing` in `crates/cli/tests/render.rs` tests it,
citing ADR-0018 alone because G-11 does not ask for it. So the reference in G-12 points at a
clause that does not exist.

Drafted text for G-11 in `docs/SPECIFICATION.md`, the change in the last line:

> **G-11** — When `nectar render` is given a language and a text, the system shall write the
> output of the code engine that supports the language, as the format `-f` or the extension of
> `-o` names, to the file `-o` names or to standard output, and shall exit non-zero and write
> nothing when no engine supports the language, the output does not allow the format, or the
> engine fails.

Accepted ADRs whose basis this may move, for the human to judge
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7):
ADR-0018, which already decides the behaviour and would gain the criterion behind it; ADR-0019,
whose *Errors* bullet cites G-5 and ADR-0018 for the same failure. The revision found no ADR the
change contradicts; that is a claim for the human to check, not a judgement.

Once the human lands the text, the test above cites `G-11` as well.

## Acceptance criteria

- [ ] Given the human's decision, when the change lands, then G-11 names the failing engine, or
      the ticket closes with the reason it does not, and G-12's reference reads true either way.
- [ ] Given G-11 changed, when the traceability check runs, then
      `an_engine_that_fails_writes_nothing` cites G-11 and ADR-0018.

## Read first

- Touches: `docs/SPECIFICATION.md`, `crates/cli/tests/render.rs`
- ADR-0018 and ADR-0019, *Decision*.

## Blocked by

- None (can start immediately)

## Mode

with the human: a change to the specification is the human's decision
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).
