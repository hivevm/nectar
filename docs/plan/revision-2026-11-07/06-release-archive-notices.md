# Ticket: A release archive carries the notices and licences of everything the binary bundles

## What to build

A Nectar release archive names every third-party work compiled into the binary, with the licence
texts its notices point to. Today `scripts/release.sh` packs `cat crates/*/NOTICE` beside the
binary: that leaves out `crates/html/theme/NOTICE`, which covers fonts and scripts the website's
theme embeds (IBM Plex Mono under a Reserved Font Name, Inter, Victor Mono, Ubuntu Sans, Material
Icons, paged.js), and `crates/pdf/NOTICE` lacks Material Icons, which the PDF embeds. No archive
carries the licence texts. The convention for bundled fonts of the design revision 2026-11-07
asks for a line in the crate's NOTICE, which the tree does not meet yet. Found by the review of
the follow-ups to PR #14.

## Acceptance criteria

- [ ] Given a release built by `scripts/release.sh`, when its archive is unpacked, then its
  NOTICE names every work the binary bundles, the theme's included, and the licence texts they
  refer to are in the archive.
- [ ] Given a bundled font or script added to a crate without a NOTICE line, when the checks run,
  then a check fails and names it, as far as a check can tell what is bundled (the files
  `include_bytes!` and `include_str!` name).
- [ ] Given `crates/pdf/NOTICE`, when it is read, then it names Material Icons.

## Read first

- Touches: `scripts/release.sh`, `crates/*/NOTICE`, `crates/html/theme/NOTICE`,
  `crates/pdf/src/world.rs`, `crates/html/src/lib.rs`, `release/` and its check
- ADRs: ADR-0016 (the release interface), ADR-0002 (checks as scripts)
- Conventions: the entry on bundled fonts in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: licence obligations are the maintainer's to judge, and which licence texts a
release must carry may need a reading of each licence.
