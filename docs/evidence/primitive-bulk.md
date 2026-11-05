# Follow-up: advice, missing Candid controls and private bulk research

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

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
