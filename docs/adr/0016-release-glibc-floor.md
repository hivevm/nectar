# ADR-0016: A Nectar release is archives built against glibc 2.35, their checksums and a provenance attestation for each, and the repository carries the installer that verifies them

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** `.github/workflows/nectar-release.yml`, `scripts/release.sh`, `release/install-nectar.sh` and the check that tests it, and every installer outside this repository that takes a Nectar release, such as the Dev Container Feature of `hivevm/features`
- **Supersedes:** [ADR-0013](0013-release-interface.md)

## Context

[ADR-0013](0013-release-interface.md) set the release interface, restated below. Its first
release, `nectar-v0.1.0`, built the Linux x86_64 archive on `ubuntu-latest`, which GitHub moved to
Ubuntu 26.04 from 19 October 2026. A binary needs at least the glibc of the host that built it,
and that one needs `GLIBC_2.43`: it fails to start on Debian 13 (glibc 2.41), Ubuntu 24.04 (2.39)
and every older base, that is on the usual Dev Container images. The arm64 archive, built on the
pinned `ubuntu-24.04-arm`, needs 2.39 and fails on Debian 12 (2.36) and Ubuntu 22.04 (2.35).
ADR-0013 named the targets but no glibc they need, so nothing caught it before the release.

The maintainer wants Nectar installable as a Dev Container Feature,
`ghcr.io/hivevm/features/nectar`,
next to `nuc` in `hivevm/features`. That repository takes a tool's release only after it checked
every binary against the release's `SHA256SUMS` and verified each binary's build-provenance
attestation against the tool's release workflow; its Feature then installs the version it names
and checks it against the checksums it carries. It never builds a tool itself.

A tag `nectar-v*` runs `nectar-release.yml`, which builds the command line for five targets with
`scripts/release.sh`, packs each into `nectar-<version>-<target>.tar.gz`, or `.zip` for Windows,
with `LICENSE` and the crates' `NOTICE` files, and publishes them with `SHA256SUMS`. It attests
nothing. Linux builds are `*-unknown-linux-gnu`, so they run on glibc images such as Debian and
Ubuntu, the usual bases of a Dev Container, and not on Alpine.

The forces:

- The archives carry the licence texts the bundled fonts and libraries require; a bare binary
  would leave them behind.
- What the archives are called, what `SHA256SUMS` holds and what signs the attestations becomes an
  interface other repositories rely on; a change to it breaks them without a test here noticing,
  unless the installer they use is tested here.
- [ADR-0007](0007-harness-one-agent-human-review.md) keeps releases with the maintainer, and
  [ADR-0011](0011-actions-pinned-by-sha.md) has every action referenced by its commit SHA.

## Decision

We will publish every Nectar release as its archives, a `SHA256SUMS` naming each, and a
build-provenance attestation for each archive signed by `nectar-release.yml`, and keep in this
repository, as `release/install-nectar.sh`, the installer that fetches the archive of one version
for the host and installs its binary only when it matches the checksums its caller carries.

- The interface, kept until a major release: the tag `nectar-vX.Y.Z`; the assets
  `nectar-X.Y.Z-<target>.tar.gz`, `.zip` for Windows, each holding `nectar-X.Y.Z-<target>/` with
  the binary, `LICENSE` and `NOTICE`; `SHA256SUMS` with one line `<sha256>  <asset>` per archive;
  one attestation per archive.
- The attestation is made in the job that publishes, by `actions/attest-build-provenance`, pinned
  by its commit SHA ([ADR-0011](0011-actions-pinned-by-sha.md)), with the `id-token` and
  `attestations` permissions that job alone holds.
- `release/install-nectar.sh` is POSIX `sh` with `curl` or `wget`, `sha256sum` or `shasum`, and
  `tar` or `unzip`, so that it runs in a container without bash. It reads the target from
  `uname`, fails on a host no archive serves, and deletes an archive whose checksum differs.
  `hivevm/features` copies it, as it copies `install-nuc.sh` from `hivevm/nuc-tools`.
- Linux archives stay `*-unknown-linux-gnu`; a host without glibc gets no archive. They are built
  on `ubuntu-22.04` and `ubuntu-22.04-arm`, never on a `*-latest` runner, so they need glibc 2.35
  at most and run on Debian 12 and 13 and Ubuntu 22.04 and later. `scripts/release.sh package`
  fails for a Linux target whose binary needs a newer glibc, so such an archive is never published.

**Out of scope:** the moment GitHub retires the Ubuntu 22.04 runners, which calls for a new floor
and a decision of its own; static `musl` builds for Alpine, which a decision of its own can add as
further targets; signing beyond the attestation, such as Sigstore bundles beside the archives;
`hivevm/features` itself, its Feature, its tests and the workflow that takes a release.

## Alternatives considered

- **Ubuntu 24.04 runners, glibc 2.39** — runners GitHub keeps longer. Not chosen: Debian 12 and
  Ubuntu 22.04, still common Dev Container bases, could not run the archive.
- **A fixed old glibc through `cargo-zigbuild`** — such as 2.28, whatever the runner. Not chosen:
  one more tool in the release chain for bases older than any the Feature serves.
- **Moving the tag `nectar-v0.1.0`** — rebuilding the same version. Not chosen: it is published
  and attested; the fix ships as `nectar-v0.1.1`.
- **Bare binaries, as `nuc` ships** — `nectar-<target>` next to the archives, attested instead of
  them. Not chosen: the licence texts would not travel with the binary that needs them.
- **Static `musl` builds as well** — the Feature on Alpine too. Not chosen now: `onig_sys` and
  `ring` build C code and need a cross toolchain, and the usual Dev Container images have glibc.
- **The installer only in `hivevm/features`** — fewer files here. Not chosen: a change to the
  archive layout would break the Feature, and no test here would notice.
- **Checksums without attestations** — what the release does today. Not chosen: `hivevm/features`
  refuses a release it cannot trace to the workflow that built it, and a checksum file replaced
  together with the archives proves nothing.
- **The tag `vX.Y.Z`, as `nuc` uses** — one form for both tools. Not chosen: the tag `nectar-v*`
  sets this repository's releases apart from the template's tags
  ([ADR-0005](0005-template-releases.md)), and the taking script reads a prefix as easily.

## Sources / Prior art

- GitHub, *Using artifact attestations to establish provenance for builds* —
  <https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations>:
  `actions/attest-build-provenance` and `gh attestation verify --signer-workflow`.
- SLSA, *Build provenance* — <https://slsa.dev/spec/v1.0/provenance>.
- GitHub, *runner images* — <https://github.com/actions/runner-images>: what `ubuntu-latest`
  points to, and the migration to Ubuntu 26 from 19 October 2026 (issue 14748).
- Development Containers, *Features* — <https://containers.dev/implementors/features/>: a Feature's
  `install.sh` runs as root at build time in the base image.
- `hivevm/features` and `hivevm/nuc-tools` — <https://github.com/hivevm/features>: the taking
  script `scripts/take-nuc-release.sh` and the installer `install-nuc.sh` it copies.

## Consequences

- Positive: a release can be traced to the workflow that built it, by `hivevm/features` and by
  anyone with `gh attestation verify`; the installer and the archive layout change in one commit,
  under one test.
- Negative / trade-offs: the release job holds two more permissions; the archive names and layout
  are an interface now, and changing them is a major release; Alpine and other hosts without glibc
  get no Feature; the Linux runners are pinned and age, and their retirement forces a new
  floor.
- Follow-ups: the first release under this interface is `nectar-v0.1.1`; `hivevm/features`
  takes it, not `nectar-v0.1.0`.

## Enforcement

A check under `scripts/`, run by `scripts/check-all.sh`, packs an archive for the host as
`scripts/release.sh package` does, writes its `SHA256SUMS`, and runs `release/install-nectar.sh`
against it from a `file://` base: the binary is installed and runs; with an archive altered after
the checksum it fails and installs nothing; with a host target no line names it fails. The check
also fails when `nectar-release.yml` does not attest the archives with
`actions/attest-build-provenance` by a commit SHA, or builds a Linux target on a runner other
than `ubuntu-22.04` or `ubuntu-22.04-arm`. `scripts/release.sh package` fails for a Linux target
whose binary needs a glibc newer than 2.35. All cite this ADR with a `Verifies:` marker.
