# Registry Runtime 0.17.1 and Types 0.16 qualification

This replaces PR #41's unpublished supplier selection with a registry lock for
Runtime/macros/exact guard 0.17.1 and canonical DTO Types 0.16.0. Pending Core 0.14
adopts the new Rust schema identity; pending Endpoint 0.4 selects execution directly.
Own package versions stay unchanged during implementation. There is one production
Serde/Postcard codec and no supplier patches, old-type schema impl or wrapper.

Principal's bytes(max 29), Account's owner/subaccount order, absent versus explicit
zero, frame IDs, limits, primitive acceptance and original retry/hash inputs are
unchanged. Consumers align DTO imports with Types 0.16 or Platform 0.17, and
align other execution suppliers before Endpoint adoption. The exact singleton
refuses legacy embedded execution. Publish coordinated Core/Endpoint and matching
Timers before application upgrades; this qualification does not publish packages.

## Selected graphs

Production Endpoint and standalone schema/Endpoint consumers share one registry
Types 0.16 and one Runtime/macros/guard 0.17.1. Schema-only graphs select no SDK;
Wasm excludes host JSON, native clients and the crypto facade. Even selecting the
fixture's native `ic` feature does not pull that client into its Wasm graph.

The unpublished fixture deliberately keeps facade 0.16 / Testkit 0.76.1 for the
existing external PocketIC client. Their old Types 0.15 exists only at that native
qualification boundary and never supplies Wire DTO schemas. This avoids changing
the native client identity while qualifying the separate execution owner. The
future-facade proof selects crypto-only capability, retaining canonical DTO and
execution identities across a simulated breaking facade/Endpoint package change.

## Focused qualification

The owning Endpoint script passes three schema/borrowing tests, the method-ID
vector, strict library/fixture Clippy, eleven expected diagnostics and their control,
optimized Wasm and Rust 1.91 Wasm. The standalone type script passes four/five
Rust tests, Rust 1.91 host/Wasm, strict Clippy, generated TypeScript and Rust/browser
byte/refusal conformance. The crypto-facade identity simulation passes with all
Core, derive, DTO, Runtime, macro and guard IDs retained. A legacy-executor
consumer is rejected specifically for its conflicting `links` singleton.

Installed `typed_calls_limits_and_transactions` passes with the explicitly prepared
PocketIC 16.0.0 server: successful and malformed frames, pre-copy limits, exact
replies, synchronous rollback, async transaction behavior and guarded dispatch.
No native client/Server ownership code was copied into Wire.

## Cost comparison

The matched comparison passes all **178 cases / 17 exports** under both existing
profiles: 5,696 measured logical calls plus 712 excluded warmups. All 356
case/profile pairs have identical instruction/cycle means, and every one of the
3,204 paired samples (warmups included) has identical instruction and cycle counts.
Caller, exact request/reply hashes, frame sizes and outcomes also match.

| Profile | Total uncompressed Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| opt-z | 100,597 → 100,637 (+40; +0.0398%) | 86,698 → 86,698 (0%) |
| opt-3 | 93,957 → 93,997 (+40; +0.0426%) | 81,012 → 81,012 (0%) |

This ownership change has a small total-module cost and no code/instruction
regression in the measured fixture; it is not a codec speed/size optimization.
Native timing is not measured. Guest instructions include physical callbacks and
page charges; cycles are actual balance debits rather than estimates. Small,
boundary and refusal controls remain visible in [all comparisons](paired-results.csv)
and the [paired summary](paired-summary.json). Distinct callers and round advances
prevent query caching from substituting for execution.

[Raw cost receipts](cost-receipts.tar.gz) preserve every physical sample, server
charge/provenance metadata and all four raw modules. The baseline is committed
Wire c09560e (already containing the writer hint); [frozen inputs](inputs.json)
record compiler, common dependencies, source hashes, target, profiles and server.
[Baseline](baseline-source.tar.gz) and [candidate](candidate-source.tar.gz) snapshots
retain the exact build sources. [Focused check logs](checks.tar.gz) retain passing
and expected-rejection evidence. The [registry lock](Cargo.lock.snapshot.gz) has
no supplier patches; [supplier identities](supplier-identities.json) distinguish
native client Types from production DTO Types. [Checksums](SHA256SUMS) bind all files.

## Reproduce

Use the checked-in registry lock and prepared Rust 1.99.1, Rust 1.91 Wasm and
TypeScript tools. From the candidate source run:

```sh
bash scripts/ci/check-endpoint.sh
bash scripts/ci/check-platform-types.sh /absolute/prepared/typescript/bin/tsc
bash scripts/ci/check-stable-identities.sh
POCKET_IC_BIN=/absolute/prepared/pocket-ic bash scripts/ci/check-endpoint-ic.sh
```

Raw source, source-bound checks and cost data remain in
`/tmp/ic-wire-pr41-registry-20261101/target`. The checked-in snapshots and receipts preserve qualification even if external
cleanup removes temporary directories.
This is Linux qualification, not a macOS or current hosted-CI pass. Required
checks and reviews still govern PR integration; do not weaken them.

For matched cost replay, restore both source snapshots into fresh roots and adapt
the absolute source/study/server/helper paths in [compare-endpoints.sh](compare-endpoints.sh).
Supply the instrumented server hash recorded in inputs.json. Run
`node compare.mjs RAW_STUDY_ROOT` to regenerate the paired summary/CSV from receipts.
