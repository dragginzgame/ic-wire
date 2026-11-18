# Standalone Wire consumer

Private `ic-wire-consumer` is an application-owned sample-upload contract using
only Wire's public API and Serde. This crate is a real executable/test workload, not a placeholder,
production service or deployed migration.

The request contains revision u64, UTF-8 key (32 bytes), opaque content (1 KiB),
explicit packed i16 samples (64 elements), and an ordinary u16 tag list (8 items).
The distinct response contains revision and acceptance. Method ID 7 and field
order are frozen by literal native vectors. Request/response limits are explicit
and independent; JS value-node policy is separate from Rust weighted admission.
It owns no transport, endpoint execution, authentication, storage or retries.

The library is no_std + alloc. The standalone binary uses std for its entry point
and assertions, runs both codec directions, and retains work with black_box. It
builds as native and wasm32-unknown-unknown without an SDK. The library uses the
current codec directly; its four application functions select the contract
identity, direction and allowances.

From the repository root:

```sh
make test-consumer
make check-consumer-wasm
cargo run --manifest-path qualification/Cargo.toml -p ic-wire-consumer --features schema --bin wire-consumer-generate --locked -- qualification/target/consumer-client
IC_WIRE_CHECK_TYPESCRIPT=1 cargo test --manifest-path qualification/Cargo.toml -p ic-wire-compatibility --features typescript --test typescript consumer_cases --locked
```

The generator writes payload.ts, methods.ts and the matching runtime.ts into the
selected directory. Schema generation is optional and host-only. Its complete
native/TS exchange is integrated into the existing compatibility harness, which
runs the clients against the bundled TypeScript runtime and checks returned frames in Rust.
Strict checking requires the private compiler installation described in the
[parent README](../README.md). The generator itself needs neither npm nor Node.

Generation always prints bulk-field diagnostics. The bounded ordinary `tags`
list deliberately produces advice; qualification retains that control instead of
silently converting it to packed words. Append `--deny-warnings` after the output
directory to enforce review in CI: the command fails before creating artifacts.
The library `clients(runtime, diagnostics)` requires a handler and reports the
payload and method generations separately. Production users acknowledge exact
paths with reasons through the Wire generation APIs, preserving field constraints.

Native tests check independent literal frames, every truncation, wrong method and
direction, oversized frames, trailing bytes and malformed boolean replies. The SDK-free Wasm workload runs
under Node with zero host imports. Its dependency graph contains no host schema JSON, Wire macro, SDK or transport dependency (Serde's
own derive remains a build-time dependency).

These checks establish standalone integration. `make measure-consumer-wasm`
separately measures current opt-z/opt-3 artifacts; see
[measurement scope](../../docs/design/performance.md). Debug artifact size and
Node execution do not establish IC costs or browser performance.
