# Ticket: Highlighted code as standalone HTML, written by the HTML renderer's crate

## What to build

`nectar render -f html` writes highlighted code through `html`, `css` and `escape` in
`crates/cli/src/render.rs`, a second copy of what `Writer::highlighted`, `token_style` and
`escape` in `crates/html/src/write.rs` do: the same `<pre><code class="language-…">`, the same
spans, the same bold, italic and text-decoration rules, and an escape that differs in one case
(the command line's leaves `'` as it is). Only the colours differ: inline for the standalone
output, custom properties for the site's stylesheet. A fix to one copy, such as a new token style,
misses the other.

`nectar-html` gains one function that writes highlighted code as a standalone `<pre>` with the
inline colours of the light theme, sharing the token writer and the escape with the site's, and
`render.rs` calls it. The ANSI writer stays in the command line.

## Acceptance criteria

- [ ] Given a TextMate language, when `nectar render <lang> -f html` runs, then its output is
      byte-identical to today's, except where the shared escape writes `&#39;` for `'`, which the
      hand-off names.
- [ ] Given the tests of `crates/cli/tests/render.rs` and `crates/html/tests/`, when the change
      lands, then they pass, that `'` aside.
- [ ] Given the change, when `crates/cli/src/render.rs` is read, then it holds no HTML escape and
      no CSS of token styles.

## Read first

- Touches: `crates/cli/src/render.rs`, `crates/html/src/lib.rs`, `crates/html/src/write.rs`
- ADR-0018, *Decision*: `html` is a `<pre>` with inline styles, without the site's stylesheet.

## Blocked by

- None (can start immediately)

## Mode

with the human: ADR-0007 lets no agent work run unattended.
