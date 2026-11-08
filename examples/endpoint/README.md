# Standalone endpoint qualification consumer

This unpublished package depends on Wire and Platform, with no Canic dependency.
The [endpoint guide](../../docs/endpoints.md) explains its supported API,
ownership and explicit PocketIC qualification commands. `src/lib.rs` is a small
installable canister covering typed/unit handlers, sync/async execution, bounded
replies, synchronous borrowed requests/replies and explicit composition around a
guarded handler. Borrowed tests verify input custody, matching schemas, malformed
request refusal before mutation and rollback on oversized replies.

The `endpoints` integration target shares one `tests/cases` catalogue between
`tests/installed` and `tests/cost_matrix`. The standard-server test retains explicit
transaction-history assertions and also executes every catalogue case; the cost
runner independently validates profiler traces. Scripts select each ignored test
by its exact module-qualified name, so ordinary installed checks never start the
profiler runner. No test helpers enter the canister library.

`tests/cost_matrix` benchmarks every endpoint exported by this fixture, including
success, malformed input, limits, policy refusals and reply traps. The runner
compares its cases with the compiled Wasm exports, so a new export cannot silently
miss cost coverage. Use `make evaluate-endpoints EVIDENCE_DIR=/absolute/new/path`
with an explicitly prepared instrumented `POCKET_IC_BIN`. Testkit owns the server;
no counters or profiling code are added to the measured canister. The report
separates complete IC instructions, observed cycle debits and raw Wasm size.
