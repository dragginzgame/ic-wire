# Full Wire/Candid matrix: 0.6.11 candidate

This measures released base `31bcef2` plus the pending shared packed-i16 decoder.
Workspace package versions still read 0.6.10. The frozen source is retained with
the evidence. This benchmark/report batch changes no production code, schema,
wire bytes, acceptance rules or canonical retry behavior. It does not measure a
new optimization; the paired before/after study is [packed sharing](packed-sharing-0611.md).

Wire has smaller total and code-section Wasm in every cohort/profile pair.
At opt-3, total Wasm is 58.7% smaller in the standard fixture, 80.6% smaller in
Toko, and 79.6% smaller with 100 ordinary derived records. Packed i16 wins all
80 paired encode/decode comparisons across both profiles, including empty inputs.
All ordinary and packed record-scaling comparisons also favor Wire instructions.

The standard codec still loses badly on some ordinary primitive lists. At opt-3,
wide Vec<i16> encoding costs 49.9x Candid, generic Vec<u8> encoding 37.2x, and wide
Vec<i64> decoding 9.55x. Owned 1 MiB byte-oriented blob decoding costs 2.37x.
PackedI16 addresses the i16 representation explicitly; it does not accelerate
ordinary lists. A byte-oriented field already avoids generic Vec<u8> encoding
cost, but owned blob decoding remains a distinct optimization opportunity.

## Coverage and interpretation

Thirteen cohorts contain 597 case entries: 214 standard, ten Toko-shaped,
1/10/100 ordinary records, twenty packed-i16 inputs, twenty ordinary-i16 controls,
and 1/10/100 packed records each with shared and varied bounds. Both opt-z and
opt-3 are measured. These cohorts overlap, so 597 is not a count of unique types
or a production traffic distribution. The ordinary control repeats the packed
cohort's logical inputs with ordinary Vec<i16> fields. Scaling cohorts repeat
structural widths; they do not predict an installed application's footprint.
Toko qualifications remain in the [consumer study](candid-consumer-comparison.md).

Wire uses one production Serde/Postcard codec. The headline uses owned ordinary
values for standard/Toko/control cases, bounded records for scaling, and public
`PackedI16<N>` for the packed-i16 cohort. Packed fields require an explicit schema
choice; their results do not fix ordinary Vec<i16>. Borrowed, buffer-reuse and
rejection lanes are retained separately in the all-lanes CSV and raw archive.
Borrowed decoding has different ownership costs from Candid's owned decode;
there is no measured Candid reuse counterpart.

Frame lengths include Wire's 11-byte method/direction envelope or Candid's
self-describing one-argument frame. Transport, authentication, persistence and
application work are outside the measurements. Wire budgets and Candid quotas
are different policies; this does not establish equivalent resource guarantees
or Candid-style schema evolution.

## Method and reproduction

Rust 1.99.1 / LLVM 23.1.1; Linux x86_64 native and wasm32-unknown-unknown; both
profiles use LTO, one codegen unit, panic abort and stripped symbols. `evaluation`
uses opt-z; `benchmark-speed` uses opt-3. The exact lock, source, flags, CPU and
compiler identities are retained with the evidence. Dependencies are Candid
0.10.39, Postcard 1.1.3, Serde 1.0.229, IC Platform 0.6.10 and Host/Testkit 0.66.0.

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

Reproduce from the frozen source in `raw.json.gz`. Its `runner` lists all thirteen
feature selections; set fresh output/source/target paths and a prepared
`POCKET_IC_BIN` before running it. Use `RUNS=2 SAMPLES=11 BATCH_MS=2` and run
cohorts serially. Each output must be new. The archived `analysis_scripts`
produce the CSV tables; adjust their root/output paths to the extracted files.
Keep the same compiler, lock, features, profiles, limits and deterministic input
fixtures when comparing results. The archive uses compression only for storage;
compressed artifact size is not measured or used to choose implementations.


## Uncompressed Wasm results

| Cohort / profile | Wire total / code bytes | Candid total / code bytes | Total size reduction |
| --- | ---: | ---: | ---: |
| Standard (214) / opt-z | 295,381 / 258,400 | 700,114 / 640,837 | 57.81% |
| Standard (214) / opt-3 | 435,212 / 383,543 | 1,053,914 / 979,866 | 58.71% |
| Toko (10) / opt-z | 87,673 / 76,446 | 373,419 / 339,307 | 76.52% |
| Toko (10) / opt-3 | 114,784 / 104,224 | 592,136 / 559,369 | 80.62% |
| 1 record / opt-z | 55,566 / 44,549 | 298,984 / 266,180 | 81.42% |
| 1 record / opt-3 | 77,934 / 51,183 | 433,489 / 385,612 | 82.02% |
| 10 records / opt-z | 107,508 / 95,632 | 434,457 / 399,223 | 75.25% |
| 10 records / opt-3 | 149,310 / 121,681 | 678,730 / 628,262 | 78.00% |
| 100 records / opt-z | 324,320 / 305,505 | 1,416,974 / 1,362,964 | 77.11% |
| 100 records / opt-3 | 424,179 / 389,650 | 2,081,252 / 2,012,071 | 79.62% |
| Packed i16 (20) / opt-z | 44,554 / 35,509 | 215,811 / 190,164 | 79.36% |
| Packed i16 (20) / opt-3 | 49,542 / 40,776 | 284,146 / 259,050 | 82.56% |
| Ordinary i16 control (20) / opt-z | 42,014 / 33,106 | 215,811 / 190,164 | 80.53% |
| Ordinary i16 control (20) / opt-3 | 45,551 / 36,897 | 284,138 / 259,050 | 83.97% |
| 1 packed record, shared bounds / opt-z | 63,197 / 51,303 | 304,794 / 271,198 | 79.27% |
| 1 packed record, shared bounds / opt-3 | 93,986 / 66,389 | 449,222 / 400,570 | 79.08% |
| 10 packed records, shared bounds / opt-z | 85,698 / 73,072 | 398,485 / 362,975 | 78.49% |
| 10 packed records, shared bounds / opt-3 | 122,108 / 93,830 | 656,974 / 606,354 | 81.41% |
| 100 packed records, shared bounds / opt-z | 263,906 / 244,524 | 1,283,379 / 1,229,497 | 79.44% |
| 100 packed records, shared bounds / opt-3 | 393,797 / 358,751 | 2,098,605 / 2,029,665 | 81.24% |
| 1 packed record, varied bounds / opt-z | 63,195 / 51,301 | 304,793 / 271,198 | 79.27% |
| 1 packed record, varied bounds / opt-3 | 93,985 / 66,388 | 449,222 / 400,570 | 79.08% |
| 10 packed records, varied bounds / opt-z | 99,187 / 86,505 | 398,485 / 362,975 | 75.11% |
| 10 packed records, varied bounds / opt-3 | 143,817 / 115,482 | 656,974 / 606,354 | 78.11% |
| 100 packed records, varied bounds / opt-z | 288,122 / 268,601 | 1,283,379 / 1,229,497 | 77.55% |
| 100 packed records, varied bounds / opt-3 | 430,402 / 395,236 | 2,098,605 / 2,029,665 | 79.49% |

## IC instruction and wire-size examples

Opt-3, owned values; packed-i16 uses PackedI16 and record cohorts use bounded fields. Each
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
| derive-100/record_099 | 46 / 102 | 2,043 / 57,545 | 3,616 / 54,780 |
| packed-i16/i16-small-0 | 12 / 10 | 567 / 13,434 | 564 / 14,839 |
| packed-i16/i16-wide-0 | 12 / 10 | 567 / 13,434 | 564 / 14,839 |
| packed-i16/i16-negative-0 | 12 / 10 | 567 / 12,636 | 564 / 14,602 |
| packed-i16/i16-mixed-0 | 12 / 10 | 567 / 13,434 | 564 / 14,839 |
| packed-i16/i16-small-1 | 14 / 12 | 678 / 13,373 | 924 / 14,764 |
| packed-i16/i16-wide-1 | 14 / 12 | 678 / 13,552 | 924 / 14,770 |
| packed-i16/i16-negative-1 | 14 / 12 | 678 / 13,548 | 924 / 15,150 |
| packed-i16/i16-mixed-1 | 14 / 12 | 678 / 13,373 | 924 / 14,764 |
| packed-i16/i16-small-128 | 269 / 267 | 957 / 15,010 | 1,710 / 26,173 |
| packed-i16/i16-wide-128 | 269 / 267 | 957 / 15,099 | 1,710 / 26,300 |
| packed-i16/i16-negative-128 | 269 / 267 | 957 / 15,010 | 1,710 / 26,173 |
| packed-i16/i16-mixed-128 | 269 / 267 | 957 / 15,010 | 1,710 / 26,173 |
| packed-i16/i16-small-1024 | 2,061 / 2,059 | 2,749 / 18,810 | 6,022 / 102,391 |
| packed-i16/i16-wide-1024 | 2,061 / 2,059 | 2,749 / 18,810 | 6,022 / 102,391 |
| packed-i16/i16-negative-1024 | 2,061 / 2,059 | 2,749 / 18,810 | 6,022 / 102,391 |
| packed-i16/i16-mixed-1024 | 2,061 / 2,059 | 2,749 / 18,810 | 6,022 / 102,391 |
| packed-i16/i16-small-16384 | 32,782 / 32,780 | 33,489 / 80,320 | 80,004 / 1,408,211 |
| packed-i16/i16-wide-16384 | 32,782 / 32,780 | 33,489 / 80,320 | 80,004 / 1,408,211 |
| packed-i16/i16-negative-16384 | 32,782 / 32,780 | 33,489 / 80,320 | 80,004 / 1,408,211 |
| packed-i16/i16-mixed-16384 | 32,782 / 32,780 | 33,489 / 80,320 | 80,004 / 1,408,211 |

Case counts below are descriptive, not weighted application scores. The control and 1/10/100
cohorts overlap and repeat structural widths; do not sum them into an overall win rate.

| Cohort / profile | Wire encode wins / cases | Wire decode wins / cases |
| --- | ---: | ---: |
| Standard (214) / opt-z | 159 / 214 | 151 / 214 |
| Standard (214) / opt-3 | 161 / 214 | 155 / 214 |
| Toko (10) / opt-z | 8 / 10 | 8 / 10 |
| Toko (10) / opt-3 | 9 / 10 | 10 / 10 |
| 1 record / opt-z | 1 / 1 | 1 / 1 |
| 1 record / opt-3 | 1 / 1 | 1 / 1 |
| 10 records / opt-z | 10 / 10 | 10 / 10 |
| 10 records / opt-3 | 10 / 10 | 10 / 10 |
| 100 records / opt-z | 100 / 100 | 100 / 100 |
| 100 records / opt-3 | 100 / 100 | 100 / 100 |
| Packed i16 (20) / opt-z | 20 / 20 | 20 / 20 |
| Packed i16 (20) / opt-3 | 20 / 20 | 20 / 20 |
| Ordinary i16 control (20) / opt-z | 8 / 20 | 8 / 20 |
| Ordinary i16 control (20) / opt-3 | 8 / 20 | 10 / 20 |
| 1 packed record, shared bounds / opt-z | 1 / 1 | 1 / 1 |
| 1 packed record, shared bounds / opt-3 | 1 / 1 | 1 / 1 |
| 10 packed records, shared bounds / opt-z | 10 / 10 | 10 / 10 |
| 10 packed records, shared bounds / opt-3 | 10 / 10 | 10 / 10 |
| 100 packed records, shared bounds / opt-z | 100 / 100 | 100 / 100 |
| 100 packed records, shared bounds / opt-3 | 100 / 100 | 100 / 100 |
| 1 packed record, varied bounds / opt-z | 1 / 1 | 1 / 1 |
| 1 packed record, varied bounds / opt-3 | 1 / 1 | 1 / 1 |
| 10 packed records, varied bounds / opt-z | 10 / 10 | 10 / 10 |
| 10 packed records, varied bounds / opt-3 | 10 / 10 | 10 / 10 |
| 100 packed records, varied bounds / opt-z | 100 / 100 | 100 / 100 |
| 100 packed records, varied bounds / opt-3 | 100 / 100 | 100 / 100 |

## Native timing, allocation and peak-heap examples

Opt-3 timing ranges are the minimum and maximum of two run medians, in µs.
Allocations and peak live bytes are separate instrumented native measurements;
each paired cell is Wire / Candid. Setup and input ownership are excluded.

| Case / operation | Wire µs | Candid µs | Allocations | Peak live bytes |
| --- | ---: | ---: | ---: | ---: |
| account-snapshot-128 / encode | 3.506–3.523 | 5.842–5.933 | 1 / 99 | 11,763 / 36,392 |
| account-snapshot-128 / decode | 14.391–15.920 | 35.824–37.304 | 265 / 1365 | 23,699 / 28,777 |
| blob-random-1048576 / encode | 17.108–17.956 | 768.205–783.618 | 1 / 17 | 1,048,590 / 2,098,807 |
| blob-random-1048576 / decode | 32.599–32.855 | 18.369–18.445 | 1 / 25 | 1,048,576 / 1,049,380 |
| generic-byte-vector-1048576 / encode | 1209.629–1210.976 | 763.178–764.337 | 1 / 16 | 1,048,590 / 2,098,807 |
| generic-byte-vector-1048576 / decode | 4763.452–4848.700 | 1998.235–1998.931 | 1 / 23 | 1,048,576 / 1,049,380 |
| i64-list-negative-4096 / encode | 9.550–9.900 | 1.465–1.596 | 1 / 16 | 4,109 / 67,189 |
| i64-list-negative-4096 / decode | 22.732–22.970 | 4.580–4.837 | 1 / 23 | 32,768 / 33,572 |

## Packed representation tradeoff

For 16,384 wide i16 values, packed Wire uses 32,782 frame bytes versus Candid's
32,780; ordinary Wire uses 49,166 in the standard fixture. For small i16 values,
ordinary Wire uses 16,398, so packed storage roughly doubles that payload. This
is a schema/size tradeoff, not a universal default-list optimization. The
[production qualification](packed-i16-production.md) documents consumer action.

In the matched twenty-case Wasm fixtures, including packed support increases
Wire total/code size by 2,540/2,403 bytes at opt-z and 3,991/3,879 at opt-3 versus
the ordinary-only control. These are whole-fixture inclusion costs, not per-type
costs or the savings from the pending visitor change. Its separate paired
[before/after study](packed-sharing-0611.md) owns that claim.

## Evidence and validation

All thirteen cohorts completed both profiles: 52 native timing runs, 52 separate
allocation/heap runs, 52 Wasm artifacts, 2,388 primary encode/decode pairs,
13,118 IC rows and 42,400 native/instrumented rows. Cross-mode input/frame hashes
and byte lengths match. Every IC row has three identical instruction samples;
all 52 artifact hashes and sizes were independently checked against the files.
The benchmark performs semantic roundtrips before timing, and the IC driver
checks encoded frames against its host oracle. This report adds no runtime code;
no additional full CI, installed consumer or big-endian execution is claimed.

- [Primary comparisons](evidence/full-comparison-0611/comparison.csv): every
  case/profile encode/decode pair, native run medians, IC instructions, frame
  bytes, allocations/reallocations, allocated bytes and peak live heap.
- [All labelled operations](evidence/full-comparison-0611/all-lanes.csv): includes
  bounded/borrowed/ordinary/packed lanes, reuse and rejection measurements.
- [Wasm artifacts](evidence/full-comparison-0611/artifacts.csv): all 52 total/code
  sizes and hashes. [Instruction wins](evidence/full-comparison-0611/instruction-wins.csv)
  preserves cohort/profile counts, including Candid wins.
- [Identity](evidence/full-comparison-0611/identity.json): compiler, source hashes,
  lock and machine identity.
- [Raw archive](evidence/full-comparison-0611/raw.json.gz): exact frozen source,
  runner, analysis scripts, per-run identities and every sample, including IC
  loop controls. It contains measurements and source, not the Wasm binaries.

Protocol unchanged. Versions, staging, commits, pushes and release remain
maintainer-owned.
