# The record names the built-in engines once, at the next normalization

Observed: three accepted ADRs together decide which code engines run without a `nectar.toml`.
ADR-0015 (a single file builds with "the built-in ones, railroad and TextMate") and ADR-0018
(`nectar render` without `-C` has "only the built-in engines … `railroad` and TextMate
highlighting") list a set the system has outgrown. ADR-0019 added `rough` as an engine with "the
`enabled` every built-in engine has". Read on its own, ADR-0015 or ADR-0018 takes the rough
engine in a single-file build or in `render` for a violation.

Decided by the maintainer on 2026-11-08 (ticket revision-2026-11-07/05): the next normalization
names the set once, without changing a decision; no superseding ADR before it. The sentence to
carry: *the built-in engines, railroad, rough and TextMate, run in every command with their
defaults, each unless `[engine.<name>] enabled = false`; without a `nectar.toml` they are the
only ones.*

Where: docs/adr/0015-single-file-build.md, docs/adr/0018-render-command.md,
docs/adr/0019-rough-engine.md, `engines` in crates/cli/src/main.rs.
