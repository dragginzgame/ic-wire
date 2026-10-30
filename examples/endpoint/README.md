# Standalone endpoint qualification consumer

This unpublished package depends on Wire and Platform, with no Canic dependency.
The [endpoint guide](../../docs/endpoints.md) explains its supported API,
ownership and explicit PocketIC qualification commands. `src/lib.rs` is a small
installable canister covering typed/unit handlers, sync/async execution, bounded
replies, synchronous borrowed requests/replies and explicit composition around a
guarded handler. Borrowed tests verify input custody, matching schemas, malformed
request refusal before mutation and rollback on oversized replies.

The `inline-baseline` feature affects only the `endpoint-cost` measurement example;
it is a qualification comparator, not an alternative production codec. The
[measurement runner](../../scripts/ci/evaluate-endpoint.sh) retains matched input,
compiler, lock, server and Wasm identities, repeated native/IC samples and raw
uncompressed/code-section sizes.

`tests/cost_matrix` benchmarks every endpoint exported by this fixture, including
success, malformed input, limits, policy refusals and reply traps. The runner
compares its cases with the compiled Wasm exports, so a new export cannot silently
miss cost coverage. Use `make evaluate-endpoints EVIDENCE_DIR=/absolute/new/path`
with an explicitly prepared instrumented `POCKET_IC_BIN`. Testkit owns the server;
no counters or profiling code are added to the measured canister. The report
separates complete IC instructions, observed cycle debits and raw Wasm size.
