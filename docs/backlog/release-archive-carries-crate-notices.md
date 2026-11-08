# A release archive carries the notices of the Rust crates the binary compiles in

Observed: the release archive names the works the crates bundle themselves (fonts, scripts,
images, Typst, giallo) and ends its NOTICE with their licence texts. The binary also compiles
in about 400 Rust crates. They are licensed under MIT, Apache-2.0, BSD-2/3-Clause, ISC,
Unicode-3.0 and Zlib, and most of these licences ask for their copyright and licence notice in a
binary distribution as well. The archive carries none of them.

Some crates bundle works of their own, which a tool over the licence files of the crates may
miss: typst-assets 0.15.1, through its default feature, ships a NOTICE for ICC profiles (CC0)
and HTML and MathML data (W3C, CC-BY), and Apache-2.0 section 4(d) asks for the NOTICE-type
files of Typst; giallo's builtin.zst holds more than 220 grammars and about 60 themes of
shikijs.

Decided by the maintainer on 2026-11-08 (ticket revision-2026-11-07/06): the bundled works now,
the crates later, through an ADR that chooses the tool. It changes the release interface
(ADR-0016) and the supply chain. Candidates: cargo-about, which generates the notices from the
licence files of the crates under a template, or a script of this repository over
`cargo metadata` and the sources in the registry.

Where: `pack` and `archive_notice` in scripts/release.sh, ADR-0016, deny.toml (the
licences allowed).
