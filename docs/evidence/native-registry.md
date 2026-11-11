# Native recursive registries and projected budgets

## Registration-independent names — 11 November 2026

Generated definitions now use `Wire_TypeName_<16-hex-suffix>` from the complete
Rust type spelling rather than `WireType{index}`. The suffix includes namespace
and generic arguments and is always present: adding a namesake must not rename
an existing export. A collision rejects rather than allocating another number.
Explicit roots consistently alias these definitions, including when registered
before another root encounters the type. Projection-collision checks remain.

Consumers regenerate schemas/clients and update old positional imports once.
Names are stable under root reordering and unrelated additions, not Rust module
renames or changes to compiler type spellings. They are metadata labels, not
contract fingerprints. Bytes, method identities, limits and runtime acceptance
are unchanged. The implementation is host-only; no runtime Wasm saving is claimed.

Matched opt-z host study: Rust 1.99.1, identical locks, isolated baseline/candidate
sources and targets, eleven samples per run, two runs each. The synthetic fixture
registers many roots sharing recursive definitions; it is **not** Toko's unavailable
1,657-distinct-type schema. Generation includes collection, validation, advice and
TypeScript rendering. Median milliseconds (first run / serial repeat):

| Roots | Before ms | After ms | Generated TS bytes before → after |
| --- | ---: | ---: | ---: |
| 1 | 0.074 / 0.067 | 0.080 / 0.078 | 2,442 → 3,580 |
| 298 | 2.280 / 2.215 | 2.520 / 2.569 | 21,828 → 36,034 |
| 1,657 | 14.616 / 14.157 | 15.158 / 16.568 | 112,836 → 186,838 |

This accepts extra host generation time and longer TypeScript identifiers for
stable imports. Native timings are shared-host observations, not IC instructions;
the initial runs could overlap and the serial repeat is the stronger timing check.
The full Toko build and its generated JavaScript bundle have not been measured.
Raw sources, compiler/lock/source hashes and logs are retained under
`target/evidence/toko-feedback-20261111/`. Reproduce with the ignored native test
`measure_named_registry_generation` using separate source/target directories.

Regression coverage includes root order, unrelated insertion, namespace clashes,
generic instantiations, explicit-name collisions, recursive schemas, projection
conflicts and unchanged rooted acknowledgement paths. Rust/TypeScript byte and
malformed-input conformance passes after regeneration.

## Historical 0.7.7 qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

This host-only addition addresses [recursive projection #20](https://github.com/dragginzgame/ic-wire/issues/20)
and [browser budget projection #21](https://github.com/dragginzgame/ic-wire/issues/21).
Publication remains maintainer-owned; local qualification does not close that gate.

`SchemaRegistry` collects native shapes through the existing describe interface;
no new generated helper couples a newer derive to a newer library. Concrete mutual
recursion also avoids redundant cyclic trait predicates. Names cannot overwrite
existing definitions; finish verifies encountered projections and rejects conflicts,
unresolved refs, alias-only cycles and excessive metadata. Inline native_schema
retains its established output and recursive Complexity refusal.

`method_limits` projects independent byte/node/depth budgets and shares the
renderer validation. Rust allocation is deliberately omitted, not approximated as
JS heap. Native/TS counters, byte encodings and envelopes remain unchanged.

## Qualification

The maintained conformance example covers 13 shapes: scalar, record, option,
newtype, transparent wrapper, empty enum, empty/nested recursive lists, recursive
option, recursive record, mutually recursive records, bytes and packed i16.
Both directions sweep node/depth allowances and exact byte boundaries, asserting
frozen admission thresholds independently for Rust encode/decode and TS encode/decode.
Generated recursive modules type-check; truncations, trailing bytes, wrong headers
and low/high-bit mutations are checked against native acceptance. The ordinary
159 vectors, 33,560 mutations and 24 native projection vectors remain controls.
Retired raw output (see retention notice) retain the differences, including the
fact that decode success does not guarantee encoding under the same limits.

Package tests/doctests, strict Clippy, native/Wasm feature forwarding, Rust 1.91
checks and compiler diagnostics qualify the local source. No sibling changes,
publication or installed-consumer acceptance are claimed. Consumers retain custom
Serde projections/validators and qualify their own budgets and real endpoint corpus.
An isolated check also compiled the new derive against the pre-change metadata
module, confirming it does not require the new registry API from older libraries.

## Host generation and artifact measurements

Baseline is 0.7.6 (`accf815`) plus the incoming cleanup batch, preserved as a patch
in the archive. The compared runtime is identical; only host metadata/derive code
changes. Same Cargo.lock, Rust 1.99.1 / LLVM 23.1.1, Linux x86_64, opt-3, LTO,
one codegen unit, panic abort and stripped symbols. Each host run alternates seven
samples per implementation; three runs before and three after. Projection/render
samples average 100 operations; limit projection samples average 1,000 operations.
Times include allocation/destruction. No CPU affinity; treat small changes as noisy.

| Host operation | Before / handwritten | After / typed | Change |
| --- | ---: | ---: | ---: |
| Inline Page<u64> projection + render, 1 named schema | 3,269 ns | 3,039 ns | −7.04% |
| Same shape under 100 names | 305,103 ns | 310,193 ns | +1.67% |
| Six budget properties, handwritten JSON versus validated helper | 146 ns | 189 ns | +29.45% (+43 ns) |

Values are pooled medians of 21 batch means. Per-run inline medians are
2,941/3,076/3,286 → 2,998/3,112/3,026 ns for one schema and
300,129/311,815/303,526 → 307,852/315,589/304,681 ns for 100. The budget control
uses identical black-boxed Limits; its extra validation is intentional. This is
not a codec speedup claim or a 100-distinct-type benchmark. The prior inline API
cannot successfully project recursive types, so it supplies no valid recursive
before/after timing baseline.

The one-type runtime Wasms are byte-identical in both profiles:

| Profile | Total bytes before = after | Code section before = after |
| --- | ---: | ---: |
| opt-z | 53,998 | 42,997 |
| opt-3 | 74,871 | 48,129 |

SHA-256: opt-z `b391e404581f917a4077e51f78367cad706f0ef54f71eba9fb0ad0e1c7fb9443`;
opt-3 `0bfbc069230d3e9b7f633d55dff7b81001ba31d41ead40c29f1ab35254d10ef6`.
No fresh IC instruction or native codec timing run was needed for these identical
artifacts. This does not requalify every consumer Wasm or the full Candid matrix.

Retired raw output (see retention notice) retain reproduction
inputs. Extract and run `bash replay.sh` with prepared Rust/dependencies; it uses
an isolated temporary tree and offline Cargo. Maintained `make check-typescript`
reproduces cross-runtime qualification with the documented Node/TypeScript setup.
Original artifacts/logs remain under `/tmp/wire-issues-20-21`.

## Reusable lint paths — 11 November 2026

The rooted advisory walker now appends to one traversal-owned path and truncates
it when returning from each child. Only warnings retain full path copies. This
removes per-edge path allocation without memoizing use sites, dropping warnings,
changing acknowledgement escaping or increasing work/depth allowances. Early
errors still discard the whole report. Codec bytes, schema output, names,
identities, limits and acceptance are unchanged; no consumer migration is needed.

Matched sources under `target/evidence/lint-paths-20261111/{before,after}/source`
use Rust 1.99.1, identical locks and separate Cargo targets. The existing
`schema-cost` example now hashes both metadata and bulk-lint sources into its
identity. Two runs/profile, 11 calibrated batches/run (at least 10 ms/batch),
opt-z evaluation and opt-3 benchmark-speed. Timing includes stats_alloc and
output destruction on a shared host. Input and generated-output hashes match.

Generation medians across 22 batches; allocations exclude reallocations:

| Shape | opt-z before → after | Allocations before → after | opt-3 before → after |
| --- | ---: | ---: | ---: |
| One-field record | 2.174 → 2.173 µs | 24 → 23 | 1.535 → 1.539 µs |
| 4,096-field record | 2.400 → 2.268 ms | 25,228 → 21,132 | 1.505 → 1.362 ms |
| 128 fields × 32 options | 1.413 → 1.211 ms | 8,997 → 4,773 | 1.024 → 0.751 ms |
| Invalid unused definition | 1.191 → 1.208 ms | 12,929 → 12,929 | 0.867 → 0.883 ms |

Large-record reallocations fall 12,218 → 4,127; nested reallocations fall
12,083 → 7,835. The invalid-definition path fails before linting and has unchanged
allocation counts; its small timing regression is recorded, not erased.
Small-input timing is essentially flat. Host-only code is excluded on Wasm:
there is no canister size or IC instruction saving claim. Existing recursive,
shared-use-site, escaped-name, tuple/result/enum and budget-boundary tests pass.

Reproduce from each frozen source with `cargo run --locked --offline
--manifest-path qualification/Cargo.toml -p ic-wire-qualification --example
schema-cost --features schema-cost --profile evaluation`, then benchmark-speed,
selecting a distinct CARGO_TARGET_DIR for each source. Raw samples are retained
beside the source directories as `<profile>-<run>.jsonl`.
