# The rebuild timing test fails under the load of the full check run

Observed: `a_one_line_edit_of_the_guide_rebuilds_in_under_200_ms` failed once at 272 ms while
`scripts/check-all.sh` ran the whole suite, on the branch that took up NUC v0.8.1, which changed
no Rust code. Three runs of the test alone and two later full runs passed. A test that fails by
the machine's load and not by the code is a flaky test (AGENTS.md section 5); a fixed wall-clock
bound measured beside parallel test binaries decides the host as much as the rebuild.

Where: crates/cli/tests/serve.rs (the assert on `timings[1]`), the test that measures the rebuild
criterion of docs/SPECIFICATION.md.
