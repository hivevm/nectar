# Nectar

[![Checks](https://github.com/hivevm/nectar/actions/workflows/checks.yml/badge.svg)](https://github.com/hivevm/nectar/actions/workflows/checks.yml)

**Nectar** turns Markdown into a website and a PDF: `nectar build` writes the site, `nectar serve`
serves it on your machine and reloads the browser on every change. The work is driven by a written
**specification** ([`docs/SPECIFICATION.md`](docs/SPECIFICATION.md)) and **Architecture Decision
Records** ([`docs/adr/`](docs/adr/)), so intent and the reasoning behind every structural choice
stay explicit and reviewable.

> For agent instructions, see [`AGENTS.md`](AGENTS.md), the single source of truth for all coding
> agents.

> [!NOTE]
> **This project still carries its template setup.** The one-time steps that turn the scaffold
> into your own project are in [`TEMPLATE-SETUP.md`](TEMPLATE-SETUP.md); ask your agent to run the
> `setup-project` skill to be walked through them, one question at a time. Once you are through
> them, `scripts/test-template-setup.sh --close` removes that file and this note.

## Overview

The repository holds two generations of the same idea: Markdown in, documentation out.

- **Nectar**, the command line `nectar`, written in Rust under [`crates/`](crates/). It turns a
  project of Markdown files into a website with a print view and a PDF, highlights code with
  TextMate grammars and draws railroad, Graphviz and PlantUML diagrams. The
  [Nectar guide](doc/nectar/README.md) describes it in full, its design is
  [`doc/nectar/design.md`](doc/nectar/design.md), and [`examples/demo/`](examples/demo/) is a
  complete project to start from.
- **The Gradle plugin** `org.hivevm.doc`, written in Java under [`src/`](src/). It renders
  Markdown to PDF through Apache FOP; the second half of [Usage](#usage) documents it.

The full problem statement and goals live in [`docs/SPECIFICATION.md`](docs/SPECIFICATION.md), the
vocabulary in [`docs/GLOSSARY.md`](docs/GLOSSARY.md).

## Prerequisites

- [VS Code](https://code.visualstudio.com/) with the
  [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
  extension, or any DevContainer-compatible IDE
- Docker / Podman (rootless) available on the host
- Outside the Dev Container: bash 4.4 or later for the checks, 3.2 for the git hooks
  ([ADR-0002](docs/adr/0002-decisions-verified-by-tests.md)); macOS ships 3.2

## Getting Started

> Setting the project up for the first time? Create the repository from the template, with
> **Use this template** on GitHub or with
> `gh repo create <owner>/<name> --template hivevm/nuc --public --clone`. A private repository
> needs a paid GitHub plan for the rulesets of [Repository settings](#repository-settings).
> The one-time steps are in [`TEMPLATE-SETUP.md`](TEMPLATE-SETUP.md); the close of the setup
> removes this paragraph with that file.

1. Open the repository in VS Code and choose **Reopen in Container**. The Dev Container and the
   preconfigured agent extensions build automatically, and the container enables the repository's
   git hooks with the merge settings they rely on ([Git hooks](#git-hooks)). Working outside the
   container? Run the three commands listed there yourself, once per clone.
2. Authenticate your coding agent inside the container (for Claude Code: `claude login`).
3. Start working with the agent. Drive the work from the specification and the ADRs.

## Build, Test & Run

<!-- This section is the single source for build/test/lint/run commands; both humans and
     agents rely on it, and the rule file names it. -->

The Dev Container brings Rust, Java 21, Graphviz and PlantUML. Nectar's commands run from the
repository root and match the [Nectar workflow](.github/workflows/nectar.yml):

- **Build:** `cargo build --workspace`
- **Test:** `cargo test --workspace`, then `cargo run -p nectar -- -C examples/demo build`,
  which has to succeed even where Graphviz and PlantUML are missing
- **Lint:** `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`; the repository checks with
  `scripts/check-all.sh`
- **Run:** `cargo run -p nectar -- -C examples/demo serve`, then open http://127.0.0.1:1313/

The Gradle plugin builds and tests with `./gradlew build`.

A release of Nectar is a tag `nectar-vX.Y.Z`: the
[release workflow](.github/workflows/nectar-release.yml) builds `nectar` for Linux, macOS and
Windows and attaches the archives, with their `SHA256SUMS`, to a GitHub release of that tag.

## Usage

### Nectar

Download the archive for your platform from the
[releases](https://github.com/hivevm/nectar/releases), unpack it and put `nectar` on your
`PATH`, or build it from source with `cargo install --path crates/cli`.

A project is a directory with a `nectar.toml`:

```toml
title = "My Manual"

[[document]]
source = "content/manual.md"
```

```sh
nectar serve            # http://127.0.0.1:1313/, reloads on every change
nectar build            # writes the site, the print view and the PDF to public/
```

| Option | Meaning |
|---|---|
| `-C <dir>` | Project directory, the one that holds `nectar.toml`; default the current one |
| `--offline` | Keep images from the web as links instead of downloading them |
| `build -o <dir>` | Output directory; overrides `output` in `nectar.toml` |
| `build --drafts` | Include files whose front matter says `draft = true` |
| `serve -p <port>`, `serve --bind <addr>` | Address to serve on; default `127.0.0.1:1313` |
| `serve --no-drafts` | Leave out drafts, which `serve` shows by default |

Chapters, front matter, the Markdown dialect, code engines, the print view, themes and every
key of `nectar.toml` are described in the [Nectar guide](doc/nectar/README.md). Nectar reads
the Markdown dialect the Gradle plugin introduced; the sections below document the plugin and
that dialect.

### Gradle plugin: Markdown for PDF generation {#Markdown_for_PDF_generation}

The plugin supports the creation of beautiful PDF's from markdown. Create a simple gradle project with the Gradle
Wrapper and configure *build.gradle* and *settings.gradle*.

The *build.gradle* applies the documentation plugin with the configuration parameters.

~~~gradle
plugins {
  id 'org.hivevm.doc' version '1.1.1'
}

markdown {
  source   = 'manual'
  template = 'template.ui.xml'
}
~~~

The latest version can always be loaded from [Gradle Plugin repository](https://plugins.gradle.org/plugin/org.hivevm.doc).

The optional *markdown.config* parameter defines the configuration file. If the parameter is omitted an internal default
template is used. The *markdown.source* parameter defines a directory of where the root markdown files are located, or
it defines a single markdown file that should be converted to PDF.

The *settings.gradle* optionally defines the plugin repository.

~~~gradle
pluginManagement {
  repositories {
    gradlePluginPortal()
  }
}
~~~

The *markdown.config* parameter is optional and allows custom configuration of the layout of the rendered PDF. By
default a standard template is applied.

### Common mark

[Markdown][1] is a plain text format for writing structured documents, based on formatting conventions from email and
usenet. *Markdown* is a simple way to format text that looks great on any device. It doesn’t do anything fancy like
change the font size, color, or type — just the essentials, using keyboard symbols you already know.

There are different implementations of Markdown, we use the [Common Markdown][2] is intended to be as easy-to-read and
easy-to-write as is feasible.

Readability, however, is emphasized above all else. A Markdown-formatted document should be publishable as-is, as plain
text, without looking like it’s been marked up with tags or formatting instructions. While Markdown’s syntax has been
influenced by several existing text-to-HTML filters — including Setext, atx, Textile, reStructuredText, Grutatext, and
EtText — the single biggest source of inspiration for Markdown’s syntax is the format of plain text email.

To this end, Markdown's syntax is composed entirely of punctuation characters, which punctuation characters have been
carefully chosen so as to look like what they mean. E.g., asterisks around a word actually look like *emphasis*.
Markdown lists look like, well, lists. Even blockquotes look like quoted passages of text, assuming you’ve ever used
email.

Common mark is a strongly defined, highly compatible specification of Markdown.

For details look the [Common Mark Manual](https://commonmark.org/help/). There are many ways to write markdown; a simple
offline editor is [Typora][3].

[1]: https://en.wikipedia.org/wiki/Markdown "Markdown is a lightweight markup language for creating formatted text using a plain-text editor."

[2]: https://spec.commonmark.org/ "A strongly defined, highly compatible specification of Markdown"

[3]: https://typora.io/ "A truly minimal Mardkown editor"

#### Headings

Starting a line with a hash # and a space makes a header.

The more #, the smaller the header.

	# Manual title
	## Chapter
	### Sub Chapter 

!w Remember: Each *markdown* file should be handled as independent document. Including a file into another it becomes
automatically a child chapter. So use the highest heading in each file.

#### Paragraphs

A paragraph is consecutive lines of text with one or more blank lines between them.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor
	invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.

#### Line Breaks

For a line break, add either a backslash \ or two blank spaces at the end of the line; this is interpreted as
continuation of the text. This is useful if the markdown text should be be wrapped for better reading, but logically the
text remains on the same line.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy··
	eirmod tempor
or

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy\
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy\
eirmod tempor

A text with less than two spaces is interpreted as simple line break

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy 
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
eirmod tempor

To organize the text thematic 3 sings(-), underscores(_), or stars(*) can be used. On PDF this will create a page break.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy 
	___
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
___
eirmod tempor

#### Blockquotes

To create a blockquote, start a line with greater than > followed by an optional space.

Blockquotes can be nested, and can also contain other formatting.

	> First line
	> Another line
	>
	> > Nested line
	>
	> Last line	

> First line
> Another line
>
> > Nested line
>
> Last line

#### Emphasis

To create bold or italic, wrap with asterisks \* or underscores \_. To avoid creating bold or italic, place a backslash
in front \\\* or \\\_. For *italics* a single asteriks or underscore is used, for **bold** 2 are used. Using 3 allows to
create a ***bold and italic*** text.

Our implementation supports an additional syntax for ____underline____ with 4 underscores and ~~strikethrough~~ with 2
tilde.

	*italic* _italic_
	**bold** __bold__
	***bold & italic text***
	~~strikethrough~~
	____underline____

#### Lists

Unordered lists can use either asterisks *, plus +, or hyphens - as list markers.

	- Apples
	- Oranges
	- Pears

- Apples
- Oranges
- Pears

Ordered lists use numbers followed by period . or right paren ).

	1. First
	2. Second
	3. Third

1. First
2. Second
3. Third

Multilevel and combined lists

    1. Fruit
     - Apple
     - Pear
    2. Vegetable
     - Potato
     - Carrot

1. Fruit
   - Apple
     1. Apple 1
     1. Apple 2
   - Pear
     1. Pear 1
     1. Pear 2
2. Vegetable
   - Potato
     1. Potato 1
     1. Potato 2
   - Carrot
     1. [x] Carrot 1
     1. [ ] Carrot 2

Task List

    - [x] Release candidate
    - [ ] Product release

- [x] Release candidate
- [ ] Product release

#### Links

Links can be either inline with the text, or placed at the bottom of the text as references.

Link text is enclosed by square brackets [], and for inline links, the link URL is enclosed by parens ().

	[text](http://a.com)

Links can be realized as footnotes, defining an identifier by square brackets [] and the link URL defined below in the
page.

	[][id]
	...
	[id]: http://b.org/ "text"

or as alternative with an inline link

	[text][id]
	...
	[id]: http://b.org/ "text"




[Tests](#Lists)

#### Images

Images are almost identical to links, but an image starts with an exclamation point !.

	![](markdown.png)
	![text](markdown.png)

![text](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC)

![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC){align=float-right}

We support the non-official specification to provide a size and alignment for the image. The optional parameters are
defined between brackets {}.

	![text](markdown.png){width=4cm align=center}

![text](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC){width=4cm align=center}

![][image]

[image]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC>


#### Code

To create `inline code`, wrap with backticks `.

	`inline code`

To create a code block, either indent each line by 4 spaces, or place 3 backticks \`\`\` on a line above and below the
code block.

	```
	code block
	```

### Extended Syntax

Advanced features that build on the basic Markdown syntax.

#### Multi-Files

The *Markdown* parser supports organizing the documentation in multiple files. Each file can be included by a simple
link with the relative path. The documentation generator will merge all files into a single *markdown* source. Each file
should be designed as independent file, the generator takes care about building the correct structure of chapters.

	[Title](local_path/to_file.md)

!w An included file will be handled as sub-chapter of the current chapter.

#### Comments

Most Markdown implementatios support comments.

	[TEXT]: # (Some comment)

[comment]: # (Still another comment)

#### Internal Links

Markdown allows to link to internal headers using as link the header name in lower case, prefixed with a hash, and
replacing all spaces with an sign (-).

	[text](#header-of-document)
	
	### Header of Document

The renderer changes to the local references automatically, that you can directly use the header. Nevertheless you
should avoid headers with special characters.

	[text](#Header of Document)
	
	### Header of Document

#### Tables

To add a table, use three or more hyphens (---) to create each column’s header, and use pipes (|) to separate each
column. You can optionally add pipes on either end of the table. We support the extended syntax to align the columns. By
default the columns are left aligned, using colons before or after the hypens it is possible to define a left (:---),
center (:---:) or right alignment (---:)

	| Auto | Center Alignment | Left Alignment | Right Alignment |
	| ---- | :--------------: | :------------- | --------------: |
	| Auto | Center | Left | Right |

| Auto | Center Alignment | Left Alignment | Right Alignment |
| ---- | :--------------: | :------------- | --------------: |
| Auto | Center | Left | Right |


Sizing of of the single table columns is not supported by markdown. We extended the tables and introduced the definition
of the column width, using the number of hyphens (---) in the header separator. The number of hyphens in a single column
relative to the sum of all hyphens in all columns define the relative width to the whole table.

#### Alerts

Alerts are an extension to provide customizable blockquotes. 4 different kind of alerts are supported, that are rendered
with a background and highlight color. The alert message have to start with **!** at the beginning of the line with a
letter defining the alert message.

	!n Note

!n Note

	!s Success

!s Success

	!w Warning

!w Warning

	!e Error

!e Error

Sometimes multiline alerts are need. Multiline alert starts with double **!!**. The message will be terminated with a
single line of **!!**.

	!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
	
	Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
	!!

!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore
magna aliquyam erat, sed diam voluptua.

Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore
magna aliquyam erat, sed diam voluptua.
!!

#### Symbols

Our implementation allows to define placeholders for font symbols. The symbol name must be defined between 2 @.

	@web@

@web@ @mobile@ @config@ @link@

As font the [Material Icons Font][material-icons] is used.


[material-icons]: https://material.io/resources/icons/ "Material Icons Font"

#### Formatted Code

The implementation supports the extended syntax with 3 tilde (~), optionally to defining the the code formatter.

	~~~ini
    ; last modified 1 April 2001 by John Doe
    [owner]
    name = John Doe
    organization = Acme Widgets Inc.
    
    [database]
    ; use IP address in case network name resolution is not working
    server = 192.0.2.62     
    port = 143
    file = "payroll.dat"
	~~~

~~~ini
; last modified 1 April 2001 by John Doe
[owner]
name = John Doe
organization = Acme Widgets Inc.

[database]
; use IP address in case network name resolution is not working
server = 192.0.2.62     
port = 143
file = "payroll.dat"
~~~

~~~yaml
# An employee record
name: Martin D'vloper
job: Developer
skill: Elite
employed: True
foods:
  - Apple
  - Orange
  - Strawberry
  - Mango
languages:
  perl: Elite
  python: Elite
  pascal: Lame
education: |
  4 GCSEs
  3 A-Levels
  BSc in the Internet of Things
~~~

~~~json
{
  "squadName": "Super hero squad",
  "homeTown": "Metro City",
  "formed": 2016,
  "secretBase": "Super tower",
  "active": true,
  "members": [
    {
      "name": "Molecule Man",
      "age": 29,
      "secretIdentity": "Dan Jukes",
      "powers": [
        "Radiation resistance",
        "Turning tiny",
        "Radiation blast"
      ]
    },
    {
      "name": "Madame Uppercut",
      "age": 39,
      "secretIdentity": "Jane Wilson",
      "powers": [
        "Million tonne punch",
        "Damage resistance",
        "Superhuman reflexes"
      ]
    },
    {
      "name": "Eternal Flame",
      "age": 1000000,
      "secretIdentity": "Unknown",
      "powers": [
        "Immortality",
        "Heat Immunity",
        "Inferno",
        "Teleportation",
        "Interdimensional travel"
      ]
    }
  ],
  "from": "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Objects/JSON"
}
~~~

~~~java
import java.awt.Rectangle;

public class ObjectVarsAsParameters {
    public static void main(String[] args) {
        go();
    }

    public static void go() {
        Rectangle r1 = new Rectangle(0, 0, 5, 5);
        System.out.println("In method go. r1 " + r1 + "\n");
        // could have been
        //System.out.prinltn("r1" + r1.toString());
        r1.setSize(10, 15);
        System.out.println("In method go. r1 " + r1 + "\n");
        alterPointee(r1);
        System.out.println("In method go. r1 " + r1 + "\n");

        alterPointer(r1);
        System.out.println("In method go. r1 " + r1 + "\n");
    }

    public static void alterPointee(Rectangle r)
    {
        System.out.println("In method alterPointee. r " + r + "\n");
        r.setSize(20, 30);
        System.out.println("In method alterPointee. r " + r + "\n");
    }

    public static void alterPointer(Rectangle r)
    {
        System.out.println("In method alterPointer. r " + r + "\n");
        r = new Rectangle(5, 10, 30, 35);
        System.out.println("In method alterPointer. r " + r + "\n");
    }
}
~~~

~~~cpp
#include <iostream>
using namespace std;

int main() {

   int num_int;
   double num_double = 9.99;

   // implicit conversion
   // assigning a double value to an int variable
   num_int = num_double;

   cout << "num_int = " << num_int << endl;
   cout << "num_double = " << num_double << endl;

   return 0;
}
~~~

~~~rust
fn main() {
    // We can use this function here, and define it somewhere later
    fizzbuzz_to(100);
}

// Function that returns a boolean value
fn is_divisible_by(lhs: u32, rhs: u32) -> bool {
    // Corner case, early return
    if rhs == 0 {
        return false;
    }

    // This is an expression, the `return` keyword is not necessary here
    lhs % rhs == 0
}

// Functions that "don't" return a value, actually return the unit type `()`
fn fizzbuzz(n: u32) -> () {
    if is_divisible_by(n, 15) {
        println!("fizzbuzz");
    } else if is_divisible_by(n, 3) {
        println!("fizz");
    } else if is_divisible_by(n, 5) {
        println!("buzz");
    } else {
        println!("{}", n);
    }
}

// When a function returns `()`, the return type can be omitted from the
// signature
fn fizzbuzz_to(n: u32) {
    for n in 1..=n {
        fizzbuzz(n);
    }
}
~~~

#### Extensions

Uml Diagrams with PlantUML

~~~uml
hide footbox
actor Caller
box "MultiAsyncExecutableTask"
    participant "Task1" as T1
    participant "Task2" as T2
    participant "Task3" as T3
end box
participant REST

Caller -> T1 : execute()
activate T1

T1 -> REST : GET

T1 -> T2 : execute()
deactivate T1
activate T2

T2 -> REST : GET

T2 -> T3 : execute()
deactivate T2
activate T3

T3 -> REST :  GET

Caller <-- T3 : return
deactivate T3

...500 ms...

T1 <-- REST : 200 OK
activate T1

T1 --> T2 : Result.ok()
deactivate T1

T2 <-- REST : 200 OK
activate T2

T2 --> T3 : Result.ok()
deactivate T2

T3 <-- REST : 200 OK
activate T3

Caller <-- T3 : Result.ok()
deactivate T3
~~~

~~~uml
interface DocumentElement
interface ParagraphElement
interface FlowElement

DocumentElement <|.. Header
DocumentElement <|.. Break
DocumentElement <|.. Table
DocumentElement <|.. ParagraphElement
ParagraphElement <|.. Paragraph
ParagraphElement <|.. BlockQuotes
ParagraphElement <|.. List
ParagraphElement <|.. CodeBlock

Document "1" *--> "*" DocumentElement : contains
Header "1" *--> "1" FlowElement : contains
Paragraph "1" *--> "1" FlowElement : contains
CodeBlock "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface FlowElement
FlowElement <|.. Text
FlowElement <|.. Link
FlowElement <|.. Style
FlowElement <|.. Image
FlowElement "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface DocumentElement
interface ParagraphElement
interface FlowElement

DocumentElement <|.. Header
DocumentElement <|.. Break
DocumentElement <|.. Table
DocumentElement <|.. ParagraphElement
ParagraphElement <|.. Paragraph
ParagraphElement <|.. BlockQuotes
ParagraphElement <|.. List
ParagraphElement <|.. CodeBlock

Document "1" *--> "*" DocumentElement : contains
Header "1" *--> "1" FlowElement : contains
Paragraph "1" *--> "1" FlowElement : contains
CodeBlock "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface FlowElement
FlowElement <|.. Text
FlowElement <|.. Link
FlowElement <|.. Style
FlowElement <|.. Image
FlowElement "1" *--> "*" FlowElement : contains
~~~

~~~uml
class Database {}
class ObjectCollection {}
class ObjectReference {}
class PersistentObject {}
class ObjectInfo {}

hide members

Database   -d-> ObjectCollection : supplies
ObjectCollection -d-> PersistentObject         : manages persistence
ObjectReference  -u-> ObjectCollection         : uses
PersistentObject "1" -u-> "*" ObjectCollection : contains
PersistentObject "1" -l-> "*" ObjectReference  : contains
ObjectInfo       -u-> PersistentObject         : describes
~~~

~~~uml
start
repeat
  :Test something;
    if (Something went wrong?) then (no)
      #palegreen:OK;
      break
    endif
    ->NOK;
    :Alert "Error with long text";
repeat while (Something went wrong with long text?) is (yes) not (no)
->//merged step//;
:Alert "Success";
stop
~~~

~~~uml
participant Participant as Foo
actor       Actor       as Foo1
boundary    Boundary    as Foo2
control     Control     as Foo3
entity      Entity      as Foo4
database    Database    as Foo5
collections Collections as Foo6
queue       Queue       as Foo7
Foo -> Foo1 : To actor
Foo -> Foo2 : To boundary
Foo -> Foo3 : To control
Foo -> Foo4 : To entity
Foo -> Foo5 : To database
Foo -> Foo6 : To collections
Foo -> Foo7: To queue
~~~

~~~jsongraph
<style>
.h1 {
  BackGroundColor
  green
  FontColor
  white
  FontStyle
  italic
}
.h2 {
  BackGroundColor
  red
  FontColor
  white
  FontStyle
  bold
}
</style>
#highlight "lastName"
#highlight "address" / "city" <<h1>>
#highlight "phoneNumbers" / "0" / "number" <<h2>>
{
"firstName": "John",
"lastName": "Smith",
"isAlive": true,
"age": 28,
"address": {
"streetAddress": "21 2nd Street",
"city": "New York",
"state": "NY",
"postalCode": "10021-3100"
},
"phoneNumbers": [
{
"type": "home",
"number": "212 555-1234"
},
{
"type": "office",
"number": "646 555-4567"
}
],
"children": [],
"spouse": null
}
~~~

~~~jsongraph
{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address": {
    "streetAddress": "21 2nd Street",
    "city": "New York",
    "state": "NY",
    "postalCode": "10021-3100"
  },
  "phoneNumbers": [
    {
      "type": "home",
      "number": "212 555-1234"
    },
    {
      "type": "office",
      "number": "646 555-4567"
    }
  ],
  "children": [],
  "spouse": null
}
~~~

~~~ebnf
title LISP Grammar
grammars_expression = atomic_symbol | "(", s_expression, ".", s_expression, ")" | list;
list = "(", s_expression, { s_expression }, ")";
atomic_symbol = letter, atom_part;
atom_part = empty | letter, atom_part | number, atom_part;
letter = ? a-z ?;
number = ? 1-9 ?;
empty = " ";
~~~

~~~regex
!option useDescriptiveNames false

\\d?\\D+\\w*\\W{1,2}|\\s.\\S
~~~

~~~ebnf
title Semantic Versioning 2.0
semver = version_core "-", pre_release "+", build;
~~~

~~~dot
digraph G {

  subgraph cluster_0 {
    style=filled;
    color=lightgrey;
    node [style=filled,color=white];
    a0 -> a1 -> a2 -> a3;
    label = "process #1";
  }

  subgraph cluster_1 {
    node [style=filled];
    b0 -> b1 -> b2 -> b3;
    label = "process #2";
    color=blue
  }
  start -> a0;
  start -> b0;
  a1 -> b3;
  b2 -> a3;
  a3 -> a0;
  a3 -> end;
  b3 -> end;

  start [shape=Mdiamond];
  end [shape=Msquare];
}
~~~

~~~railroad
H2_SELECT =
    'SELECT' [ 'TOP' term ] [ 'DISTINCT' | 'ALL' ] selectExpression {',' selectExpression} \
    'FROM' tableExpression {',' tableExpression} [ 'WHERE' expression ] \
    [ 'GROUP BY' expression {',' expression} ] [ 'HAVING' expression ] \
    [ ( 'UNION' [ 'ALL' ] | 'MINUS' | 'EXCEPT' | 'INTERSECT' ) select ] [ 'ORDER BY' order {',' order} ] \
    [ 'LIMIT' expression [ 'OFFSET' expression ] [ 'SAMPLE_SIZE' rowCountInt ] ] \
    [ 'FOR UPDATE' ];
~~~

The *language* attribute allows to define the formatting for a specific language. *ini*, *xml*, *json*, *java* and *cpp*
are supported. Support for *rest* is missing.

### Rendering Configuration

The *markdown.config* parameter allows to define a custom configuration for rendering.

- **template**: A template the defines the rendering configuration. The template allows to define the default page size
with the *width* and *height* parameters, and the default font configuration with the parameter *font-family*.
- **font**: Declares a named font, composed by different TTF (*True Type Font*) files for each metric. Optionally a
codepoints file can be declared to define named symbols of a symbol font file.
- **style**: Allows to define for each text block a different font configuration, line height or coloring. The *style*
can be applied for text styles, emphasise and special blocks. On blocks the styles supports additional attributes, like
*border*, *padding* or *background-color*
- **page**: defines the setting of a single page. On the page you can redefine the page *size*, the *margin* and 
the *padding*. Each page can have a *head*, *tail*, *start* and *end* block with a *size*. The page *margin* defines the
intend from the page borders, inclusive the blocks. The page *padding* defines the intend of the body from
the blocks. On the page you can define optionally a watermark image or a background color.
On the page it's possible to place static elements. On each block we can define flow elements.
- **page-set**: defines a collection of pages for a specific configuration. Usually you need a single set, but
you can define different sets, if you want a special rendering for chapters.

~~~xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE template>
<template width="210mm" height="297mm" font-family="ubuntu">
    <font name="ubuntu">
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Regular.ttf"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Bold.ttf" bold="true"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Italic.ttf" italic="true"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-BoldItalic.ttf" bold="true" italic="true"/>
    </font>
    <font name="victor">
        <font-metric uri=":fonts/victor/VictorMono-Medium.ttf"/>
        <font-metric uri=":fonts/victor/VictorMono-Bold.ttf" bold="true"/>
        <font-metric uri=":fonts/victor/VictorMono-MediumItalic.ttf" italic="true"/>
        <font-metric uri=":fonts/victor/VictorMono-BoldItalic.ttf" bold="true" italic="true"/>
    </font>
    <font name="material" codepoints=":fonts/MaterialIcons-Regular.ttf.codepoints">
        <font-metric uri=":fonts/MaterialIcons-Regular.ttf"/>
    </font>

    <style match="subtitle" color="white" font-weight="bold" font-size="26pt" break-after="page" line-height="1.4em"/>
    <style match="h1" color="#130806" font-weight="bold" text-align="left" break-after="page" font-size="42pt"
           line-height="1.1em"/>
    <style match="h[2-6]" font-size="24pt" font-weight="bold" line-height="1.2em" text-align="start"
           keep-with-next="always"
           color="#130806" space-before="1em * 0.8,1em,1em * 1.2" space-after="0.5em * 0.8,0.5em,0.5em * 1.2"/>

    <style match="p" line-height="1.4em" space-before="0.5em" space-after="0.5em" text-align="justify"/>
    <style match="block" line-height="1.4em" margin-left="0.15in" padding-left="0.7em" border-left="3pt solid #ffbe35"/>
    <style match="link" color="#9c8b54"/>

    <style match="info" margin="0" padding="0.2em,1em" background="#bde5f8" keep-with-next="always"
           border=".5px solid #5bc2f3" border-left=".1in solid #5bc2f3" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="success" margin="0" padding="0.2em,1em" background="#dff2bf" keep-with-next="always"
           border=".5px solid #4f8a10" border-left=".1in solid #4f8a10" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="warning" margin="0" padding="0.2em,1em" background="#ffedbd" keep-with-next="always"
           border=".5px solid #ffbe35" border-left=".1in solid #ffbe35" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="error" margin="0" padding="0.2em,1em" background="#f78082" keep-with-next="always"
           border=".5px solid #cc0013" border-left=".1in solid #cc0013" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="code" text-align="start" line-height="1em" keep-with-next="always"
           padding="0.1in" font-family="victor" font-size="9pt"
           space-before="0.8em,1em,1.2em" space-after="0.8em,1em,1.2em"/>
    <style match="styled" padding="0.2em,0.2em"
           font-family="ubuntu" background="#eeeeee"
           border="0.5pt solid #aaaaaa" border-radius="0.2em"/>

    <page name="cover" padding="210mm,0,70mm,2.7in" background=":background.jpg">
        <region position="top" extent="297mm">
            <column top="187mm" left="2.5in" right="0" bottom="70mm" background="#130806">
                <column top="0" left="0.2in" right="0.2in" bottom="0">
                    <row color="#cfa721" font-size="48pt" font-weight="bold" line-height="1.4em">User Manual</row>
                </column>
            </column>
        </region>
    </page>

    <page name="title" padding="15cm,1in,10cm,1in" background="#cfa721">
        <region position="top" extent="107mm">
            <column top="1in" left="1in" color="#130806" font-weight="bold" text-align="left" font-size="256pt"
                    line-height="1.5em">
                <row>
                    {{$CHAPTER}}
                </row>
            </column>
        </region>
        <region position="bottom" extent="107mm" column-count="2" column-gap="12pt">
            <column top=".15in" left="1in" right="1in">
                <br style="solid" size="1pt" color="#000000"/>
            </column>
        </region>
    </page>

    <page name="odd" padding="0.75in">
        <region position="top" extent="8mm">
            <column top="0mm" left="130mm" right="10mm" bottom="0mm" background="#cfa721">
                <row color="#130806" font-size="13pt" text-align="center" top="0.2em">{{$TITLE}}</row>
            </column>
        </region>
        <region position="bottom" extent="10mm">
            <column top="0mm" left="190mm" right="10mm" bottom="0mm" background="#cfa721">
                <row color="#130806" text-align="center" top="0.5em">{{$PAGE_NUMBER}}</row>
            </column>
        </region>
    </page>

    <page name="even" padding="0.75in">
        <region position="top" extent="8mm">
            <column top="0mm" left="10mm" right="130mm" bottom="0mm" background="#cfa721">
                <row color="#130806" font-size="13pt" text-align="center" top="0.2em">{{$TITLE}}</row>
            </column>
        </region>
        <region position="bottom" extent="10mm">
            <column top="0mm" left="10mm" right="190mm" bottom="0mm" background="#cfa721">
                <row color="#130806" text-align="center" top="0.5em">{{$PAGE_NUMBER}}</row>
            </column>
        </region>
    </page>

    <page name="blank"/>

    <page-set name="book">
        <page-entry name="cover" match="First"/>
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>

    <page-set name="chapter">
        <page-entry name="title" match="First"/>
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>

    <page-set name="standard">
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>
</template>
~~~

Currently the renderer support 3 different fonts:

- TEXT: Defines the font for the default text.
- MONO: Defines the font for the code snippets.
- SYMBOLS: Defines the symbol font, used for font icons.

## Project Layout

```
TEMPLATE-SETUP.md     # one-time template setup; delete it when the project is yours
README.md             # overview & setup for humans
AGENTS.md             # single source of truth for coding agents
CONTRIBUTING.md       # how to contribute, pointing at the rules
SECURITY.md           # how to report a vulnerability, and what the configuration guarantees
CODE_OF_CONDUCT.md    # the standards everyone taking part is held to
LICENSE               # the license the project is released under
docs/SPECIFICATION.md # the specification: problem, goals, success criteria
docs/GLOSSARY.md      # the vocabulary everyone uses, kept current inline
docs/CONVENTIONS.md   # how this project writes what no check decides, kept current inline
docs/ARCHITECTURE.md  # the system as it currently stands
docs/adr/             # Architecture Decision Records (+ template)
docs/plan/            # open work: feature specs and their tickets, deleted as they land
scripts/check-all.sh  # runs the checks, sensors, and self-tests, as the hooks and CI do
scripts/check-*.sh    # checks: each fails on what breaks a rule
scripts/sensor-*.sh   # sensors: report for the reviewer; sensor-repo-settings.sh runs by hand
scripts/test-*.sh     # self-tests of the checks, the sensors, the hooks, the setup, and the scripts
                      # run by hand
scripts/lib/          # what the scripts share: the ADR header, git, the ticket path, the self-tests
scripts/*.sh          # run by hand, steps 7 and 8 of the integration branches: number-adrs.sh,
                      # ready-for-main.sh, drop-merged-branch.sh
.githooks/            # git hooks: nothing lands on main locally; checks before a merge and a push
.github/workflows/    # checks.yml runs the checks; review-binding.yml reads each pull request's hand-off
.github/              # Dependabot, issue & pull request templates, code owners
.devcontainer/        # Dev Container definition (base image + Features)
.vscode/              # shared editor settings
.editorconfig         # editor-neutral formatting baseline
.gitattributes        # line-ending normalization (LF everywhere), the ADR index merged by its driver
.gitignore            # what git leaves out, .scratch/ for session artifacts among it
.agents/skills/       # the procedures of AGENTS.md as Agent Skills, one directory each (ADR-0003),
                      # with the scripts and templates they carry (claim.sh, remap.sh, ticket.md)
.claude/CLAUDE.md     # pointer for Claude Code to read AGENTS.md
.claude/skills/       # pointers (one symlink per skill) for Claude Code to read .agents/skills/
.claude/settings.json # Claude Code permissions: deny reading .env files (see SECURITY.md)
Cargo.toml            # Cargo workspace of Nectar
crates/core/          # Nectar: Markdown dialect, document model and transform passes
crates/html/          # Nectar: website, print view and the built-in theme
crates/pdf/           # Nectar: PDF typeset with Typst, with its fonts
crates/textmate/      # Nectar: code highlighting with TextMate grammars and themes
crates/railroad/      # Nectar: railroad diagrams from (E)BNF
crates/process/       # Nectar: code engines that run a local command, such as Graphviz
crates/server/        # Nectar: local server that rebuilds and reloads the browser
crates/cli/           # Nectar: the nectar command line
examples/demo/        # a complete Nectar project, built in CI
doc/nectar/           # Nectar's user guide and design
build.gradle          # Gradle plugin org.hivevm.doc: build and publication
src/main/             # Gradle plugin: Java sources, templates, fonts and grammars
```

## Dev Container

The environment is defined entirely in
[`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json). It starts from a prebuilt
base image and layers Dev Container Features and VS Code extensions on top; no Dockerfile or
Compose file is required. Customise the environment by adding Features, switching the base image,
or adding extensions. Features are pinned by major tag and resolved in the committed
`devcontainer-lock.json`. Extensions are listed by identifier only, because a published extension
version cannot be repointed and a version suffix is VS Code-specific.

### Host container management

The Dev Container deliberately has **no access to the host Docker daemon**: the socket is not
mounted ([ADR-0004](docs/adr/0004-dev-container-runtime.md)). To manage the host's containers from
VS Code, run the **Container Tools** extension (`ms-azuretools.vscode-containers`) on the **host**
side: install it in your host VS Code. [`.vscode/settings.json`](.vscode/settings.json) already pins
it to run locally via `remote.extensionKind`, so it keeps talking to the host engine even when this
folder is reopened in the container.

## Coding Agents

This Dev Container preinstalls the **Claude Code** and **Mistral Vibe** VS Code extensions (see
[`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json)); other agents (OpenAI Codex,
Cursor, OpenCode, GitHub Copilot) work too once you add them.

The rules every agent follows live in [`AGENTS.md`](AGENTS.md); that there is exactly one such file
is decided in [ADR-0001](docs/adr/0001-agent-governance-model.md). An agent that cannot read it
natively gets a pointer file instead. [`.claude/CLAUDE.md`](.claude/CLAUDE.md) is the one shipped
here; it carries the pointer and no rules of its own ([`AGENTS.md` §3](AGENTS.md#3-adr-rules)). It
exists for one gap only: Claude Code loads `CLAUDE.md`, not `AGENTS.md`, tracked upstream as
[anthropics/claude-code#34235](https://github.com/anthropics/claude-code/issues/34235). Once that
lands, delete the pointer file rather than keeping a second file to maintain.

The procedures those rules name (proposing an ADR, planning a feature, working its tickets
alone or in parallel, reviewing a change, revising the design, normalizing the record) are Agent
Skills under [`.agents/skills/`](.agents/skills/), the open format and the directory Codex,
Gemini CLI, and Cursor read
([ADR-0003](docs/adr/0003-procedures-as-skills.md)). An agent invokes one by name (`/review`) or
picks it up when the task matches its description. Claude Code reads `.claude/skills/` only, so
that directory holds one symlink per skill into `.agents/skills/`; a directory-level symlink is
not followed. The symlinks are pointers of the same kind as `CLAUDE.md`, tracked upstream as
[anthropics/claude-code#31005](https://github.com/anthropics/claude-code/issues/31005). Delete them
when that lands, and add one for every skill added meanwhile; the documentation check fails on a
skill without its pointer. What enforces a rule is what every agent hits
([`AGENTS.md` §6](AGENTS.md#6-project-rules)): the git hooks under [`.githooks/`](.githooks/),
the checks, and the repository settings below.

Agents work tickets in parallel without any one vendor's mechanism: a dispatching session claims
each ticket by its branch in a git worktree of its own and starts a fresh session there, with the
non-interactive command of whichever agent the project's harness ADR names, and the session works
the ticket through the `work-tickets` skill.

## Git hooks

Git does not version its hooks, so the repository keeps them under [`.githooks/`](.githooks/).
They hold for every agent and every human. The Dev Container enables them, with the merge
settings they rely on, on every start; outside it, run once per clone:

```sh
git config core.hooksPath .githooks
git config merge.ff false
git config pull.ff only
git config merge.adr-index.driver '.githooks/merge-adr-index %O %A %B %S %X %Y'
```

`merge.ff false` makes every merge a merge commit, as a `feat/` branch keeps them
([`AGENTS.md` §6](AGENTS.md#6-project-rules)); a fast-forward would make no commit and run no
hook. `pull.ff only` keeps a pull from then making a merge. Without the two settings the hooks
still hold: a fast-forward skips the documentation check of `pre-merge-commit`, and the push
still gates what it brought. The merge driver of the ADR index keeps the rows both sides add and
stops on its prose ([`.githooks/merge-adr-index`](.githooks/merge-adr-index)); without it, every
row two branches append conflicts.

| Hook | Refuses | Bypassed by |
|---|---|---|
| `reference-transaction` | moving the local `main` to a commit `origin/main` lacks: by a commit, merge, cherry-pick, reset, or `git branch -f`. It lets through `git merge --ff-only origin/main`, a reset to it, `git fetch origin main:main`, a write of `main` where it already is, and a deletion. Needs git 2.28 or later. | running git without the hooks (`git -c core.hooksPath=/dev/null`), or moving `origin/main` by hand; not by `--no-verify` |
| `pre-commit` | a commit on `main`; the conclusion of a conflicted merge while `scripts/check-all.sh --changed` is red | `--no-verify` |
| `pre-merge-commit` | a merge on `main`; a merge into any other branch while `scripts/check-docs.sh` is red | `--no-verify` |
| `pre-push` | a push to the remote's `main`, a deletion included; any other push while `scripts/check-all.sh --changed` is red, except one that only deletes, and one from a clean working tree whose tree the checks passed on before, as after a rework that only regroups | `--no-verify`, as the claim on a ticket does ([`AGENTS.md` §6](AGENTS.md#6-project-rules)); a tree written by hand into `checks-green-trees` of the git directory |

With `--changed`, the self-tests of what the branch left untouched since `main` are skipped; CI
runs every one before anything reaches `main`. A bypass is a deliberate act, not a default;
[`AGENTS.md` §6](AGENTS.md#6-project-rules) says when an agent may. Each refusal says what to do
instead.
[`scripts/test-git-hooks.sh`](scripts/test-git-hooks.sh) exercises every row.

## Integration branches

A change reaches `main` through a pull request. Several branches may integrate first on a
`feat/` branch, by local merges or by pull requests against it, and it stays the human's to
rework until its pull request into `main` merges ([`AGENTS.md` §6](AGENTS.md#6-project-rules)).
Each step waits for a human's instruction.

1. Start it at the remote's `main`: `git fetch origin`, then
   `git switch -c feat/<topic> origin/main`.
2. Merge a reviewed branch into it with its hand-off as the message, written to a file since an
   editor strips its headings: `git merge --no-ff -F .scratch/<hand-off>.md <branch>`. The
   documentation check gates the merge, and the push of step 6 runs the rest. One that stops on a
   conflict is concluded, once resolved and staged, with `git commit -F .scratch/<hand-off>.md`,
   and all the checks gate that commit.
3. Or merge it by a pull request against the `feat/` branch, so that its pipelines run first
   ([`AGENTS.md` §6](AGENTS.md#6-project-rules)). With both branches pushed, on the branch:
   `gh pr create --base feat/<topic> --body-file .scratch/<hand-off>.md`, and once it is green,
   `gh pr merge <number> --merge --body-file .scratch/<hand-off>.md`, never by the web interface,
   whose merge commit drops the hand-off. Then, on the `feat/` branch, `git fetch origin` and
   `git merge --ff-only origin/feat/<topic>`.
4. Rework it as often as needed: rebase it, regroup its commits, or drop what did not hold,
   with `git rebase -i --autosquash --rebase-merges`. `--autosquash` folds in the fixups a review
   leaves, as the `review` skill says; without `--rebase-merges` the merges flatten, and their
   hand-offs with them. A pull request still open against the `feat/` branch is rebased onto the
   rework.
5. After a rework, review the whole branch at its head
   ([`AGENTS.md` §5](AGENTS.md#5-quality-bar--definition-of-done)) and name that commit on the
   `Reviewed at:` line of the pull request of step 6, which
   [`review-binding.yml`](.github/workflows/review-binding.yml) reads on every push and edit.
6. Test it in the pipelines: `git push -u origin feat/<topic>`, then open a draft pull request,
   `gh pr create --draft --base main`, which runs them on every push against what the merge
   into `main` would produce. A branch kept without a pull request, such as an experiment, runs
   them by hand: `gh workflow run checks.yml --ref feat/<topic>`. A rework already pushed goes out
   with `git push --force-with-lease`.
7. Make the pull request ready: [`scripts/ready-for-main.sh`](scripts/ready-for-main.sh) refuses
   a fixup steps 4 to 6 left, takes up the remote's `main`, and numbers the draft ADRs
   ([ADR-0002](docs/adr/0002-decisions-verified-by-tests.md)); then the push it names and
   `gh pr ready`. What `main` gains later is taken up the same way, and an ADR it gains moves the
   branch's numbers on ([mechanics](docs/adr/README.md#mechanics)).
8. Once the pull request into `main` has merged as a merge commit, take it up and drop the branch:
   on it, [`scripts/drop-merged-branch.sh`](scripts/drop-merged-branch.sh) refuses a branch
   `origin/main` lacks, takes up the remote's `main`, and deletes the local branch; then the
   `git push origin --delete feat/<topic>` it names.

## Repository settings

Some of this project's rules cannot be enforced by files in the repository
([`AGENTS.md` §6](AGENTS.md#6-project-rules)). They are settings on the GitHub repository itself,
configured once by a maintainer and worth re-checking after a repository move or transfer:

- a **ruleset on `main`** that requires pull requests and allows the merge method **merge commit**
  only, since squash and rebase discard the merges a `feat/` branch carries and the commits it
  was reworked into ([`AGENTS.md` §6](AGENTS.md#6-project-rules)); requires the
  <!-- required-checks begin — compared with the workflow's jobs by scripts/check-docs.sh -->
  `docs`, `traceability`, `devcontainer`, `sensors`, and `shell`
  <!-- required-checks end -->
  jobs of the **Checks** workflow as required status checks (rulesets list checks by their job
  name) and a branch up to date with `main` before it merges, so that the checks see what the
  merge produces, such as two ADRs proposed in parallel under one number, and blocks force
  pushes and branch deletion;
- enable **require a review from Code Owners** on that ruleset, so the two artifacts only a human
  decides, [`docs/SPECIFICATION.md`](docs/SPECIFICATION.md) and everything under
  [`docs/adr/`](docs/adr/) ([`AGENTS.md` §3](AGENTS.md#3-adr-rules)), cannot be merged without the
  owner named in [`.github/CODEOWNERS`](.github/CODEOWNERS), which has to carry a real handle or
  team for the requirement to mean anything;
- enable **secret scanning with push protection**, the only mechanical backstop behind the secrets
  rule ([`AGENTS.md` §7](AGENTS.md#7-secrets)), which is otherwise carried by review alone;
- enable **private vulnerability reporting** (see [`SECURITY.md`](SECURITY.md)).

[`scripts/sensor-repo-settings.sh`](scripts/sensor-repo-settings.sh) reads these settings
through `gh` and reports each one that differs from this list. It runs on a human's
instruction ([ADR-0002](docs/adr/0002-decisions-verified-by-tests.md)).

## Template

- **Template release:** v0.5.0 — the release of [NUC](https://github.com/hivevm/nuc) this
  repository carries; a project moves the line when it takes up a later one
  ([ADR-0005](docs/adr/0005-template-releases.md)).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow (specification- and ADR-driven, small
reviewable changes) and [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) for the community standards we
expect of everyone taking part. Security issues: please follow [`SECURITY.md`](SECURITY.md) instead
of opening a public issue.

## License

Released under the BSD 3-Clause License. See [`LICENSE`](LICENSE).
