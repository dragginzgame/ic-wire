# Wire contract qualification

This unpublished workspace tests the local Wire implementation. Product packages
do not depend on it. Tests use independent byte/schema expectations and the
current Rust and TypeScript implementations, with no older Wire dependency.

| Path | Responsibility |
| --- | --- |
| `ic-wire-compatibility/` | Native, allocation and Rust/TypeScript contract tests |
| `ic-wire-consumer/` | Standalone upload workload and client generator |
| `ic-instructions/ic-wire-instructions/` | Native instruction measurement driver in its isolated workspace |
| `typescript/` | Private compiler dependency for strict generated-client checks |
| `browser/` | Explicit Chromium smoke fixture |

Package directories match Cargo package names. `wire` is the renamed dependency
on the local product crate, also exercising derive support for renamed imports.

## Focused checks

Run from the repository root after `cargo fetch --locked --manifest-path
qualification/Cargo.toml`. Checks use locked offline dependencies.

```sh
cargo test --manifest-path qualification/Cargo.toml -p ic-wire-compatibility --test compatibility --test codec_contracts --locked --offline
make test-allocation
make test-consumer
make check-consumer-wasm
make test-typescript
```

The literal tests cover every u16/i16 encoding, short primitive acceptance,
representative payload/request/response bytes, truncated prefixes, reusable/slice
custody and borrowed input pointers. Additional frame tests cover nested
truncation/trailing spans and nonfinite or partial packed float words at first,
middle and last positions. Product tests independently cover precise admission
boundaries and malformed-input precedence.

The [standalone consumer](ic-wire-consumer/README.md) checks a bounded upload contract
against literal frames. Its native and SDK-free Wasm workload uses only the
current codec. Optional installed measurements use an
[isolated native driver](ic-instructions/README.md).

`make test-compatibility` and `make lint-compatibility` are aggregate checks of
this workspace with default features. They do not enable TypeScript or start a
simulator. The root Makefile admits the tooling snapshot and includes all three
workspaces in formatting/dependency commands. Each owns a lockfile and lint policy.

## Allocation qualification

The `allocation` executable runs eight groups serially without the concurrent
Rust test harness. Published `stats_alloc` wraps the system allocator; the crate
forbids unsafe code. Fixture preparation and assertions are outside measured
regions, and outputs stay live until statistics are read.

Checks cover zero-allocation sizing/preflight refusal, fallible output reservation
between passes, reused/slice output, field rejection before owned allocation,
borrowed spans, successful allocation controls, failed second-pass cleanup and
domain boundaries. A valid prefix can allocate before a later invalid field.
Text allocates owned bytes before UTF-8 validation, and custom Serde can allocate
independently. These are selected admission contracts, not a process heap cap.

## Rust/TypeScript exchange

`make test-typescript` requires explicitly installed Node supporting
`--experimental-transform-types`. The harness emits the current bundled runtime
and generated clients into a unique temporary directory, executes assertions,
checks returned bytes/values in Rust and removes the directory. It covers literal
records, recursive/shared schemas, large bytes, packed/ordinary fields, domain
adapters, platform identities, scalar/custom checks, malformed inputs and budgets.
Short u16/i16 and boolean inputs are checked against independent protocol rules.

For strict generated-client checking, prepare the private TypeScript compiler:

```sh
npm ci --prefix qualification/typescript --ignore-scripts --no-audit --no-fund
make check-typescript
```

The lock selects TypeScript 5.9.3. Checks enable strict mode, unchecked index
access and exact optional properties, targeting ES2022 modules. Positive and
expected-error fixtures exercise generated types. These commands do not claim
browser-engine coverage or identical Rust/JavaScript budget accounting.

The retained large-byte fixture was adapted from source revision
`2dc434b10bec2fca92345d940dbfedc05993e9d5`, paths
`crates/ic-wire/examples/typescript_conformance/large_bytes/{mod.rs,checks.mjs}`.
Their SHA-256 values are `6033c1eb16dcb25eb3fba0b0bddfa7fffb8850a26e90b4e2a8035a3a7151ae07`
and `a9e511850e6325164d948d31e1ca5283618dc514df9e31a4134d0cb351a063e7`.
Packed-i16 distributions came from
`qualification/ic-wire-benchmarks/benches/fixtures/packed_i16/mod.rs` at that
revision, SHA-256 `e2f873f88d0e190652ae43e51ff88507689f42843b85aca0d6bb3202fd102f22`.
Only workload patterns and boundary ideas are retained, under the root licence.
These provenance records introduce no external dependency or comparison gate.

## Measurement and distribution

[Performance scope](../docs/design/performance.md) describes current-only opt-z/
opt-3 Wasm and installed IC counters. They are separate from correctness checks.
`make check-packages` verifies extracted archives with the real consumer;
`make check-browser` executes generated clients in an explicitly selected
Chromium binary. See [release preparation](../docs/release.md) for prerequisites
and delivery boundaries. Neither command publishes or migrates consumers.
