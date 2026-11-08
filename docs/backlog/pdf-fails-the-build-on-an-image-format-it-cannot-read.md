# The PDF fails the build on an image it cannot read, where G-4 asks for a warning

Observed: a project with an image `old.bmp` in a Markdown source, whose file held the two bytes
`BM`, failed `nectar build` with `error: main.md: PDF: unknown image format`, naming a line of
generated Typst source. G-4 asks the system to report an image the PDF cannot embed as a warning
with file and line. The project built once `pdf = false` was set for that document. Only this file
was tried; whether a valid BMP, AVIF or TIFF fails the same way, all of which the core accepts as
images, is unconfirmed.

Where: `crates/pdf/src/lib.rs` (the errors of `typst::compile`), and the list of picture extensions
in `crates/core/src/load.rs`, against what the PDF can embed.
