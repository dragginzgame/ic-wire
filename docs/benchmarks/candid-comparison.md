# Latest recorded Candid comparison

## Core 0.20 full matrix — 10 November 2026

**39/39 cohorts completed and verified:** **2,314 overlapping case
entries**, **9,256 primary encode/decode comparisons**,
**48,610 IC rows** and **156,640 native/frame checks**.
All 156 guest builds, baked source/lock identities, exact retained frames,
matching native/IC row inventories and three identical instruction samples per
operation verify. No missing cohort remains.

This refresh uses released source `3c5176c`, Rust 1.99.1, Candid 0.10.40 and
prepared PocketIC 16.0.0. The source and both locks are frozen under
`target/evidence/codec-matrix-core020/source`; builds use the independent
`target/codec-matrix-core020-build`. Canonical Types is 0.19.0 in these
measurements. A separate edit selected Types 0.19.1 in the normal root lock
while the run was active; it was preserved and is not qualified by this matrix.
No runtime code, protocol bytes, identities, budgets or acceptance rules changed
in this benchmark task. No new public release is needed for the report.

Wire wins **4,155/4,628 opt-z** and **4,260/4,628
opt-3** primary instruction comparisons. Counts overlap across controlled
representations and type scales; they are not unique types, traffic weights or
an application-wide speedup. `research-bulk` includes private float prototypes,
public adapters and ordinary controls; it is not uniformly production-ready.
The copying-container and preconstructed DTO cohorts also remain labelled controls.

### Uncompressed Wasm

Wire is smaller in **both total module and code-section bytes in 78/78
profile/cohort pairs**. Selected opt-z linked fixture sizes:

| Fixture | Wire total / code bytes | Candid total / code bytes |
| --- | ---: | ---: |
| Standard | 592,245 / 545,637 | 1,426,944 / 1,352,295 |
| Toko-shaped | 80,011 / 68,887 | 375,576 / 341,390 |
| 100 derived types | 310,413 / 291,709 | 1,424,995 / 1,370,933 |
| Expanded application shapes | 264,145 / 228,928 | 806,250 / 743,369 |
| 100 domain-adapter types | 345,747 / 326,798 | 1,495,696 / 1,441,311 |
| 100 Packed DTO types | 337,728 / 318,784 | 1,495,696 / 1,441,311 |

[All 78 Wasm pairs](current-wasm.csv) retain both opt-z and opt-3 absolute sizes.
These are compiler-linked benchmark modules, not full Toko applications or
per-type savings. Both profiles use LTO, one codegen unit, aborting panics and
symbol stripping; no additional post-link wasm-opt pass is applied. Code-section
size is the independently checked section-10 payload of each retained raw module.
No compressed-size metric is used.

### New application shapes

Twenty appended cases cover five shapes at 0, 1, 128 and 4,096 entries. Each runs
in all four follow-up cohorts. In the ordinary production-shape cohort, Wire
wins **34/40 opt-z** and **32/40 opt-3** encode/decode comparisons; every new
case's decode beats Candid in both profiles. Opt-z absolute results at 4,096
entries (each cell is **Wire / Candid**):

| Shape | Frame bytes | Encode instructions | Decode instructions |
| --- | ---: | ---: | ---: |
| Paginated Packed records | 150,339 / 175,067 | 10,429,732 / 7,394,525 | 11,859,545 / 31,445,948 |
| Mixed enum changes | 65,534 / 71,027 | 4,772,387.5 / 2,817,442 | 4,199,621.5 / 27,947,615 |
| Sparse optional blobs | 50,519 / 50,521 | 3,041,617.5 / 2,538,982.5 | 2,551,626.5 / 13,957,945 |
| Result maps with blobs | 104,333 / 112,676 | 7,814,676 / 5,320,770 | 13,393,927 / 44,501,864 |
| Mixed-width tuples | 122,893 / 98,329 | 13,956,856 / 5,300,546 | 13,091,998 / 25,681,498 |

The clearest new hotspot is mixed-width tuple encoding: **2.63× Candid at
opt-z and 6.68× at opt-3**, with 122,893 versus 98,329 frame bytes (+25.0%).
At opt-z the other four large encodes cost 1.20–1.69× Candid; their decodes use
0.15–0.38× its instructions. The complete [80 application rows](current-application-cases.csv)
include both profiles, all sizes, absolute IC costs, frame bytes, repeated native
timing, allocation counts/bytes and peak live heap for both codecs.

### Remaining gaps and supported representations

Ordinary controls still expose large gaps. At opt-z, wide u16-list encode with
16,384 elements costs 10,487,432.5 versus 86,364.5 instructions (121.43×);
generic 1 MiB byte-vector encode costs 158,336,929 versus 2,117,967 (74.76×).
Keep these visible alongside explicit Packed/byte-field alternatives and lint
advice; do not silently rewrite consumer Serde or claim every payload wins.

Supported representations materially change the outcome:

- Packed i16, 16,384 wide values: 34,338.5 / 86,093.5 encode instructions and
  33,932.5 / 1,219,098.5 decode instructions; 32,782 / 32,780 frame bytes.
- Public nonzero-i64 adapter, 4,096 values: encode/decode cost 0.40× / 0.33×
  Candid. Public heap-u64 adapter: 0.40× / 0.50×. Wire validates nonzero values
  and reconstructs a heap; Candid controls are integer vectors.
- Packed ID16 owned list, 4,096 IDs: 67,613 / 155,582 encode instructions and
  67,109 / 90,553 decode instructions; 65,550 / 65,548 bytes. Candid's control
  is an already-flattened blob. Borrowed-source Wire lanes include copying and
  retain separate results; DTO construction is otherwise outside timing.

These are explicit representation choices with compatibility and small-value
size tradeoffs. They do not add a second production codec. The next useful
experiment is mixed-width tuple and aggregate-record encoding/admission cost,
with pre-allocation limits preserved. Do not revive the private parser or
introduce generic per-type codec variants solely to improve this table.

### Method and reproduction

Each profile retains two native runs of eleven calibrated batches (2 ms minimum),
plus separate allocation and peak-heap builds. CSV native nanoseconds are the
median of 22 per-operation batch means; they are shared-host timing evidence,
not IC instructions or request-tail latency. Memory metrics are native allocator
observations, not IC heap measurements. IC counts include result destruction and
loop overhead; loop controls are retained, not subtracted. Setup and reply
transport are excluded. Wire budgets and Candid quotas have different semantics.
Build times are operational records, not controlled cold-compilation comparisons.

This is a current codec comparison, not a matched regression study against the
historical source below, whose fixtures and locks differ. No before/after cost
screen is claimed to pass. Prior accepted optimization tradeoffs remain recorded
in their paired studies.

Run the maintained matrix against a fresh output and target with the prepared
server (never reuse this completed directory). Raw output, logs, manifests,
server/artifact hashes, samples, frozen sources and verifiers are retained under
`target/evidence/codec-matrix-core020`. Reproduce the compact data with:

`node target/evidence/codec-matrix-core020/wire020-matrix-summary.mjs target/evidence/codec-matrix-core020`

`node target/evidence/codec-matrix-core020/write-report-data.mjs target/evidence/codec-matrix-core020 docs/benchmarks`

## Completed full baseline — 10 November 2026

The full frozen baseline now verifies **39/39 cohorts**, **2,234 overlapping
case entries**, **8,936 primary encode/decode comparisons**, **46,978 IC rows**
and **151,392 native/frame checks**. All 156 guest builds and baked source/input
identities verify. Rust 1.99.1, direct Candid 0.10.40, identical repeated IC
samples, native repeats and both profiles are recorded under
`target/evidence/codec-matrix-20261110/` (`summary.json`, `summary-final.txt`,
frozen `source/` and per-cohort results). No missing cohort remains.

This source predates the four retained runtime optimizations and the 20 new
application-composition cases below. It is a completed baseline, not a full
qualification of those changes. Those have their own matched studies.

Wire wins 4,028/4,468 primary comparisons at opt-z and 4,126/4,468 at opt-3.
Outside the mixed research cohort, wins are 7,652/8,400; within it, 502/536.
These are overlapping corpus counts, not unique types or traffic weights.
The research cohort includes public bool adapters, private float prototypes
and controls; its results are not uniformly production-ready.

Wire has smaller total **and** code-section Wasm in all **78 profile/cohort
pairs**. Selected uncompressed total / code-section bytes:

| Fixture | Profile | Wire | Candid |
| --- | --- | ---: | ---: |
| Standard | opt-z | 593,655 / 547,058 | 1,426,816 / 1,352,168 |
| Standard | opt-3 | 893,367 / 832,967 | 2,074,804 / 1,985,363 |
| Toko-shaped | opt-z | 80,478 / 69,354 | 375,448 / 341,263 |
| Toko-shaped | opt-3 | 103,282 / 92,906 | 593,903 / 561,172 |
| 100 derived types | opt-z | 310,553 / 291,848 | 1,424,867 / 1,370,806 |
| 100 derived types | opt-3 | 395,846 / 361,556 | 2,086,335 / 2,017,152 |
| 100 DTO adapters | opt-z | 337,944 / 319,000 | 1,495,696 / 1,441,311 |
| 100 DTO adapters | opt-3 | 551,262 / 516,765 | 2,421,226 / 2,351,730 |

These linked fixtures are not complete Toko applications. Native time remains
shared-host evidence, separate from deterministic instructions. Ordinary list
controls remain expensive: the standard 1 MiB generic byte vector encodes in
158,336,943 instructions versus Candid's 2,117,967 (74.76×), with frame sizes
1,048,590 / 1,048,588 bytes. Explicit byte fields remain the supported remedy;
schema advice must not silently change consumer serialization. Large ordinary
bool and float lists likewise remain deliberate controls. The subsequent
matched studies below address finite adapters and aggregate encoding, not every
ordinary-list disadvantage. No new protocol change follows from this baseline.

## Expanded hotspot sweep — 10 November 2026

A completed **154-case** focused sweep includes the 20 new application cases and
retained f32 scanner. It is separate from the completed 39-cohort baseline,
whose frozen source predates both changes. Rust 1.99.1, direct Candid 0.10.40,
locked sources, two native runs of 11 batches (2 ms minimum), three identical
IC samples per operation and fresh guests are verified. Evidence:
`target/evidence/expanded-hotspots-20261110/` (source, hashes, frames,
allocation/heap observations, native runs, instructions and Wasm).
Verification covers 3,202 IC rows and 10,280 native/frame comparisons.

Subsequent matched [encoder studies](../evidence/encoder-followup.md#newtype-enum-tags--10-november-2026)
retain unit/newtype tag and depth improvements. Their latest 154-case fixture
is 264,432 total / 229,215 code bytes at opt-z, and 348,920 / 316,157 at opt-3;
Candid stays identical to the table below. Large result-map and mixed-enum
opt-z encodes now cost 7,814,676 and 4,772,387.5 instructions respectively.
The newtype study explicitly accepts secondary-profile regressions and fails
the strict opt-3 cost screen; see its before/after report. The sweep table below
retains its original source-specific values rather than mixing measurements.

Wire wins **273/308** opt-z and **277/308** opt-3 primary encode/decode comparisons.
These are corpus counts, not traffic weights. Among the 20 new cases, Wire wins
32/40 in each profile: all new decodes win; eight encode comparisons lose.

Selected **opt-z IC instructions/op**, fresh-output encoding, 4,096 entries:

| New shape | Wire encode | Candid encode | Wire/Candid | Wire / Candid bytes |
| --- | ---: | ---: | ---: | ---: |
| Mixed-width tuples | 14,042,872 | 5,300,546 | 2.65× | 122,893 / 98,329 |
| Mixed enum batch | 5,774,676.5 | 2,817,442 | 2.05× | 65,534 / 71,027 |
| Map of results with blobs | 9,459,476 | 5,320,628 | 1.78× | 104,333 / 112,676 |
| Paginated Packed records | 10,405,151 | 7,394,525 | 1.41× | 150,339 / 175,067 |
| Sparse optional blobs | 3,096,219.5 | 2,538,982.5 | 1.22× | 50,519 / 50,521 |

Their opt-z decode ratios are respectively 0.48×, 0.14×, 0.29×, 0.37× and 0.18×.
Opt-3 tuple encode is the new worst ratio: **5,333,974 / 798,786 = 6.68×**;
no new comparison reaches 10×. Full-width tuple varints also produce a larger
frame. Packed fields already exist in the page case: ordinary-list advice alone
cannot remove all aggregate encoding cost. Attribution to traversal, allocation
or varint handling needs a matched experiment; these results do not establish
which implementation change would help.

Existing ordinary controls still lose substantially: opt-z bool-list encode
46.04×, heap encode 46.02× and nonzero-i64-list encode 38.40×. Their explicit
adapters remain the first consumer choice to evaluate. Raw IEEE f32/f64 controls
lose 27.60×/14.05× on large encode; finite adapters narrow acceptance and must not
be substituted silently. The retained finite f32/f64 encode ratios are
**2.29×/1.95×**, while decode costs 0.47×/0.51× Candid. Raw Candid accepts non-finite
values; the finite checks remain intentional.

| Linked 154-case fixture | Wire total / code bytes | Candid total / code bytes |
| --- | ---: | ---: |
| opt-z | 265,567 / 230,348 | 806,250 / 743,369 |
| opt-3 | 348,715 / 315,953 | 1,195,978 / 1,135,110 |

These are uncompressed fixture modules, not application savings or a matched
before/after optimization claim. Native opt-z repeats also show the five new
large encodes slower (roughly 1.1–1.8×) and decodes faster; shared-host times are
separate from deterministic IC instructions. No production code changed during
this sweep. Bytes, identities, limits and acceptance remain unchanged.

## Historical full matrix — 9 November 2026

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
Supported host metadata emits [bulk advice](../guides/packed-fields.md), including bool
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

The [earlier matched study](https://github.com/dragginzgame/ic-wire/blob/bcbcec53ce9eaadf5a839efdb95595b834d2583b/docs/benchmarks/candid-comparison.md#borrowed-adapter-investigation)
records why the public adapters were adopted. Current 100-type DTO → adapter
Wasm total/code is **337,928/318,968 → 346,145/327,180** at opt-z and
**551,262/516,765 → 567,411/532,912** at opt-3. Already-packed data stays smaller;
these DTO controls omit domain conversion. This is not a new before/after codec
change. Follow the [consumer guidance](../../crates/ic-wire/README.md#representation-and-adoption-details).

## Signed32 follow-up (8 November, Core 0.17.14)

The historical baseline and rejected writer experiments remain in the
[encoder study](../evidence/encoder-followup.md#signed-i32-writer-screen-8-november-2026).
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
public Packed<float> support. See [the original studies](../evidence/primitive-bulk.md).

## Next experiments

1. **Measure finite fields in consumer-shaped records.** The pending public
   adapters now have [Rust/TS qualification and matched costs](../evidence/primitive-bulk.md#public-finite-adapters--9-november-2026).
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
   [outer-visitor experiments](../evidence/bound-sharing.md#outer-list-visitor-screen-7-november)
   barely changed the broad corpus. Do not adopt complexity for stress-only wins.

Do not repeat blanket one-pass encoding, forced varint-reader sharing or common
buffer-reader extraction without a new hypothesis. The
[encoder](../evidence/encoder-followup.md) and [runtime](../evidence/runtime.md) studies
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

Reproduce using the [maintained matrix command](../guides/development.md#candid-comparison-benchmarks)
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


## Upstream Postcard hard cut — 10 November 2026

Retain upstream Postcard 1.1.3 decoding in place of the 618-line private reader.
This is a maintenance hard cut with measured instruction regressions, not a
universal speed optimization. The bounded Serde adapter, encoder and framing
remain authoritative and unchanged. No second reader or fallback is retained.

Acceptance broadens: integer `80 00` becomes zero and char `02 61 62` becomes
`'a'`, previously Malformed. Browser integer/Result tags follow Postcard; options
and bools still have byte tags. Collection lengths follow target usize: Wasm32
and browser readers allow five groups; a 64-bit Rust host allows ten. A six-group
zero length is therefore host-only. Explicit target-width tests cover that
exception outside the cross-runtime equality oracle. Valid encoder bytes,
identities and budget rules are unchanged. Regenerate clients, review rejection
assumptions, retain original retry/hash bytes; existing persisted encodings need
no rewrite. See the [runtime contract](../../crates/ic-wire/README.md#upstream-postcard-decoding-hard-cut).

Fresh isolated baseline/candidate sources and separate Cargo targets compare the
same 154-case coverage-shapes corpus, including tiny controls, buffers, Packed,
Principal/Account, maps and composed record batches. Rust/Cargo 1.99.1,
wasm32-unknown-unknown, fat LTO, one codegen unit, panic abort, stripped symbols;
opt-z is primary and opt-3 secondary. Qualification lock SHA-256 is
`80518b084229e6856d948d3e7e62a9436ce3988441a272094b1f5b4f489df2f0`.
The candidate native module SHA-256 is
`5ec8fdab89eab4a195b08f46a2251a5bfcc6b70d8bec3a955131974814ba1951`.
At qualification, Rust codec files and both locks matched the measured source
exactly. The subsequent dev-only Toko Tooling 0.39.7 adoption changes the current
locks; the recorded study retains its original 0.39.6 selection.

Uncompressed Wasm, **total / code-section bytes**:

| Profile | Private reader | Upstream | Change |
| --- | ---: | ---: | ---: |
| opt-z | 264,432 / 229,215 | 264,145 / 228,928 | -287 / -287 |
| opt-3 | 348,920 / 316,157 | 348,673 / 315,911 | -247 / -246 |

These are linked qualification guests, not projected application savings. Candid
sizes are unchanged: 806,250 / 743,369 at opt-z; 1,195,978 / 1,135,110 at opt-3.
Its opt-z artifact digest differs despite equal sizes and all equal instruction
counters; do not claim byte-identical Candid modules.

IC decode instructions (three identical samples per row; small cases use four
repetitions, large cases one):

| Case | Profile | Private reader | Upstream | Change |
| --- | --- | ---: | ---: | ---: |
| NonZeroU16 65535 | opt-z | 513.5 | 570.5 | +11.10% |
| 4,096 byte-array hashes | opt-z | 1,471,494 | 1,606,678 | +9.19% |
| 4,096 mixed-width tuples | opt-z | 12,399,758 | 13,091,998 | +5.58% |
| 4,096 paginated records | opt-z | 11,509,544 | 11,859,545 | +3.04% |
| 4,096 byte-array hashes | opt-3 | 1,031,735 | 1,048,106 | +1.59% |
| 4,096 mixed-width tuples | opt-3 | 5,119,657 | 3,870,359 | -24.40% |
| 4,096 paginated records | opt-3 | 7,969,509 | 7,946,845 | -0.28% |

All 3,202 IC rows per lane retain matched inputs and frame hashes. All 1,540
Candid rows are unchanged. Of 831 Wire rows per profile, opt-z has 314 regressions
and 517 unchanged; opt-3 has 125 improvements, 153 regressions and 553 unchanged.
The largest opt-3 slowdown is 3.13% (+8 instructions) for NonZeroU16 1.
Direct application of the `baseline * 1.05 + 64` instruction screen finds
**19 opt-z failures and zero opt-3 failures**. Retain these losses explicitly in
exchange for removing the fork; this is not a passing no-regression result.

The automated cost checker stops at driver identity: the private-parser source
hash field became an upstream-owner marker. Independent comparison verifies
that this metadata line is the only driver-code difference; measurement logic,
fixture/operation hashes, compiler, locks and server match. The differing opt-z
Candid artifact digest is a further strict-check limitation. Neither check was
relaxed or its evidence overwritten. Fresh guest records and baked driver/source
hashes verify independently in `compare.py`.

All 5,140 allocation/heap observations match, including fresh output capacities.
Two native runs/profile, each with 11 batches and a 2 ms batch target, verify
5,140 corresponding frame/input rows per lane. Native timing is noisy shared-host
evidence: the 4,096-hash decode medians are 61,933/63,779 ns before versus
64,599/72,513 ns after at opt-z; opt-3 is 22,475/24,047 versus 25,504/56,243 ns.
Do not attribute the large second-run native fluctuation to IC execution.

Focused qualification passes: all-feature Core tests/doctests (83 native tests,
three existing ignored probes), strict Core Clippy, 178 Rust/browser vectors,
39,092 differential mutations with normalized re-encode comparisons, explicit
Wasm32 length/Result tests, standalone Principal/Account host/Wasm/Endpoint graphs,
18 coverage and ten signed32 corpus tests, and 10,000 sanitizer fuzz inputs.
The fuzzer now compares semantic roundtrips instead of assuming received bytes
are canonical. Its sandboxed final leak check was blocked by ptrace; the same
prepared test passes outside that restriction. The generated package contains
upstream decoder selection and no private parser. Packaging used `--no-verify`;
source tests supply compilation evidence. Full release CI and the 39-cohort matrix
were not rerun for this cut.

Raw evidence: `target/evidence/upstream-postcard-20261110/`, containing isolated
`baseline/source` and `candidate/source`, each lane's `qualified` output,
`compare.py`, `verification.json`, `deltas.json` and `cost-screen.txt`. The initial
sandbox server failures remain separate. To reproduce each lane, run from its
frozen `source` with `CORPUS_FEATURE=coverage-shapes`, a new lane-specific
`CARGO_TARGET_DIR`, `CODEC_SOURCE_SNAPSHOT="$PWD"` and the explicitly prepared
`POCKET_IC_BIN`; invoke `bash scripts/ci/evaluate-codecs.sh ../rerun` (the output
must not already exist). PocketIC needs permission to bind localhost. Run
`python3 target/evidence/upstream-postcard-20261110/compare.py` from the repository
root to verify the retained original paired results. No release was performed.
