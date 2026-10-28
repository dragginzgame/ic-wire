# Caller-storage and payload-client qualification (0.7.19 draft)

Prepared for [#29](https://github.com/dragginzgame/ic-wire/issues/29) and
[#30](https://github.com/dragginzgame/ic-wire/issues/30), against released 0.7.18
`c748379325266d7fa614fefee9d745eb4a88acca`. These are codec/host measurements,
not SDK echo costs, installed consumer qualification or a Candid comparison.
Existing wire bytes, primitive acceptance, identities and admission are unchanged.
Explicit TS payload codecs add Rust payload parity: u64 7 is `07`, unit is empty.

## Reproduction and identity

Use the committed [runner](../../../scripts/ci/evaluate-caller-storage.sh):

```sh
POCKET_IC_BIN=/path/to/prepared/pocket-ic \
  bash scripts/ci/evaluate-caller-storage.sh target/new-storage-evidence \
  c748379325266d7fa614fefee9d745eb4a88acca
```

The runner retains the current source, a baseline with only the two published
crates restored from the selected commit, the shared lockfile, compiler identity,
Wasm artifacts and raw samples. Baseline has a separate Cargo target directory;
no artifacts are shared across physical source copies. Candidate Vec and slice
builds use the same physical source and ordinary Cargo feature invalidation.
The private `caller-slice` feature selects only a measurement cohort, not a codec.
Native ignored tests and PocketIC run explicitly, outside ordinary CI gates.

Measured here: Rust 1.99.1/LLVM 23.1.1, x86_64 Linux and wasm32-unknown-unknown;
`evaluation` is opt-z, fat LTO, one codegen unit, panic abort and stripped symbols;
`benchmark-speed` changes only opt-level to 3. See [compiler](compiler.txt) and
[hashes](hashes.txt) plus [artifact/environment identity](identity.json).
Postcard 1.1.3, Serde 1.0.229, Platform 0.6.21 and
Build/Testkit/host 0.68.12 come from the unchanged incoming lockfile. PocketIC
server 16.0.0 has SHA256
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`;
no current-replica source parity or mainnet latency claim is made.

All cases use payload encoding with bytes 131,072, allocation 8 MiB, nodes 131,072,
depth 64. Inputs are unit, u64 7, Vec<u32> of 0/8/4096 maximum values, ByteBuf of
0/32/65536 bytes `a5`, and a record `{ counts: [u64::MAX,1], name: "é🦀" }`.
Setup, input storage and the preallocated 131,072-byte caller buffer are excluded
from operation counters. Output destruction is included. This oversized common
scratch is a control, not an SDK buffer policy or a total-heap saving.

Local full artifacts/source: `target/caller-storage-0719-qualified/`. The earlier
`target/caller-storage-0719/` stopped on a benchmark-fixture compilation error;
none of its results qualify this batch. Subsequent TS-only writer cleanup is
measured separately below; it does not affect the Wasm cohort, which disables
TypeScript. Documentation updates also change no measured execution.

## IC instructions and uncompressed Wasm

Three physical queries per case/build; medians from [raw samples](instructions.json).
All repeated instruction samples agree. The Vec control's before/after Wasm is
byte-identical in each profile, and its instruction counts are identical.

| Case | Payload bytes | opt-z Vec → slice (change) | opt-3 Vec → slice (change) |
| --- | ---: | ---: | ---: |
| Unit | 0 | 455 → 310 (-31.87%) | 255 → 275 (+7.84%) |
| u64 | 1 | 988 → 595 (-39.78%) | 582 → 304 (-47.77%) |
| Empty Vec | 1 | 1,256 → 860 (-31.53%) | 665 → 319 (-52.03%) |
| Vec, 8 | 41 | 7,072 → 6,292 (-11.03%) | 2,833 → 2,480 (-12.46%) |
| Vec, 4096 | 20,482 | 2,979,099 → 2,782,107 (-6.61%) | 1,094,333 → 1,016,322 (-7.13%) |
| Empty ByteBuf | 1 | 1,306 → 856 (-34.46%) | 779 → 462 (-40.69%) |
| ByteBuf, 32 | 33 | 1,448 → 991 (-31.56%) | 921 → 599 (-34.96%) |
| ByteBuf, 65536 | 65,539 | 67,066 → 66,621 (-0.66%) | 226,918 → 66,140 (-70.85%) |
| Record | 18 | 2,953 → 2,433 (-17.61%) | 1,349 → 955 (-29.21%) |

| Profile | Vec total / code bytes, before = after | Slice total / code bytes | Slice total / code change |
| --- | ---: | ---: | ---: |
| opt-z | 22,847 / 20,532 | 21,696 / 19,218 | -1,151 (-5.04%) / -1,314 (-6.40%) |
| opt-3 | 22,390 / 20,329 | 22,200 / 20,046 | -190 (-0.85%) / -283 (-1.39%) |

These are complete fixture modules, not predicted savings for another canister.
Raw [Wasm sizes](wasm.csv) and [instruction comparison](ic.csv) are retained.
The opt-3 empty-unit regression is 20 instructions; the API is an explicit storage
choice and does not replace the existing heap-backed path.

## Native time and allocation

Seven samples per workload/lane, with 4,096 repetitions for outputs under 1 KiB
and 64 otherwise. [Native medians](native.csv) retain fresh Vec, warmed reusable
Vec and slice controls; nanoseconds are host time, not IC instructions.
For example opt-z Vec-4096 is 50,733.66 → 46,535.63 ns (-8.27%). Opt-3 is
25,364.41 → 24,133.64 ns (-4.86%). Regressions: opt-3 unit is 1.90 → 2.51 ns
(+32.19%, dominated by a tiny operation/loop); the 64 KiB blob is
1,091.95 → 1,195.23 ns (+9.46%), despite its lower IC count. No universal speed-up
is claimed. Input setup and reporting are outside timing; allocations/drop inside
encoding remain included.

Allocator and DHAT builds run separately from timing, seven samples per case.
All nonempty fresh outputs make one allocation, request exactly the encoded byte
length, and have that many peak live output bytes; unit allocates zero. Reused
Vec and slice operations allocate zero, with zero additional live output bytes.
The slice still owns no storage: its caller supplies a 131,072-byte buffer.
Reported Vec capacity is the retained Vec capacity; requested/live/capacity are
separate quantities and do not measure allocator overhead or the total process.
Raw samples: [opt-z](evaluation-native/native-time.json),
[opt-3](benchmark-speed-native/native-time.json), and sibling
`native-allocations.json` / `native-heap.json` files in those directories.

## JavaScript and host generation

[Runtime timing runner](runtime-bench.mjs) compares compiled 0.7.18 runtime source
with the final shared writer/reader implementation, plus explicit payload calls.
Compile each source as `.mts` using the prepared strict ES2022/NodeNext compiler,
then run `node runtime-bench.mjs BEFORE.mjs AFTER.mjs OUTPUT.json`. Repeat in
three fresh Node processes. Node/compiler configuration and compiled runtime
hashes are retained in each [raw result](javascript/results-shared-writer.json).
The compiler is TypeScript 7.0.2 and Node is 24.21.0. Use `.mts` inputs and
`--strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022
--module NodeNext --moduleResolution NodeNext` for compilation. Before source is
`git show c748379:crates/ic-wire/src/typescript/runtime.ts`; after is the current
runtime. The runtime source and compiled-module hashes identify these separately.
Each process records seven warmed samples, alternating lane order. Unit/u64,
small and 4,096-word lists, 32-byte/64-KiB blobs and a record are controlled.
Frames are byte-identical; payload bytes equal the original frame after byte 11.

[Aggregate medians](javascript.csv) use 21 batch samples. Existing framed unit
encode is 487.23 → 442.68 ns (-9.14%); unit decode is 106.11 → 126.69 ns (+19.40%).
Large-list encode is 194,226.41 → 198,986.73 ns (+2.45%); record decode is
803.13 → 872.00 ns (+8.58%). Other gains/regressions remain in the CSV; GC/JIT
variation prevents interpreting small shifts as stable savings. An earlier
conditional writer constructor was rejected after a tiny-frame slowdown; these
results use the simpler shared buffer with explicit header writing.

Existing host generation uses seven 100-iteration samples at 1 and 100 types,
with identical metadata, compiler, lockfile, profile and redirected diagnostic
callback. [Before](host-before/native-schema-host-benchmark.json) and
[after](host-after/native-schema-host-benchmark.json) retain every sample.
Derived 1-type generation is 13,162 → 12,614 ns (-4.16%); 100 types are
1,233,504 → 1,222,774 ns (-0.87%). These include metadata construction and
printing diagnostics to redirected stderr; no generator savings claim is made.
220 existing generated TypeScript modules are byte-identical, including direct
and re-exported Serde crate-path fixtures. Only the shared runtime changes;
payload/facade modules are additions. Compile-time derive token equality is also
unit-tested, including retained rejection of layout-changing attributes.

Run [analyze.mjs](analyze.mjs) on the retained evidence directory to reproduce the
CSV medians and percentage changes. New source/binary identity requires a new
output directory; keep prior observations and failures intact.
