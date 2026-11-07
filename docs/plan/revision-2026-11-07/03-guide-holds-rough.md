# Ticket: The guide's feature test requires a rough sketch

## What to build

ADR-0014 has the guide use a diagram of every engine itself, and
`the_guide_uses_every_feature_it_explains` in `crates/cli/tests/build.rs` holds a railroad, a
Graphviz and a PlantUML diagram, but no `rough` block. The guide has one today
(`docs/guide/chapters/code-and-diagrams.md`), so nothing is broken, but deleting it would pass:
the rough engine (ADR-0019) arrived without its line in the test's list of features. Add it.

## Acceptance criteria

- [ ] Given the guide without any `rough` block, when the test runs, then it fails and names the
      missing rough sketch.
- [ ] Given the guide as it is, when the test runs, then it passes.

## Read first

- Touches: `crates/cli/tests/build.rs`
- ADR-0014, *Decision* and *Enforcement*.

## Blocked by

- None (can start immediately)

## Mode

with the human: ADR-0007 lets no agent work run unattended.
