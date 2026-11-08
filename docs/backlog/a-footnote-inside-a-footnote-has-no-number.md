# A footnote referenced from inside a footnote has no number

Observed: with `A[^b]`, `[^b]: B[^c]` and `[^c]: C`, the website links to `#fn-0`, which no element
carries, and the text of `[^c]` appears nowhere. The numbering pass walks the blocks of the chapters
and not the footnote definitions, so such a reference keeps the number 0, with no diagnostic. The
EPUB leaves it out and reports it; the website, the print view and the PDF were not changed.

Where: `number_chapter` in `crates/core/src/passes.rs`.
