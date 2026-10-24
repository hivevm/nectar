+++
slug = "basics"
+++
# Basics

Text with a footnote[^note], //highlighted// and ____underlined____ words, a link to
[diagrams](diagrams.md) and a [reference link][commonmark] that turns into a footnote.

!!w A multi-line alert.

It spans several paragraphs.
!!

| Feature | Syntax | Width follows the dashes |
| ------- | :----: | -----------------------: |
| Alert   | `!w`   | right                    |
| Symbol  | `@web@` | right                   |

- [x] task done
- [ ] task open

![Draft](../img/draft.png){width=120 align=center}

```rust
fn main() {
    println!("Hello, Nectar!");
}
```

A source file embedded through an image link, highlighted like a code block:

![Main class](../code/Main.java)

[^note]: Footnotes are numbered across the whole document.

[commonmark]: https://commonmark.org "CommonMark"
