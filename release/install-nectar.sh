#!/bin/sh
#
# Installs the nectar of one release, verified against the checksums the caller carries
# (ADR-draft-release-interface). The Dev Container Feature of hivevm/features runs a copy of it
# with the SHA256SUMS and the VERSION its release commit holds; an archive whose checksum differs
# is deleted and the installation fails.
#
# POSIX sh, curl or wget, sha256sum or shasum, and tar or unzip, so that it runs in a container
# without bash. Usage:
#     install-nectar.sh <SHA256SUMS> <VERSION file> <destination directory>
# NECTAR_RELEASE_BASE replaces https://github.com/hivevm/nectar/releases/download, the base the
# archive is fetched from, as scripts/release.sh does with a file:// URL; NECTAR_TARGET replaces
# the target read from uname. Neither can pass an archive the checksums do not name.
# Exit code 0 when nectar is installed, 1 otherwise.

set -eu

fail() { echo "install-nectar: $1" >&2; exit 1; }

[ $# -eq 3 ] || fail "usage: install-nectar.sh <SHA256SUMS> <VERSION file> <destination directory>"
sums="$1" version_file="$2" dest="$3"
if [ ! -f "$sums" ] || [ ! -f "$version_file" ]; then
  fail "$sums or $version_file is missing — no release of nectar is named"
fi
version="$(tr -d '[:space:]' < "$version_file")"
printf '%s\n' "$version" | grep -Eqx '(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)' \
  || fail "VERSION holds '$version', not X.Y.Z"

target="${NECTAR_TARGET:-}"
if [ -z "$target" ]; then
  os="$(uname -s)" arch="$(uname -m)"
  case "$os/$arch" in
    Linux/x86_64 | Linux/amd64) target=x86_64-unknown-linux-gnu ;;
    Linux/aarch64 | Linux/arm64) target=aarch64-unknown-linux-gnu ;;
    Darwin/x86_64) target=x86_64-apple-darwin ;;
    Darwin/arm64) target=aarch64-apple-darwin ;;
    MINGW*/x86_64 | MSYS*/x86_64 | CYGWIN*/x86_64) target=x86_64-pc-windows-msvc ;;
    *) fail "no release archive for $os on $arch" ;;
  esac
  # The Linux archives need glibc: its loader is there, whatever else is installed beside it.
  case "$target" in
    x86_64-unknown-linux-gnu) loader=/lib64/ld-linux-x86-64.so.2 ;;
    aarch64-unknown-linux-gnu) loader=/lib/ld-linux-aarch64.so.1 ;;
    *) loader= ;;
  esac
  if [ -n "$loader" ] && [ ! -e "$loader" ]; then
    fail "the release builds nectar for glibc, and this host has no $loader, as Alpine has none"
  fi
fi
name="nectar-$version-$target"
binary=nectar
case "$target" in
  *windows*) asset="$name.zip" binary=nectar.exe ;;
  *) asset="$name.tar.gz" ;;
esac

expected="$(awk -v name="$asset" '$2 == name || $2 == "*" name { print $1 }' "$sums")"
[ -n "$expected" ] || fail "SHA256SUMS names no $asset"

base="${NECTAR_RELEASE_BASE:-https://github.com/hivevm/nectar/releases/download}"
url="$base/nectar-v$version/$asset"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
if command -v curl > /dev/null 2>&1; then
  curl -fsSL --proto '=https,file' -o "$work/$asset" "$url" || fail "could not fetch $url"
elif command -v wget > /dev/null 2>&1; then
  wget -q -O "$work/$asset" "$url" || fail "could not fetch $url"
else
  fail "neither curl nor wget is on the PATH"
fi

# Through stdin, so that no path reaches the output: a backslash in one, as on Windows, makes
# sha256sum escape the line with a leading backslash.
if command -v sha256sum > /dev/null 2>&1; then
  actual="$(sha256sum < "$work/$asset" | awk '{ print $1 }')"
elif command -v shasum > /dev/null 2>&1; then
  actual="$(shasum -a 256 < "$work/$asset" | awk '{ print $1 }')"
else
  fail "neither sha256sum nor shasum is on the PATH"
fi
[ "$actual" = "$expected" ] \
  || fail "$asset has the checksum $actual, not the $expected of release $version — not installed"

case "$asset" in
  *.zip) unzip -q "$work/$asset" -d "$work" || fail "could not unpack $asset" ;;
  *) tar -xzf "$work/$asset" -C "$work" || fail "could not unpack $asset" ;;
esac
[ -f "$work/$name/$binary" ] || fail "$asset holds no $name/$binary"

mkdir -p "$dest"
chmod 755 "$work/$name/$binary"
mv "$work/$name/$binary" "$dest/$binary"
echo "install-nectar: nectar $version installed as $dest/$binary"
