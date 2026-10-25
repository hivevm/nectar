# ADR-0007: One coding agent implements, the maintainer reviews every change, nothing runs unattended

- **Status:** 🟢 accepted
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** every change, every agent session, the specification, the ADR record, and the release workflows under `.github/workflows/`

## Context

Nectar has one maintainer, who works with one coding agent
([specification](../SPECIFICATION.md#constraints)); its work fits a session as a rule, and it has
no assets that a second reviewer of another kind would protect beyond what the maintainer reads.
[`AGENTS.md` §4](../../AGENTS.md#4-working-style) asks the harness ADR who decides, which agents
work, how much agent work may be open at once, and which parts are kept with the human; and
[§5](../../AGENTS.md#5-quality-bar--definition-of-done) excludes the implementer from reviewing
its own change, whatever the harness says.

The catalogue's *One agent, the human reviews every change* fits a one-person project that grows
the harness when the work does. Its *Does not fit* names unattended work without a cap, which this
decision avoids by allowing no unattended work at all.

## Decision

We will have one coding agent, Claude Code with Claude Opus 5.5, implement every change in a
session with the maintainer, Markus Brigl, who reviews every change before it lands, decides every
ADR and every change to the specification, and lets no agent work run unattended.

- **Open agent work:** one session at a time, no parallel sessions, no worktrees.
- **Kept with the human,** also once the harness grows to unattended tickets: the specification,
  the ADR record, and the release workflows with their checksums, which hold the integrity of the
  released binaries.
- **Non-interactive start:** none is granted. `claude -p` is the command once a later ADR allows
  unattended work.

**Out of scope:** the toolchain the agent works with; the review skill as a second sensor before
the maintainer's review, which the maintainer may run without a decision; a second maintainer and
the Code Owners review, which a superseding ADR takes up when one joins.

## Alternatives considered

- **Implementer and reviewer apart** — a second agent reviewing in a fresh context before the
  maintainer reads the change. Not chosen as a rule: with every change read by the maintainer it
  adds a step without adding a guarantee; the review skill stays available.
- **Parallel tickets under a cap** — several agents working tickets from `docs/plan/` without the
  human. Not chosen: the work fits one session, and there is one reviewer to outrun.

## Sources / Prior art

- Birgitta Böckeler, *Harness engineering* —
  <https://martinfowler.com/articles/harness-engineering.html>: guides that feed forward and
  sensors that feed back, and committing early to what the sensors can hold.
- Addy Osmani, *Agentic autonomy levels* — <https://addyosmani.com/blog/agentic-autonomy-levels/>:
  how quickly a wrong result is noticed, how cleanly it is undone, what would prove it right.

## Consequences

- Positive: every change passes the one person who decides; no claim, cap or dispatch machinery to
  run or maintain.
- Negative / trade-offs: throughput is bounded by the maintainer's sessions; a change waits while
  the maintainer is away.
- Follow-ups: unattended tickets, with a cap and the start command, when the work outgrows one
  session at a time; a second maintainer and the Code Owners review when one joins.

## Enforcement

**Not mechanically decidable:** who reviews a change, who decides an ADR, and that no session runs
unattended are decided outside the tree. What the harness relies on mechanically is already
checked: the git hooks under `.githooks/` and the required checks listed under *Repository
settings* in the README.
