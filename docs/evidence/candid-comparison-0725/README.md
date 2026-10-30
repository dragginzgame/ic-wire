# Current Wire versus Candid: full codec matrix

Measured 30 October 2026 from current 0.7.24 source, with pending 0.7.25
benchmark-only changes. Production codec, dependencies and protocol are unchanged.
All **22 cohorts / 1,208 case entries / 4,832 primary comparisons** pass the
retained analyzer. Wire wins 4,332 successful encode/decode instruction comparisons;
Candid wins 500, with no ties. These overlapping controls are not weighted by
production traffic. Wire has smaller raw Wasm and code sections in **all 44**
cohort/profile pairs. Frame sizes favor Wire in 758 case entries, tie in 15 and
favor Candid in 435. All 25,670 IC rows have three equal fresh samples; all medians
match the earlier cache controls exactly (0% change).

## Wasm footprint

Total uncompressed bytes / code-section bytes; complete modules with encoding
and decoding reachable:

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 298 cases | opt-z | 377,812 / 337,784 | 950,754 / 885,544 |
| Standard, 298 cases | opt-3 | 573,508 / 518,796 | 1,409,623 / 1,329,553 |
| Toko-shaped, 10 cases | opt-z | 81,091 / 69,899 | 384,801 / 349,658 |
| Toko-shaped, 10 cases | opt-3 | 103,841 / 93,308 | 609,004 / 575,323 |
| 1 ordinary derived type | opt-z | 53,966 / 42,965 | 309,885 / 276,074 |
| 1 ordinary derived type | opt-3 | 74,871 / 48,129 | 449,732 / 400,926 |
| 10 ordinary derived types | opt-z | 99,439 / 87,602 | 445,893 / 409,652 |
| 10 ordinary derived types | opt-3 | 129,641 / 102,105 | 695,693 / 644,359 |
| 100 ordinary derived types | opt-z | 310,707 / 291,939 | 1,435,165 / 1,380,149 |
| 100 ordinary derived types | opt-3 | 398,739 / 364,245 | 2,106,847 / 2,036,736 |
| Packed, 109 cases | opt-z | 98,460 / 87,278 | 453,947 / 417,900 |
| Packed, 109 cases | opt-3 | 122,570 / 112,096 | 693,723 / 659,090 |

At 100 ordinary types, opt-3 Wire is 81.07% smaller in total Wasm and 82.12%
smaller in code section. These are fixture comparisons, not linker savings in
an application that also retains external Candid endpoints.

## IC instructions and frame bytes

Standard-corpus wins: opt-z Wire wins 229/298 encodes and 249/298 decodes;
opt-3 wins 233/298 encodes and 277/298 decodes. The 100 ordinary-type cohort
wins every primary comparison in both profiles. Toko-shaped opt-3 wins 9/10
encodes and 10/10 decodes; `toko-token-card-projection-50` encode remains a
loss: 217,529.25 versus 213,182.25 instructions (+2.04%).

The explicit 109-case Packed cohort wins all 436 primary instruction comparisons
across both profiles. Ordinary controls remain visible. Selected opt-3 rows:

| Shape / representation | Wire encode | Candid encode | Wire decode | Candid decode | Wire / Candid bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 MiB byte-oriented blob | 1,049,704 | 2,115,234 | 1,049,508 | 1,068,979 | 1,048,590 / 1,048,588 |
| 16,384 wide i16, ordinary standard vector | 2,638,675.25 | 80,397.25 | 1,904,910.5 | 1,412,079.5 | 49,166 / 32,780 |
| 16,384 wide i16, matched Packed fixture | 33,489.25 | 80,320.25 | 33,476 | 1,411,787 | 32,782 / 32,780 |

The two i16 fixtures have the same logical values; their different compiled
contexts are retained rather than combining Candid counters across fixtures.
Small controls remain important: empty Packed-i16 decode costs 347 instructions
versus 299 for ordinary Wire in that matched fixture, and fixed-width Packed can
expand small integers relative to ordinary varints. Representation changes need
coordinated contracts; no automatic rewrite or protocol change is made here.

There is no universal performance win. The largest successful encode ratio is
ordinary wide u16 at opt-z: 10,323,594.5 versus 86,295.5 instructions (119.63×).
The worst successful decode ratio is ordinary wide u64 in the opt-z bulk control:
14,670,163 versus 1,193,038 (12.30×). The opt-3 worst decode is ordinary wide i64
in the bulk control: 6,246,822 versus 1,166,367 (5.36×).

Rejection cost is separate from successful decoding. A truncated generic 1 MiB
`Vec<u8>` at opt-3 costs 42,472,926 versus 4,289 instructions; the byte-oriented
blob counterpart costs 636 versus 4,290. At opt-z the generic-vector rejection
reaches 268,965,605 versus 6,574. Wire's ordinary sequence traverses a valid prefix
before finding truncation; the byte representation permits a range check before
copying. Candid's bounded admission can reject this unavailable range early too.
See [field selection](../../packed-fields.md) for byte-oriented and Packed types.

## Scope and method

The matrix covers all 21 maintained cohorts plus the focused hotspot corpus.
It retains ordinary vectors alongside explicit byte-oriented and `Packed<T>`
representations, Toko-shaped records, and independent/shared type-count fixtures
at 1, 10 and 100 types. These overlap intentionally; totals are case entries,
not a claim that every entry is a unique Rust type.

Both profiles use stable Rust 1.99.1, wasm32-unknown-unknown, LTO, one codegen unit,
panic abort and stripped symbols. `evaluation` uses opt-z; `benchmark-speed` uses
opt-3. The locked graph selects Candid 0.10.39, Postcard 1.1.3, Serde 1.0.229,
serde_bytes 0.11.19, Platform 0.8.3 and Testkit/Build/Host 0.70.4. Manifest minimums
of 0.70.3 do not describe the resolved graph. Testkit admits the explicitly
prepared PocketIC 16.1.0 server.

Native timing uses two serial runs per profile, eleven samples per operation and
calibrated batches of at least two milliseconds. Result destruction is included;
fixture setup and correctness checks are excluded. Allocation counts and heap
measurements use separate instrumented builds and one warm measured operation;
the timing sample-count argument does not make these repeated allocation samples.
Recorded build times include cache effects and are not a controlled compiler
performance comparison.

IC results use three fresh queries per case/lane/profile. A unique eight-byte
caller prevents identical queries from reusing PocketIC's response cache; the
guest does not inspect the caller. Counters surround the codec operation inside
the guest, with four repetitions for payloads up to 64 KiB and one above that.
The loop control is retained, not subtracted. These are codec-body instructions,
not complete endpoint costs, transport latency or cycle debits. For a separate
complete-call comparison, see the [Tripwire study](../tripwire-comparison-0725/README.md).
Its nightly compiler differs; do not subtract absolute values across the studies.

All Wasm sizes are whole uncompressed modules and their code sections, with
encoding and decoding reachable. They are not projected application linker savings.
Native nanoseconds, IC instructions, allocation/heap metrics, frame bytes and
artifact sizes remain separate. Compression only stores evidence.

## Native and allocation examples

Opt-3 native medians in nanoseconds, shown separately for the two runs:

| Operation | Wire run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: |
| Unit encode | 8.96 / 9.10 | 208.57 / 197.88 |
| Unit decode | 5.13 / 5.24 | 246.13 / 210.53 |
| 1 MiB byte blob encode | 17,501.74 / 18,440.42 | 750,527.50 / 773,619.50 |
| 1 MiB byte blob owned decode | 17,540.71 / 17,792.73 | 18,902.84 / 20,311.44 |

These are warmed batch means on the recorded Threadripper host, not request
latency or IC performance. All eleven raw samples and calibrated repetitions
are retained. The separate allocation-count build reports one Wire allocation
for that blob encode/decode, versus Candid's seventeen/thirty-one allocations
(and three encode reallocations). Separate heap instrumentation reports peak
live bytes of 1,048,590 / 1,048,576 for Wire encode/decode, versus Candid's
2,098,807 / 1,049,380. Instrumentation implementations differ; do not equate
allocation totals, peak live bytes and IC page charges.

## Qualification boundaries

The previous [0.7.10 matrix](../../full-comparison-0710.md) used another dependency
and simulator graph. This refresh is not a matched old-release regression test.
Borrowed decoding begins with an already-owned input slice. Hash-array Candid
fixtures retain nested schemas; flat-byte controls are separate. Candid and Wire
resource policies and schema-evolution rules differ. Successful roundtrips do
not establish identical malformed-input acceptance or browser metadata support
for every Rust collection.

The initial IC samples are retained as cache controls, then rerun against the
same byte-identical Wasm with the fresh-caller driver. A native timing overlap
in packed-shared-100 is retained separately and replaced with serial repeats;
the overlapping timings do not contribute to the reported comparisons.
The [excluded timing control](native-overlap-control.json.gz) preserves all four
original runs; their input/frame hashes match the serial replacements.

## Retained results

- [comparison.csv](comparison.csv): all 4,832 primary instruction, frame, native
  timing, allocation-count and heap comparisons.
- [rejection-comparison.csv](rejection-comparison.csv): all 4,832 paired rejection rows.
- [instruction-wins.csv](instruction-wins.csv), [summary.json](summary.json) and
  [artifacts.csv](artifacts.csv): cohort/profile totals and all 88 module identities.
- [raw.json.gz](raw.json.gz): all native samples, IC samples and run identities;
  [all-lanes.json.gz](all-lanes.json.gz) also exposes borrowing/reuse/control lanes.
- [sampling-controls.json.gz](sampling-controls.json.gz): original anonymous-query
  controls; every median matches its fresh-query counterpart on the same artifact.

Strict Clippy for the changed host sampler, script/analyzer syntax, paired-input
and frame checks, artifact hashes and complete cohort coverage pass. No full CI,
macOS execution or production endpoint regression run is claimed by this codec
study. The separate Tripwire study retains its own complete-call evidence.

## Reproduction

Prepare the locked dependencies, stable compiler, wasm32 target and exact server
recorded in the identities. Run outside other native timing work:

```sh
export POCKET_IC_BIN=/absolute/prepared/pocket-ic
export CARGO_TARGET_DIR=/absolute/isolated-target
bash scripts/ci/evaluate-codec-matrix.sh /absolute/fresh-output
```

The matrix deliberately leaves the focused hotspot corpus separate. Run it from
the matrix's frozen source, using the same target and server:

```sh
cd /absolute/fresh-output/source
CODEC_SOURCE_SNAPSHOT="$PWD" CORPUS_FEATURE=hotspots RUNS=2 SAMPLES=11 BATCH_MS=2 \
  bash scripts/ci/evaluate-codecs.sh /absolute/fresh-output/hotspots
node /absolute/ic-wire/docs/evidence/candid-comparison-0725/analyze.mjs \
  /absolute/fresh-output /absolute/analysis-output
```

Retain Git metadata when running the snapshot outside a checkout, as the matrix
runner does with `GIT_DIR` and `GIT_WORK_TREE`. The analyzer requires fresh caller
identities, verifies artifact hashes and pairs logical-input and encoded-frame
hashes across native/IC measurements and both profiles. Historical cache-control
folders are optional on a new run; when present, every cohort must have one.

[source-identity.json](source-identity.json) records the frozen input file hashes
and separate fresh-driver hash. [benchmark.patch](benchmark.patch) contains the
benchmark changes against the recorded source HEAD; the retained [Cargo.lock](Cargo.lock)
and [cohort roster](cohorts.psv) fix the graph and feature selection. Copied browser
node_modules are excluded from the source manifest because they are not Cargo
benchmark inputs. Native/IC identities retain compiler, parameters and server
hashes alongside the samples.

Original source, build logs and artifacts are retained under
`/tmp/wire-candid-0725-20261030/`. The separately frozen host-only counter fix is
under `/tmp/wire-candid-0725-fresh-counters/`. Nothing was committed or published.
