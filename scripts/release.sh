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
  ((errors == 0)) || { echo "Release check FAILED." >&2; return 1; }
  echo "Release check passed."
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
