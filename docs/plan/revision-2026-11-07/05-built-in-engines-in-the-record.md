# Ticket: The record names the built-in engines once

## What to build

Three accepted ADRs together decide which code engines run without a `nectar.toml`, and two of
them list a set the system has outgrown:

- ADR-0015: a single file builds with "the built-in ones, railroad and TextMate, on with their
  defaults";
- ADR-0018: `nectar render` without `-C` has "only the built-in engines with their defaults:
  `railroad` and TextMate highlighting";
- ADR-0019 adds `rough` as an engine with "the `enabled` every built-in engine has", so the
  command line composes railroad, rough and TextMate in every command
  (`engines` in `crates/cli/src/main.rs`), and `nectar render rough` works without `-C`, as
  ADR-0019's *Enforcement* and G-12 require.

Each ADR is true read with the later one, but a reader of ADR-0015 or ADR-0018 alone takes the
rough engine in a single-file build or in `render` for a violation. The set said in one sentence
is: *the built-in engines, railroad, rough and TextMate, run in every command with their defaults,
each unless `[engine.<name>] enabled = false`; without a `nectar.toml` they are the only ones.*

The human decides how the record says it once:

1. a `proposed` ADR that consolidates the sentence above and supersedes the engine bullets of
   ADR-0015 and ADR-0018, which means superseding both whole;
2. leaving it to the next normalization, which restates the ADRs in force and can name the set in
   one place without changing a decision;
3. leaving it as it is, the later ADR read as amending the earlier.

The revision recommends 2: superseding two ADRs for one list is ceremony out of proportion. The
ADR on settings under `[engine.rough]`, which touches the same composition, is now accepted on
the same branch as this ticket, so nothing else waits on this choice.

## Acceptance criteria

- [ ] Given the human's choice, when this ticket closes, then either a `proposed` ADR names the
      built-in engines once, or the choice and its reason are recorded where the next
      normalization reads them (its hand-off or a ticket of its own).

## Read first

- Touches: `docs/adr/0015-single-file-build.md`, `docs/adr/0018-render-command.md`,
  `docs/adr/0019-rough-engine.md`, `crates/cli/src/main.rs` (`engines`)
- The skill `normalize`, for option 2; the skill `propose-adr`, for option 1.

## Blocked by

- None (can start immediately)

## Mode

with the human: the ADR record is kept with the human (ADR-0007).
