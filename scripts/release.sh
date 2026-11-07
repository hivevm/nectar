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
#   refuses a target SHA256SUMS does not name; the workflow builds every Linux target on
#   ubuntu-22.04 or ubuntu-22.04-arm, never on a *-latest runner; and the glibc floor package
#   enforces refuses a binary that needs more than glibc 2.35, in package too, which writes no
#   archive then (Verifies: ADR-0016);
# - package <target> builds nectar for the Rust target in release mode with the locked versions,
#   fails for a Linux target whose binary needs a glibc newer than 2.35, and packs the binary,
#   LICENSE and the NOTICE files of the crates into
#   nectar-<version>-<target>.tar.gz, or .zip for Windows, the version taken from GITHUB_REF_NAME;
#   the archive's name goes to GITHUB_ENV as ARCHIVE where that is set, and to stdout;
# - publish <directory> writes SHA256SUMS of the archives in the directory and creates the GitHub
#   release of GITHUB_REF_NAME in GITHUB_REPOSITORY with them, through gh and GH_TOKEN.
#
# Usage:
#     scripts/release.sh [package <target> | publish <directory>]
# Exit code 0 on success, 1 when a step fails, 2 on an unknown command.

set -euo pipefail

# The newest glibc a Linux archive may need: that of ubuntu-22.04, where it is built.
GLIBC_FLOOR=2.35

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# check — what every archive carries is in the tree.
check() {
  local errors=0 notices=(crates/*/NOTICE) job
  [[ -f LICENSE ]] || { echo "LICENSE is missing, which every archive carries" >&2; errors=1; }
  [[ -f "${notices[0]}" ]] \
    || { echo "no crates/*/NOTICE, whose concatenation every archive carries" >&2; errors=1; }
  grep -qx 'name = "nectar"' crates/cli/Cargo.toml \
    || { echo "crates/cli/Cargo.toml does not name the package nectar, which the release builds" >&2; errors=1; }
  # The job that publishes, the last of the workflow, attests and alone holds what that needs.
  job="$(awk '/^  release:$/ { found = 1 } found' .github/workflows/nectar-release.yml)"
  if ! grep -Eq '^ +- uses: actions/attest-build-provenance@[0-9a-f]{40} ' <<< "$job" \
    || ! grep -Eq '^ +subject-path: "dist/nectar-\*"$' <<< "$job" \
    || ! grep -Eq '^ +id-token: write( |$)' <<< "$job" \
    || ! grep -Eq '^ +attestations: write( |$)' <<< "$job"; then
    echo "the release job of nectar-release.yml does not attest the archives by actions/attest-build-provenance pinned to a commit, with id-token and attestations to write" >&2
    errors=1
  fi
  # Every Linux target is built on a runner with glibc 2.35, never on one that moves.
  local linux
  linux="$(awk '/^ +- target: / { target = $3 }
    /^ +os: / && target ~ /linux/ { print target, $2 }' .github/workflows/nectar-release.yml)"
  if [[ -z "$linux" ]] || grep -Evq ' ubuntu-22\.04(-arm)?$' <<< "$linux"; then
    echo "nectar-release.yml builds a Linux target on a runner other than ubuntu-22.04 or" \
      "ubuntu-22.04-arm: ${linux:-none}" >&2
    errors=1
  fi
  check_glibc || errors=1
  check_installer || errors=1
  ((errors == 0)) || { echo "Release check FAILED." >&2; return 1; }
  echo "Release check passed."
}

# check_glibc — the floor package enforces: versions up to GLIBC_FLOOR pass, newer ones fail, and
# on a Linux host the versions a binary of the host needs are read.
check_glibc() {
  local errors=0 version
  for version in 2.2.5 2.17 2.34 2.35; do
    glibc_within_floor "$version" || { echo "the glibc floor refuses $version" >&2; errors=1; }
  done
  for version in 2.36 2.39 2.43 3.0; do
    ! glibc_within_floor "$version" || { echo "the glibc floor allows $version" >&2; errors=1; }
  done
  if [[ "$(uname -s)" == Linux ]] && command -v objdump > /dev/null 2>&1; then
    version="$(glibc_needed "$(command -v ls)")"
    [[ "$version" =~ ^[0-9]+(\.[0-9]+)+$ ]] \
      || { echo "no glibc version read from $(command -v ls): '$version'" >&2; errors=1; }
  fi
  check_package_floor || errors=1
  return "$errors"
}

# check_package_floor — package itself, with stand-ins for cargo and objdump: a binary that needs
# glibc 2.43 fails and leaves no archive, one that needs 2.35 is packed.
check_package_floor() {
  local work errors=0 target=x86_64-floorcheck-linux-gnu out
  local archive=nectar-9.9.9-x86_64-floorcheck-linux-gnu.tar.gz
  work="$(mktemp -d)"
  # shellcheck disable=SC2064 # the paths are fixed now
  trap "rm -rf '$work' 'target/$target' '$archive'" RETURN
  printf '#!/bin/sh\nmkdir -p "target/%s/release" && echo stand-in > "target/%s/release/nectar"\n' \
    "$target" "$target" > "$work/cargo"
  # shellcheck disable=SC2016 # $NEEDED expands in the stand-in, not here
  printf '#!/bin/sh\necho "0000000000000000 DF *UND* 0000000000000000 (GLIBC_$NEEDED) memcpy"\n' \
    > "$work/objdump"
  chmod 755 "$work/cargo" "$work/objdump"
  try_package() {
    NEEDED="$1" PATH="$work:$PATH" GITHUB_REF_NAME=nectar-v9.9.9 GITHUB_ENV='' \
      bash scripts/release.sh package "$target" 2>&1
  }
  if out="$(try_package 2.43)" || [[ -e "$archive" ]] || [[ "$out" != *"needs glibc 2.43"* ]]; then
    echo "package did not refuse a binary that needs glibc 2.43: $out" >&2; errors=1
  fi
  if ! out="$(try_package 2.35)" || [[ ! -e "$archive" ]]; then
    echo "package did not pack a binary that needs glibc 2.35: $out" >&2; errors=1
  fi
  return "$errors"
}

# glibc_needed <binary> — the newest GLIBC_ symbol version the binary needs, empty for none; fails
# when objdump cannot read the binary.
glibc_needed() {
  local symbols
  symbols="$(objdump -T "$1")" || return 1
  grep -o 'GLIBC_[0-9][0-9.]*' <<< "$symbols" | sed 's/^GLIBC_//' | sort -uV | tail -n 1 || true
}

# glibc_within_floor <version> — whether a glibc version is GLIBC_FLOOR or older.
glibc_within_floor() {
  [[ "$(printf '%s\n%s\n' "$1" "$GLIBC_FLOOR" | sort -V | tail -n 1)" == "$GLIBC_FLOOR" ]]
}

# check_installer — release/install-nectar.sh against archives packed and summed by the functions
# package and publish use, served from a file:// base; the binary is a stand-in, so no release
# build is needed.
check_installer() {
  local work errors=0 target=x86_64-unknown-linux-gnu version=9.9.9 archive out
  work="$(mktemp -d)"
  # shellcheck disable=SC2064 # the directory is fixed now
  trap "rm -rf '$work'" RETURN
  mkdir -p "$work/base/nectar-v$version"
  printf '#!/bin/sh\necho "nectar %s"\n' "$version" > "$work/nectar"
  chmod 755 "$work/nectar"
  archive="$(pack "$target" "$version" "$work/nectar" "$work/base/nectar-v$version")"
  sums "$work/base/nectar-v$version"
  cp "$work/base/nectar-v$version/SHA256SUMS" "$work/SHA256SUMS"
  echo "$version" > "$work/VERSION"
  try_install() {
    NECTAR_RELEASE_BASE="file://$work/base" NECTAR_TARGET="$1" \
      sh release/install-nectar.sh "$work/SHA256SUMS" "$work/VERSION" "$2" 2>&1
  }

  if ! out="$(try_install "$target" "$work/ok")" || [[ "$(sh "$work/ok/nectar")" != "nectar $version" ]]; then
    echo "install-nectar.sh did not install a matching archive: $out" >&2; errors=1
  fi
  # Where the host is the one the archive serves, the installer finds the target itself.
  if [[ "$(uname -s)/$(uname -m)" == Linux/x86_64 && -e /lib64/ld-linux-x86-64.so.2 ]]; then
    if ! out="$(try_install "" "$work/detected")" || [[ ! -x "$work/detected/nectar" ]]; then
      echo "install-nectar.sh did not find the target of this host: $out" >&2; errors=1
    fi
  fi
  if out="$(try_install aarch64-apple-darwin "$work/other")" || [[ "$out" != *"names no"* ]]; then
    echo "install-nectar.sh did not refuse a target SHA256SUMS does not name: $out" >&2; errors=1
  fi
  # A VERSION that is not X.Y.Z: a leading zero, a part missing, a suffix, a second line, empty.
  local bad
  for bad in 09.9.9 9.9 9.9.9x $'9.9.9\n9.9.9' ''; do
    printf '%s\n' "$bad" > "$work/VERSION"
    if out="$(try_install "$target" "$work/bad")" || [[ "$out" != *"not X.Y.Z"* ]]; then
      echo "install-nectar.sh did not refuse VERSION '$bad': $out" >&2; errors=1
    fi
  done
  echo "$version" > "$work/VERSION"
  # A valid archive with another binary, under the checksum of the first.
  printf '#!/bin/sh\necho "replaced"\n' > "$work/nectar"
  rm "$work/base/nectar-v$version/$archive"
  pack "$target" "$version" "$work/nectar" "$work/base/nectar-v$version" > /dev/null
  if out="$(try_install "$target" "$work/altered")" || [[ -e "$work/altered/nectar" ]] \
    || [[ "$out" != *"checksum"* ]]; then
    echo "install-nectar.sh installed an archive altered after its checksum: $out" >&2; errors=1
  fi
  return "$errors"
}

# pack <target> <version> <binary> <directory> — packs the binary, LICENSE and the NOTICE files of
# the crates as nectar-<version>-<target>/ into nectar-<version>-<target>.tar.gz, or .zip for
# Windows, in the directory, and prints the archive's name.
pack() {
  local target="$1" version="$2" binary="$3" dir="$4" name archive stage
  name="nectar-$version-$target"
  stage="$(mktemp -d)"
  mkdir "$stage/$name"
  cp LICENSE "$stage/$name/"
  cat crates/*/NOTICE > "$stage/$name/NOTICE"
  if [[ "$target" == *windows* ]]; then
    cp "$binary" "$stage/$name/nectar.exe"
    archive="$name.zip"
    (cd "$stage" && 7z a "$archive" "$name" > /dev/null)
  else
    cp "$binary" "$stage/$name/nectar"
    archive="$name.tar.gz"
    tar czf "$stage/$archive" -C "$stage" "$name"
  fi
  mv "$stage/$archive" "$dir/"
  rm -rf "$stage"
  echo "$archive"
}

# sums <directory> — writes SHA256SUMS of the archives in the directory, one line
# '<sha256>  <archive>' each.
sums() {
  (
    cd "$1"
    if command -v sha256sum > /dev/null 2>&1; then
      sha256sum -- nectar-*
    else
      shasum -a 256 -- nectar-*
    fi
  ) > "$1/SHA256SUMS.partial"
  mv "$1/SHA256SUMS.partial" "$1/SHA256SUMS"
}

# package <target> — the archive of one target.
package() {
  local target="$1" ref="${GITHUB_REF_NAME:?the tag nectar-v* names the version}" binary archive
  cargo build --release --locked -p nectar --target "$target"
  binary="target/$target/release/nectar"
  [[ "$target" == *windows* ]] && binary="$binary.exe"
  if [[ "$target" == *-linux-gnu ]]; then
    local needed
    needed="$(glibc_needed "$binary")" \
      || { echo "objdump cannot read the glibc versions $binary needs" >&2; return 1; }
    glibc_within_floor "${needed:-0}" \
      || { echo "$binary needs glibc $needed, newer than the floor $GLIBC_FLOOR" >&2; return 1; }
  fi
  archive="$(pack "$target" "${ref#nectar-v}" "$binary" .)"
  [[ -n "${GITHUB_ENV:-}" ]] && echo "ARCHIVE=$archive" >> "$GITHUB_ENV"
  echo "$archive"
}

# publish <directory> — the GitHub release of the tag, with the archives and their checksums.
publish() {
  local dir="$1" ref="${GITHUB_REF_NAME:?the tag nectar-v* names the release}"
  sums "$dir"
  gh release create "$ref" "$dir"/* --repo "${GITHUB_REPOSITORY:?}" \
    --title "Nectar ${ref#nectar-v}" --generate-notes
}

case "${1:-}" in
  "") check ;;
  package) [[ $# -eq 2 ]] || { echo "usage: $0 package <target>" >&2; exit 2; }; package "$2" ;;
  publish) [[ $# -eq 2 ]] || { echo "usage: $0 publish <directory>" >&2; exit 2; }; publish "$2" ;;
  *) echo "usage: $0 [package <target> | publish <directory>]" >&2; exit 2 ;;
esac
