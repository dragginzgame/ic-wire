# Standalone Wire consumer

Private `ic-wire-consumer` is an application-owned sample-upload contract using
only Wire's public API and Serde. It is independent of the legacy consumers and
other recodes. This crate is a real executable/test workload, not a placeholder,
production service or deployed migration.

The request contains revision u64, UTF-8 key (32 bytes), opaque content (1 KiB),
explicit packed i16 samples (64 elements), and an ordinary u16 tag list (8 items).
The distinct response contains revision and acceptance. Method ID 7 and field
order are frozen by literal native vectors. Request/response limits are explicit
and independent; JS value-node policy is separate from Rust weighted admission.
It owns no transport, endpoint execution, authentication, storage or retries.

The library is no_std + alloc. The standalone binary uses std for its entry point
and assertions, runs both codec directions, and retains work with black_box. It
builds as native and wasm32-unknown-unknown without an SDK. Candidate and reference
are mutually exclusive supplier features, so their builds do not accidentally
include both codecs. The four narrow API mappings in lib.rs adapt the old native
functions to the candidate Encoding API; the types, fixtures and workload remain
the same. No compatibility facade enters the product crate.

From the repository root:

```sh
make test-consumer
make check-consumer-wasm
cargo run --manifest-path qualification/Cargo.toml -p ic-wire-consumer --features schema --bin wire-consumer-generate --locked --offline -- qualification/target/consumer-client
IC_WIRE_CHECK_TYPESCRIPT=1 cargo test --manifest-path qualification/Cargo.toml -p ic-wire-compatibility --features typescript --test typescript consumer_cases --locked --offline
```

The generator writes payload.ts, methods.ts and the matching runtime.ts into the
selected directory. Schema generation is optional and host-only. Its complete
native/TS exchange is integrated into the existing compatibility harness, which
runs the clients against both TS runtimes and checks returned frames in Rust.
Strict checking requires the private compiler installation described in the
[parent README](../README.md). The generator itself needs neither npm nor Node.

Native tests check independent literal frames, every truncation, wrong method and
direction, oversized frames, trailing bytes and malformed boolean replies. Both
supplier builds pass. Separate candidate/reference debug Wasm artifacts also run
under Node with zero host imports. The selected candidate Wasm dependency graph
contains no host schema JSON, Wire macro, SDK or transport dependency (Serde's
own derive remains a build-time dependency).

These checks establish standalone integration and a stable comparison workload.
`make measure-consumer-wasm` now runs the frozen matched opt-z/opt-3 comparison;
see [measurement contract and results](../../docs/design/consumer-measurement.md).
Debug artifact size and Node execution are not matched opt-z/opt-3 or IC-instruction
evidence. Before measuring those, freeze profiles and acceptance criteria and
retain separate fresh build directories. A deployed consumer, browser-engine
execution, installed package distribution and persisted-state adoption remain
separate gates. This initial case does not claim every field bound is saturated.
