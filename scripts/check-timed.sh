#!/usr/bin/env bash
#
# Check: the timed tests, those `cargo test` leaves out with #[ignore], run alone and one after
# another, and at least one of them. A bound in milliseconds, such as Q-4's rebuild in under
# 200 ms, holds only on a machine that runs nothing else beside the test: the parallel tests of
# scripts/check-build.sh and the other checks of scripts/check-all.sh decide the timing as much as
# the code. So check-all.sh starts this script only once every other check has finished, and the
# build job of .github/workflows/checks.yml runs it after scripts/check-build.sh
# (OPS-cargo-clippy-deny-toolchain).
#
# Needs cargo on PATH, as scripts/check-build.sh does.
#
#     scripts/check-timed.sh
# Exit code 0 when every timed test passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-timed.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
set -- "${ARGS[@]}"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "$@"

# The output is read for the count of tests that ran: with none, the check would pass on nothing,
# and a timed test that lost its #[ignore] would run beside the others again unnoticed.
output="$(cargo test --quiet --workspace -- --ignored --test-threads=1 2>&1)"
rc=$?
printf '%s\n' "$output"
ran=0
while read -r n; do
  ran=$((ran + n))
done < <(grep -oE '[0-9]+ passed; [0-9]+ failed' <<< "$output" | awk '{print $1 + $3}')
if ((rc != 0)); then
  echo "Timed tests FAILED." >&2
  exit 1
elif ((ran == 0)); then
  echo "Timed tests FAILED: no test ran — each timed test carries #[ignore] for cargo test." >&2
  exit 1
fi
echo "Timed tests passed ($ran)."
