# ADR-0021: A ZIP writer of our own packs the EPUB container, on `miniz_oxide` and `crc32fast`

- **Status:** 🟢 accepted
- **Date:** 2026-11-08
- **Deciders:** Markus Brigl
- **Applies to:** the ZIP writer in `crates/html/`, the dependencies on `miniz_oxide` and `crc32fast` in `crates/html/Cargo.toml`, and the tests that verify the container

## Context

[ADR-0020](0020-epub-renderer.md) leaves open how the container is packed. An EPUB
is a ZIP archive under rules of its own (EPUB 3.3, sections 4.3.2 and 4.3.3): only stored and
Deflate entries, no encryption, no extra field, `mimetype` as the first file and not compressed,
and a `version needed to extract` of 10, 20 or 45. ZIP64 is allowed and is to be used only where
the content requires it.

Three forces bear on the choice:

- **Determinism (Q-3).** The same sources must give the same bytes, so timestamps and the order of
  the entries must be fixed.
- **Portability (Q-1, G-8).** The packing runs in `nectar-html`, which links for
  `wasm32-unknown-unknown` and must not read or create files
  ([ADR-0009](0009-ports-and-adapters-technology.md)).
- **Scale.** A document has up to a few hundred pages, by the specification's assumptions. The
  output needs writing only, of far less than 4 GiB.

`cargo tree` for `nectar-wasm-link` on that target shows `miniz_oxide` and `crc32fast` already in
the module, through `flate2` and `png`, so this adds no new code to the dependency graph.

## Decision

We will pack the EPUB container with a ZIP writer of our own in `nectar-html`, on `miniz_oxide` and
`crc32fast`.

- The writer must store `mimetype` as the first entry, without compression.
- The writer must compress every other entry with Deflate at one fixed level.
- The writer must write the other entries in the order of their sorted names.
- The writer must set every timestamp to 1980-01-01 00:00:00.
- The writer must not write an extra field, encryption, or ZIP64 structures.
- The writer must mark entry names as UTF-8.
- The renderer must report an error diagnostic for an entry or a container that ZIP without ZIP64
  cannot hold.

**Out of scope:** reading ZIP archives; any use of the writer beyond the EPUB container.

## Alternatives considered

- **The `zip` crate, 9.4.2** — the established library, under MIT. Not chosen: its manifest makes
  `getrandom` a dependency that is not optional, built with `wasm_js` on WebAssembly, for an output
  that needs no randomness. Its `deflate` feature brings `zopfli` and `zlib-rs` as well. An archive
  that must have a fixed layout, without extra fields, would still have to be tested byte by byte.
- **Stored entries only** — EPUB 3.3 allows them, and only `crc32fast` remains as a dependency; no
  time goes to compression under Q-4. Not chosen: the text of a book is several times larger
  without compression, and Deflate costs one call. It stays the fallback if the first ticket shows
  that compression endangers Q-4.
- **`flate2` directly** — the usual interface to Deflate, already in the graph. Not chosen: it
  adds a layer of streams and backends over the `miniz_oxide` it calls, for one function on a
  buffer.

## Sources / Prior art

- W3C, *EPUB 3.3*, sections 4.3.2 and 4.3.3 — <https://www.w3.org/TR/epub-33/>: the rules of the
  OCF ZIP container.
- `zip` 9.4.2 on crates.io, its features and dependencies, read on 2026-11-08:
  <https://docs.rs/crate/zip/latest/source/Cargo.toml.orig>.
- `cargo tree -p nectar-wasm-link --target wasm32-unknown-unknown`, on this repository's lockfile.

## Consequences

- Positive: full control of the bytes (Q-3). No new crate enters the WebAssembly module (Q-1). The
  writer is small enough to read in one sitting, and `cargo deny` already holds the licences of
  both dependencies.
- Negative / trade-offs: the project owns a small implementation of a file format, with its tests.
  A document over the limits of ZIP without ZIP64 cannot give an EPUB, which the assumptions of the
  specification do not expect. `miniz_oxide` output may change between its versions, which Q-3
  accepts, since it compares two builds of one binary.
- Follow-ups: none.

## Enforcement

`crates/html/tests/epub.rs` must cite this ADR with a `Verifies:` marker. It reads the bytes of a
built container and fails in these cases:

- `mimetype` is not the first entry, is compressed, or differs from `application/epub+zip`;
- an entry uses a method other than 0 or 8, or carries an extra field or the encryption flag;
- a timestamp differs from the fixed one, or the other entries are not in sorted order;
- a checksum or an inflated entry disagrees with the source;
- two builds of one project differ in any byte.

`miniz_oxide` is a regular dependency of `nectar-html`, so the test inflates with it. EPUBCheck, run
by the check of [ADR-0020](0020-epub-renderer.md), judges the container from
outside.
