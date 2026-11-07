# Ticket: A code block with CRLF line ends gets no extra empty line

## What to build

Highlighted code whose text ends in a CRLF line end (`\r\n`) is shown with as many lines as the
text has, in the website, the PDF and `nectar render` (`html` and `ansi`), as a text ending in
`\n` already is. Today the TextMate engine strips only a final `\n` before highlighting, so the
`\r` it leaves becomes one extra empty line. Found by the review of the lesser fixes of
`nectar render` (PR #14); older than that change.

## Acceptance criteria

- [ ] Given a Rust snippet of three lines with CRLF line ends, the last one included, when
  `nectar render rust -f ansi` and `-f html` run on it, then the output has three lines, the
  same as for the snippet with LF line ends.
- [ ] Given a document with that snippet as a fenced code block, when it is built, then the
  highlighted block has three lines.
- [ ] Given a snippet whose last line has no line end, when it is rendered, then nothing
  changes from today.

## Read first

- Touches: `crates/textmate/src/lib.rs`, `crates/cli/tests/render.rs`, the tests of
  `crates/textmate/`
- Seam: the `CodeEngine` port (`crates/core/src/engine.rs`); test the engine through it.

## Blocked by

- None (can start immediately)

## Mode

unattended: a test decides each criterion; the scope is one engine and its tests, a wrong result
costs one revert; it touches nothing ADR-0007 keeps with the human; a wrong result shows as a
failing test or an extra line in `nectar render`; stop and hand off after two attempts that leave
the suite red.
