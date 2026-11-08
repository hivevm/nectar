#!/usr/bin/env bash
#
# Check: the EPUB of the user guide and those of a fixture conform to EPUB 3.3, as EPUBCheck judges
# them; any error or warning fails the check. The fixture holds every case a finding of EPUBCheck
# has shown so far, each of which must build and give a valid publication.
#
# Verifies: G-13, ADR-0020, ADR-0021
#
# Needs cargo, curl, java 8 or later, sha256sum and unzip on PATH. EPUBCheck is pinned by version
# and checksum; its archive stays under target/, so a later run needs no network.
#
#     scripts/check-epub.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

EPUBCHECK_VERSION=5.3.0
EPUBCHECK_SHA256=6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5
TOOL_DIR="target/epubcheck-$EPUBCHECK_VERSION"
ARCHIVE="$TOOL_DIR/epubcheck.zip"
JAR="$TOOL_DIR/epubcheck-$EPUBCHECK_VERSION/epubcheck.jar"

if ! command -v java >/dev/null; then
  echo "check-epub: java is not on PATH; EPUBCheck needs it" >&2
  exit 1
fi

mkdir -p "$TOOL_DIR"
if [[ ! -f "$ARCHIVE" ]]; then
  # Through a second name, so that an interrupted download leaves no archive behind.
  curl -fsSL --retry 5 --retry-delay 20 --retry-all-errors -o "$ARCHIVE.part" \
    "https://github.com/w3c/epubcheck/releases/download/v$EPUBCHECK_VERSION/epubcheck-$EPUBCHECK_VERSION.zip"
  mv "$ARCHIVE.part" "$ARCHIVE"
fi
echo "$EPUBCHECK_SHA256  $ARCHIVE" | sha256sum -c - >/dev/null
unzip -q -o "$ARCHIVE" -d "$TOOL_DIR"

out="$(mktemp -d)"
trap 'rm -rf -- "$out"' EXIT

fixture="$out/fixture"
mkdir -p "$fixture/sub"
cp docs/guide/img/logo.png "$fixture/logo.png"
cp docs/guide/img/logo.png "$fixture/not-a-jpeg.jpg"
printf 'BM' > "$fixture/old.bmp"
printf 'x' > "$fixture/data.csv"
svg='<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"'
printf '%s/>\n' "$svg" > "$fixture/fine.svg"
printf '%s><script>x()</script></svg>\n' "$svg" > "$fixture/scripted.svg"
printf '%s onload="x()"/>\n' "$svg" > "$fixture/event.svg"
printf '%s><image href="https://example.org/r.png"/></svg>\n' "$svg" > "$fixture/remote.svg"
printf '%s><image href="other.png"/></svg>\n' "$svg" > "$fixture/local.svg"
printf '%s><a href="other.html"><rect/></a></svg>\n' "$svg" > "$fixture/linked.svg"
printf '%s><a href="https://example.org/"><rect width="1" height="1"/></a></svg>\n' "$svg" > "$fixture/web-linked.svg"
printf '<?xml-stylesheet href="https://example.org/s.css"?>%s/>\n' "$svg" > "$fixture/styled.svg"
printf '%s><rect width="1" height="1" style="fill:URL(https://example.org/#a)"/></svg>\n' "$svg" > "$fixture/upper.svg"
printf '%s><foreignObject><img xmlns="http://www.w3.org/1999/xhtml" src="https://example.org/r.png" alt=""/></foreignObject></svg>\n' "$svg" > "$fixture/foreign.svg"
printf '<svg' > "$fixture/broken.svg"

cat > "$fixture/nectar.toml" <<'TOML'
title = "Fixture & more"
html = true

[[document]]
source = "main.md"
path = "main"

[document.epub]
enabled = true
language = "de-AT"
cover_image = "logo.png"

# The PDF refuses what these two documents hold, which the EPUB has to leave out by itself.
[document.print]
pdf = false

[[document]]
source = "images.md"
path = "images"

[document.print]
pdf = false

[document.epub]
enabled = true
language = "en"

[[document]]
source = "bare.md"
path = "bare"
contents = false

[document.epub]
enabled = true
language = "en"
identifier = "urn:isbn:9780000000000"

[[document]]
source = "untitled.md"
path = "untitled"

[document.epub]
enabled = true
language = "en"
TOML

cat > "$fixture/main.md" <<'MD'
# Main & "more"

See [the part](sub/part.md), [details](#details), [a section](sub/part.md#details), and
[the web](https://example.org/?a=1&b=2). A note[^one], and [a file](data.csv).

Links without a target: [nowhere](#nope), [absolute](/abs/path), [outside](../../o.txt),
[spaced](<https://example.org/b c>), [empty](), [bad escape](https://example.org/%zz).

A chapter without a heading: [second](sub/second.md); a heading in a list: [inner](sub/part.md#inner).

[A link[^two]](https://example.org/), with a footnote in its text.

Line one
line two

***

<div class="raw">unclosed<br>raw</div> inline <b>bold</b>, @settings@ a symbol

- [x] done
- [ ] open
    1. nested
    2. list

> [!NOTE]
> An alert

| A | B |
|:--|--:|
| 1 | `x<y` |

```rust
fn main() { println!("<&>"); }
```

```
plain <code> & more
```

![An image](logo.png "With a title"){width=10}
![From the web](https://example.org/offline.png)
![A data URI](data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7)

## fn 1

A heading whose id a footnote's could repeat.

## Repeated {#again}

## Repeated too {#again}

## Taken {#_fn-1}

## Percent {#100%}

## Escape {#a%20b}

A link to [the percent](#100%), and URLs with a mistyped host or port:
[two dots](https://example..org/), [big port](https://example.org:65536), [address](https://999.1.1.1/).

URLs that only look fine: [brackets](https://example.org/?x[]=1), [two hashes](https://example.org/#a#b),
[a file](file:///etc/x), [data](data:text/html,x), [script](javascript:x()), [other](foo:bar),
[bad port](https://example.org:port/), and one that is: [fine](https://example.org:8080/a_(b)?c=d#e).

An image with a width that is no length: ![a width](logo.png){width="url(https://example.org/)"}.

A footnote with a heading[^five].

[^five]: ## Heading in a note

[^one]: The note, with [a link](#details) and another note[^three].

[^two]: Note in a link.

[^three]: The note inside a note.

[Missing](missing.md)

[Part](sub/part.md)

[Second](sub/second.md)
MD

cat > "$fixture/sub/part.md" <<'MD'
# Part

## Details

Text[^four], again[^one].

- ## Inner

  In a list.

```railroad
rule ::= 'a' ;
```

[^four]: Note four.

[^one]: A note of this file.

A heading in a note, with the id of a chapter below[^six].

[^six]: ## In a note {#silent}

[Nested](nested.md)

[Silent](silent.md)

[Odd id](odd.md)
MD
printf '# Nested\n\nBack to [the start](../main.md).\n' > "$fixture/sub/nested.md"
printf 'A nested chapter without a heading.\n' > "$fixture/sub/silent.md"
printf '# Odd {#_fnord}\n\nBack to [myself](odd.md).\n' > "$fixture/sub/odd.md"
printf 'No heading here, just a note[^a].\n\n[^a]: The note.\n' > "$fixture/sub/second.md"
printf 'A document without any heading.\n' > "$fixture/bare.md"
printf '# <b></b>\n\nA title without any text.\n' > "$fixture/untitled.md"
cat > "$fixture/images.md" <<'MD'
# Images

![fine](fine.svg) ![old](old.bmp) ![not a jpeg](not-a-jpeg.jpg)

![scripted](scripted.svg) ![event](event.svg) ![remote](remote.svg) ![local](local.svg)

![broken](broken.svg)

![linked](linked.svg) ![web linked](web-linked.svg) ![styled](styled.svg) ![upper](upper.svg)

![foreign](foreign.svg)
MD
# A form feed and an escape, as text pasted from a terminal gives them.
printf '\n## Control\f characters\n\n%s\nterminal \033[0m output\n%s\n' '```' '```' >> "$fixture/images.md"

build() {
  if ! cargo run --quiet -p nectar -- --offline -C "$1" build --output "$2" 2>"$out/build.log"; then
    cat "$out/build.log" >&2
    echo "check-epub: the build of $1 failed" >&2
    exit 1
  fi
}
build docs/guide "$out/guide"
build "$fixture" "$out/fixture-out"

checked=0
while IFS= read -r -d '' epub; do
  if ! java -jar "$JAR" "$epub" --failonwarnings >"$out/report" 2>&1; then
    cat "$out/report" >&2
    echo "check-epub: EPUBCheck rejects ${epub#"$out"/}" >&2
    exit 1
  fi
  checked=$((checked + 1))
done < <(find "$out/guide" "$out/fixture-out" -name '*.epub' -print0)

if [[ "$checked" -ne 5 ]]; then
  echo "check-epub: expected 5 EPUBs, found $checked" >&2
  exit 1
fi
echo "EPUB check passed: EPUBCheck $EPUBCHECK_VERSION accepts $checked publications."
