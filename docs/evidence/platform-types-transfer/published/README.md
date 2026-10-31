# Published Platform 0.11 adoption

The single workspace Platform dependency now selects registry 0.11.0 for both
`endpoint` and `platform-types`. The lock changes only Platform, its macros and
its runtime guard, plus their consumer references. Testkit's native-only
Platform 0.9.1 remains outside the actual canister runtime. There are no path or
Cargo patch dependencies for consumers. Own Wire versions remain 0.9.0 until
maintainer release preparation; the pending release is 0.10.0 because endpoint
consumers must coordinate their runtime dependency identity.

Consumers replace Platform `wire-schema` with Wire `platform-types` and align
runtime-consuming dependencies on Platform 0.11. Principal's 29-byte bound,
Account's owner/subaccount order, absent versus zero subaccount bytes, Serde
serialization, resource budgets, frame identities and accepted inputs stay
unchanged. CDK remains selected only by the independent endpoint feature.

## Supplier review and qualification

The [source review](source-review.json) records the published commit and verified
registry archive checksums. All 120 facade source files, five macro source files
and the runtime guard match the [frozen preparation](../README.md) byte-for-byte.
Platform has no `wire-schema` feature or production Wire/serde_json dependency.

The published supplier passes standalone Principal/Account derives, four schema
checks and five combined-endpoint checks, Rust 1.91 native/Wasm compilation,
strict Clippy, feature-graph isolation, exact Rust/TypeScript byte vectors and
malformed-input refusals. Endpoint native/schema tests, 11 expected compiler
diagnostics plus passing control, strict Clippy, Wasm/MSRV and installed
limits/transaction rollback checks pass. The actual checkout's three Candid
comparison corpus roundtrip/rejection tests and pin/boundary checks also pass.
No full CI, macOS, sibling suite or downstream application qualification was run.

## Matched endpoint comparison

The baseline is Wire 0.9.0 at `edb5d1b` with published Platform 0.10.0. The
candidate applies the recorded Wire transfer and published 0.11 supplier.
The [lock delta](supplier-lock.patch) binds all remaining dependency inputs.
Both lanes use the same [compiler](compiler.txt), [instrumented server](server.sha256),
inputs, budgets and host driver. Profiles use LTO, one codegen unit, panic abort
and symbol stripping, with opt-z or opt-3 respectively.

The maintained 12-case subset covers seven exports with empty/32-byte controls,
a 65,522-byte blob, borrowed and async paths, composite self-call, mutation/reply
failure and malformed inputs. Each lane has one warmup and eight measured
repetitions per case: 384 measured logical calls and 608 physical executions.

| Metric | Platform 0.10 | Platform 0.11 | Change |
| --- | ---: | ---: | ---: |
| opt-z total uncompressed Wasm | 99,947 B | 99,950 B | +3 B (+0.003%) |
| opt-z code section | 86,068 B | 86,068 B | 0 |
| opt-3 total uncompressed Wasm | 93,518 B | 93,518 B | 0 |
| opt-3 code section | 80,630 B | 80,630 B | 0 |
| opt-z empty echo instructions | 253,815 | 253,815 | 0% |
| opt-z largest blob instructions | 3,884,431 | 3,884,431 | 0% |
| opt-z mutate/reply-32 instructions | 492,549 | 492,549 | 0% |
| opt-z mutate/reply-32 observed cycle debit | 7,000,549 | 7,000,549 | 0% |
| opt-3 empty echo instructions | 247,492 | 247,492 | 0% |
| opt-3 largest blob instructions | 3,879,734 | 3,879,734 | 0% |

Every measured instruction/cycle sample matches, not just means. Frames,
outcomes and transaction oracles also match. Zero query cycle debits are observed
balance changes in this harness, not general billing claims. The only measured
regression is three total Wasm bytes at opt-z; neither code section grows.
This is a focused supplier comparison, not a full endpoint/Candid matrix or a
host-generation/native-time benchmark. The earlier preparation's shorter local
source paths do not establish a published Wasm saving.

[Full per-case results](comparison.json) include samples, ranges and percentage
changes. [Raw execution evidence](samples.json.gz) retains the physical traces;
compression is evidence storage only, not a Wasm optimization metric.

## Reproduction

With cached registry dependencies, the recorded compiler, Rust 1.91 native/Wasm
targets and the explicitly prepared browser compiler and instrumented server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic-server \
  bash docs/evidence/platform-types-transfer/reproduce.sh \
  /absolute/fresh/output - published
```

The same runner reconstructs the original baseline and transferred Wire source,
selects registry 0.11, verifies the lock delta, runs schema/endpoint checks and
measures both lanes. The `published` selection does not read a sibling checkout.
It does not fetch, install, publish or release anything. The completed run is
`/tmp/wire-platform011-published/`; its standalone consumer is
`candidate/target/platform-types.CMjavi`. Pin/boundary and corpus logs are
`/tmp/wire-platform011-boundaries.log` and `/tmp/wire-platform011-roundtrip.log`.

## Published Wire consumer follow-up

On 2026-10-31, issue [#32](https://github.com/dragginzgame/ic-wire/issues/32) was
qualified against **both published packages**, Wire 0.10.0 and Platform 0.11.1,
without path dependencies or Cargo patches. This is an additional consumer
composition check, not a rerun of the historical performance comparison above.
The [standalone manifest](composition-Cargo.toml.snapshot) replaces Platform's
retired `wire-schema` with Wire's `platform-types`; its `endpoint` feature enables
the same published runtime supplier. Five fixture tests and Rust 1.91 Wasm
compilation pass. The schema-only graph selects no runtime owner; the combined
Wasm graph has one Platform/runtime guard, no Candid and no host serde_json.
[Logs and dependency graphs](composition-checks.txt) retain the fixture hash.

Reproduce from the repository root with cached registry packages and Rust 1.91:

```sh
consumer="$(mktemp -d)"
mkdir "$consumer/src"
cp docs/evidence/platform-types-transfer/published/composition-Cargo.toml.snapshot "$consumer/Cargo.toml"
cp crates/ic-wire-qualification/fixtures/platform_types/mod.rs "$consumer/src/lib.rs"
cargo metadata --manifest-path "$consumer/Cargo.toml" --offline --format-version 1 > "$consumer/metadata.json"
cargo +1.91.0 test --manifest-path "$consumer/Cargo.toml" --locked --offline --features endpoint --lib
cargo +1.91.0 check --manifest-path "$consumer/Cargo.toml" --locked --offline --features endpoint --target wasm32-unknown-unknown
cargo tree --manifest-path "$consumer/Cargo.toml" --locked --offline --edges normal,build
cargo tree --manifest-path "$consumer/Cargo.toml" --locked --offline --features endpoint --target wasm32-unknown-unknown --edges normal,build
```

Transitive packages resolve from the available registry index; the two integration
suppliers are exact. This check establishes composition, not a performance result
or a downstream Canic/Blob deployment qualification. Platform 0.12 consumers still
need the pending compatible Wire release tracked by #34.
