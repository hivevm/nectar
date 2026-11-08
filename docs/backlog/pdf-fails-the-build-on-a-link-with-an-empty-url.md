# The PDF fails the build on a link with an empty URL

Observed: a Markdown source with the link `[empty]()` failed `nectar build` with
`error: main.md: PDF: URL must not be empty`, while the website built. G-4 asks for a warning with
file and line where the PDF cannot show an element. A link keeps its text in every other output.

Where: the links of `crates/pdf/src/typeset.rs`.
