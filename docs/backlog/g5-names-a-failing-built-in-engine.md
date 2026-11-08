# G-5 names a failing built-in engine only on a loose reading

Observed: G-12 has text that is no SVG fail "as an engine does under G-5 and G-11". G-5 names an
engine "missing on the host" or whose "program is missing or fails". A built-in engine such as
`rough` or `railroad` has no program, so its failure during a build is covered only by reading
"its program" loosely, as ADR-0019 and docs/DESIGN.md:154 do. Found in the review of the G-11
change (ticket revision-2026-11-07/04), which named a failing engine for `nectar render` only.
A change to G-5 is the maintainer's decision (AGENTS.md section 3, rule 7).

Where: G-5 and G-12 in docs/SPECIFICATION.md, the Errors bullet of
docs/adr/0019-rough-engine.md, docs/DESIGN.md:154.
