# Installed runtime adoption evidence

This trial qualifies the [implemented Rust API](runtime.md) for an initial
controlled Canic integration. It is not a claim that Canic source has adopted it.
Both the installed canister and native Rust caller compile against the same
[shared contracts](../crates/ic-wire-canister-evaluation/src/commands/mod.rs).
The [adapter](../crates/ic-wire-canister-evaluation/src/replay/mod.rs) uses the
production encode/decode functions at query, update and outbound-call boundaries.

## Qualified behavior

- A maximum-size nested ordinary query round-trips four UTF-8 labels and an
  optional maximum u64, returning the current counter without changing state.
- The protected update validates actual caller, audience and expiry, then commits
  the local counter and exact receipt bytes together. Exact retries return retained
  responses; conflicting bytes for a retained identity reject.
- A trap after stable write rolls back effect and receipt. Identical-Wasm upgrade
  preserves the current contract's stable bytes and exact replay.
- Overflow is a typed application error with a retained response. A full eight-slot
  table rejects new identities but continues to serve known requests.
- A forwarding canister encodes one typed request, sends it twice to the receiver
  and validates both typed responses and byte equality. The receiver applies one
  effect. An unauthorized forwarding call cannot change the receiver's state.
- Wrong method identity and query truncations reject. Update malformed/admission
  cases check stable memory remains unchanged after each rejection.

The [saved JSON](evidence/runtime-adoption.json) records exact vectors, counts,
compiler, flags and source/artifact hashes. The script builds the `replay` feature
with Rust 1.99.0, `wasm32-unknown-unknown` and `canister-evaluation` (opt-level z,
LTO, one codegen unit, panic abort, debug-info stripping). Postcard is 1.1.3,
PocketIC client 16.1.0/server 16.0.0, on a default application subnet.

## Limits and reproduction

This replaces the private tuple replay probe in current source. The runtime
contract has a distinct identity and changed stable receipt width. Every run
installs fresh state; it does not upgrade or migrate a 0.1.4 fixture. The upgrade
assertion is exclusively within this one contract and identical Wasm artifact.
Historical [0.1.4 evidence](command-replay.md) is preserved independently.

The runtime still has no Canic, Candid or CDK dependency. The evaluation adapter
uses CDK platform calls and its unrelated historical comparison endpoints remain
in this evaluation package. This artifact is not a Candid-removal size comparison.
The peer CDK transport allocates raw replies before codec admission; production
pre-copy limits belong in Canic's callback layer. Exact-retry policy here is
specific to the qualified receipt-backed receiver. No general update retry policy,
delegated authentication or external-effect reconciliation is implied.

With prepared dependencies and a local server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-replay
cargo test --locked -p ic-wire --test runtime
cargo test --locked -p ic-wire --doc
```

The runner retains Wasm and a JSON report under `target/runtime-adoption-evidence/`.
The new runtime header and static-bound/canonical checks have not been benchmarked
against Candid; historical savings must not be relabeled as runtime measurements.

## Validation of this batch

The saved run records 43 update admission/truncation rejections, 155 query
truncation rejections and two successful inter-canister deliveries producing one
effect. Eight runtime tests pass on Rust 1.99 and 1.91; the compiling quickstart
and four runtime compile-fail examples pass. The combined peer/replay fixture
passes seven native tests. All-feature Clippy, Rust 1.91 host/Wasm adapter checks,
formatting, dependency pins, documentation links and the local Make/hook routing
fixture pass. The four historical typed-journey compiler rejection checks also
pass after adding the runtime/derive workspace members. No sibling suite or full
repository CI was run. Source hashes in the saved report match the final inputs.
