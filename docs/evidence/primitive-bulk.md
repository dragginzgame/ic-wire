# Follow-up: advice, missing Candid controls and private bulk research

> The 7 November and older raw artifacts were found retired on 8 November.
> Those sections are historical; their original paths no longer provide proof.
> New qualification uses the maintained ID cohorts and fresh output directories.

## Byte-aligned Packed copy (8 November)

Keep the byte-aligned slice-copy change for Core **0.17.14**. On little-endian
targets, the existing Packed visitor copies `i8`, 16-byte IDs and 32-byte hashes
through a checked bytemuck slice into fallibly reserved storage. Wider integers
and big-endian targets retain their existing readers. No unsafe code, dependency,
public API, new codec or per-record generated machinery is added. The initial
zero-initialize-then-copy candidate was rejected: it grew total Wasm by 321/549
bytes in the ID fixture and 1,265/1,104 bytes in the bulk fixture at opt-z/3.

**IC decode instructions per operation**, normal Wire framing included:

| Workload | Profile | Before | After | Matched Candid |
| --- | --- | ---: | ---: | ---: |
| 4,096 owned 16-byte IDs | opt-z | 112,047 | 67,100 | 96,648 |
| 4,096 owned 16-byte IDs | opt-3 | 66,620 | 66,609 | 85,978 |
| 16,384 signed bytes | opt-z | 394,599.5 | 17,879.5 | 1,127,362.5 |
| 16,384 signed bytes | opt-3 | 17,432.5 | 17,432.5 | 1,461,213.5 |
| 1,024 32-byte hashes | opt-z | 37,348.5 | 34,383.5 | 5,268,300.5 |
| 1,024 32-byte hashes | opt-3 | 33,957.5 | 33,945.5 | 4,741,164.5 |

The ID Candid DTO is a flat blob plus width checks. The older bulk corpus keeps
its nested hash DTO; do not compare the two Candid rows as equivalent layouts.
Large ID opt-z decode improves **40.1%** and signed bytes **95.5%**. Smaller inputs
can regress: singleton ID opt-z decode is **1,105.5 → 1,203.5** instructions;
eight IDs are **1,384.5 → 1,405.5**. The worst observed increase is **104** for a
singleton hash (1,145.5 → 1,249.5), including the corresponding nested late-failure
path. Opt-3 empty ID decode adds **3** instructions. The maintained regression
policy passes all **1,032 Wire operation pairs** across the 20-case ID and
109-case bulk cohorts, including rejection; every IC row has three identical
samples with distinct callers. Candid counters, frames and input hashes match.

Whole uncompressed Wasm **total / code-section bytes**, no post-link optimizer:

| Fixture | Profile | Before | After | Total / code delta |
| --- | --- | ---: | ---: | ---: |
| ID | opt-z | 50,256 / 40,910 | 50,226 / 40,880 | -30 / -30 |
| ID | opt-3 | 55,757 / 46,616 | 55,738 / 46,597 | -19 / -19 |
| Bulk | opt-z | 98,118 / 86,965 | 98,048 / 86,895 | -70 / -70 |
| Bulk | opt-3 | 122,644 / 112,201 | 122,616 / 112,173 | -28 / -28 |

These are complete fixture modules, not application savings or per-type
attribution. Code size is the final module's section-10 payload. Two serial
native ID runs/profile retain eleven samples: large opt-z decode medians are
**1,975/1,944 → 1,048/1,083 ns**, while opt-3 is **1,058/1,064 → 1,052/1,049 ns**.
Native time is distinct from IC instructions. Bulk native probes retain only one
three-sample run for frame/allocation controls; no stable native speed claim is
made for that cohort. Allocation counts and peak payload heap are unchanged in
both cohorts. Existing bytes, schemas, identities, budgets and malformed-input
acceptance remain unchanged: one packed ID is still `10 + 16 bytes`, and eight
IDs remain `80 01 + 128 bytes`. No consumer migration or regeneration is needed.

Reproduction: `target/evidence/packed-copy-20261108/` retains `screen.sh`,
`screen-aligned.sh`, `measure.sh`, `measure-bulk.sh`, both source snapshots,
separate Cargo targets, the rejected candidate, raw measurements and analysis.
Base is **3ac49b5** plus the preserved Testkit **0.93.1** lock selection, SHA-256
`e779f17c4f9a0bcbd5d773434d4a2741b6a701ab320e99592108f4fb0838f64a`.
Rust/Cargo **1.99.1**, LLVM **23.1.1**, Linux host, `wasm32-unknown-unknown`, fat
LTO/one codegen unit/panic abort, `evaluation` opt-z and `benchmark-speed` opt-3;
PocketIC **16.1.0**. Platform 0.26.0, Types 0.18.2 and Runtime 0.20.1 are fixed.
The maintained cost checker verifies fresh library/guest builds, exact loaded
artifacts and paired controls; baked Packed/lock hashes match each source.
Native macOS, actual big-endian execution and installed application performance
are not claimed. Focused Packed byte/admission/endian tests and lint qualify the
implementation; this is not a new full Candid matrix.

## Public 16-byte IDs and flat Candid (8 November)

Keep explicit `Packed<[u8; 16]>` support for pending Core **0.17.13**. It uses the
existing sealed word implementation and byte-buffer codec, with no arbitrary-width
generic family, second codec or automatic field rewriting. Host metadata uses
`hash16`; generated clients use a flat `Uint8Array` with length divisible by 16.
Default advice recognizes ordinary ID lists and retains exact-path suppression.

The maintained `id16` / `id16-control` cohorts compare the same 20 workloads:
lists and filter records at 0/1/8/128/4,096 IDs, with owned and borrowed-input
encode paths. Candid now receives **one flattened blob**, including a decode-time
width check, rather than thousands of byte vectors. Filter cursors also validate
16-byte width. Candid setup is outside counters; its decoded bytes are not copied
into an array vector. Wire's borrowed-input control includes cloning its existing
array vector before wrapping it. This favors an already-flat Candid consumer and
is not an equal-cost application conversion benchmark.

**IC instructions/op for 4,096 owned IDs**, including normal codec framing:

| Representation | opt-z encode / decode | opt-3 encode / decode | Frame bytes |
| --- | ---: | ---: | ---: |
| Ordinary Wire arrays | 10,098,515 / 11,609,194 | 5,096,419 / 7,483,283 | 65,549 |
| Public Packed IDs | 67,609 / 112,047 | 66,637 / 66,620 | 65,550 |
| Flattened Candid + width check | 155,418 / 96,673 | 149,171 / 85,862 | 65,548 |

The earlier approximately 129× Candid decode comparison was against nested
vectors. Against this flattened control, opt-3 Wire uses **55.3% fewer encode
and 22.4% fewer decode instructions**. At opt-z, large-list decode is **15.9%
more expensive than Candid**, while encode still wins. These are the two losses
among 80 primary comparisons, counting both ownership controls; all 40 opt-3
comparisons favor Wire. Keep this remaining gap visible rather than adding a
separate decoder to chase it.

All 64 nonempty comparisons improve over ordinary arrays. Thirteen empty-input
comparisons regress by 23–186 instructions. Empty-list opt-3 encode/decode is
**673.25 / 411 → 709.25 / 467**; singleton is **1,966.25 / 2,557 → 835.25 / 804**.
For 128 IDs it is **160,040.25 / 236,116 → 2,950.25 / 2,942**, with unchanged
2,061-byte Wire frames. Borrowed-input large encode costs **133,873 / 132,725**
opt-z/3 instructions. It adds a 64 KiB temporary and changes measured peak payload
heap from 65,549 to 131,086 bytes (two allocations instead of one). Owned-vector
wrapping/recovery preserves the allocation. Packed decode keeps the same 64 KiB
peak while eliminating the ordinary vector's ten reallocations. Separate native
allocator/heap probes record these costs; they are not IC heap measurements.

Whole ID fixture **uncompressed Wasm total / code-section bytes**:

| Representation | opt-z | opt-3 |
| --- | ---: | ---: |
| Ordinary Wire arrays | 51,772 / 42,422 | 56,205 / 47,075 |
| Public Packed IDs | 50,288 / 40,942 | 55,757 / 46,616 |
| Flattened Candid | 280,150 / 250,103 | 457,612 / 428,658 |

Packed saves **1,484 / 1,480** total/code bytes at opt-z and **448 / 459** at
opt-3 in this maintained fixture. Do not compare these deltas with the earlier
prototype as if only production code changed: the fixture also changed. These
are complete fixture modules, not projected application savings.

A separate existing **109-case bulk** fixture is byte-identical before/after
adding the unused element implementation at both profiles: **98,123 / 86,970**
opt-z and **122,644 / 112,201** opt-3 total/code bytes. The new support adds no
linked Wasm in this control when it is unused; this is not a proof for every app.

Native medians in **µs**, run 1 / run 2 (eleven samples per run):

| Profile / owned operation | Ordinary Wire | Packed Wire | Flat Candid |
| --- | ---: | ---: | ---: |
| opt-z encode | 299.910 / 325.416 | 1.046 / 1.086 | 2.676 / 2.857 |
| opt-z decode | 383.936 / 399.820 | 0.986 / 1.068 | 1.862 / 1.903 |
| opt-3 encode | 103.909 / 96.323 | 1.028 / 1.035 | 2.489 / 2.508 |
| opt-3 decode | 143.826 / 125.319 | 1.033 / 1.037 | 1.567 / 1.605 |

Native time and IC instructions are different measurements; opt-z demonstrates
why one cannot substitute for the other. The host is a shared Threadripper 7970X,
not latency-isolated. Generator timing, full application Wasm, actual native
big-endian execution and installed consumer integration were not measured here.

Rust exact-byte/width/prefix/unaligned/admission tests pass. Browser qualification
passes 178 bidirectional vectors, 39,448 differential mutations, and 66 simulated
big-endian cases. Standalone no_std host/Wasm consumers pass with and without
metadata, including codec-only dependency isolation. Core and both benchmark
cohorts pass Clippy; default advice visibility/suppression and dependency
boundaries pass. The consumer-owned conversion and JSON projection still need
review when adopting a field.

**Adoption:** one ID changes `01 + 16 bytes` to `10 + 16 bytes`; eight change
`08 + 128 bytes` to `80 01 + 128 bytes`. Opaque ID bytes retain their order.
Coordinate identities and clients, preserve original retry/hash bytes, and
requalify buffer budgets and field bounds. Existing fields keep their bytes,
identities, limits and acceptance. No compatibility reader is introduced.

Fresh evidence: `target/evidence/id16-flat-20261108/`, with paired `ordinary` and
`packed` source copies and separate Cargo targets, `run.sh`, `compare.mjs`,
`comparison.json`, `verified-artifacts.json`, and `size-controls.sh` /
`unchanged-size.csv`. Baseline is **72ab905** Core source plus the new ordinary
fixture and current lock; candidate is the public implementation and Packed
fixture selection. All eight measured guest builds are fresh, as are the four
existing-field size controls. Baked runtime/fixture/lock hashes and all 16 native
run identities match retained inputs/executables. Each of the 800 IC rows has
three identical samples. Candid frames, counters and total/code sizes match
between cohorts; its binary hashes differ between feature builds, so they are
not claimed byte-identical.

Rust/Cargo **1.99.1**, LLVM **23.1.1**, Candid **0.10.40**, Platform **0.25.1**,
Types **0.18.2**, Runtime **0.20.0**, Testkit **0.93.0**, PocketIC **16.1.0**.
The pre-existing 0.25.1 selection stayed unchanged throughout qualification.
Profiles use fat LTO, one codegen unit, panic abort and stripped symbols, without
post-link optimization. Reproduce both cohorts with the documented evaluator in
fresh directories and the same prepared server. No full release gate or complete
38-cohort matrix was run; the historical 36-cohort results are not relabelled.

## Packed 16-byte ID experiment (7 November)

This historical prototype is superseded by the public qualification above.

**Promising for explicit field adoption; research only.** Extend the existing
sealed Packed element set in disposable sources with `[u8; 16]`, using the same
reader and byte-buffer codec as `[u8; 32]`. Do not introduce an arbitrary-width
generic family or a second codec. Production sources, supported schemas, bytes,
identities, budgets and acceptance remain unchanged; no consumer action is needed
for this experiment. Public promotion still needs Rust/TypeScript schema,
cross-endian, admission, standalone-consumer and default-advice qualification.

Twenty workloads cover lists and filter records at 0/1/8/128/4,096 IDs, each with
owned fields and conversion from borrowed ordinary data. Compare ordinary
`Vec<[u8; 16]>`, existing `Vec<ByteArray<16>>`, and prototype `Packed<[u8; 16]>`.
Filters also contain an optional ordinary ID cursor, a bool and a u16 limit.
IDs use identical deterministic mixed bytes. Candid's identical external DTOs
use lists of byte vectors; their construction is outside measurements and they
do not enforce the fixed 16-byte width. These are valid-value codec comparisons,
not equivalent malformed-value policies or whole application measurements.

IC instructions/op for **4,096 owned IDs**, with complete frame bytes:

| Representation | opt-z encode / decode | opt-3 encode / decode | Frame bytes |
| --- | ---: | ---: | ---: |
| Ordinary arrays | 10,098,515 / 11,609,169 | 5,096,419 / 7,483,281 | 65,549 |
| ByteArray fields | 2,967,380 / 967,796 | 488,416 / 704,399 | 69,645 |
| Packed prototype | 67,609 / 112,022 | 66,637 / 66,618 | 65,550 |
| External Candid DTO | 2,072,287 / 13,407,128 | 979,501 / 8,595,661 | 69,645 |

Packed reduces opt-3 encode/decode instructions by **98.7% / 99.1%** against
ordinary arrays. It beats this Candid control in all 80 primary comparisons.
All 64 nonempty comparisons improve over ordinary arrays, including singleton
and record controls. Thirteen empty-input comparisons regress by **23–186**
instructions; for example empty-list opt-3 encode/decode changes
**673.25 / 409 → 709.25 / 465**. This is not an unconditional improvement.
For 128 owned IDs, opt-3 encode/decode changes
**160,040.25 / 236,114 → 2,950.25 / 2,940**, with unchanged 2,061-byte frames.
The corresponding filter changes **162,497.25 / 238,165 → 4,032.25 / 4,825**.

Wrapping an owned vector in Packed and recovering it preserves its allocation;
the focused test checks the pointer. Borrowed-input encoding includes one clone
before encoding, not hidden setup: 4,096-ID Packed encode becomes
**133,873 / 132,725** opt-z/3 instructions. Its peak measured payload heap grows
from **65,549 to 131,086 bytes**, with two allocations instead of one. Owned
Packed decode keeps a 65,536-byte peak but eliminates ordinary/ByteArray decoding's
ten reallocations. Native allocator and peak-heap probes are separate builds.

Repeated native medians, µs; each cell is run 1 / run 2, with eleven samples/run:

| 4,096 IDs | opt-z ordinary | opt-z Packed | opt-3 ordinary | opt-3 Packed |
| --- | ---: | ---: | ---: | ---: |
| Owned encode | 291.540 / 300.831 | 1.100 / 1.099 | 97.338 / 99.281 | 1.026 / 1.194 |
| Owned decode | 336.286 / 339.947 | 1.815 / 1.782 | 127.843 / 127.480 | 1.026 / 1.237 |
| Encode including borrowed-input copy | 295.297 / 294.703 | 2.028 / 2.050 | 96.040 / 97.935 | 2.291 / 2.698 |

These are serial warm microbenchmarks on a shared Threadripper 7970X host, not IC
instructions or request latency. Sample variation remains visible; do not project
these ratios onto consumer traffic. Truncating the large list frame costs
**11,609,043 / 7,483,225 → 777 / 686** opt-z/3 instructions: the packed byte count
permits refusal before payload allocation/traversal. Truncating a filter's final
scalar still decodes its preceding ID field; this is not universal constant-time
malformed-input rejection. Wrong-header and loop controls are also retained.

Whole 20-workload fixture **uncompressed Wasm total / code-section bytes**:

| Representation | opt-z | opt-3 |
| --- | ---: | ---: |
| Ordinary arrays | 51,419 / 42,067 | 55,768 / 46,630 |
| ByteArray fields | 51,049 / 41,696 | 57,012 / 47,860 |
| Packed prototype | 49,935 / 40,587 | 56,370 / 47,213 |
| Candid | 287,343 / 257,048 | 468,862 / 439,710 |

Packed saves **1,484 total / 1,480 code bytes at opt-z**, but adds
**602 / 583 bytes at opt-3**. Both ownership paths remain reachable in these
modules. This is a field-representation tradeoff, not a general Wasm reduction
or an application savings claim. No consumer compiler settings were changed.

Potential adoption changes the payload prefix from an **element count** to a
**byte count**: one ID changes `01 + 16 bytes` to `10 + 16 bytes`; eight change
`08 + 128 bytes` to `80 01 + 128 bytes`. ID bytes retain their original order.
The byte count must be divisible by 16; partial words are malformed. Adopters
must coordinate contract identities/clients and requalify buffer admission and
field bounds. Preserve original retry/hash bytes. Never rewrite existing Serde
fields silently or add an old-layout reader. The current public API is unchanged.

Reproduction: `target/evidence/id16-20261107/{screen.sh,run.sh,compare.mjs,
verify-screen.mjs}` with `ordinary`, `bytes` and `packed` paired source/target
directories. Each source includes the same experimental element impl; only the
selected fixture representation differs. The normal `cost-smoke` fixture is
replaced only in these disposable sources, retaining the existing evaluator.
Use fresh output directories and a prepared PocketIC server for reruns. Baseline
is **72ab905** plus pending Testkit **0.93.0**; Rust/Cargo **1.99.1**, LLVM
**23.1.1**, Candid **0.10.40**, Platform **0.25.0**, Types **0.18.2**, Runtime
**0.20.0**, PocketIC **16.1.0**. Both Wasm profiles use fat LTO, one codegen unit,
panic abort and symbol stripping, without a post-link optimizer.

All three seven-test runs pass, including exact bytes, prefix truncations,
trailing input, partial packed width and the relevant corpus roundtrips.
Twelve fresh guest modules, baked source/lock/input identities and all 24 native
run identities are verified. The 1,200 IC rows each have three identical samples;
Candid modules and counters are identical across all three representations.
`comparison.json` retains every row, regression, native run and allocation/heap
probe. No full production matrix or installed consumer qualification was rerun.

## Public bool decoder size screen (7 November)

Reject three further candidates; no production change is kept. The current
124-case `wire,research-bulk` fixture exercises the public bool adapter alongside
the existing private float controls. All candidates grow both total uncompressed
Wasm and its code section, so none advances to native timing or IC qualification.
Compilation and size measurements do not prove their runtime performance or
malformed-input correctness. Production bytes, schemas, identities, budgets and
acceptance remain unchanged; consumers take no action.

| Variant | opt-z total / code bytes | opt-3 total / code bytes |
| --- | ---: | ---: |
| Current decoder | 215,993 / 181,506 | 271,026 / 238,940 |
| Outline the validation/allocation/copy visitor | 216,080 / 181,592 | 271,114 / 239,027 |
| Outline only the safe checked cast | 216,055 / 181,567 | 271,126 / 239,039 |
| Validate 32-byte chunks by OR reduction, then construct bools | 216,103 / 181,600 | 271,327 / 239,241 |

The two outlining variants add 62–100 total bytes. Chunked validation adds
110/301 total bytes at opt-z/3. These are complete fixture modules, not projected
application savings. Native time, IC instructions, allocation/heap comparisons
and new Candid measurements were deliberately not run after the size refusal.

Reproduction and all eight fresh modules remain under
`target/evidence/bool-outline-20261107/`: `screen.sh`, `chunked.sh`, `size.csv`,
`sources.json`, per-variant sources, separate Cargo targets and build JSON/logs.
Baseline is **72ab905** plus the pending Testkit **0.93.0** adoption; each candidate
differs only in `serde_packed/bools/mod.rs`. Source inventories verify identical
manifests, lock, corpus and profiles; Cargo marks every Core/guest artifact as
rebuilt, and retained module hashes match the emitted artifacts. Rust/Cargo
**1.99.1**, LLVM **23.1.1**, `wasm32-unknown-unknown`, fat LTO, one codegen unit,
panic abort, symbol stripping, no post-link optimizer. `evaluation` uses opt-z;
`benchmark-speed` uses opt-3. No consumer profile setting is changed.

## Checked bool-copy follow-up (7 November)

Keep the safe checked-slice copy in **private research**. Bytemuck validates
every bool bit pattern before allocation; the decoder then copies the valid
slice instead of reconstructing each element. Float decoding is unchanged.
No public adapter, schema or client is added. Production bytes, identities,
limits and acceptance are unchanged; the private representation also preserves
its bytes, bounds and rejection rules (`02 00 01` still means `[false, true]`,
and any bool byte above `01` is rejected). Consumers take no migration action.

IC decode instructions/op, before → after:

| Bool count | opt-z | opt-3 |
| ---: | ---: | ---: |
| 0 | 561.5 → 564.5 | 312.5 → 312.5 |
| 1 | 993.5 → 1,078.5 | 578.5 → 651.5 |
| 128 | 6,794.5 → 3,704.5 | 4,614.5 → 3,228.5 |
| 16,384 | 738,626.5 → 329,136.5 | 516,939.5 → 328,609.5 |
| 1,048,576 | 47,187,427 → 20,973,137 | 33,031,148 → 20,972,610 |

Large decode improves **55.6% / 36.5%**, with singleton costs **+85 / +73**
instructions. Bool encode and every non-bool IC counter are unchanged. The
matched Candid large decode costs remain **403,732,016 / 208,686,329**.
Wire's large frame remains **1,048,590 bytes**.

Whole 124-case fixture uncompressed Wasm, total / code-section bytes:

| Profile | Before | After | Delta |
| --- | ---: | ---: | ---: |
| opt-z | 215,968 / 181,497 | 215,958 / 181,487 | -10 / -10 |
| opt-3 | 271,183 / 239,113 | 271,010 / 238,940 | -173 / -173 |

Candid stays **710,264 / 648,591** at opt-z and **1,053,767 / 994,259** at
opt-3. These are fixture sizes, not application savings. Native large decode
run medians in µs are **668.429 / 636.207 → 278.003 / 259.611** at opt-z,
but **266.104 / 252.335 → 270.854 / 262.325** at opt-3: a **1.8% / 4.0%
regression** on the shared host. This is not a universal native-time speedup.
All allocation/heap probe rows agree; large decode retains one allocation,
zero reallocations and 1 MiB peak payload heap.

Qualification uses Rust/Cargo **1.99.1**, LLVM **23.1.1**, native Linux and
`wasm32-unknown-unknown`, fat LTO, one codegen unit and panic abort. Baseline
**7983e8fb33d8be9ff52553ecc4c6664c4e37a48c** and the candidate use isolated
source snapshots and separate fresh targets, with identical lock SHA-256
`e0b2abc6f6bc868faf7fd8c76cf923ccdb77c5ea49ddfbf1ccac3c3b4a43d1a6`.
Build 0.78.0, Testkit 0.91.0 and PocketIC 16.1.0 supply this study.

Reproduce with the retained `run.sh`: the maintained `evaluate-codecs.sh` runner,
`CORPUS_FEATURE=research-bulk RUNS=2 SAMPLES=11 BATCH_MS=2`, a prepared
`POCKET_IC_BIN`, and separate `CODEC_SOURCE_SNAPSHOT` / `CARGO_TARGET_DIR` per
variant. Native, allocation and heap probes run separately. IC measurements use
three distinct callers, with four repetitions for small values and one for large
values; counters include result destruction/loop overhead, excluding transport
and setup. All **2,594 paired IC rows** preserve input/frame identities; all
three samples agree, and **2,585 rows** have identical instructions. The verifier
checks unchanged source inventories, baked host source hashes and fresh Wasm
artifact hashes. All **14** research tests and strict caller Clippy pass, including
every invalid bool byte at every position of lengths 1/31/32/33. Public promotion
still requires schema/client and resource-admission qualification.

### Bool promotion: byte and admission qualification

The next focused checks use Core 0.17.10 with Testkit 0.92.0, Rust 1.99.1 on
x86_64 Linux. They change tests only, not runtime code. Across 11 lengths from
0 to 16,384 (including 127/128 and 16,383/16,384) and all-false, all-true and
alternating patterns, ordinary `Vec<bool>` and private `Bulk<bool>` produce
identical payloads and method frames and decode each other's values. Truncation
and nonminimal length prefixes remain `Malformed`; extra bytes remain `Trailing`.
The existing exhaustive invalid-bit-pattern check also passes.

For 128 bools, the exact successful native allowances are below. Reducing any
one of bytes, allocation, nodes or depth by one refuses with `Limit`.
These are **64-bit host admission thresholds**, not measured heap allocation
or Wasm costs; custom Serde projections and record wrappers can change charges.

| Representation / operation | Bytes | Allocation allowance | Nodes | Depth |
| --- | ---: | ---: | ---: | ---: |
| Ordinary encode | 130 | 16,898 | 516 | 4 |
| Bulk encode | 130 | 1,538 | 4 | 3 |
| Ordinary decode | 130 | 2,370 | 257 | 2 |
| Bulk decode | 130 | 1,346 | 1 | 1 |

Serde's ordinary `collect_seq` hands the serializer a reference for each element;
encoding therefore charges the reference size rather than one bool byte. Bulk
admission charges one byte buffer. This confirms why byte equality alone cannot
justify silently replacing consumer Serde or keeping limits without review.
No budget policy changes here.

Run the three focused tests with `cargo test --locked --offline -p
ic-wire-benchmarks --example codec-wasm --features wire,candid,research-bulk
private_bulk_bool`. Logs are under `target/bool-admission-qualification/`.
No new native-time, IC-instruction or Wasm-size claim follows from test-only work.
This established the remaining promotion requirement: existing Packed words
require every bit pattern to be valid, while bool requires validation. The
explicit adapter below preserves that invariant and reuses the existing codec.

## Public bool adapter for 0.17.11

Keep `serde_packed::bools` for explicit `Vec<bool>` fields. It uses the checked
copy algorithm without weakening `PackedElement` or adding a wrapper type,
dependency, const-bound codec variant or second codec. The metadata derive
recognizes the canonical adapter path, including a leading `::`, and checks
type equality. Generated values are `boolean[]`; the shared browser runtime
checks 00/01 before allocating its decoded array. Default bool-list advice points
to this adapter while retaining bounds/admission guidance.

Adoption preserves valid bytes (`[false,true]`: `02 00 01`), but changes traversal
to byte-buffer admission. Regenerate clients and review field bounds and budgets;
`List<bool, N>` cannot silently become an unbounded vector. Existing method
identities, ordinary lists, Packed words, budgets and primitive acceptance are
unchanged. This compatible opt-in addition requires no migration for existing users.

The paired 124-case study lives under `target/evidence/public-bools-20261107/`.
Baseline is Core 0.17.10 source **cb9b29a** plus the pending byte/admission tests;
the candidate uses the public adapter for its bool cases and retains private
float research. Both use Rust/Cargo 1.99.1, LLVM 23.1.1, Testkit 0.92.0,
PocketIC 16.1.0 and lock SHA-256
`5e91f850a9af1fc8af6aac59af2dda9b5b36f0d4da47e158a21cd37147ae490e`.
Separate source snapshots and fresh Cargo targets retain full source inventories,
baked host hashes (including the new adapter) and fresh guest artifact records.
The maintained runner uses `CORPUS_FEATURE=research-bulk RUNS=2 SAMPLES=11 BATCH_MS=2`
with each variant's `CODEC_SOURCE_SNAPSHOT`, `CARGO_TARGET_DIR` and prepared server.
Profiles and IC sampling match the checked-copy study above.

All **2,594 paired IC rows** retain identical input/frame hashes and lengths;
all three counters per row agree. **Every Wire IC count is unchanged**, including
empty, singleton, large and rejection controls. All allocation/heap rows match.
Twenty Candid bool rows drift after the fixture changes from its generic word
case to the explicit adapter pair; there is no Candid dependency change. Do not
attribute that control-side code-generation difference to the production adapter.

Current 1,048,576-bool results, IC instructions/op:

| Profile | Wire encode | Candid encode | Wire decode | Candid decode |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 1,050,616 | 2,118,104 | 20,973,137 | 403,732,214 |
| opt-3 | 1,049,585 | 2,112,301 | 20,972,610 | 208,686,520 |

Frame sizes are **1,048,590 Wire / 1,048,588 Candid**. Whole fixture uncompressed
Wasm, total / code-section bytes (these are not application savings):

| Profile | Private baseline Wire | Public adapter Wire | Candid before → after |
| --- | ---: | ---: | ---: |
| opt-z | 215,958 / 181,487 | 215,993 / 181,506 | 710,264 / 648,591 → 710,372 / 648,683 |
| opt-3 | 271,010 / 238,940 | 271,026 / 238,940 | 1,053,767 / 994,259 → 1,053,861 / 994,337 |

Wire adds **35 total / 19 code bytes** at opt-z and **16 total / 0 code bytes**
at opt-3. Large decode still allocates once with zero reallocations and 1 MiB
peak payload heap. Native timing initially overlapped independent builds, so
both profiles were repeated serially in alternating before/after order, twice,
with eleven samples and the same 2 ms minimum batches. Large Wire decode medians
in µs are **249.335 / 253.303 → 249.037 / 252.781** at opt-z and
**244.875 / 268.250 → 493.751 / 468.683** at opt-3. The latter is a real
**1.75–2.02× native regression against the private prototype**, despite identical
IC counts. Candidate Candid opt-3 decode is **3,292.355 / 3,394.289 µs**.
Native code-generation follow-up remains useful; do not claim universal speed
parity or substitute native timings for IC instructions.

Host generation was also run twice per isolated source, serially, using the
maintained `typescript-conformance` example in the default dev profile. Each
entry below is a median of seven samples, 100 generations/sample; diagnostic
printing is excluded. Absolute before → after medians in µs:

| Types / metadata | Run 1 | Run 2 |
| --- | ---: | ---: |
| 1 / manual | 18.297 → 18.530 | 18.424 → 18.488 |
| 1 / derived | 21.242 → 21.559 | 21.681 → 21.555 |
| 100 / manual | 1,686.181 → 1,740.435 | 1,698.623 → 1,748.898 |
| 100 / derived | 2,014.974 → 2,048.926 | 2,003.822 → 2,045.939 |

The 100-type controls regress about 1.7–3.2% on this shared host; these are
host generator times, not codec instructions. All 36 existing projection files
remain byte-identical. Shared browser runtime TypeScript source grows
21,574 → 22,500 bytes (+926); this is source size, not bundled JavaScript or Wasm.
The paired verifier checks all source/frame identities, including the serial
native reruns. No browser timing or application-linked Wasm claim is made.

Focused qualification passes: public/private byte and all-four-budget boundaries,
all invalid bool bytes, nested Rust fields, 41 generated exact-frame vectors,
malformed/sparse/non-boolean browser input, independent decoded ownership and
single-buffer node admission. Standalone host/Wasm feature graphs and eight
compile-fail diagnostics with the passing control also pass. Logs live under
`target/public-bools-checks/`. No release or full CI gate is claimed by this work.

## Earlier primitive-bulk study

This supplements the frozen [Platform 0.21 full matrix](../benchmarks/candid-comparison.md).
The production runtime remains one Serde/Postcard codec. Candid is used only by
unpublished qualification through the Platform facade; owned Principal/Account
values are measured on Wire's side and external DTO conversion stays outside
primary codec counters. No production bytes, identities, budgets or primitive
acceptance rules change in this batch.

The [host generator study](schema-generation.md) qualifies newly detected
`Vec<[u8;32]>` advice and an eight-entry default diagnostic cap, retaining every
warning and exact acknowledgement in the report. Generated TypeScript bytes are
unchanged in all 24 matched outputs. Added advice has a measured host generation
cost; see both rounds and tiny controls rather than treating it as a speedup.

The separate 94-case `coverage-shapes` cohort fills the previously identified
owned identity, byte-adapted Candid, nested packed, deterministic hash collection,
bounded Nat, standard-library and larger IEEE-bit/bool controls. `research-bulk`
uses identical logical inputs and Candid DTOs while changing only the private
float/bool fields on Wire's side. Neither cohort changes existing case indices.

The research serializer uses byte-buffer admission and little-endian word data;
the bool decoder validates every byte before output allocation. Float prefixes
change from element counts to byte counts (`01 00 00 80 3f` becomes
`04 00 00 80 3f` for `[1_f32]`). Bool bytes remain `02 00 01` for `[false,true]`,
but buffer admission has different budgets from element visits. These are
explicit private qualification representations, not new public Packed kinds,
automatic Serde rewrites, generated schemas or browser-client support.
Any future production adoption requires contract/client/bounds qualification and
coordinated consumers, including retained retry/hash inputs. No promotion is
made by this study.

## Measured result

Two cohorts × 94 cases, **752 primary encode/decode comparisons**, **724 Wire
wins / 28 Candid wins**, no ties. These overlapping entries are not unique types
or a traffic weighting. All **3,988 IC rows** retain three distinct-caller samples
with identical counters. All eight measured Wasm modules are freshly compiled,
source-hash verified and match their loaded bytes; every frame matches native.
Candid frames and instruction counts agree across the paired cohorts. Candid
modules have equal total/code sizes but different hashes (feature-build TypeIds
may differ); we verify each loaded artifact rather than assuming byte identity.

Selected **opt-3 IC instructions/op**, 1 MiB of values:

| Shape | Operation | Ordinary Wire | Private bulk Wire | Candid | Wire change |
| --- | --- | ---: | ---: | ---: | ---: |
| bool | encode | 60,818,316 | 1,049,585 | 2,112,270 | -98.27% |
| bool | decode | 44,570,576 | 33,031,148 | 208,686,283 | -25.89% |
| f32 | encode | 12,583,841 | 1,049,588 | 2,112,195 | -91.66% |
| f32 | decode | 12,326,591 | 1,049,688 | 21,515,183 | -91.48% |
| f64 | encode | 6,292,385 | 1,049,588 | 2,112,195 | -83.32% |
| f64 | decode | 7,607,721 | 1,049,688 | 9,849,764 | -86.20% |

The prototype eliminates the large successful float/bool gaps **in these new
cohorts**. It does not fix existing production ordinary vectors automatically.
The private cohort's worst successful Wire/Candid ratios are **1.106 encode /
1.296 decode** at opt-z and **1.191 / 1.035** at opt-3. Existing ordinary controls
remain visible, reaching **46.04× / 28.79×** Candid encode for the 1 MiB bool list.

All largest frames are **1,048,590 Wire / 1,048,588 Candid bytes**. Frame lengths
remain equal between the ordinary and research fields for the selected lengths;
nonempty float prefix values differ. Other lengths can grow (e.g. 16 f64 values:
ordinary `10` prefix becomes byte-count `80 01`). Bool bytes are identical.

### Uncompressed Wasm

Whole 94-case fixtures; total / code-section bytes. Nat conversion and its
reachable big-integer work are included, so these are not minimal Core modules
or per-type savings. Candid sizes are unchanged across feature builds.

| Profile | Ordinary Wire | Research Wire | Candid | Wire total/code delta |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 199,089 / 165,437 | 198,150 / 164,499 | 658,034 / 597,813 | -939 / -938 |
| opt-3 | 245,688 / 214,426 | 244,124 / 212,864 | 988,710 / 930,676 | -1,564 / -1,562 |

The total Wire reductions are **0.47% / 0.64%**. Compression is only archive
storage and has no role in this conclusion.

### Regressions, native time and allocation

There are **14 tiny-input IC regressions** across the 60 successful paired
float/bool rows; no non-word primary IC regressions. Worst relative change is
singleton bool encode at opt-3: **520.25 → 672.25 (+29.22%, +152 instructions)**,
still below Candid's 13,655.25. Empty bool encode is **443.25 → 562.25 (+26.85%)**.
Every regression and both profiles are retained in Retired raw output (see retention notice)
and tables; instruction wins do not imply native-time wins.

Two serial native runs/profile each retain eleven samples. At opt-3, 1 MiB f32
encode is **321,909 → 17,978 ns** and **323,433 → 18,269 ns** (-94.42% / -94.35%);
decode is **187,576 → 17,482 ns** and **188,795 → 18,174 ns** (-90.68% / -90.37%).
Tiny native cases include noisy regressions; the full two-run tables preserve
them. Native time is not an IC counter or a physical-cycle measurement.

For 1 MiB f32 decode, allocator probes drop **16 reallocations → 0**, retaining
one allocation and the same 1 MiB peak live payload heap. f64 drops 15 → 0 and
bool 17 → 0. Encoding retains one output allocation with the same frame bytes
and peak live heap. These separate instrumented builds are not timed results.

## Better Candid controls and labelled conversion

At opt-3, the byte-oriented Candid Account control (128 explicit-zero accounts)
reduces decode **1,640,902.5 → 1,199,953.5 (-26.87%)**, but encode increases
**100,580.25 → 106,566.25 (+5.95%)**. The 128-entry hash-key control reduces decode
**1,571,524.5 → 1,130,147.5 (-28.09%)**, with encode **+2.74%**. Both pairs retain
identical Candid bytes. We keep plain and byte-adapted controls instead of
assuming one adapter wins every metric. These are this fixture's linked context,
not a source-matched regression against the older full-matrix modules.

The largest separately labelled Nat conversion lane costs **3,619.25 encode /
3,284.5 decode** opt-3 instructions, versus raw bounded bytes **876.25 / 697.5**
and Candid **13,749.25 / 27,106.5**. It includes application conversion; primary
codec rows do not. The 64 KiB borrowed record decodes at **938** instructions
versus owned Wire **67,272** and Candid **108,299**, using the identical frame.
Borrowing is inside an already-owned frame and is not ingress zero-copy or a
claim about application work that later requires ownership.

## Finite float field investigation — 9 November 2026

Recommendation: investigate two explicit Serde field adapters for `Vec<f32>` and
`Vec<f64>`, provisionally named `serde_packed::finite_f32` and `finite_f64`.
These are **proposals, not available APIs**. Reuse the existing byte-buffer codec;
keep ordinary lists and integer/hash Packed unchanged. No new crate, codec,
feature switch or automatic derive rewrite is needed.

The [full matrix](../benchmarks/candid-comparison.md) measures raw IEEE-bit research fields,
not these finite adapters. Its f32 input includes infinities, quiet/signalling
NaNs, signed zero, subnormals and finite extremes
(`benches/fixtures/followup/mod.rs`, `f32-bits-*`/`f64-bits-*`). A finite adapter
would reject those non-finite fixtures. Consequently the 55,838,553 → 1,050,623
opt-z encode result for 1 MiB is evidence for bulk transport, **not a measured
finite-adapter saving**. The combined bool/float Wasm difference likewise is not
an isolated float result.

### Boundary and simplest implementation

- `src/float/mod.rs` already defines finite-only schema floats, preserving negative
  zero and subnormals. `typescript/runtime.ts` rejects non-finite scalar values;
  `typescript/native_schema/mod.rs` deliberately omits raw Rust float metadata.
  Follow that existing browser contract instead of quietly extending it to NaNs.
- `packed/mod.rs` seals words behind `Pod + Eq`. Raw floats lack `Eq`; finite
  wrappers cannot be `Pod` because non-finite bit patterns are invalid. Do not
  alter this integer/hash owner merely to fit the finite domain. The established
  explicit bool adapter is the better model.
- On little-endian Rust, validate the borrowed primitive slice, then serialize its
  safe bytemuck byte view. No temporary encoding Vec is needed. Sizing and writing
  each invoke Serde, so the finite scan can run twice: this must be measured.
  Big-endian encoding needs an explicit fallible LE buffer path, as existing
  Packed encoding has; the current private float prototype rejects that host.
- Decode checks width and finite exponent bits in the borrowed byte span before
  reserving an output Vec, then reads unaligned LE words into owned storage.
  The existing native `CheckedVisitor::visit_bytes` charges the buffer before
  the adapter visitor runs. Reuse that admission; do not invent float budgets.
  Validation scans add work and can inhibit an optimized copy loop. Measure
  scan-then-copy against alternatives rather than assuming the raw result survives.
- Browser values can be `Float32Array` / `Float64Array`, with finite-word checking
  before allocating the decoded copy. Existing packed machinery already handles
  byte offsets, alignment, ownership and endian swapping. Check raw exponent
  words with DataView before copying; no NaN arithmetic or bigint loop is needed.
  A Float32Array is already rounded to f32 by its caller. Do not silently accept
  arbitrary `number[]` and round it inside the codec.

The adapter's custom scan before `serialize_bytes` is trusted Serde work, as
specified by the runtime contract; it is not bounded by per-element node visits.
Do not claim strict instruction admission for arbitrary custom Serde. Decode
buffer admission still precedes the scan and allocation. `Vec<f32>` is not
`Vec<FiniteF32>`: do not transmute wrappers, add unsafe casts or accidentally
accept a different domain type through syntactic macro matching.

### Adoption contract and complete integration scope

An adopting f32 field changes `[1.0]` from `01 00 00 80 3f` to
`04 00 00 80 3f`; f64 changes its singleton count `01` to byte count `08`.
Finite value bits stay exact, but count-prefix widths can change payload size.
Starting from raw float lists, NaN/infinity acceptance changes to rejection.
Starting from finite-value lists, that rejection policy already exists. Traversal
changes from per-element admission to a buffer, so bounds and allocation/node
allowances need requalification. Consumers must coordinate identities, regenerate
clients and preserve retained retry/hash bytes. No automatic migration/dual reader.

Promotion would update these existing owners together:

| Owner | Required work |
| --- | --- |
| Core `serde_packed` | Two concrete finite field adapters and endian/error tests |
| Derive `native_schema::attributes` / `fields` | Recognize exact adapter paths; verify actual Vec element types, including aliases |
| TypeScript metadata/runtime | Finite packed schema words, typed-array rendering, finite-before-copy checks, endian handling |
| Bulk lint | Recommend supported adapters only after qualification; retain ordinary float controls |
| Qualification fixtures | Rust/TS exact bytes, scalar finite-policy parity, malformed and budget refusal, standalone derives |
| Docs/consumer adoption | Explicit old/new bytes, field-bound preservation, identity and budget action |

The proposed Vec adapters do not introduce a per-field length bound. Existing
consumer bounds/custom checks must remain explicit; runtime byte limits are not
a substitute for all field constraints. Numeric per-element range constraints
must not disappear when moving from scalar/list metadata to a packed field.

### Executed feasibility probes and remaining proof

Evidence: `target/evidence/finite-float-investigation-20261109/`. With the admitted
Rust 1.99.1 compiler, four standalone tests pass: f32/f64 boundary classification,
offsets 0–15, partial words, empty input, invalid first/middle/last values, and
one million deterministic bit-pattern iterations against scalar `is_finite`.
The Node probe passes 1,280 classification/copy assertions, checking finite bit
preservation and that owned output does not retain the caller's buffer.

These are isolated semantic helpers, **not implemented adapters or generated-client
conformance**. Native typed-array copying was tested on little-endian only. No
new IC instruction, native timing, allocation or Wasm saving is claimed.

The next measured experiment needs matched finite-only f32/f64 inputs in both
lanes, separate from the existing all-IEEE controls: 0/1/small/large lengths,
signed zeros, subnormals and finite extremes. Compare ordinary finite lists,
raw bulk and checked finite bulk; retain Candid inputs identically. Measure
opt-z first, opt-3 losses, repeated IC samples, total/code Wasm and native
allocations. Separately exercise non-finite refusal, truncated/misaligned spans,
exact/one-short frame/allocation/node/depth limits, and check rejection precedes
allocation. Qualify the actual generated client and big-endian strategy before
promotion. No production bytes, identities, budgets, acceptance or API changed
in this investigation.

## Matched finite float cost study — 9 November 2026

**Keep the chunked scan as research; do not publish an adapter yet.** Finite
bulk fields substantially improve ordinary finite lists and reduce this fixture's
Wasm, but large f32 encoding still costs 3.40 times Candid at opt-z. Candid's raw
float DTO does not enforce Wire's finite-only policy. This is a matched finite
input comparison, not identical non-finite acceptance or an application saving.

### Matched inputs and executed proof

Six isolated sources use the same manifests/lock, Rust/Cargo 1.99.1, LLVM 23.1.1,
Postcard 1.1.3, Serde 1.0.229, Candid 0.10.40, Platform 0.27.1, Types 0.18.3,
Runtime 0.20.3, Testkit 0.95.0 and PocketIC 16.1.0. Source is df071a4 plus the
pending tooling/supplier adoptions. Lock SHA-256:
`a1b3e79781eb208c581a850aa2151f111df263ed8daaef93366c6c5e8e6e51e8`.
Each source has a separate Cargo target. The maintained evaluation script runs
opt-z/evaluation and opt-3/benchmark-speed, two native runs with eleven samples
and 2 ms batches, separate allocation/heap modes, and three distinct-caller IC
samples per operation. Every IC repeat agrees exactly.

The 36 cases include empty/singleton, small lengths around byte-prefix boundaries,
127/128 elements, 16,384 elements, and 1 MiB: 262,144 f32 or 131,072 f64 values.
Eight singleton edge cases per width cover signed zeros, signed subnormals,
finite extremes, smallest normal and one. Ordinary Wire stores public FiniteF32 /
FiniteF64 values, constructed before measurement; Candid stores primitive floats.
Inputs are identified by their exact bits. This replaces the research corpus
only in disposable sources; it does not add a second production codec.

The lanes are ordinary finite lists, raw bulk, scalar-checked bulk, checked plus
inline hints, explicit exponent masks, and an eight-word checked loop. The last
uses one non-inlined helper instantiated only for f32/f64; eight bit predicates
are combined before branching, with a scalar tail. Both encoding passes validate;
decode validates the borrowed bytes before reserving output. No per-field bound
or per-consumer codec specialization is introduced.

All 24 Wasm builds are fresh; baked source identities, loaded-module hashes,
code-section lengths, 4,320 IC rows and 13,824 native/frame comparisons verify.
All six Candid controls have identical frames and instruction counts. The five
bulk lanes have identical finite Wire frames. Initial allocator-test setup and
an incorrect test allowance search were fixed before completed measurements;
`ordinary/attempt-1` is incomplete and excluded. The filtered contract test passes
for both checked and chunked sources; this is not a pass of the original replaced
corpus's unrelated tests.

### IC instructions and uncompressed Wasm

Instructions per operation for 1 MiB of finite values:

| Profile / operation | Ordinary finite list | Raw bulk | Scalar checked | Eight-word checked | Candid |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z f32 encode | 72,877,809 | 1,050,470 | 12,060,534 | 7,210,930 | 2,118,122 |
| opt-z f32 decode | 19,930,633 | 7,079,382 | 13,108,705 | 10,159,614 | 19,166,309 |
| opt-z f64 encode | 36,964,081 | 1,050,470 | 6,555,510 | 4,130,731 | 2,118,225 |
| opt-z f64 decode | 10,362,009 | 3,540,438 | 6,555,105 | 5,080,572 | 9,991,272 |
| opt-3 f32 encode | 12,583,845 | 1,049,592 | 12,059,658 | 7,144,510 | 2,112,181 |
| opt-3 f32 decode | 14,685,793 | 1,049,690 | 7,079,011 | 4,097,148 | 18,893,809 |
| opt-3 f64 encode | 6,292,389 | 1,049,592 | 6,554,640 | 4,097,090 | 2,112,181 |
| opt-3 f64 decode | 8,525,137 | 1,049,690 | 4,064,355 | 2,573,438 | 9,063,423 |

| Lane | opt-z total / code bytes | opt-3 total / code bytes |
| --- | ---: | ---: |
| Ordinary finite list | 44,200 / 35,483 | 47,681 / 39,006 |
| Raw bulk | 42,776 / 34,087 | 45,615 / 36,975 |
| Scalar checked (also inline / mask) | 43,144 / 34,455 | 46,003 / 37,363 |
| Eight-word checked | 43,727 / 34,940 | 46,656 / 38,014 |
| Candid | 279,705 / 250,195 | 392,612 / 364,445 |

Chunking saves 473 total / 543 code bytes against ordinary at opt-z, and
1,025 / 992 at opt-3. Against scalar checked it costs 583 / 485 bytes at opt-z
and 653 / 651 at opt-3. Inline hints and integer masks produce **no instruction
or size improvement**. The final Wasm backend already lowers scalar is_finite
into integer checks; intermediate LLVM floating operations were not evidence of
floating arithmetic in the final loop. The remaining cost includes two full
validation scans during sizing/writing, absent from raw bulk.

Small controls remain material. Chunked empty opt-z encode/decode costs
1,460.5 / 607.5 versus ordinary 1,379.5 / 556.5. Singleton f32 costs
1,620.5 / 1,030.5 versus 1,657.5 / 1,069.5. At opt-3 singleton f32 encode
rises 645.25 to 777.25; singleton f64 encode rises 638.25 to 783.25 (22.7%).
Chunking beats ordinary in 68/72 primary opt-z comparisons and 29/72 at opt-3;
it beats Candid in 68/72 at each profile. Applying the instruction screen
`candidate <= 1.05 * ordinary + 64` yields zero opt-z failures and **20 opt-3
failures** (including repeated singleton edge inputs). Scalar checked fails 13
opt-3 comparisons. This is not a passing all-profile screen or promotion approval.

For 1 MiB, both ordinary and bulk frames are 1,048,590 bytes; Candid is
1,048,588. Prefix boundaries differ: 32 f32 / 16 f64 elements require a two-byte
bulk byte count while the ordinary element count still fits one byte. See the
old/new singleton bytes and coordinated-consumer requirements above.

### Native time, memory and refusal

Repeated native medians in microseconds, shown as run 1 / run 2. These are
shared-host measurements, not IC instructions or isolated latency guarantees.

| Profile / operation | Ordinary | Scalar checked | Eight-word checked | Candid in chunked run |
| --- | ---: | ---: | ---: | ---: |
| opt-z f32 encode | 1,931.65 / 1,967.96 | 245.89 / 259.42 | 156.62 / 153.03 | 36.96 / 37.61 |
| opt-z f32 decode | 965.57 / 1,013.59 | 235.93 / 236.56 | 132.20 / 135.81 | 561.39 / 561.87 |
| opt-z f64 encode | 957.34 / 1,000.97 | 87.28 / 83.42 | 110.06 / 112.87 | 37.34 / 37.97 |
| opt-z f64 decode | 520.89 / 541.54 | 110.65 / 109.82 | 84.15 / 81.98 | 272.12 / 272.23 |
| opt-3 f32 encode | 325.49 / 333.40 | 150.50 / 148.80 | 61.80 / 62.07 | 39.53 / 38.38 |
| opt-3 f32 decode | 213.01 / 216.25 | 102.97 / 101.04 | 39.13 / 41.74 | 312.35 / 325.57 |
| opt-3 f64 encode | 170.90 / 171.17 | 82.29 / 91.33 | 65.10 / 66.53 | 37.13 / 36.90 |
| opt-3 f64 decode | 139.61 / 132.73 | 75.55 / 80.51 | 41.60 / 43.12 | 151.49 / 166.56 |

Native opt-z f64 encode regresses versus scalar checked; do not hide that loss
behind the IC improvement. Chunked opt-z empty encode/decode medians are
41.99/41.10 ns and 22.03/22.51 ns; singleton f32 is 44.88/46.12 ns and
31.77/32.86 ns. All Wire large encodes allocate one 1,048,590-byte output.
Bulk decode allocates one 1,048,576-byte output with no reallocations, versus
ordinary f32/f64's 16/15 reallocations. Peak live bytes remain 1,048,576;
ordinary total allocation traffic is 2,097,136/2,097,120 versus bulk 1,048,576.
These are native allocator measurements, not a measured Wasm heap profile.

Contract probes reject infinities and quiet/signalling NaNs before allocation,
including first/middle/last positions and counts 1/7/8/9/15/16/17 for chunk edges.
They preserve finite bits at frame offsets 0–15, reject partial words and all
small-frame truncations, and check exact/one-short frame, allocation, node and
depth allowances. Non-finite decode returns typed Malformed; allowance refusal
returns Limit, with zero observed allocations. Invalid values may inspect up to
eight words before rejecting in the chunked scan. Browser-generated conformance,
actual big-endian execution and non-finite IC rejection costs remain unqualified.

### Decision and reproduction

Keep one production Serde/Postcard codec. Retain the chunked finite scan as a
bounded research candidate, not public Packed<float> support. A plain Vec field
adapter necessarily validates again during both encoder visits; avoiding this
would require a separately justified invariant-bearing collection or an encoder
change. Neither is authorized by these measurements. Do not remove finite checks
or reintroduce blanket one-pass encoding merely to match Candid.

Before promotion, qualify explicit field adapters end to end: endian encoding,
metadata/derive, generated typed arrays, preserved field bounds, Rust/TS bytes,
and consumer identity/budget action. Resolve or explicitly accept the opt-3 small
controls. Production APIs, bytes, identities, budgets and acceptance are unchanged.

Raw source, scripts and results: `target/evidence/finite-float-cost-20261109/`.
`prepare.py` creates the first three frozen fixture variants; `run.sh` executes
them serially. `run-inline.sh`, `run-bits.sh` and `run-chunks.sh` run the subsequent
frozen variants through the same maintained evaluate-codecs.sh entrypoint. Each
sets RUNS=2, SAMPLES=11, BATCH_MS=2, CORPUS_FEATURE=research-bulk, its own
CARGO_TARGET_DIR and adjacent CODEC_SOURCE_SNAPSHOT. `analyze.py` independently
checks identities, frames, samples, freshness and Wasm sections. Reruns require
fresh output directories/targets; do not overwrite retained measurements. The
scratch scripts are research evidence, not new maintained repository tooling.

## Public finite adapters — 9 November 2026

The next local implementation qualifies explicit `serde_packed::finite_f32` /
`finite_f64` fields end to end; it is not published. This supersedes the earlier
"proposed API" status, not the measurements or caveats above. Ordinary lists and
Packed integer/hash types remain unchanged. One shared macro emits two concrete
finite scanners, independent of consumer type or field bound. Derive checks
actual primitive Vec types (including aliases), and generated schemas use
`finite-f32` / `finite-f64` typed arrays. The existing Postcard byte codec remains
the only production codec. Float lint advice names the explicit primitive-vector
adapters and preserves consumer bound/admission review.

The public implementation was measured in `public` and then `public-qualified`
under the same study directory, using separate fresh targets. The final run adds
hashes of all three adapter source files to the maintained evidence producer;
its four Wasm modules are byte-identical to the preceding public run. Final
adapter source bytes match the checkout. Eight lanes now verify 32 fresh guests,
5,760 IC rows and 18,432 native/frame checks. Candid controls remain equal.
The final run retains the same compiler, lock, inputs and parameters above.

At opt-z, public instruction counts and total/code Wasm match the chunked
prototype exactly. At opt-3, the public fixture is **47,036 / 38,394 bytes**:
645 / 612 below ordinary, but 380 / 380 above the prototype. Public 1 MiB
f32 encode/decode costs **7,144,510 / 5,932,077** instructions; f64 costs
**4,097,096 / 2,966,575**. Do not reuse the prototype's better opt-3 decode
numbers. Public wins 68/72 primary comparisons against ordinary at opt-z and
47/72 at opt-3; it retains **20 opt-3 instruction-screen failures**. This is an
explicit large-field option, not a default replacement or a passing all-profile
cost screen. Small ordinary lists remain a valid choice. Large f32/f64 encode
still costs about 3.40/1.95 times raw Candid at opt-z, with additional finite
validation that raw Candid does not require.

Final native repeated medians (microseconds; shared host):

| Profile / operation | Run 1 | Run 2 |
| --- | ---: | ---: |
| opt-z f32 encode | 144.71 | 144.69 |
| opt-z f32 decode | 181.32 | 183.11 |
| opt-z f64 encode | 105.20 | 103.71 |
| opt-z f64 decode | 80.43 | 77.16 |
| opt-3 f32 encode | 58.90 | 59.11 |
| opt-3 f32 decode | 41.69 | 40.27 |
| opt-3 f64 encode | 66.99 | 62.07 |
| opt-3 f64 decode | 45.21 | 43.88 |

Native opt-z f32 decode is slower than the private prototype despite equal IC
counts. Native allocation/heap observations retain one output allocation with no
reallocations for large bulk encode/decode; this is not a Wasm heap measurement.
The public isolated contract test passes with zero observed allocation for
non-finite and one-short-limit refusal. Rust preserves bits across chunk/prefix
boundaries and offsets 0–15; generated clients validate before copying, reject
partial/non-finite words and preserve input custody. Rust/TS vectors, differential
mutations and 78 simulated big-endian browser checks pass. Actual native
big-endian execution remains unqualified; Rust's branch uses checked-length,
fallible LE staging, following the existing Packed strategy.
An isolated copy selecting that Rust staging branch on the native host passes
the same finite-bit, malformed, chunk/prefix, offset and exact-limit test. Only
its two endian cfg selections change. This compiles and exercises the fallback,
but does not replace execution on a real big-endian target.

Focused qualification passes 78 native tests (three timing probes ignored), nine
compiler refusals with a passing control, standalone codec/schema host/Wasm
consumers, and Core/derive/example Clippy. Adding two vectors required raising
only the standalone fixture's allocation allowance from 4,096 to 8,192; runtime
admission rules did not change. Coverage and research correctness each pass 16
tests with ten additional maintained finite-only cases, taking those corpora to
134. Existing 94 historical names stay ordered; subsequent container indices
move under the new source identity. No full matrix of the enlarged corpus or
application build is claimed. Raw all-IEEE controls remain distinctly labelled.

Adopting a field still changes f32 singleton prefix `01` to `04` and f64 `01`
to `08`; NaN/infinity acceptance narrows if starting from raw floats. Consumers
must coordinate identities/clients, preserve count/range constraints, retain
retry/hash bytes and requalify buffer budgets. No dependent crate needs changing
merely to retain an ordinary field. This is additive to Wire's public API; the
pending Core patch records it, with no Endpoint change, commit or publication.

## Paired f32 validation — 10 November 2026

The retained scanner checks eight f32 values per chunk using four u64 loads.
Each masked exponent independently carries into its sign bit only for exponent
255. The f64 scanner and scalar remainder remain unchanged. This preserves
finite-only acceptance, exact bytes, identities and budgets, including rejection
before allocation. No consumer migration or per-consumer codec is introduced.

A simple two-value loop regressed large encode costs and was rejected. A
sixteen-value unrolled prototype improved large inputs but regressed 8/9-value
controls. Its relocated Candid artifacts also differed; it was not accepted as
a matched cost-screen result. The final eight-value study executes both lanes
from the same checkout path with separate fresh Cargo targets and captured
sources. Candid modules and all Candid instruction controls are identical.

Raw evidence: `target/evidence/finite-pair-20261110/{live-baseline,live-chunk4}`.
Both use Rust 1.99.1, the same qualification lock, Candid 0.10.40's direct oracle,
19 inputs, two native runs of 11 batches (2 ms minimum), and three independent
identical IC samples. `verify.mjs` verifies baked source identities, eight fresh
guests, 772 IC rows and 2,480 native/frame comparisons across the pair.
The final source adds only the exponent explanation and formatting to the
measured scanner; the full follow-up corpus is restored after the study.

| Measurement | Baseline | Retained candidate |
| --- | ---: | ---: |
| opt-z 1 MiB f32 encode instructions | 7,211,037 | 4,851,741 |
| opt-z 1 MiB f32 decode instructions | 10,159,614 | 8,979,966 |
| opt-3 1 MiB f32 encode instructions | 7,144,510 | 4,785,214 |
| opt-3 1 MiB f32 decode instructions | 5,932,077 | 4,752,429 |
| opt-z 8-value f32 encode / decode | 1,907.5 / 1,291.5 | 1,835.5 / 1,255.5 |
| opt-z Wasm total / code bytes | 55,737 / 45,413 | 55,742 / 45,418 |
| opt-3 Wasm total / code bytes | 62,303 / 52,731 | 62,306 / 52,734 |

The 158-operation instruction screen passes its margins; the overall strict
screen **fails** only for 5/3 bytes of opt-z/opt-3 total and code growth. This is
an explicit small size tradeoff for instruction savings, not a passing no-growth
screen. f64 instructions are unchanged. Finite validation still makes large f32
encode about 2.29 times raw Candid; this does not establish universal parity.
These are linked fixture sizes, not full application savings.
All 620 paired native allocation/heap observations are unchanged; these do not
measure Wasm heap use.

Repeated native medians below are microseconds on a shared host, not IC counters:

| Profile / 1 MiB f32 operation | Baseline runs 1 / 2 | Candidate runs 1 / 2 |
| --- | ---: | ---: |
| opt-z encode | 224.09 / 162.98 | 78.97 / 75.02 |
| opt-z decode | 213.03 / 180.91 | 149.20 / 142.63 |
| opt-3 encode | 55.18 / 57.55 | 49.15 / 48.34 |
| opt-3 decode | 37.63 / 38.12 | 34.91 / 34.58 |

Unchanged f64 native decode moved from 77.28/75.94 to 118.78/96.07 microseconds
at opt-z despite identical IC instructions; host noise remains material.
All-feature Core tests and doctests, 79 native tests and the full 134-case
coverage corpus pass. A new scalar-reference test covers all 65,536 exponent
pairs, distributed across chunk positions; existing tests cover bit preservation,
non-finite inputs, truncation, offsets and exact limits. Native big-endian
execution remains unqualified. The concurrently refreshed full matrix measures
the frozen pre-optimization source; it is separate from this matched study.

### Rejected f64 exponent-carry scan

The follow-up `live-f64` lane tried eight masked u64 exponent additions in the
f64 scanner, against `live-chunk4` with the same 19 cases, compiler, lock,
checkout path, native parameters and fresh target. Both Candid modules and all
Candid counters match. Baked sources, 386 new IC rows and 1,240 native/frame
comparisons verify. Exhaustive f64 exponent/sign/mantissa controls passed, but
correctness alone does not justify keeping the rewrite.

| 1 MiB f64 / fixture metric | Retained baseline | Rejected probe |
| --- | ---: | ---: |
| opt-z encode / decode instructions | 4,130,838 / 5,080,572 | 4,360,214 / 5,195,260 |
| opt-3 encode / decode instructions | 4,097,096 / 2,966,575 | 4,326,474 / 3,081,264 |
| opt-z total / code Wasm bytes | 55,742 / 45,418 | 55,839 / 45,515 |
| opt-3 total / code Wasm bytes | 62,306 / 52,734 | 62,410 / 52,838 |

The strict screen rejects four large-encode instruction regressions and both
profiles' total/code growth. Restore the scalar f64 scan; no production f64,
protocol or acceptance change remains. The temporary 19-case corpus and
probe-only test are removed from the working tree; captured sources, test and
raw native/IC results remain under the existing paired-study evidence directory.
