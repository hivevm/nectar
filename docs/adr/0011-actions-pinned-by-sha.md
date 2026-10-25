# ADR-0011: The workflows reference every action by its full commit SHA, kept current by Dependabot

- **Status:** 🟡 proposed
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** every `uses:` line under `.github/workflows/`, `.github/dependabot.yml`, and `scripts/check-all.sh`

## Context

The [threats](../SPECIFICATION.md#threats--forbidden-actions) name the integrity of the released
binaries as an asset. `nectar-release.yml` builds and publishes them with a token that can write
releases, and the workflows run third-party actions: `Swatinem/rust-cache` by major tag and
`dtolnay/rust-toolchain` by the branch `stable`. A tag or a branch can be repointed, and the code
it then names runs in CI before anyone sees a diff.

The catalogue's *Action references* fits a full commit SHA for third-party actions, a release
pipeline, or a token with write access, all of which hold here. Its *Does not fit* warns of SHA
pins without a tool that updates them; Dependabot is that tool.

## Decision

We will reference every action of the workflows by its full commit SHA with the version as a
comment, `actions/*` included, and let Dependabot's `github-actions` updates raise each new
version as a pull request.

**Out of scope:** the Rust toolchain version that `dtolnay/rust-toolchain` installs; Dependabot for
Cargo dependencies; where secrets live, since the specification names no credential beyond the
workflow's own token.

## Alternatives considered

- **Major version tag** — as the template ships; fixes within a major arrive without a pull
  request. Not chosen: a repointed tag of a third-party action would run in the release pipeline
  unseen, and `dtolnay/rust-toolchain@stable` is a branch, not a tag.
- **Tags for `actions/*`, SHAs for everything else** — a mixed rule whose check needs a list of
  trusted owners. Not chosen: one rule for all is simpler to hold and to check.

## Sources / Prior art

- GitHub, *Security hardening for GitHub Actions* —
  <https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions>:
  a full SHA is the only immutable reference; a tag can be moved.
- GitHub Advisory GHSA-mrrh-fwg8-r2c3 / CVE-2025-30066 —
  <https://github.com/advisories/GHSA-mrrh-fwg8-r2c3>: the tags of `tj-actions/changed-files`
  repointed to a commit that printed the runner's secrets.
- GitHub, *Keeping your actions up to date with Dependabot* —
  <https://docs.github.com/en/code-security/dependabot/working-with-dependabot/keeping-your-actions-up-to-date-with-dependabot>.

## Consequences

- Positive: every change of the code that runs in CI and in the release is a reviewed diff.
- Negative / trade-offs: fixes within a major arrive as pull requests the maintainer merges, not by
  themselves; the SHAs make the workflows harder to read without their comments.
- Follow-ups: the comment of `.github/dependabot.yml`, which describes the tag form today.

## Enforcement

`scripts/sensor-action-refs.sh --require sha` runs in `scripts/check-all.sh` and in the `sensors`
job of `checks.yml`, beside a comment that cites this ADR with a `Verifies: ADR-0011` marker; it
fails on any action not referenced by a full SHA and on a missing `github-actions` entry in
`.github/dependabot.yml`.
