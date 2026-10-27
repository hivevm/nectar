#!/usr/bin/env bash
#
# Builds, tests and lints Nectar (ADR-0008).
#
# The one script behind the Build, Test & Run section of README.md: scripts/check-all.sh runs it
# before a push, and the build job of .github/workflows/checks.yml runs it on Linux, macOS and
# Windows after setting up the toolchain. Every step runs even after an earlier one fails, so a
# single run reports everything that is wrong.
#
# - rustfmt and Clippy, with every warning denied, rustc's dead_code lint among them;
# - the tests of the workspace;
# - a build of examples/demo, which has to succeed where Graphviz and PlantUML are missing: their
#   blocks stay code and are reported as warnings;
# - a check of the core, the renderers and the railroad engine for wasm32-unknown-unknown
#   (Verifies: G-8, ADR-0012), so nothing that cannot run in a browser creeps in, nectar-textmate
#   included, which runs on native hosts only; the target comes with the toolchain set-up;
# - cargo-deny against deny.toml: licences compatible with BSD-3-Clause only, the exception the
#   specification names aside; the sources of every crate; and none of the technology crates
#   ADR-0009 keeps out of the core and the renderers. Advisories are not part of the gate.
#
# Needs cargo with the wasm32-unknown-unknown target, and cargo-deny, on PATH. The Dev Container
# installs them (its Rust feature and .devcontainer/setup.sh); elsewhere a missing cargo-deny is
# an error, since no other gate holds the licences.
#
# Usage:
#     scripts/check-build.sh        (or: bash scripts/check-build.sh)
# Exit code 0 when every step passes, 1 otherwise.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

failed=()

# step <label> <command...> — run one step, recording a failure without aborting the run.
step() {
  local label="$1"
  shift
  printf '\n--> %s\n' "$label"
  "$@" || failed+=("$label")
}

step "Formatting" cargo fmt --all --check
step "Clippy" cargo clippy --workspace --all-targets -- -D warnings
step "Tests" cargo test --workspace
step "Demo build" cargo run --quiet -p nectar -- -C examples/demo build --output "$ROOT/target/demo"
step "WebAssembly" cargo check --quiet --target wasm32-unknown-unknown \
  -p nectar-core -p nectar-html -p nectar-pdf -p nectar-railroad
step "Dependencies" cargo deny --all-features check licenses bans sources

printf '\n'
if ((${#failed[@]})); then
  echo "Build FAILED:" >&2
  printf '  - %s\n' "${failed[@]}" >&2
  exit 1
fi
echo "Build passed."
