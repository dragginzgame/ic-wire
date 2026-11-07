# Follow-up: advice, missing Candid controls and private bulk research

> Earlier findings below are historical: their raw artifacts were retired.
> The checked bool-copy follow-up retains its current raw artifacts under
> `target/evidence/bool-copy-20261107/`, outside Git. A summary/hash cannot
> verify missing payloads; use the maintained commands for fresh measurements.

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

This supplements the frozen [Platform 0.21 full matrix](../candid-comparison.md).
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
