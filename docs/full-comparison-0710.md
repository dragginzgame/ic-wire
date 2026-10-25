# Full Wire/Candid comparison: released 0.7.10

The suite contains **22 cohorts and 1,208 overlapping case entries**, across two profiles. Wire has smaller total Wasm in **44/44 pairs** and smaller code sections in **44/44 pairs**. Of 4,832 primary owned encode/decode IC comparisons, Wire wins 4,308, with 0 ties. These counts are not traffic-weighted averages.

Frames are smaller for Wire in **758 case entries**, equal in **15**, and larger in **435**. Each case is counted once here, regardless of profile or operation; overlapping cohorts remain separate entries.

The full matrix is an explicit qualification run. The normal `check-benchmarks` correctness gate does not execute these performance measurements. Protocol bytes, acceptance rules and production code are unchanged by this reporting batch.

## Release regression verdict

**No Wire IC instruction regression was found against released 0.7.9.** All
13,590 matched Wire operation rows have exactly equal before/after counters,
including owned/borrowed decoding, fresh/reused encoding, rejection and loop
controls. Exact frame/input hashes match throughout. All 88 matched code-section
sizes, across both codecs and profiles, are unchanged.

| Comparison | Result |
| --- | --- |
| 0.7.9 code normalized to 0.7.10 manifests/lock | All 88 complete Wasm artifacts byte-identical |
| Original 0.7.9 manifests/lock | 25/88 complete artifacts byte-identical; all 25,670 IC rows remeasured |
| Wire IC counters | 13,590/13,590 unchanged |
| Wire code-section sizes | 44/44 unchanged |
| Wire total Wasm | 41/44 unchanged; two −2-byte changes and one +2-byte change |
| All frame/input identities | Unchanged |

Wire's sole total-size increase is the standard opt-3 fixture:
**573,480 → 573,482 bytes (+2; +0.00035%)**, entirely in the element section.
Its code section remains 518,734 bytes. Shared-bound and varied-bound one-type
opt-z fixtures each shrink by two bytes. Equal code-section sizes do not imply
identical code-section contents; differing modules were remeasured.

Candid's original-release controls have 146 changed instruction rows out of
12,080: 95 decreases and 51 increases, ranging from −0.233% to +0.347%.
The largest increase is `page-option-result-0` opt-3 encode:
65,630.25 → 65,858.25 instructions (+228). Candid itself remains version 0.10.39.
These differences occur across the original package/dependency selections; the
normalized comparison removes them. They must not be attributed to the Wire host
cleanup, nor silently discarded from the comparison. Candid total Wasm changes
range from −64 to +10 bytes; its code-section sizes are unchanged.

See [normalized artifacts](evidence/full-comparison-0710/normalized-artifacts.csv),
[original-release artifacts](evidence/full-comparison-0710/release-artifacts.csv)
and [all remeasured counters](evidence/full-comparison-0710/instruction-regressions.csv).
The remaining ordinary-vector and rejection-cost gaps below were already present
in the matched 0.7.9 measurements; they are not regressions introduced by 0.7.10.

## Uncompressed Wasm sizes

| Cohort | Profile | Wire total / code bytes | Candid total / code bytes | Total reduction |
| --- | --- | ---: | ---: | ---: |
| standard | opt-z | 377,806 / 337,777 | 933,386 / 869,186 | 59.52% |
| standard | opt-3 | 573,482 / 518,734 | 1,386,033 / 1,306,893 | 58.62% |
| toko | opt-z | 81,093 / 69,901 | 373,419 / 339,307 | 78.28% |
| toko | opt-3 | 103,851 / 93,318 | 592,136 / 559,369 | 82.46% |
| derive-1 | opt-z | 53,966 / 42,965 | 298,984 / 266,180 | 81.95% |
| derive-1 | opt-3 | 74,871 / 48,129 | 433,489 / 385,612 | 82.73% |
| derive-10 | opt-z | 99,439 / 87,602 | 434,457 / 399,223 | 77.11% |
| derive-10 | opt-3 | 129,641 / 102,105 | 678,666 / 628,262 | 80.90% |
| derive-100 | opt-z | 310,707 / 291,939 | 1,416,974 / 1,362,964 | 78.07% |
| derive-100 | opt-3 | 398,739 / 364,245 | 2,081,252 / 2,012,071 | 80.84% |
| packed-i16 | opt-z | 43,331 / 34,431 | 215,803 / 190,164 | 79.92% |
| packed-i16 | opt-3 | 48,515 / 39,765 | 284,146 / 259,050 | 82.93% |
| packed-control | opt-z | 40,818 / 32,056 | 215,811 / 190,164 | 81.09% |
| packed-control | opt-3 | 44,902 / 36,264 | 284,138 / 259,050 | 84.20% |
| packed-shared-1 | opt-z | 61,572 / 49,706 | 304,793 / 271,198 | 79.80% |
| packed-shared-1 | opt-3 | 90,721 / 63,134 | 449,222 / 400,570 | 79.80% |
| packed-shared-10 | opt-z | 83,165 / 70,578 | 398,484 / 362,975 | 79.13% |
| packed-shared-10 | opt-3 | 118,580 / 90,316 | 656,974 / 606,354 | 81.95% |
| packed-shared-100 | opt-z | 261,373 / 242,030 | 1,283,379 / 1,229,497 | 79.63% |
| packed-shared-100 | opt-3 | 390,268 / 355,237 | 2,098,605 / 2,029,665 | 81.40% |
| packed-varied-1 | opt-z | 61,570 / 49,704 | 304,793 / 271,198 | 79.80% |
| packed-varied-1 | opt-3 | 90,720 / 63,133 | 449,222 / 400,570 | 79.81% |
| packed-varied-10 | opt-z | 91,750 / 79,111 | 398,484 / 362,975 | 76.98% |
| packed-varied-10 | opt-3 | 133,497 / 105,174 | 656,974 / 606,354 | 79.68% |
| packed-varied-100 | opt-z | 278,181 / 258,698 | 1,283,379 / 1,229,497 | 78.32% |
| packed-varied-100 | opt-3 | 415,573 / 380,422 | 2,098,605 / 2,029,665 | 80.20% |
| bulk | opt-z | 98,460 / 87,278 | 442,079 / 407,050 | 77.73% |
| bulk | opt-3 | 122,570 / 112,096 | 676,458 / 642,739 | 81.88% |
| bulk-control | opt-z | 105,042 / 93,827 | 442,079 / 407,050 | 76.24% |
| bulk-control | opt-3 | 136,359 / 125,845 | 676,458 / 642,739 | 79.84% |
| bulk-1 | opt-z | 70,615 / 58,664 | 365,689 / 331,589 | 80.69% |
| bulk-1 | opt-3 | 100,333 / 72,663 | 531,930 / 482,712 | 81.14% |
| bulk-10 | opt-z | 110,247 / 97,554 | 502,633 / 466,594 | 78.07% |
| bulk-10 | opt-3 | 150,150 / 121,803 | 776,659 / 725,476 | 80.67% |
| bulk-100 | opt-z | 457,655 / 438,206 | 1,793,974 / 1,739,556 | 74.49% |
| bulk-100 | opt-3 | 650,829 / 615,774 | 2,639,766 / 2,570,257 | 75.35% |
| bulk-control-1 | opt-z | 77,406 / 65,411 | 365,689 / 331,589 | 78.83% |
| bulk-control-1 | opt-3 | 111,325 / 83,648 | 531,920 / 482,712 | 79.07% |
| bulk-control-10 | opt-z | 119,352 / 106,609 | 502,633 / 466,594 | 76.25% |
| bulk-control-10 | opt-3 | 166,650 / 138,286 | 776,659 / 725,476 | 78.54% |
| bulk-control-100 | opt-z | 466,760 / 447,261 | 1,793,974 / 1,739,556 | 73.98% |
| bulk-control-100 | opt-3 | 667,333 / 632,257 | 2,639,766 / 2,570,257 | 74.72% |
| hotspots | opt-z | 83,622 / 72,630 | 384,365 / 350,688 | 78.24% |
| hotspots | opt-3 | 104,403 / 94,077 | 593,929 / 561,474 | 82.42% |

## Primary IC instruction wins

Each cell is Wire wins / total cases. Losses and absolute counters remain in [comparison.csv](evidence/full-comparison-0710/comparison.csv); reuse, borrowing and rejection rows remain in the raw evidence.

| Cohort | opt-z encode | opt-z decode | opt-3 encode | opt-3 decode |
| --- | ---: | ---: | ---: | ---: |
| standard | 228/298 | 241/298 | 233/298 | 275/298 |
| toko | 8/10 | 8/10 | 9/10 | 10/10 |
| derive-1 | 1/1 | 1/1 | 1/1 | 1/1 |
| derive-10 | 10/10 | 10/10 | 10/10 | 10/10 |
| derive-100 | 100/100 | 100/100 | 100/100 | 100/100 |
| packed-i16 | 20/20 | 20/20 | 20/20 | 20/20 |
| packed-control | 10/20 | 16/20 | 10/20 | 17/20 |
| packed-shared-1 | 1/1 | 1/1 | 1/1 | 1/1 |
| packed-shared-10 | 10/10 | 10/10 | 10/10 | 10/10 |
| packed-shared-100 | 100/100 | 100/100 | 100/100 | 100/100 |
| packed-varied-1 | 1/1 | 1/1 | 1/1 | 1/1 |
| packed-varied-10 | 10/10 | 10/10 | 10/10 | 10/10 |
| packed-varied-100 | 100/100 | 100/100 | 100/100 | 100/100 |
| bulk | 109/109 | 109/109 | 109/109 | 109/109 |
| bulk-control | 47/109 | 60/109 | 49/109 | 74/109 |
| bulk-1 | 1/1 | 1/1 | 1/1 | 1/1 |
| bulk-10 | 10/10 | 10/10 | 10/10 | 10/10 |
| bulk-100 | 100/100 | 100/100 | 100/100 | 100/100 |
| bulk-control-1 | 1/1 | 1/1 | 1/1 | 1/1 |
| bulk-control-10 | 10/10 | 10/10 | 10/10 | 10/10 |
| bulk-control-100 | 100/100 | 100/100 | 100/100 | 100/100 |
| hotspots | 61/87 | 72/87 | 66/87 | 78/87 |

The run retains 25,670 IC result rows, with three counter samples per row. 0 rows have unequal repeated counters. Native sample distributions are separate and must not be interpreted as IC instruction variance.

## Representative payloads at opt-3

| Cohort / payload | Frame bytes Wire / Candid | Encode instructions Wire / Candid | Decode instructions Wire / Candid |
| --- | ---: | ---: | ---: |
| standard / unit | 11 / 7 | 385.25 / 5,957.75 | 207.5 / 6,309 |
| standard / blob-random-1048576 | 1,048,590 / 1,048,588 | 1,049,704 / 2,115,234 | 1,049,508 / 1,065,374 |
| standard / generic-byte-vector-1048576 | 1,048,590 / 1,048,588 | 78,644,279 / 2,112,114 | 42,472,921 / 119,553,417 |
| standard / i16-list-wide-16384 | 49,166 / 32,780 | 2,638,675.25 / 80,397.25 | 2,035,982.5 / 1,408,452.5 |
| packed-i16 / i16-wide-16384 | 32,782 / 32,780 | 33,489.25 / 80,320.25 | 33,476 / 1,408,211 |
| bulk / i8-wide-16384 | 16,398 / 16,396 | 17,298.25 / 47,643.25 | 17,432.5 / 1,244,403.5 |
| bulk / u16-wide-16384 | 32,782 / 32,780 | 33,823.25 / 80,505.25 | 33,843.5 / 1,375,520.5 |
| standard / u64-list-wide-16384 | 163,854 / 131,084 | 7,750,790 / 277,189 | 6,656,290 / 1,965,686 |

## Malformed-input cost

Rejection is a separate measurement. The following standard-corpus controls remove the final encoded byte; their input construction is outside the IC counters. Full primary rejection rows are retained in [rejection-comparison.csv](evidence/full-comparison-0710/rejection-comparison.csv).

| Profile | Payload | Wire instructions | Candid instructions | Wire / Candid |
| --- | --- | ---: | ---: | ---: |
| opt-z | generic-byte-vector-1048576 | 268,965,605 | 32,943 | 8164.58× |
| opt-z | blob-random-1048576 | 816 | 33,913 | 0.02× |
| opt-3 | generic-byte-vector-1048576 | 42,472,926 | 24,367 | 1743.05× |
| opt-3 | blob-random-1048576 | 636 | 24,861 | 0.03× |

Generic sequences can traverse a long valid prefix before discovering truncation. A byte-oriented field exposes the complete byte length and rejects this input before copying its payload. This is a material admission-cost difference, even when valid payload throughput is acceptable. Use byte-oriented binary fields and appropriate runtime budgets; successful-codec wins do not establish cheap rejection for ordinary collections.

## Remaining performance gaps

The ordinary standard corpus still contains expensive per-element encoders.
At opt-3, generic 1 MiB `Vec<u8>` encoding costs 37.23× Candid; 16,384 wide
`i16` values cost 32.82×. At opt-z, wide `u16` encoding reaches 119.67×.
These are current codec comparisons, not newly introduced release regressions.
Use byte-oriented fields for binary data and explicitly coordinated Packed fields
for supported homogeneous words. Ordinary vectors remain useful when compact
varints matter more than bulk speed.

Ordinary `Vec<i8>` also remains expensive: the 4,096-item standard cases cost
13.39× Candid to encode at opt-z. Current `Packed<i8>` supplies the signed-byte
bulk path; the bulk corpus includes ten signed-byte controls. The old 0.6.13
report predates that support. No automatic conversion is claimed. Ordinary byte
and integer-vector losses remain visible alongside packed wins.

The standard corpus's worst primary owned decode is 7.25× Candid at opt-z and
3.39× at opt-3, both on wide `u64` vectors. A byte-oriented 1 MiB blob uses
1,049,508 decode instructions versus Candid's 1,065,374 at opt-3. That blob costs
two extra frame bytes (1,048,590 versus 1,048,588). No payload-size win follows
from an instruction win; Packed can also expand small values relative to Wire
varints. See the complete CSV for absolute counters, payload lengths and controls.

## Regression qualification against 0.7.9

Two baselines distinguish code changes from dependency/version changes:

- **Normalized:** 0.7.9 crate source with 0.7.10's root manifest and lock. This
  isolates the derive/host-registry cleanup under identical dependency selections.
- **Original release:** 0.7.9 source with its own manifests and lock, rebuilt using
  the same pinned compiler, profiles, corpus and prepared server as the new run.

The benchmark fixture source is identical between releases. Every baseline Wasm
is rebuilt and compared with the current artifact, retaining total bytes,
code-section bytes and SHA-256. Identical complete artifacts on identical inputs
supply execution-equivalence evidence; they are not separately sampled baseline
counter measurements. A differing cohort runs the baseline IC driver with the
same inputs and server, verifying frame hashes and all operation counters.

The release changed Platform 0.6.15 → 0.6.16 and Build/Testkit/host support packages
0.68.1 → 0.68.2. Platform's only production-source change removes redundant error
conversion from its native Principal decoder. The benchmark's Candid-facing
Principal uses ExternalPrincipal instead. The artifact comparison checks the
compiled effect rather than assuming this source observation proves equivalence.
Candid itself remains 0.10.39 in both releases.

This matched release verdict covers **IC instructions, exact frames and Wasm
artifacts**. Native timing/allocation/heap rows are fresh Wire/Candid comparisons,
not separately rerun 0.7.9 native measurements. Host metadata-generation changes
have their own [matched qualification](evidence/host-cleanup-0710/README.md).
The run does not qualify macOS timing or consumer endpoint behavior.

## Scope and method

This is a codec qualification, not an installed Canic/Relay endpoint benchmark.
The source is released 0.7.10 (`c2adbfb47da6c180bc470c0d5204a35d7b61d574`).
The 21 maintained cohorts plus the focused hotspot corpus use unchanged fixtures
from released 0.7.9 (`00b1686a80cd7a3068be0a24aaee0dc8239009c7`). Cohort
entries overlap; they are not a traffic distribution or a count of unique types.
Borrowed, buffer-reuse and malformed-input lanes are retained separately from the
primary owned encode/decode comparison. Small inputs and empty collections remain
in the corpus. No production code, wire bytes, acceptance rules or limits changed
in this reporting batch.

Compiler: Rust 1.99.1, LLVM 23.1.1; native Linux x86_64 on Ryzen Threadripper
7970X and Wasm `wasm32-unknown-unknown`. Candid 0.10.39, Postcard 1.1.3,
Serde 1.0.229, Platform 0.6.16 and Testkit 0.68.2 are locked. Both profiles use
LTO, one codegen unit, panic abort and stripped symbols. `evaluation` uses opt-z;
`benchmark-speed` uses opt-3. These are qualification controls, not recommended
consumer build settings or production codec modes. Gzip size is not measured as
a performance objective.

Native timing uses two serial runs per profile, eleven samples per operation and
calibrated batches of at least 2 ms, capped at 1,048,576 calls. Allocator requests
and peak live heap come from separate instrumented runs. Host activity is not
controlled; native run medians are not confidence intervals or request-tail
latencies. Task-owned compilation does not overlap native timing batches.

IC instruction counts use the explicitly prepared PocketIC 16.0.0 server,
SHA-256 `69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`,
admitted and managed by Testkit. Three queries per case measure four repetitions
up to 64 KiB and one above. Counter 0 wraps the warm operation loop; result
destruction, assertions and loop overhead remain included. Setup, transport,
fixture construction and roundtrip equality are outside the counters. The
loop-control lane is retained without subtraction. Each IC output must match
its native fixture bytes. Wire and Candid retain their distinct resource policies.

Every Wasm size is the total uncompressed fixture module or its parsed code
section, including enabled codec lanes and dispatch. These are not symbol-based
savings estimates or net savings in a consumer that retains external Candid
endpoints. Frame lengths include Wire's 11-byte method/direction envelope or
Candid's one-argument DIDL envelope. Packed fields and byte-oriented fields are
explicit schema choices; they do not silently optimize ordinary vectors.

The old 0.6.13 report used PocketIC 16.1.0 and an older decoder contract. Its
instruction numbers must not be used as a matched regression baseline here.
MacOS execution, end-to-end application latency and installed consumer behavior
are outside this run.

Scaling fixtures contain 1/10/100 nominal record types, with repeated structural
widths. Wire uses ordinary Serde derives; Candid uses CandidType and Serde.
Wire's host-only metadata generator is not linked into these codec modules.
The compiler may share identical generated code. A Wire cohort artifact includes
all enabled Wire lanes, including ordinary controls where present. Generic packed
hash fixtures retain Candid's nested `Vec<[u8; 32]>` schema; flat byte-oriented
blob/manifest controls are separate. This does not claim Candid cannot select a
different schema. Container roundtrips do not establish generated TypeScript
support for every tested Rust shape, and the benchmark does not cover arbitrary
custom Serde implementations or every malformed-input family.

Wire limits are 2 MiB frame bytes, 256 MiB allocation admission, 8,388,608 nodes
and depth 64. Candid limits are 2 MiB reply bytes, decoding/skipping quotas of
134,217,728, 4,096 type entries and 64 KiB headers. These policies are not equivalent
resource guarantees. Limits and exact fixture/input hashes are retained with the
raw identities rather than inferred from the resulting ratios.

Lane names `wire-native`, `wire-bounded` and `wire-packed-i16` identify fixture
representations within that one production codec. The generic Packed cohort's
`wire-bounded` label does not imply a per-field const bound: Packed<T> uses
runtime budgets, while PackedI16<N> also retains its explicit element bound.

A concurrent checkout lockfile update selected Build/Testkit/host support 0.68.3
while this run was in progress. It was preserved. The matrix and hotspot run both
use the frozen 0.7.10 lock (0.68.2), so this report does not qualify that later
working-tree update. Reproduction and regression scripts use the frozen source,
not the moving checkout's dependency selection.

Artifacts come directly from the recorded Cargo profiles; no post-link wasm-opt
pass is applied. Installed consumers may have different reachable code and build
pipelines, so these fixture sizes are not a replacement for measuring their final
canister artifacts.

## Reproduction and checks

The [evidence directory](evidence/full-comparison-0710/README.md) retains the source snapshot, baseline patches, scripts, raw samples/identities and per-case CSVs. All measured operations pass the corpus roundtrip/rejection assertions, and every IC frame matches its native oracle. The analysis checks input/payload identities and matched baseline server/driver/fixture settings. Source reconstruction, script syntax, documentation links and whitespace checks pass. Full CI and fresh macOS execution were not run for this evidence/documentation batch.
