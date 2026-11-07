# The plain-rules sensor splits an avoided term where the glossary wraps it

Observed: the sensor reports "file" and "message" as avoided words on lines the NUC v0.8.1
take-up added (AGENTS.md:123; docs/adr/0006-feature-layer.md:87, 246, 251). The glossary avoids
"include file" and "log message", each wrapped over a line break in its _Avoid:_ list. The
reader of the lists takes each glossary line on its own, so a wrapped term becomes its single
words, and the sensor reports common words no entry avoids. The sensor fails nothing; its reports
mislead the reviewer. The defect is the template's, so the fix belongs upstream in hivevm/nuc
and reaches Nectar with a release.

Where: avoid_terms in scripts/lib/rules.sh, scripts/sensor-plain-rules.sh, docs/GLOSSARY.md
(the entries Chapter and Diagnostic).
