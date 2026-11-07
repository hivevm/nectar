#!/usr/bin/env bash
#
# Builds, packages and publishes the Nectar command line for a tag nectar-v*.
#
# The one script behind .github/workflows/nectar-release.yml, a workflow that publishes rather than
# checks and that scripts/check-all.sh names as no gate; check-all.sh still runs this script, so
# what the release packs is checked before a tag is pushed (ADR-0002). The release itself
# is the maintainer's (ADR-0007): the workflow runs it on a pushed tag, never this repository's
# hooks.
#
# - without a command, the check scripts/check-all.sh runs: the files every archive carries are
#   there — LICENSE, at least one crates/*/NOTICE — and the package nectar is the command line;
#   the workflow attests every archive by an action pinned to a commit; and release/install-nectar.sh
#   installs an archive packed as package packs it, refuses one altered after its checksum, and
#   refuses a target SHA256SUMS does not name (Verifies: ADR-draft-release-interface);
# - package <target> builds nectar for the Rust target in release mode with the locked versions,
#   and packs the binary, LICENSE and the NOTICE files of the crates into
#   nectar-<version>-<target>.tar.gz, or .zip for Windows, the version taken from GITHUB_REF_NAME;
#   the archive's name goes to GITHUB_ENV as ARCHIVE where that is set, and to stdout;
# - publish <directory> writes SHA256SUMS of the archives in the directory and creates the GitHub
#   release of GITHUB_REF_NAME in GITHUB_REPOSITORY with them, through gh and GH_TOKEN.
#
# Usage:
#     scripts/release.sh [package <target> | publish <directory>]
# Exit code 0 on success, 1 when a step fails, 2 on an unknown command.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# check — what every archive carries is in the tree.
check() {
  local errors=0 notices=(crates/*/NOTICE)
  [[ -f LICENSE ]] || { echo "LICENSE is missing, which every archive carries" >&2; errors=1; }
  [[ -f "${notices[0]}" ]] \
    || { echo "no crates/*/NOTICE, whose concatenation every archive carries" >&2; errors=1; }
  grep -qx 'name = "nectar"' crates/cli/Cargo.toml \
    || { echo "crates/cli/Cargo.toml does not name the package nectar, which the release builds" >&2; errors=1; }
  if ! grep -Eq '^ +- uses: actions/attest-build-provenance@[0-9a-f]{40} ' .github/workflows/nectar-release.yml \
    || ! grep -Eq '^ +subject-path: "dist/nectar-\*"$' .github/workflows/nectar-release.yml; then
    echo "nectar-release.yml does not attest the archives by actions/attest-build-provenance pinned to a commit" >&2
    errors=1
  fi
  check_installer || errors=1
  ((errors == 0)) || { echo "Release check FAILED." >&2; return 1; }
  echo "Release check passed."
}

# check_installer — release/install-nectar.sh against archives packed as package packs them,
# served from a file:// base; the binary is a stand-in, so no release build is needed.
check_installer() {
  local work errors=0 target=x86_64-unknown-linux-gnu version=9.9.9 name out
  work="$(mktemp -d)"
  # shellcheck disable=SC2064 # the directory is fixed now
  trap "rm -rf '$work'" RETURN
  name="nectar-$version-$target"
  mkdir -p "$work/base/nectar-v$version" "$work/pack/$name"
  printf '#!/bin/sh\necho "nectar %s"\n' "$version" > "$work/pack/$name/nectar"
  cp LICENSE "$work/pack/$name/"
  tar czf "$work/base/nectar-v$version/$name.tar.gz" -C "$work/pack" "$name"
  (cd "$work/base/nectar-v$version" && sha256sum -- * > "$work/SHA256SUMS")
  echo "$version" > "$work/VERSION"
  try_install() {
    NECTAR_RELEASE_BASE="file://$work/base" NECTAR_TARGET="$1" \
      sh release/install-nectar.sh "$work/SHA256SUMS" "$work/VERSION" "$2" 2>&1
  }

  if ! out="$(try_install "$target" "$work/ok")" || [[ "$(sh "$work/ok/nectar")" != "nectar $version" ]]; then
    echo "install-nectar.sh did not install a matching archive: $out" >&2; errors=1
  fi
  if out="$(try_install aarch64-apple-darwin "$work/other")" || [[ "$out" != *"names no"* ]]; then
    echo "install-nectar.sh did not refuse a target SHA256SUMS does not name: $out" >&2; errors=1
  fi
  # A valid archive with another binary, under the checksum of the first.
  printf '#!/bin/sh\necho "replaced"\n' > "$work/pack/$name/nectar"
  tar czf "$work/base/nectar-v$version/$name.tar.gz" -C "$work/pack" "$name"
  if out="$(try_install "$target" "$work/altered")" || [[ -e "$work/altered/nectar" ]]; then
    echo "install-nectar.sh installed an archive altered after its checksum: $out" >&2; errors=1
  fi
  return "$errors"
}

# package <target> — the archive of one target.
package() {
  local target="$1" ref="${GITHUB_REF_NAME:?the tag nectar-v* names the version}"
  local name="nectar-${ref#nectar-}-$target" archive
  cargo build --release --locked -p nectar --target "$target"
  mkdir "$name"
  cp LICENSE "$name/"
  cat crates/*/NOTICE > "$name/NOTICE"
  if [[ "$target" == *windows* ]]; then
    cp "target/$target/release/nectar.exe" "$name/"
    archive="$name.zip"
    7z a "$archive" "$name" > /dev/null
  else
    cp "target/$target/release/nectar" "$name/"
    archive="$name.tar.gz"
    tar czf "$archive" "$name"
  fi
  [[ -n "${GITHUB_ENV:-}" ]] && echo "ARCHIVE=$archive" >> "$GITHUB_ENV"
  echo "$archive"
}

# publish <directory> — the GitHub release of the tag, with the archives and their checksums.
publish() {
  local dir="$1" ref="${GITHUB_REF_NAME:?the tag nectar-v* names the release}"
  (cd "$dir" && sha256sum -- * > SHA256SUMS)
  gh release create "$ref" "$dir"/* --repo "${GITHUB_REPOSITORY:?}" \
    --title "Nectar ${ref#nectar-v}" --generate-notes
}

case "${1:-}" in
  "") check ;;
  package) [[ $# -eq 2 ]] || { echo "usage: $0 package <target>" >&2; exit 2; }; package "$2" ;;
  publish) [[ $# -eq 2 ]] || { echo "usage: $0 publish <directory>" >&2; exit 2; }; publish "$2" ;;
  *) echo "usage: $0 [package <target> | publish <directory>]" >&2; exit 2 ;;
esac
