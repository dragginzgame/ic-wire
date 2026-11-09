# Latest recorded Candid comparison

This recorded matrix predates the direct Candid oracle migration. New runs use
upstream Candid 0.10.40 with explicit quotas and exact single-result framing,
without Platform's former depth-16 pre-scan. That changes Candid admission cost
and nesting acceptance; these numbers are historical, not a qualified performance
baseline for the new comparator. Rerun matched sources before claiming a gain.

Measured **9 November 2026**, Core **0.18.0** with pending tooling/supplier
adoptions. The complete **39-cohort** run contains **2,194 overlapping case
entries** and **8,776 primary encode/decode comparisons**. These are not unique
types or traffic weights. Wire wins **7,587/8,280** comparisons outside the mixed
research cohort, and **470/496** inside it; Candid wins 693 and 26, with no ties.
There remains one production Serde/Postcard codec. Candid is test-only.

The `research-bulk` name predates the public bool adapter. That cohort now mixes
**public bool-adapter cases**, **private float bulk prototypes**, and unchanged
controls. Its aggregate must not be presented as either wholly private research
or wholly production-ready. This run changes no production code, bytes, schemas,
identities, budgets or acceptance rules.

## Uncompressed Wasm

Wire is smaller in total and code-section bytes in **all 78 cohort/profile pairs**.
Below: total / code-section bytes, without a post-link optimizer. Opt-z is the
primary decision profile; opt-3 remains a secondary check. These are linked
fixture modules, not full Toko builds or projected application savings.

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 391 cases | opt-z | 594,277 / 547,562 | 1,452,370 / 1,376,612 |
| Standard, 391 cases | opt-3 | 893,678 / 833,247 | 2,054,997 / 1,964,584 |
| Toko-shaped, 10 cases | opt-z | 80,498 / 69,354 | 387,078 / 351,883 |
| Toko-shaped, 10 cases | opt-3 | 103,306 / 92,906 | 596,200 / 562,506 |
| 100 ordinary derived types | opt-z | 310,569 / 291,848 | 1,460,475 / 1,405,395 |
| 100 ordinary derived types | opt-3 | 395,846 / 361,556 | 2,130,654 / 2,060,333 |
| Packed ID16, 20 cases | opt-z | 50,226 / 40,880 | 279,862 / 249,821 |
| Packed ID16, 20 cases | opt-3 | 55,738 / 46,597 | 450,374 / 421,393 |
| Signed32, 49 cases | opt-z | 75,386 / 63,304 | 348,519 / 313,294 |
| Signed32, 49 cases | opt-3 | 92,579 / 81,454 | 529,997 / 496,386 |

The 100-type controls retain reachable Serde/CandidType codec paths. Wire's
metadata derive is host-only. Source reductions and stress-fixture differences
are not application Wasm savings.

## Instructions and frame bytes

Selected **opt-z IC instructions/op**; encode allocates fresh output. Each row
uses its own labelled fixture context. Frame sizes include codec framing.

| Cohort / payload | Wire encode / decode | Candid encode / decode | Wire / Candid bytes |
| --- | ---: | ---: | ---: |
| Standard: owned 1 MiB byte buffer | 1,050,683 / 1,049,829 | 2,121,766 / 1,080,091 | 1,048,590 / 1,048,588 |
| Ordinary i16: 16,384 wide values | 3,999,090.5 / 1,988,434.5 | 86,112.5 / 1,225,077.5 | 49,166 / 32,780 |
| Packed-i16: same values | 34,327.5 / 34,001.5 | 86,112.5 / 1,225,077.5 | 32,782 / 32,780 |
| Standard: 128 explicit-zero Accounts | 278,676.5 / 224,995.5 | 289,520 / 2,484,773.5 | 8,205 / 8,349 |
| Byte-key map: 128 keys | 172,131.5 / 266,594.5 | 221,552.5 / 1,703,083 | 6,477 / 6,485 |
| Public adapter: 4,096 nonzero i64 | 34,539.5 / 181,477.5 | 87,286 / 549,883.5 | 32,782 / 32,779 |
| Public adapter: heap of 4,096 u64 | 34,985.5 / 274,053.5 | 87,287 / 549,890.5 | 32,782 / 32,779 |
| Packed ID16: 4,096 owned IDs | 67,598 / 67,100 | 155,601 / 96,694 | 65,550 / 65,548 |
| Packed ID16: borrowed input, copying included | 133,862 / 67,098 | 155,601 / 96,694 | 65,550 / 65,548 |
| Signed32: 4,096 ordinary wide values | 2,348,650.5 / 2,520,051.5 | 53,494.5 / 557,894.5 | 20,493 / 16,395 |
| Signed32: explicit Packed control | 18,058.5 / 95,595.5 | 53,494.5 / 557,957.5 | 16,398 / 16,395 |

Generic Packed wins **436/436**, bounded Packed-i16 **80/80**, and Packed ID16
**80/80** primary comparisons. Toko-shaped inputs win **38/40**; the two losses
are opt-z ID-filter encodes at 500/1,000 IDs (1.09× / 1.27× Candid). The ID16
controls are separate shapes, not an installed Toko adoption. Their Candid DTOs
receive already-flat blobs and validate ID/cursor widths; DTO setup is untimed.
The labelled Wire borrowed-input encode includes copying into Packed.

Profile losses remain visible. The Account row encodes in **122,726.25 vs
104,445.75** instructions at opt-3, despite winning opt-z. Ordinary wide i32
encodes in **1,323,901.25 vs 47,618.25** at opt-3; Packed takes **17,215.25**.
Use each control's measured Candid decode: the signed32 ordinary/Packed controls
share inputs and Candid bytes but differ by 63/62 decode instructions at opt-z/3.
Accounts use plain-array external Candid DTOs here; other controls exercise
byte-adapted DTOs. Wire's production types do not require CandidType.

## Remaining hotspots and representation tradeoffs

Outside the mixed research cohort, **114 / 82 encode** and **10 / 6 decode**
comparisons are at least 10× slower than Candid at opt-z/3. These include
repeated ordinary controls. The largest ratio is ordinary ID16-array decode:
**11,609,194 vs 96,694** instructions (120.06×) at 4,096 IDs; the public Packed
control takes **67,100**. This is a flat-blob Candid control, not an unexplained
regression in the same representation.

Large ordinary byte vectors and wide integer/hash arrays dominate absolute
avoidable costs. At opt-z a generic 1 MiB byte-vector encode takes **158,336,943
vs 2,117,991** instructions; the explicit byte-buffer row above takes **1,050,683**.
Supported host metadata emits [bulk advice](packed-fields.md), including bool
adapter advice and bounded-cost warnings for unsupported float/i128 packing.
Codec-only/custom Serde layouts are not universally linted. Never silently
rewrite consumer Serde or infer set semantics from vector results.

Packing trades representation costs. In the i16 fixture it adds **2,619 total /
2,481 code bytes** at opt-z (43,065/34,298 vs 40,446/31,817), while the 100-type
generic bulk fixture becomes **8,343 total / 8,316 code bytes smaller**. Small
varints can produce much smaller payloads than fixed-width Packed fields.
Byte-key maps at 128 keys reduce opt-z encode **356,635.5 → 172,131.5** and decode
**892,485.5 → 266,594.5**, adding 128 frame bytes. Consumer adoption must review
layout, bounds and retained retry/hash bytes; this measurement changes none.

Not all residual costs have a bulk replacement:

| Opt-z case | Wire / Candid instructions | Interpretation |
| --- | ---: | --- |
| f32 list, 1 MiB, encode | 55,838,553 / 2,118,212 | No public Packed float contract |
| f64 list, 1 MiB, encode | 28,444,505 / 2,118,315 | Same policy/qualification gap |
| 4,096 Result-of-unit items, encode | 2,942,555.5 / 1,288,478.5 | Repeated variant/admission traversal |
| 4,096 mixed i32 deque values, decode | 1,822,579.5 / 595,531.5 | Container traversal; not raw Vec bulk |
| 4,096 unique i32 set values, decode | 6,617,962.5 / 5,571,708.5 | Includes ordered-set reconstruction |
| Toko-shaped 1,000-ID filter, encode | 861,544.5 / 678,003.5 | Separate field-adoption opportunity |

Zero-byte elements still cost traversal: 4,096 units encode in **181,839.5 vs
19,192.75** instructions at opt-z, but **783.25 vs 13,145.5** at opt-3. Do not
change resource admission just to erase that profile-dependent work.

## Expanded container coverage

Nested options/results, hash/nonzero lists and heaps retain empty/singleton
controls. The public nonzero/heap adapters include validation/reconstruction;
Candid vectors do not perform the same domain work. `packed-containers` copies
inside Serde during sizing and writing. `adapter-dto-1/10/100` constructs DTOs
before timing; do not present these controls as end-to-end conversion timings.

## Borrowed adapter investigation

The [earlier matched study](https://github.com/dragginzgame/ic-wire/blob/bcbcec53ce9eaadf5a839efdb95595b834d2583b/docs/candid-comparison.md#borrowed-adapter-investigation)
records why the public adapters were adopted. Current 100-type DTO → adapter
Wasm total/code is **337,928/318,968 → 346,145/327,180** at opt-z and
**551,262/516,765 → 567,411/532,912** at opt-3. Already-packed data stays smaller;
these DTO controls omit domain conversion. This is not a new before/after codec
change. Follow the [consumer guidance](../crates/ic-wire/README.md#representation-and-adoption-details).

## Signed32 follow-up (8 November, Core 0.17.14)

The historical baseline and rejected writer experiments remain in the
[encoder study](evidence/encoder-followup.md#signed-i32-writer-screen-8-november-2026).
The 49-case cohort is now included in this complete 9 November matrix with the
released Core 0.18.0 encoder. It covers zigzag scalars, small/negative/wide/mixed
lists, deque/set/tuple/option/result shapes and Packed controls. Set cardinality
and physical deque split tests remain meaningful; no TypeScript support is added.
Do not compare the historical counters to this supplier graph as a matched regression.

## Public bool and private float controls

At opt-z, the public bool adapter reduces 1 MiB encode **97,519,445 → 1,050,620**
instructions; Candid takes **2,118,112**. Adapter decode takes **20,973,137** vs
Candid **403,732,189**, including validation of every bool byte.

The private f32 prototype encodes in **1,050,623** and decodes in **7,079,401**
instructions, versus Candid **2,118,212 / 19,166,352**. The paired complete
coverage → mixed-cohort fixture changes total/code Wasm **216,523/182,068 →
215,661/181,206** at opt-z and **272,914/240,946 → 271,306/239,339** at opt-3.
These whole-fixture differences combine bool and float changes; they are not
isolated float savings or application savings.

Float fields remain private research: element-count prefixes become byte counts
(`01` → `04` for one f32). Promotion needs an explicit finite/IEEE-bit policy,
malformed-input/admission proofs, endian handling, schema/browser conformance and
consumer coordination. Current bool support must not be confused with nonexistent
public Packed<float> support. See [the original studies](evidence/primitive-bulk.md).

## Next experiments

1. **Measure finite fields in consumer-shaped records.** The pending public
   adapters now have [Rust/TS qualification and matched costs](evidence/primitive-bulk.md#public-finite-adapters--9-november-2026).
   They save 473 opt-z Wasm bytes and cut large f32 encode 90.1% versus ordinary
   finite lists in the focused fixture, but remain 3.40 times Candid and fail
   20 opt-3 small-control comparisons. Preserve explicit adoption and bounds;
   these are not application savings. Ten new maintained finite cases have
   correctness coverage, not a new full-matrix measurement. The all-IEEE
   prototype remains separate research; do not drop finite validation to copy
   its better results.
2. **Measure shared Serde/admission traversal on non-packable payloads.** Target
   Result/unit lists, deques and batches of tiny buffers, retaining exact budgets,
   typed failures and opt-3 losses. Avoid per-consumer codec specializations.
3. **Prove bound-sharing value in a real consumer before redesigning it.** The
   100-type varied-bound fixture adds 16,811 total bytes at opt-z, but prior
   [outer-visitor experiments](evidence/bound-sharing.md#outer-list-visitor-screen-7-november)
   barely changed the broad corpus. Do not adopt complexity for stress-only wins.

Do not repeat blanket one-pass encoding, forced varint-reader sharing or common
buffer-reader extraction without a new hypothesis. The
[encoder](evidence/encoder-followup.md) and [runtime](evidence/runtime.md) studies
record rejected instruction/Wasm tradeoffs. Existing byte/Packed/adapters and
lint guidance remain the first action for the ordinary-control gaps.

## Native time, memory and reproduction

Standard 1 MiB byte buffer, two native run medians in **µs/op**:

| Profile / operation | Wire run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: |
| opt-z fresh encode | 17.598 / 17.303 | 760.735 / 761.406 |
| opt-z owned decode | 17.267 / 17.480 | 18.894 / 19.551 |
| opt-3 fresh encode | 18.158 / 17.405 | 741.388 / 737.800 |
| opt-3 owned decode | 18.076 / 17.687 | 18.528 / 18.692 |

The host was shared and sometimes heavily loaded; small timing differences are
not isolated latency evidence. Separate probes record Wire one allocation and
zero reallocations per operation. Candid encode uses 17 allocations plus three
reallocations; decode uses 31 allocations. Peak live bytes for encode/decode are
Wire **1,048,590 / 1,048,576**, Candid **2,098,807 / 1,049,380** in both profiles.

Rust/Cargo **1.99.1**, LLVM **23.1.1**, Linux x86_64 and wasm32-unknown-unknown;
opt-z/opt-3, fat LTO, one codegen unit, abort, stripped symbols. Candid **0.10.40**,
Serde **1.0.229**, Postcard **1.1.3**, Platform **0.27.1**, Types **0.18.3**,
Runtime **0.20.3**, Testkit **0.95.0**, Build **0.81.0**, Host **0.76.1**;
prepared PocketIC **16.1.0**. Lock SHA-256:
`a1b3e79781eb208c581a850aa2151f111df263ed8daaef93366c6c5e8e6e51e8`.

Reproduce using the [maintained matrix command](development.md#candid-comparison-benchmarks)
with a fresh output directory/target and an explicitly prepared server. Two native
runs/profile retain eleven samples and minimum 2 ms batches. IC measurements use
three distinct callers; counters include codec work, result destruction and loop
overhead, excluding transport/authentication, ingress copies and physical cycles.
Rejection costs remain separate: Wire limits and Candid quotas are not equivalent
policies. No full CI, installed Endpoint, consumer build, macOS or real big-endian
qualification is implied. Older supplier graphs are not regression baselines.

Raw evidence: `target/evidence/codec-matrix-20261109/`; frozen source base
**df071a4** plus recorded working edits. `summarize.mjs`, `pairs.py`, `summary.json`
and per-cohort logs/CSVs retain the analysis. All **351 source-identity records**,
**156 fresh guest module hashes/code sizes**, **46,178 IC rows** with three equal
samples, **148,832 native/IC input/frame checks**, and **13 representation pairs**
verify. Core dependency artifacts are shared only within this one frozen source;
all guest builds are observed fresh. Nothing was committed, pushed or published.
