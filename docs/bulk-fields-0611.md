# Bulk integers and hash manifests: 0.6.11 research

The shared bulk-field prototype beats Candid on all 68 field/manifest encode and
decode cases in both profiles, with the normal production Serde/Postcard runtime,
admission and canonical verification enabled. The runtime-limited design also
reduces total and code-section Wasm versus ordinary fields at 1/10/100 types.
All 1/10/100-record encode/decode cases also beat Candid in both profiles.
These measurements cover the unpublished prototype before promotion. The current
[public API and migration guide](packed-fields.md) describe `ic_wire::Packed<T>`;
this archive retains the research source and does not measure the promoted artifact.

At opt-3, a manifest with 1,024 hashes falls from 2,516,422 to 35,175 encode
instructions and from 5,037,486 to 82,936 decode instructions. Candid's corresponding
costs are 156,158 and 4,233,663. The Wire frame grows by one byte, 32,830 to 32,831;
Candid uses 33,883. The hash-vector Rust value is retained as Vec<[u8; 32]> inside
the prototype; serialization borrows its contiguous bytes rather than building
a flattened temporary buffer on little-endian targets. Decoding remains owned.

## Design selection and scope

Three implementations were investigated:

1. `Packed<T, N>` shared one serializer/visitor design across integer widths and
   hashes, with the exact per-field bound checked before allocation. It wins
   instructions, but distinct bounds introduce more concrete Rust field types.
   With 100 records, total Wasm grows 48,122 bytes at opt-z and 35,346 at opt-3
   relative to ordinary fields. This version is archived, not selected.
2. Moving serialization/deserialization into explicit non-inlined entry helpers
   preserves those bounds but makes the 100-record artifact another 200/862 bytes
   larger at opt-z/opt-3. This Wasm-only screen is rejected; no speed claim is made.
3. `Packed<T>` keeps only the word type as a type parameter. Like ordinary Vec<T>,
   it relies on native runtime Limits, without a field-specific N. It retains
   word-width checks before allocation and normal canonical verification. Total
   Wasm falls 27,936 bytes at opt-z and 19,234 at opt-3 for 100 records. This is the
   selected **research implementation**, not a public replacement for bounded fields.

The prototype supports i16/u16/i32/u32/i64/u64 and [u8; 32]. It has one generic
Serde implementation and specializes the read loop by element type, without
unsafe code in this crate. Existing bytemuck supplies the safe byte-slice view.
No serializer-generating derive, second production codec, dispatch framework or
consumer-specific transport is added. Bytemuck is an existing dependency; this
batch adds its benchmark dev-dependency role without changing its version.

This result supports explicit bulk fields where numeric vectors or hash
collections are hot. It does not make ordinary Vec<T> automatically bulk, and
it is not a reason to pack every small value. The ordinary field corpus and
existing PackedI16 production comparisons remain available as controls.

Small controls still matter: at opt-3, empty u32 packed decode rises from 411.5
to 572.5 instructions (+39.13%) versus ordinary Wire, and a single small u32
encode rises 759.25 to 767.25 (+1.05%). Both still beat Candid. At opt-z the worst
empty decode increase is 7.85%. The one-record opt-3 runtime-limited artifact is
135 bytes larger than the first const-bound prototype, though 13,693 bytes smaller
than ordinary fields. These are not universal savings across every comparison.

## Contract and consumer implications

Production API, encoding and acceptance are unchanged by this research batch.
The experimental field representation changes element-count/per-element encoding
to a byte count followed by fixed-width little-endian words:

- One i16 value 32767: `01 fe ff 03` becomes `02 ff 7f`.
- One 32-byte hash: prefix `01` becomes `20`, followed by the same 32 hash bytes.
- Small integers can grow: 16,384 small i64 values use 16,398 ordinary frame bytes
  versus 131,086 packed bytes. Packing is a speed/size choice.

Serde admission changes from per-element visits to byte-buffer charges. Native
budget checks still occur before the byte visitor allocates; consumers must
requalify resource allowances when changing a field representation. The selected
prototype deliberately has no per-field maximum N. It is **not** a drop-in
replacement for PackedI16<N> or a promise to preserve their tighter field limits.
A consumer requiring such a maximum needs an explicit field constraint.

Adoption must coordinate schemas and clients, resolve retained
old-layout retry frames, and keep one current layout without fallback readers.
This research stage did not implement TypeScript metadata or installed consumer
migration. The subsequent [promotion](packed-fields.md) supplies the public API
and metadata while preserving PackedI16<N> bounds. Consumer adoption is separate.

For hashes, Candid retains its original nested array-of-hashes schema. We do not
compare this candidate against a newly flattened Candid blob schema in this
matched fixture. Candid's primitive integer-vector fast path is already active.
The larger [full matrix](full-comparison-0611.md) remains a separate fixture;
optimizer/context differences mean its absolute numbers must not be spliced into
this before/after table.

## Method, coverage and reproduction

Released base `31bcef2` plus the pending shared PackedI16 decoder, package version
0.6.10. Rust 1.99.1 / LLVM 23.1.1; Linux x86_64 Threadripper 7970X native and
wasm32-unknown-unknown. Candid 0.10.39, Postcard 1.1.3, Serde 1.0.229, IC Platform
0.6.10 and Host/Testkit 0.66.0 use the archived lock. Evaluation is opt-z;
benchmark-speed is opt-3; both use LTO, one codegen unit, panic abort and stripped
symbols. No gzip optimization metric is used.

Sixty numeric inputs cover six widths, small and maximum values, and lengths
0/1/128/1024/16384. Eight hash/manifest cases cover 0/1/32/1024 hashes. Scaling uses
1/10/100 nominally distinct records, each with six numeric collections and one
hash collection plus scalar/label fields. Each collection contains eight elements.
The first prototype uses sixteen field-bound values; the selected prototype and
ordinary Vec baseline do not encode those bounds in the field type. These are
synthetic fixtures, not installed-application size forecasts or traffic weights.

Two serial native runs/profile use eleven calibrated samples/run, at least 2 ms
per batch with a 1,048,576-operation cap. These unpinned warm-cache observations
include result destruction/loop overhead and exclude setup and semantic checks.
Ranges are run medians, not confidence intervals or request-tail latency.
Separate instrumented runs measure allocations/reallocations and native peak live
heap, not Wasm heap. PocketIC 16.1.0 uses counter 0, three samples, four repetitions
through 64 KiB and one above; setup/transport is excluded and no loop overhead is
subtracted. Whole Wasm includes fixture construction, dispatch and harness code.
All before/after savings are measured artifacts, not symbol attribution.

The archive retains the initial frozen `source`/`runner`, selected
`runtime_source`/`runtime_runner`, and the rejected helper screen's exact module,
runner and artifact hashes. Extract sources into separate directories and adjust
root/output/target and POCKET_IC_BIN paths before replay. Use separate build roots
for the two source versions and fresh output directories. Analysis scripts reproduce
CSV tables after adjusting their root/output paths. Full results use RUNS=2,
SAMPLES=11, BATCH_MS=2. The selected live corpus can be run with:

```sh
CORPUS_FEATURE=bulk bash scripts/ci/evaluate-codecs.sh NEW_OUTPUT
CORPUS_FEATURE=bulk,bulk-baseline bash scripts/ci/evaluate-codecs.sh NEW_BASELINE
CORPUS_FEATURE=bulk-scale bash scripts/ci/evaluate-codecs.sh NEW_SCALE_OUTPUT
```

Add `derive-scale-1` or `derive-scale-10` for the smaller rosters, and
`bulk-baseline` for ordinary scale fields. The existing consumer-benchmark gate
now covers these cohorts. Both representations still call the same production
native codec; the historical lane label `wire-bounded` does not imply a per-field
N in the selected prototype.

## Wasm size gate

Total / code-section bytes, uncompressed. Negative deltas favor bulk.
These are matched whole fixtures, not per-type or symbol-attributed savings.

| Cohort / profile | Ordinary Wire | `Packed<T, N>` | `Packed<T>` | `Packed<T>` total delta | Candid |
| --- | ---: | ---: | ---: | ---: | ---: |
| fields / evaluation | 80,068 / 69,487 | 70,365 / 59,888 | 69,599 / 59,154 | -10,469 (-13.08%) | 376,381 / 342,985 |
| fields / benchmark-speed | 97,760 / 87,815 | 81,567 / 71,688 | 80,985 / 71,143 | -16,775 (-17.16%) | 577,487 / 545,426 |
| scale-1 / evaluation | 81,959 / 69,820 | 73,977 / 61,901 | 72,995 / 61,023 | -8,964 (-10.94%) | 365,689 / 331,589 |
| scale-1 / benchmark-speed | 118,672 / 90,975 | 104,844 / 77,156 | 104,979 / 77,291 | -13,693 (-11.54%) | 531,930 / 482,712 |
| scale-10 / evaluation | 129,259 / 116,360 | 151,233 / 138,364 | 113,514 / 100,791 | -15,745 (-12.18%) | 502,633 / 466,594 |
| scale-10 / benchmark-speed | 174,280 / 145,896 | 188,231 / 159,771 | 155,051 / 126,687 | -19,229 (-11.03%) | 776,659 / 725,476 |
| scale-100 / evaluation | 488,858 / 469,139 | 536,980 / 517,140 | 460,922 / 441,443 | -27,936 (-5.71%) | 1,793,974 / 1,739,556 |
| scale-100 / benchmark-speed | 675,032 / 639,867 | 710,378 / 675,041 | 655,798 / 620,658 | -19,234 (-2.85%) | 2,639,766 / 2,570,257 |

## Instruction and payload examples

Opt-3 instructions, rounded. Payload column includes the complete Wire or Candid
frame. The packed candidate still uses normal Wire admission/canonical verification.

| Case / operation | Ordinary Wire | Bulk Wire | Change | Candid | Frame bytes: ordinary / bulk / Candid |
| --- | ---: | ---: | ---: | ---: | ---: |
| manifest-1024 / encode | 2,516,422 | 35,175 | -98.60% | 156,158 | 32,830 / 32,831 / 33,883 |
| manifest-1024 / decode | 5,037,486 | 82,936 | -98.35% | 4,233,662 | 32,830 / 32,831 / 33,883 |
| hashes-1024 / encode | 2,504,594 | 33,685 | -98.66% | 129,242 | 32,781 / 32,782 / 33,805 |
| hashes-1024 / decode | 5,034,818 | 80,508 | -98.40% | 4,208,984 | 32,781 / 32,782 / 33,805 |
| i16-wide-16384 / encode | 4,014,882 | 33,689 | -99.16% | 80,492 | 49,166 / 32,782 / 32,780 |
| i16-wide-16384 / decode | 5,902,468 | 80,362 | -98.64% | 1,408,282 | 49,166 / 32,782 / 32,780 |
| u32-wide-16384 / encode | 4,375,565 | 66,637 | -98.48% | 146,212 | 81,934 / 65,550 / 65,548 |
| u32-wide-16384 / decode | 7,664,582 | 159,378 | -97.92% | 1,375,675 | 81,934 / 65,550 / 65,548 |
| i64-wide-16384 / encode | 6,931,453 | 132,173 | -98.09% | 277,284 | 163,854 / 131,086 / 131,084 |
| i64-wide-16384 / decode | 12,194,602 | 317,074 | -97.40% | 1,277,361 | 163,854 / 131,086 / 131,084 |
| i16-small-16384 / encode | 3,457,826 | 33,689 | -99.03% | 80,523 | 16,398 / 32,782 / 32,780 |
| i16-small-16384 / decode | 2,945,440 | 80,362 | -97.27% | 1,408,474 | 16,398 / 32,782 / 32,780 |
| i64-small-16384 / encode | 2,081,909 | 132,173 | -93.65% | 277,315 | 16,398 / 131,086 / 131,084 |
| i64-small-16384 / decode | 3,470,810 | 317,074 | -90.86% | 1,277,552 | 16,398 / 131,086 / 131,084 |
| u32-small-1 / encode | 759 | 767 | +1.05% | 12,897 | 13 / 16 / 14 |
| u32-small-1 / decode | 856 | 1,098 | +28.40% | 15,098 | 13 / 16 / 14 |
| u32-wide-0 / encode | 671 | 654 | -2.53% | 13,577 | 12 / 12 / 10 |
| u32-wide-0 / decode | 412 | 572 | +39.13% | 14,988 | 12 / 12 / 10 |

## Native and allocation examples

Two opt-3 run medians as a range in µs. Allocations/reallocations and peak live
bytes come from separate native instrumentation, not Wasm heap measurements.

| Case / operation / representation | Native µs | Allocations / reallocations | Peak bytes |
| --- | ---: | ---: | ---: |
| manifest-1024 / encode / baseline | 47.947–50.064 | 1 / 0 | 32,830 |
| manifest-1024 / encode / runtime | 0.564–0.627 | 1 / 0 | 32,831 |
| manifest-1024 / decode / baseline | 67.019–69.509 | 2 / 8 | 32,792 |
| manifest-1024 / decode / runtime | 1.140–1.164 | 2 / 0 | 32,792 |
| i16-wide-16384 / encode / baseline | 89.020–89.383 | 1 / 0 | 49,166 |
| i16-wide-16384 / encode / runtime | 0.534–0.539 | 1 / 0 | 32,782 |
| i16-wide-16384 / decode / baseline | 147.698–148.080 | 1 / 12 | 32,768 |
| i16-wide-16384 / decode / runtime | 0.971–0.976 | 1 / 0 | 32,768 |
| i64-wide-16384 / encode / baseline | 140.012–142.440 | 1 / 0 | 163,854 |
| i64-wide-16384 / encode / runtime | 1.988–2.056 | 1 / 0 | 131,086 |
| i64-wide-16384 / decode / baseline | 228.301–231.921 | 1 / 12 | 131,072 |
| i64-wide-16384 / decode / runtime | 3.773–3.799 | 1 / 0 | 131,072 |

## Qualification and evidence

Twelve cohorts completed both profiles: 2,148 primary comparisons, 10,740 IC rows,
34,368 native/instrumented rows and 48 Wasm artifacts. These are overlapping
fixtures, not 537 unique application types. Every IC row has three identical
samples. Input/frame identities match across measurement modes; logical inputs
match across representations; all Candid frame and instruction controls match.
All artifact sizes/hashes were independently verified. Wire frame hashes change
where the field layout changes, as expected.

Focused tests pass for both selected field/scale representations, including
corpus selection and damaged frames. Dedicated tests cover signed extrema,
unaligned decode, all 65,536 i16 values against existing PackedI16 bytes, invalid
word lengths, noncanonical lengths and native byte/allocation-limit refusal.
Strict Clippy passes all targets for the four selected feature combinations;
Rust 1.91 native and Wasm checks pass. The standard corpus roundtrip check passes.
No full CI, big-endian execution, TypeScript implementation or installed-consumer
qualification is claimed.

At the research handoff, source differed from the selected frozen snapshot only in
formatting/comments, a test name and removal of an unused bulk-only bound constant.
The archived snapshots identify exactly what produced each measurement; final
focused tests/lints checked that research code.

- [Primary comparison CSV](evidence/bulk-fields-0611/comparison.csv): all cases,
  wire sizes, IC instructions, both native run medians, allocations and peak heap.
- [Before/after CSV](evidence/bulk-fields-0611/before-after.csv): absolute results,
  percentage changes and regressions for both prototypes against ordinary fields.
- [All operation lanes](evidence/bulk-fields-0611/all-lanes.csv),
  [Wasm artifacts](evidence/bulk-fields-0611/artifacts.csv), and
  [instruction wins](evidence/bulk-fields-0611/instruction-wins.csv).
- [Identity](evidence/bulk-fields-0611/identity.json) and
  [raw archive](evidence/bulk-fields-0611/raw.json.gz): exact source versions,
  lock, runner/scripts, samples, validation logs and rejected helper screen.
  Archive compression is storage only, not a Wasm-size optimization metric.

This research batch left production runtime/API and protocol unchanged. The
subsequent promotion is documented separately; staging, versions, commits, pushes and releases remain maintainer-owned.
