# Full Wire/Candid comparison: 0.6.7 candidate

This measures runtime commit `066b618` with expanded benchmark fixtures; workspace
package versions still read 0.6.6. It is a current-codec comparison, not a claim
that enlarging the suite improved performance. Production code, bytes, admission
and canonical acceptance are unchanged by this benchmark batch.

Wire retains a large Wasm advantage: 58.7% smaller for the expanded standard
fixture and 79.7% smaller for 100 records at opt-3. It wins most standard IC
comparisons, but large primitive-vector encoding remains the clearest weakness:
wide i16 lists cost about 50x Candid, generic byte lists 37x. Owned 1 MiB blob
decode remains 2.37x Candid. The largest new account snapshot decodes at 0.30x
Candid's instruction cost; the Toko 50-card projection encodes at 1.019x.

## Coverage and interpretation

The standard suite grows from 154 to 214 cases. Sixty appended cases preserve
existing case indices and add negative/mixed-width integers, varint and blob
boundaries, float/boolean/optional lists, nested blobs and account snapshots with
maps, mixed events and optional signed values. These are deterministic synthetic
inputs, not a production traffic distribution. The existing Toko-shaped cohort
adds ten cases; the 1/10/100 nominal record cohorts bring the total to 335 cases.
The record cohorts repeat sixteen structural widths; they do not predict an
installed application's footprint. Toko projections retain the qualifications
in the [consumer study](candid-consumer-comparison.md).

Wire uses one production Serde/Postcard codec. `wire-native`, `wire-bounded` and
`wire-borrowed` denote field representation or ownership, not alternate codecs.
The headline comparison uses ordinary owned Wire values in standard/Toko cases
and the existing bounded record representation in the scaling cohorts. Complete
raw evidence preserves alternate ownership/representation lanes and rejection
and buffer-reuse operations. Candid has no measured reuse counterpart.

Wire frame lengths include the 11-byte method/direction envelope. Candid lengths
include its self-describing one-argument encoding. Method integration, transport,
authentication, persistence and application work are outside these measurements.
Wire budgets and Candid decode quotas have different semantics; this comparison
does not establish equivalent resource policies or Candid-style compatibility.

## Method and reproduction

Rust 1.99.1 / LLVM 23.1.1; Linux x86_64 native and wasm32-unknown-unknown; both
profiles use LTO, one codegen unit, panic abort and stripped symbols. `evaluation`
uses opt-z; `benchmark-speed` uses opt-3. The exact lock, source, flags, CPU and
compiler identities are retained with the evidence. Dependencies are Candid
0.10.39, Postcard 1.1.3, Serde 1.0.229, IC Platform 0.6.9 and Testkit 0.65.0.

Native timing uses two sequential runs per profile, eleven samples per run,
and calibration to at least 2 ms per batch (1,048,576-operation cap). These are
unpinned warm-cache microbenchmarks with result destruction and loop/dynamic-call
overhead included. Input construction, semantic roundtrip checks and reporting
are excluded. Reported timing ranges span the two run medians; they are not
confidence intervals or request-tail latency. No other benchmark/build was
launched by this task during a timed run; unrelated host activity is uncontrolled.

Allocation counts and peak live heap use separate instrumented native builds
and one warmed operation. Counts, allocated bytes and peak live bytes are
different metrics; they do not measure Wasm heap behavior. Input/setup allocations
are excluded. IC instructions use PocketIC 16.1.0, performance counter 0, three
samples, four repetitions through 64 KiB and one above. Transport/setup is outside
the counter; destruction and loop overhead are included without subtraction.

All sizes are total uncompressed Wasm and code-section payload bytes. Whole
artifacts include fixtures, dispatch and harness code. Adding cases changes those
artifacts, so compare Wire with Candid within this run, not with earlier smaller
corpora. No compressed-size objective or symbol-attributed savings claim is used.
Cached build durations are retained for provenance, not treated as fair compiler
performance comparisons.

Reproduce from the archived source with `RUNS=2 SAMPLES=11 BATCH_MS=2`, a prepared
`POCKET_IC_BIN` and `bash scripts/ci/evaluate-codecs.sh NEW_OUTPUT`. Run sequentially
with `CORPUS_FEATURE` empty, `toko`, `derive-scale-1`, `derive-scale-10`, then
`derive-scale`. Each output must be new. Keep the same compiler, lock, features,
profiles, limits and deterministic input definitions when comparing results.

## Uncompressed Wasm results

| Cohort / profile | Wire total / code bytes | Candid total / code bytes | Total size reduction |
| --- | ---: | ---: | ---: |
| 214 standard cases / opt-z | 295,445 / 258,463 | 700,385 / 641,100 | 57.82% |
| 214 standard cases / opt-3 | 435,212 / 383,543 | 1,053,914 / 979,866 | 58.71% |
| 10 Toko-shaped cases / opt-z | 87,705 / 76,478 | 373,607 / 339,489 | 76.52% |
| 10 Toko-shaped cases / opt-3 | 114,784 / 104,224 | 592,136 / 559,369 | 80.62% |
| 1 record / opt-z | 55,598 / 44,581 | 299,172 / 266,354 | 81.42% |
| 1 record / opt-3 | 77,930 / 51,179 | 433,476 / 385,608 | 82.02% |
| 10 records / opt-z | 107,443 / 95,567 | 434,583 / 399,342 | 75.28% |
| 10 records / opt-3 | 149,196 / 121,567 | 678,552 / 628,148 | 78.01% |
| 100 records / opt-z | 324,033 / 305,218 | 1,416,914 / 1,362,897 | 77.13% |
| 100 records / opt-3 | 422,871 / 388,342 | 2,079,944 / 2,010,763 | 79.67% |

## IC instruction and wire-size examples

Opt-3, ordinary owned values except the existing bounded record cohort. Each
cell shows **Wire / Candid**; instructions are rounded here, exact values are
in the CSV. Results include the weak cases rather than selecting only wins.

| Case | Frame bytes | Encode instructions | Decode instructions |
| --- | ---: | ---: | ---: |
| standard/account-snapshot-0 | 66 / 195 | 5,308 / 169,257 | 10,100 / 205,132 |
| standard/account-snapshot-128 | 11,763 / 12,607 | 229,888 / 264,162 | 468,854 / 1,561,674 |
| standard/nested-blobs-128 | 91,576 / 91,576 | 142,691 / 302,626 | 354,368 / 336,648 |
| standard/blob-random-1048576 | 1,048,590 / 1,048,588 | 1,049,715 / 2,115,305 | 2,524,535 / 1,065,406 |
| standard/generic-byte-vector-1048576 | 1,048,590 / 1,048,588 | 78,644,290 / 2,112,185 | 134,747,700 / 119,553,449 |
| standard/i64-list-negative-4096 | 4,109 / 32,779 | 521,016 / 80,485 | 851,578 / 330,914 |
| standard/i16-list-wide-16384 | 49,166 / 32,780 | 4,014,941 / 80,468 | 5,919,094 / 1,408,484 |
| standard/i64-list-wide-16384 | 163,854 / 131,084 | 6,931,462 / 277,260 | 12,194,344 / 1,277,563 |
| toko/toko-token-card-projection-50 | 10,798 / 11,004 | 217,529 / 213,440 | 742,872 / 1,845,558 |
| derive-scale/record_099 | 46 / 102 | 2,043 / 57,307 | 3,616 / 54,780 |

Case counts below are descriptive, not weighted application scores. The 1/10/100
cohorts overlap and repeat structural widths; do not sum them into an overall win rate.

| Cohort / profile | Wire encode wins / cases | Wire decode wins / cases |
| --- | ---: | ---: |
| 214 standard cases / opt-z | 159 / 214 | 151 / 214 |
| 214 standard cases / opt-3 | 161 / 214 | 155 / 214 |
| 10 Toko-shaped cases / opt-z | 8 / 10 | 8 / 10 |
| 10 Toko-shaped cases / opt-3 | 9 / 10 | 10 / 10 |
| 1 record / opt-z | 1 / 1 | 1 / 1 |
| 1 record / opt-3 | 1 / 1 | 1 / 1 |
| 10 records / opt-z | 10 / 10 | 10 / 10 |
| 10 records / opt-3 | 10 / 10 | 10 / 10 |
| 100 records / opt-z | 100 / 100 | 100 / 100 |
| 100 records / opt-3 | 100 / 100 | 100 / 100 |

## Native timing, allocation and peak-heap examples

Opt-3 timing ranges are the minimum and maximum of two run medians, in µs.
Allocations and peak live bytes are separate instrumented native measurements;
each paired cell is Wire / Candid. Setup and input ownership are excluded.

| Case / operation | Wire µs | Candid µs | Allocations | Peak live bytes |
| --- | ---: | ---: | ---: | ---: |
| account-snapshot-128 / encode | 3.611–3.666 | 6.001–6.282 | 1 / 99 | 11,763 / 36,392 |
| account-snapshot-128 / decode | 14.638–15.076 | 38.944–39.375 | 265 / 1365 | 23,699 / 28,777 |
| blob-random-1048576 / encode | 17.633–18.422 | 774.844–796.547 | 1 / 17 | 1,048,590 / 2,098,807 |
| blob-random-1048576 / decode | 34.582–35.295 | 18.437–18.521 | 1 / 25 | 1,048,576 / 1,049,380 |
| generic-byte-vector-1048576 / encode | 1253.732–1368.880 | 749.986–761.273 | 1 / 16 | 1,048,590 / 2,098,807 |
| generic-byte-vector-1048576 / decode | 4895.158–5077.893 | 2047.892–2143.166 | 1 / 23 | 1,048,576 / 1,049,380 |
| i64-list-negative-4096 / encode | 9.987–10.092 | 1.575–1.619 | 1 / 16 | 4,109 / 67,189 |
| i64-list-negative-4096 / decode | 23.725–24.556 | 5.779–5.909 | 1 / 23 | 32,768 / 33,572 |

## Evidence and validation

The [complete paired table](evidence/full-comparison-067/comparison.csv) contains
1,340 encode/decode comparisons across both profiles, including frame sizes,
IC instructions, both native run medians, allocation/reallocation counts,
allocated bytes and peak live heap. [Artifact sizes and hashes](evidence/full-comparison-067/artifacts.csv),
[instruction case counts](evidence/full-comparison-067/instruction-wins.csv) and
[compiler/dependency/machine identity](evidence/full-comparison-067/identity.json)
provide the accompanying context.

The [raw archive](evidence/full-comparison-067/raw.json.gz) retains baseline
source (the current runtime plus expanded fixtures), runner, reporting scripts,
final validation source, all 24,672 native
and instrumented rows, all 7,598 IC rows and run identities. It includes all
measured ownership lanes, buffer reuse and malformed-header/truncation operations.
Archive compression is storage only; no compressed Wasm measurement is produced.
Extract its `source` mapping into a clean temporary workspace to reproduce the
commands above. Native sample arrays preserve variation within each run.

All native, allocation, heap and IC rows agree on input identities and encoded
frames for each case/lane. Every IC row has three identical instruction samples.
Benchmark semantic roundtrips, selected-case identity, duplicate-name rejection,
header/truncation/trailing-data rejection and buffer reuse pass. Focused benchmark
Clippy, Rust 1.91 checking, formatting and runner shell syntax pass. The Wasm
artifacts execute successfully in PocketIC. No full CI, macOS or installed
consumer qualification is claimed.
