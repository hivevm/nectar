# The tests-kept sensor misses an #[ignore] that gives a reason

Observed: the Q-4 test in `crates/cli/tests/serve.rs` gained `#[ignore = "timed: …"]`, and
`scripts/sensor-tests-kept.sh` did not report it; it reported the deleted backlog entry on the
timing test as a deleted test file instead, since its name holds "test". Its marker for Rust
reads only the bare `#[ignore]`, so the form with a reason, which Clippy and the Rust reference
both offer, and `#[cfg_attr(…, ignore)]` pass unreported. OPS-cargo-clippy-deny-toolchain says the
sensor reports added `#[ignore]` markers; for these forms it does not. The sensor is the
template's, so the fix belongs upstream in hivevm/nuc and reaches Nectar with a release.

Where: the MARKERS table of scripts/sensor-tests-kept.sh (the line for `rs`), and its fixture
under scripts/fixtures/.
