# Scoped payload output (0.7.21 draft)

`native::with_encoded` accepts an owned value (or a borrowed projection), performs
ordinary admission/sizing, selects caller scratch or an owned buffer once, then
serializes through the existing bounded Postcard encoder. It drops owned input
before a synchronous callback. Errors never invoke the callback; bytes cannot
escape as a borrow/future. Existing APIs, framing, bytes and admission rules are
unchanged. No migration is required: u64 7 remains payload `07`, unit is empty.

This adopts only the scoped-output helper proposed in the
[SDK branch](https://github.com/infu/ic-wire/commit/5d535c0a7e7514885c4f7253e229586b5f8b137f),
with the current published payload/generator/slice interfaces preserved.

## Inputs and reproducible scope

Baseline is released 0.7.20, `1608df0661137003c7126040e8656598b98d5bd8`.
Candidate is the dirty source identified in [identity.json](identity.json).
The [maintained runner](../../../scripts/ci/evaluate-caller-storage.sh) copies
current source/benchmark wiring and the lock, then restores only Wire/derive
from baseline for the old-library build. Baseline has its own Cargo target;
candidate variants share the same physical source and normal feature invalidation.
The captured source precedes one added documentation-only compile-fail example;
identity records both native-module hashes. No implementation changed afterward.

Compiler: [Rust 1.99.1 / LLVM 23.1.1](compiler.txt); targets x86_64 Linux and
wasm32-unknown-unknown. `evaluation` is opt-z, fat LTO, one codegen unit,
panic abort, stripped symbols; `benchmark-speed` changes only opt-level to 3.
Both revisions use the same Cargo.lock: Postcard 1.1.3, Serde 1.0.229,
Platform 0.7.3, Build/Testkit 0.69.6. Prepared PocketIC 16.0.0 is identified by
its SHA-256; no mainnet or complete replica-source parity is claimed.

Limits: bytes 131,072, allocation 8 MiB, nodes 131,072, depth 64. Every path
admits the same root type `&T`; passing a borrowed projection by value to the
helper must be compared with encoding that same reference, not a differently
sized owned root. Inputs are unit, u64 7, ordinary Vec<u32> of 0/8/4,096 maximum
values, byte buffers of 0/32/65,536 bytes `a5`, and a record containing
`[u64::MAX,1]` and `é🦀`. Payload sizes are respectively 0/1/1/41/20,482/1/33/65,539/18.
Scoped scratch is 128 bytes; large vector/blob cases select owned fallback.
The full caller buffer is prepared outside counters; these results do not measure
an SDK's stack reservation, endpoint copying, network latency or rollback.

## IC instructions

Each cell compares baseline fresh owned encode with candidate scoped encode.
The existing fresh-encode control has identical instruction counts and
byte-identical Wasm before/after in both profiles. Setup/input preparation and
transport are outside Counter0 windows; one operation and output destruction
are inside. Three identical anonymous query returns are retained per row; these
may reuse execution and are not three independent Wasm executions. All returned
counters agree. See [raw counters](instructions.json).

| Case | opt-z before → scoped (change) | opt-3 before → scoped (change) |
| --- | ---: | ---: |
| Unit | 424 → 291 (-31.37%) | 255 → 271 (+6.27%) |
| u64 | 842 → 425 (-49.52%) | 586 → 289 (-50.68%) |
| Empty vector | 1,223 → 807 (-34.01%) | 674 → 306 (-54.60%) |
| Vector, 8 | 6,975 → 6,239 (-10.55%) | 2,845 → 2,474 (-13.04%) |
| Vector, 4,096 | 2,946,298 → 2,938,078 (-0.28%) | 1,094,347 → 1,090,337 (-0.37%) |
| Empty blob | 1,267 → 807 (-36.31%) | 789 → 461 (-41.57%) |
| Blob, 32 | 1,409 → 942 (-33.14%) | 931 → 598 (-35.77%) |
| Blob, 65,536 | 67,027 → 66,995 (-0.05%) | 226,928 → 66,484 (-70.70%) |
| Record | 2,583 → 2,060 (-20.25%) | 1,358 → 945 (-30.41%) |

Unit regresses by 16 instructions at opt-3. The large opt-3 blob difference is a
whole compiled-path observation, not an allocation-count improvement: both
fallback and fresh output allocate once. No universal consumer speedup follows.

## Uncompressed Wasm

| Profile | Owned control total / code | Scoped total / code | Total / code change |
| --- | ---: | ---: | ---: |
| opt-z | 22,443 / 20,129 | 23,165 / 20,833 | +722 / +704 |
| opt-3 | 22,493 / 20,432 | 24,983 / 22,927 | +2,490 / +2,495 |

These are complete nine-case fixture modules replacing the selected encoding
path, not a universal marginal application cost. [Size receipts](wasm.csv) also
retain the explicit slice controls; [identity](identity.json) retains all eight
Wasm hashes. Merely adding the unused helper changes no owned-control artifact.

## Native time and operation allocation

Seven samples per case/path/profile. Each timing sample repeats 4,096 operations
below 1 KiB or 64 otherwise. Report median amortized nanoseconds; setup, warmup
and reporting are excluded, encoding/callback/output destruction included.
Comparisons below use fresh-owned and scoped lanes in the same candidate native
executable, not independent before/after native builds. Raw samples retain
reusable-Vec and explicit-slice controls too:
[opt-z](evaluation-time.json), [opt-3](benchmark-speed-time.json).

| Case | opt-z fresh → scoped ns (change) | opt-3 fresh → scoped ns (change) |
| --- | ---: | ---: |
| Unit | 20.82 → 5.45 (-73.85%) | 2.73 → 2.93 (+7.53%) |
| u64 | 38.46 → 8.82 (-77.06%) | 26.85 → 6.27 (-76.66%) |
| Empty vector | 40.91 → 18.33 (-55.19%) | 32.56 → 9.61 (-70.47%) |
| Vector, 8 | 135.01 → 102.10 (-24.37%) | 80.06 → 54.07 (-32.47%) |
| Vector, 4,096 | 58,184.33 → 56,522.63 (-2.86%) | 25,526.00 → 25,656.97 (+0.51%) |
| Empty blob | 42.25 → 16.86 (-60.09%) | 32.77 → 10.65 (-67.50%) |
| Blob, 32 | 45.68 → 16.34 (-64.24%) | 35.23 → 10.02 (-71.55%) |
| Blob, 65,536 | 1,309.30 → 1,296.80 (-0.95%) | 1,027.16 → 1,006.98 (-1.96%) |
| Record | 64.71 → 39.71 (-38.64%) | 41.79 → 17.33 (-58.53%) |

Shared-host timing and tiny operations limit interpretation; native timing is
not IC cost. The opt-3 unit and large-vector regressions remain visible.

Separate allocator/DHAT runs record seven single warmed operations per row.
All scratch-fitting scoped cases allocate zero output bytes. Large fallback
makes one allocation of 20,482 or 65,539 bytes, no reallocations, zero retained
bytes after callback, and the same peak output bytes as fresh encoding. Unit
already allocates zero on fresh encode. Caller storage and prepared input are
outside instrumentation; no total-heap saving is claimed. Raw
[opt-z allocations](evaluation-allocations.json),
[opt-3 allocations](benchmark-speed-allocations.json),
[opt-z heap](evaluation-heap.json), [opt-3 heap](benchmark-speed-heap.json).

## Reproduce and validate

With the pinned toolchain/dependencies and an explicitly prepared server:

```sh
POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-caller-storage.sh target/NEW-STUDY \
  1608df0661137003c7126040e8656598b98d5bd8
```

Use a fresh directory and no active build sharing target/. Testkit owns server
startup/cleanup; unset `IC_TESTKIT_POCKET_IC_URL`. In this run sandbox networking
refused local server binding. All builds/native runs completed, then only the
focused IC test was rerun with local binding permitted:

```sh
IC_WIRE_STORAGE_EVIDENCE="$PWD/target/NEW-STUDY" \
POCKET_IC_BIN=/path/to/pocket-ic \
  cargo test --locked --offline -p ic-wire-benchmarks \
  --example caller-storage --features ic ic_measurement -- --ignored --test-threads=1
```

Full local artifacts/source/logs: `target/with-encoded-0721/`. Percent change is
`100 * (scoped / control - 1)`; native sample cost is `native_ns / repetitions`,
then the median of seven. Artifact hashes compare bytes, not symbol attribution.

Focused checks pass: four scoped integration tests covering parity/destination,
exact admission, two visits/drop ordering and second-pass failure; three adjacent
slice tests; one executable and two lifetime compile-fail doctests; strict runtime
and fixture Clippy; Rust 1.91 library compilation; native time/allocation/heap
studies and the IC study. No full CI, macOS, installed SDK or full Candid matrix
was run. TypeScript production code is unchanged by the helper addition.
