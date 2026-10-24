# Full Wire/Candid comparison: expanded 0.6.13 batch

The subsequent [hotspot optimization](hotspot-encoding-0613.md) supersedes the
ordinary i16 runtime figures below; this full-matrix snapshot remains frozen.

This study freezes the working tree based on `f470bcd9` (commit subject 0.6.13;
workspace packages still 0.6.12), including the incoming Shared Tooling 0.26.10
adoption. This benchmark batch changes only qualification fixtures/runners and
documentation. Production codecs, schemas, wire bytes, acceptance rules, limits,
canonical hashing and retries are unchanged.

Wire has smaller total and code-section Wasm in all **42 cohort/profile pairs**.
The generic packed-field and 1/10/100-record cases win all **840** primary IC
encode/decode comparisons against Candid. All ordinary record-scaling comparisons
also favor Wire. In the 298-case standard corpus, opt-3 Wire wins **232 encodes**
and **236 decodes**; this does not imply every ordinary container is fast.

At opt-3, 100 ordinary records use **424,179 total / 389,650 code bytes** for Wire
versus **2,081,252 / 2,012,071** for Candid. The 100 mixed-bulk records use
**655,798 / 620,658** versus **2,639,766 / 2,570,257**. These are whole fixtures,
not projected savings in an installed application.

Weak cases remain explicit: standard wide Vec<i16> encode costs **49.94x Candid**
at opt-3; generic 1 MiB Vec<u8> encode **37.23x**; a 4,096-item Vec<i8> encode
**8.61x**; owned byte-oriented 1 MiB blob decode **2.37x**. At opt-z, ordinary
wide-u16 encode reaches **123.85x**. Packed fields solve the qualified 16/32/64-bit
integer/hash representations; they do not alter ordinary vectors, and Packed<i8>
is not currently supported. Byte-oriented Serde fields avoid the generic Vec<u8>
encoding cost. Scalar/list usability, wire-byte compactness and instruction cost
remain separate choices.

## Coverage

Twenty-one cohorts contain 1,101 case entries across two optimization profiles:
298 standard cases, ten Toko-shaped cases, 1/10/100 ordinary derived records,
twenty bounded packed-i16 cases and twenty ordinary controls, 1/10/100 packed
records with shared/varied bounds, 99 generic packed-field cases and 99 ordinary
controls, plus 1/10/100 mixed bulk records with matching ordinary controls.
Cohorts overlap; case entries are not unique Rust types or a traffic distribution.
The 100-record fixtures contain nominal types with repeated structural widths.

The expanded standard corpus adds signed minima, 128-bit integer lists, ordered
sets, queues, linked lists, nested maps, generic blob/optional/result/nested pages,
fixed arrays, boxed slices, Duration, tuple structs and generic enum variants.
Generic packed fields now cover negative/mixed signed and mixed unsigned
values, plus envelopes combining packed i16/u64/hash fields with headers,
optional cursors and ordinary events. NaN/infinity, nondeterministic hash-container
iteration, arbitrary custom serializers and every possible Serde shape are not
claimed as covered. Container timing/roundtrip results do not establish automatic
TypeScript metadata support for each Rust collection. Principal controls use their explicit benchmark declaration;
they do not change consumer-owned principal/Candid boundaries.

Wire uses its one production Serde/Postcard codec. Primary rows use owned values;
scaling/bulk cohorts use their stated bounded or packed field types. Packed is
an explicit coordinated schema choice. Ordinary vectors retain varints. Bounded
PackedI16 retains its field limit; generic Packed relies on runtime limits.
CSV labels such as `wire-native`, `wire-bounded` and `wire-packed-i16` identify
fixture representations within that same codec; they are not production codec
modes. A cohort's Wire Wasm includes all its enabled Wire lanes, not just the
primary table row. Borrowed/reuse/rejection lanes are labelled separately. Hash controls retain
Candid's nested hash-array schema; the standard corpus separately includes flat
byte-oriented manifests and blobs. There is no claim that Candid could not select
another representation. Wire budgets and Candid quotas are different policies.

## Method

Rust 1.99.1 / LLVM 23.1.1, Linux x86_64 and wasm32-unknown-unknown, same frozen
source and lock across codecs. Candid 0.10.39, Postcard 1.1.3, Serde 1.0.229,
IC Platform 0.6.12 and IC Host/Testkit 0.66.3. A concurrent root-lock update to
Platform 0.6.14 was preserved after the run. Its Candid/ic0 source directories
match 0.6.12 byte-for-byte, but these are frozen-0.6.12 measurements, not a
0.6.14 artifact qualification. Both profiles use LTO, one codegen
unit, panic abort and stripped symbols. Evaluation uses opt-z; benchmark-speed
uses opt-3. Every artifact is a complete codec/fixture/dispatch Wasm, not a
symbol estimate or an installed application's footprint. Applications retaining
required external Candid endpoints still link the Candid code those endpoints
need; isolated fixture differences are not net application migration savings.

Native timings use two sequential runs/profile, eleven samples/run and calibrated
batches of at least 2 ms (1,048,576-call cap). Result destruction and loop/dynamic
call overhead are included. Allocation requests/bytes and peak live heap use
separate instrumented native builds, with one warmed operation and input/setup
excluded. They do not measure Wasm heap. Native timings are unpinned warm-cache
microbenchmarks; run-median ranges are not confidence intervals or request tails.
No other task-owned build/benchmark runs concurrently with timed native batches;
unrelated host activity is uncontrolled. Build elapsed times are cached provenance,
not a fair compiler-performance comparison.

IC measurements use the prepared PocketIC 16.1.0 server, admitted and owned by
IC Testkit. Counter 0 wraps warm encode/decode/reuse/rejection loops; three queries
per case report four repetitions through 64 KiB and one above. Setup, input
construction, transport and semantic roundtrip equality checks are outside the
measured loops; decode success/failure assertions, result destruction and loop
overhead are included, without subtraction.
Encoded frames are checked against the native oracle. All frame sizes include
Wire's 11-byte method/direction envelope or Candid's one-argument DIDL envelope.
No gzip size is measured or used as an objective.

## Reproduction

Prepare the pinned compiler, target, locked dependency cache and an explicit
PocketIC 16.1.0 binary. Run from a source checkout:

```sh
POCKET_IC_BIN=/path/to/pocket-ic RUNS=2 SAMPLES=11 BATCH_MS=2 \\
  bash scripts/ci/evaluate-codec-matrix.sh NEW_OUTPUT
```

The full runner freezes source/lock and runs every cohort serially. It requires
a fresh output directory; old evidence is retained. The archived source, cohort
plan, analysis scripts and all raw samples permit regeneration. When extracting
the source, supply the original Git source identity or run the orchestrator from
a Git checkout; the timing runner records Git identity as well as content hashes.
Keep compiler, target, features, profiles, exact inputs and limits identical for
paired comparisons. Adding cases changes whole-fixture artifacts, so this is a
within-run Wire/Candid comparison, not a size-regression comparison with earlier
smaller corpora. No sibling repository or installed application is qualified.
## Uncompressed Wasm sizes

| Cohort / profile | Wire total / code bytes | Candid total / code bytes | Total reduction |
| --- | ---: | ---: | ---: |
| standard / opt-z | 402,893 / 362,784 | 933,712 / 869,518 | 56.85% |
| standard / opt-3 | 611,388 / 556,574 | 1,386,033 / 1,306,893 | 55.89% |
| toko / opt-z | 87,705 / 76,478 | 373,547 / 339,434 | 76.52% |
| toko / opt-3 | 114,784 / 104,224 | 592,136 / 559,369 | 80.62% |
| derive-1 / opt-z | 55,598 / 44,581 | 299,112 / 266,307 | 81.41% |
| derive-1 / opt-3 | 77,934 / 51,183 | 433,489 / 385,612 | 82.02% |
| derive-10 / opt-z | 107,540 / 95,664 | 434,585 / 399,350 | 75.25% |
| derive-10 / opt-3 | 149,310 / 121,681 | 678,730 / 628,262 | 78.00% |
| derive-100 / opt-z | 324,352 / 305,537 | 1,417,102 / 1,363,091 | 77.11% |
| derive-100 / opt-3 | 424,179 / 389,650 | 2,081,252 / 2,012,071 | 79.62% |
| packed-i16 / opt-z | 44,602 / 35,556 | 216,364 / 190,714 | 79.39% |
| packed-i16 / opt-3 | 49,542 / 40,776 | 284,138 / 259,050 | 82.56% |
| packed-control / opt-z | 42,065 / 33,154 | 216,356 / 190,714 | 80.56% |
| packed-control / opt-3 | 45,551 / 36,897 | 284,146 / 259,050 | 83.97% |
| packed-shared-1 / opt-z | 63,229 / 51,335 | 304,929 / 271,325 | 79.26% |
| packed-shared-1 / opt-3 | 93,986 / 66,389 | 449,222 / 400,570 | 79.08% |
| packed-shared-10 / opt-z | 85,730 / 73,104 | 398,612 / 363,102 | 78.49% |
| packed-shared-10 / opt-3 | 122,108 / 93,830 | 656,974 / 606,354 | 81.41% |
| packed-shared-100 / opt-z | 263,938 / 244,556 | 1,283,507 / 1,229,624 | 79.44% |
| packed-shared-100 / opt-3 | 393,797 / 358,751 | 2,098,605 / 2,029,665 | 81.24% |
| packed-varied-1 / opt-z | 63,227 / 51,333 | 304,929 / 271,325 | 79.27% |
| packed-varied-1 / opt-3 | 93,985 / 66,388 | 449,222 / 400,570 | 79.08% |
| packed-varied-10 / opt-z | 99,218 / 86,537 | 398,612 / 363,102 | 75.11% |
| packed-varied-10 / opt-3 | 143,817 / 115,482 | 656,974 / 606,354 | 78.11% |
| packed-varied-100 / opt-z | 288,154 / 268,633 | 1,283,507 / 1,229,624 | 77.55% |
| packed-varied-100 / opt-3 | 430,402 / 395,236 | 2,098,605 / 2,029,665 | 79.49% |
| bulk / opt-z | 100,943 / 89,805 | 430,600 / 395,717 | 76.56% |
| bulk / opt-3 | 124,423 / 113,996 | 659,701 / 626,144 | 81.14% |
| bulk-control / opt-z | 108,617 / 97,442 | 430,600 / 395,717 | 74.78% |
| bulk-control / opt-3 | 140,144 / 129,676 | 659,701 / 626,144 | 78.76% |
| bulk-1 / opt-z | 73,000 / 61,028 | 365,817 / 331,716 | 80.04% |
| bulk-1 / opt-3 | 104,979 / 77,291 | 531,930 / 482,712 | 80.26% |
| bulk-10 / opt-z | 113,519 / 100,796 | 502,761 / 466,721 | 77.42% |
| bulk-10 / opt-3 | 155,051 / 126,687 | 776,659 / 725,476 | 80.04% |
| bulk-100 / opt-z | 460,927 / 441,448 | 1,794,102 / 1,739,683 | 74.31% |
| bulk-100 / opt-3 | 655,798 / 620,658 | 2,639,766 / 2,570,257 | 75.16% |
| bulk-control-1 / opt-z | 81,429 / 69,410 | 365,817 / 331,716 | 77.74% |
| bulk-control-1 / opt-3 | 118,672 / 90,975 | 531,930 / 482,712 | 77.69% |
| bulk-control-10 / opt-z | 124,923 / 112,144 | 502,761 / 466,721 | 75.15% |
| bulk-control-10 / opt-3 | 174,280 / 145,896 | 776,659 / 725,476 | 77.56% |
| bulk-control-100 / opt-z | 472,331 / 452,796 | 1,794,102 / 1,739,683 | 73.67% |
| bulk-control-100 / opt-3 | 675,032 / 639,867 | 2,639,766 / 2,570,257 | 74.43% |

## Representative IC instructions and frame bytes

Opt-3 owned values, with explicitly labelled packed fields. Each cell is Wire / Candid.

| Cohort / case | Frame bytes | Encode instructions | Decode instructions |
| --- | ---: | ---: | ---: |
| standard / unit | 11 / 7 | 385 / 5,958 | 208 / 6,309 |
| standard / bool-false | 12 / 8 | 622 / 6,776 | 230 / 6,288 |
| standard / scalar-record | 48 / 100 | 2,423 / 52,664 | 2,890 / 37,610 |
| standard / tuple | 20 / 30 | 1,310 / 23,679 | 1,756 / 22,098 |
| standard / option-some | 14 / 18 | 858 / 14,442 | 612 / 14,558 |
| standard / result-error | 30 / 37 | 1,253 / 23,478 | 1,792 / 21,915 |
| standard / variant-record | 158 / 215 | 2,340 / 68,371 | 3,738 / 63,756 |
| standard / tree-depth-4 | 73 / 302 | 21,470 / 39,126 | 31,282 / 198,626 |
| standard / ordered-set-128 | 141 / 1,035 | 27,819 / 28,538 | 116,258 / 118,932 |
| standard / deque-128 | 141 / 1,035 | 15,549 / 25,409 | 27,262 / 32,270 |
| standard / linked-list-128 | 204 / 1,035 | 27,356 / 24,142 | 82,788 / 77,260 |
| standard / nested-map-128 | 2,061 / 3,093 | 167,479 / 82,422 | 380,636 / 951,730 |
| standard / page-blob-128 | 4,245 / 4,273 | 57,553 / 91,388 | 137,588 / 278,416 |
| standard / page-option-result-128 | 572 / 905 | 71,008 / 85,570 | 125,472 / 514,244 |
| standard / nested-page-128 | 1,173 / 2,373 | 144,904 / 114,397 | 292,942 / 1,072,614 |
| standard / fixed-u64-32 | 331 / 266 | 13,089 / 14,609 | 24,984 / 19,844 |
| standard / boxed-slice-u64 | 25 / 34 | 1,569 / 16,743 | 2,378 / 15,466 |
| standard / duration | 21 / 33 | 1,132 / 22,840 | 1,252 / 20,466 |
| standard / f32-list-1024 | 4,109 / 4,107 | 49,965 / 23,014 | 221,396 / 99,432 |
| standard / f64-list-1024 | 8,205 / 8,203 | 49,967 / 31,206 | 108,460 / 92,252 |
| standard / bool-list-1024 | 1,037 / 1,035 | 49,953 / 16,945 | 101,798 / 129,194 |
| standard / u128-extra-list-wide-4096 | 77,837 / 77,835 | 4,088,808 / 3,856,944 | 5,121,637 / 10,137,270 |
| standard / i128-extra-list-wide-4096 | 77,837 / 77,835 | 4,219,880 / 7,203,376 | 5,260,901 / 9,895,620 |
| standard / i8-extra-list-wide-4096 | 4,109 / 4,107 | 197,409 / 22,931 | 269,910 / 322,746 |
| standard / u8-extra-list-wide-4096 | 4,109 / 4,107 | 308,073 / 22,931 | 527,780 / 482,510 |
| standard / i16-list-wide-16384 | 49,166 / 32,780 | 4,014,930 / 80,397 | 5,919,238 / 1,408,452 |
| standard / i64-list-wide-16384 | 163,854 / 131,084 | 6,931,451 / 277,189 | 12,194,488 / 1,982,071 |
| standard / generic-byte-vector-1048576 | 1,048,590 / 1,048,588 | 78,644,279 / 2,112,114 | 134,747,920 / 119,553,417 |
| standard / blob-random-32 | 44 / 42 | 948 / 17,337 | 1,046 / 15,742 |
| standard / blob-random-4096 | 4,109 / 4,107 | 5,028 / 26,050 | 11,026 / 20,706 |
| standard / blob-random-1048576 | 1,048,590 / 1,048,588 | 1,049,704 / 2,115,234 | 2,524,535 / 1,065,374 |
| standard / packed-manifest-1024 | 32,831 / 32,858 | 35,394 / 112,282 | 82,814 / 73,196 |
| bulk / i16-wide-16384 | 32,782 / 32,780 | 33,823 / 80,505 | 80,510 / 1,408,288 |
| bulk / i16-mixed-16384 | 32,782 / 32,780 | 33,823 / 80,505 | 80,510 / 1,408,288 |
| bulk / u16-wide-16384 | 32,782 / 32,780 | 33,823 / 80,505 | 80,378 / 1,375,520 |
| bulk / i32-wide-16384 | 65,550 / 65,548 | 66,771 / 146,225 | 159,393 / 1,408,449 |
| bulk / u32-wide-16384 | 65,550 / 65,548 | 66,771 / 146,257 | 159,393 / 1,375,683 |
| bulk / i64-wide-16384 | 131,086 / 131,084 | 132,307 / 277,297 | 317,089 / 1,277,367 |
| bulk / u64-wide-16384 | 131,086 / 131,084 | 132,307 / 277,329 | 317,222 / 1,392,055 |
| bulk / hashes-1024 | 32,782 / 33,805 | 33,685 / 129,242 | 80,508 / 4,208,984 |
| bulk / manifest-1024 | 32,831 / 33,883 | 35,455 / 156,259 | 83,708 / 4,233,816 |
| bulk / mixed-envelope-1 | 223 / 327 | 6,563 / 159,390 | 14,660 / 217,558 |
| bulk / mixed-envelope-16 | 1,906 / 2,024 | 8,305 / 164,174 | 20,212 / 299,980 |
| bulk / mixed-envelope-1024 | 114,805 / 115,929 | 121,454 / 455,946 | 291,636 / 5,784,059 |

## Instruction win counts

Each count is per-cohort, per-profile, per-operation. Overlapping controls are not unique types.

| Cohort / profile | Cases | Wire encode wins | Wire decode wins | Wire smaller frames |
| --- | ---: | ---: | ---: | ---: |
| standard / opt-z | 298 | 228 | 221 | 118 |
| standard / opt-3 | 298 | 232 | 236 | 118 |
| toko / opt-z | 10 | 8 | 8 | 10 |
| toko / opt-3 | 10 | 9 | 10 | 10 |
| derive-1 / opt-z | 1 | 1 | 1 | 1 |
| derive-1 / opt-3 | 1 | 1 | 1 | 1 |
| derive-10 / opt-z | 10 | 10 | 10 | 10 |
| derive-10 / opt-3 | 10 | 10 | 10 | 10 |
| derive-100 / opt-z | 100 | 100 | 100 | 100 |
| derive-100 / opt-3 | 100 | 100 | 100 | 100 |
| packed-i16 / opt-z | 20 | 20 | 20 | 0 |
| packed-i16 / opt-3 | 20 | 20 | 20 | 0 |
| packed-control / opt-z | 20 | 8 | 8 | 6 |
| packed-control / opt-3 | 20 | 8 | 10 | 6 |
| packed-shared-1 / opt-z | 1 | 1 | 1 | 1 |
| packed-shared-1 / opt-3 | 1 | 1 | 1 | 1 |
| packed-shared-10 / opt-z | 10 | 10 | 10 | 10 |
| packed-shared-10 / opt-3 | 10 | 10 | 10 | 10 |
| packed-shared-100 / opt-z | 100 | 100 | 100 | 100 |
| packed-shared-100 / opt-3 | 100 | 100 | 100 | 100 |
| packed-varied-1 / opt-z | 1 | 1 | 1 | 1 |
| packed-varied-1 / opt-3 | 1 | 1 | 1 | 1 |
| packed-varied-10 / opt-z | 10 | 10 | 10 | 10 |
| packed-varied-10 / opt-3 | 10 | 10 | 10 | 10 |
| packed-varied-100 / opt-z | 100 | 100 | 100 | 100 |
| packed-varied-100 / opt-3 | 100 | 100 | 100 | 100 |
| bulk / opt-z | 99 | 99 | 99 | 11 |
| bulk / opt-3 | 99 | 99 | 99 | 11 |
| bulk-control / opt-z | 99 | 41 | 39 | 49 |
| bulk-control / opt-3 | 99 | 43 | 44 | 49 |
| bulk-1 / opt-z | 1 | 1 | 1 | 1 |
| bulk-1 / opt-3 | 1 | 1 | 1 | 1 |
| bulk-10 / opt-z | 10 | 10 | 10 | 10 |
| bulk-10 / opt-3 | 10 | 10 | 10 | 10 |
| bulk-100 / opt-z | 100 | 100 | 100 | 100 |
| bulk-100 / opt-3 | 100 | 100 | 100 | 100 |
| bulk-control-1 / opt-z | 1 | 1 | 1 | 1 |
| bulk-control-1 / opt-3 | 1 | 1 | 1 | 1 |
| bulk-control-10 / opt-z | 10 | 10 | 10 | 10 |
| bulk-control-10 / opt-3 | 10 | 10 | 10 | 10 |
| bulk-control-100 / opt-z | 100 | 100 | 100 | 100 |
| bulk-control-100 / opt-3 | 100 | 100 | 100 | 100 |

## Largest Candid instruction advantages

Opt-3; all primary lanes are eligible. These include ordinary representation controls.

| Cohort / case / operation | Wire | Candid | Wire / Candid cost |
| --- | ---: | ---: | ---: |
| bulk-control / i16-wide-16384 / encode | 4,473,749 | 80,505 | 55.57x |
| bulk-control / i16-mixed-16384 / encode | 4,206,174 | 80,505 | 52.25x |
| bulk-control / u16-wide-16384 / encode | 4,113,267 | 80,505 | 51.09x |
| packed-control / i16-wide-16384 / encode | 4,014,993 | 80,320 | 49.99x |
| standard / i16-list-wide-16384 / encode | 4,014,930 | 80,397 | 49.94x |
| bulk-control / i16-small-16384 / encode | 3,916,693 | 80,536 | 48.63x |
| bulk-control / i16-negative-16384 / encode | 3,916,693 | 80,536 | 48.63x |
| packed-control / i16-mixed-16384 / encode | 3,742,969 | 80,320 | 46.60x |
| packed-control / i16-small-16384 / encode | 3,457,985 | 80,320 | 43.05x |
| packed-control / i16-negative-16384 / encode | 3,457,985 | 80,320 | 43.05x |
| standard / i16-list-small-16384 / encode | 3,457,874 | 80,484 | 42.96x |
| bulk-control / u16-mixed-16384 / encode | 3,091,921 | 80,505 | 38.41x |
| standard / generic-byte-vector-1048576 / encode | 78,644,279 | 2,112,114 | 37.23x |
| standard / u32-list-wide-16384 / encode | 4,883,545 | 146,117 | 33.42x |
| bulk-control / i32-wide-16384 / encode | 4,637,775 | 146,225 | 31.72x |

## Explicit packed fields versus ordinary Wire controls

These are current field representations in the same production codec, with equal logical inputs. Selecting packed fields changes schemas/bytes and requires coordinated consumers.

| Opt-3 input | Ordinary / packed Wire frame bytes | Ordinary / packed encode instructions | Ordinary / packed decode instructions |
| --- | ---: | ---: | ---: |
| i16-wide-16384 | 49,166 / 32,782 | 4,473,749 / 33,823 | 7,295,302 / 80,510 |
| u64-small-16384 | 16,398 / 131,086 | 1,852,604 / 132,307 | 3,864,140 / 317,222 |
| hashes-1024 | 32,781 / 32,782 | 2,504,594 / 33,685 | 5,034,818 / 80,508 |
| mixed-envelope-1 | 247 / 223 | 15,202 / 6,563 | 29,080 / 14,660 |
| mixed-envelope-1024 | 139,378 / 114,805 | 9,192,856 / 121,454 | 15,187,755 / 291,636 |

| Wire cohort / profile | Ordinary total / code bytes | Packed total / code bytes |
| --- | ---: | ---: |
| bulk / opt-z | 108,617 / 97,442 | 100,943 / 89,805 |
| bulk / opt-3 | 140,144 / 129,676 | 124,423 / 113,996 |
| bulk-1 / opt-z | 81,429 / 69,410 | 73,000 / 61,028 |
| bulk-1 / opt-3 | 118,672 / 90,975 | 104,979 / 77,291 |
| bulk-10 / opt-z | 124,923 / 112,144 | 113,519 / 100,796 |
| bulk-10 / opt-3 | 174,280 / 145,896 | 155,051 / 126,687 |
| bulk-100 / opt-z | 472,331 / 452,796 | 460,927 / 441,448 |
| bulk-100 / opt-3 | 675,032 / 639,867 | 655,798 / 620,658 |


## Native timing and memory examples

Native times below span the medians of the two opt-3 runs; these are Rust host ns/op, not IC instructions. Full sample distributions are archived.

| Cohort / case / operation | Wire ns (run-median range) | Candid ns (run-median range) |
| --- | ---: | ---: |
| standard / scalar-record / encode | 67.0–71.9 | 1,375.1–1,423.6 |
| standard / scalar-record / decode | 69.2–71.7 | 946.9–958.6 |
| standard / blob-random-1048576 / encode | 17,071.5–17,104.7 | 716,909.8–781,569.8 |
| standard / blob-random-1048576 / decode | 33,533.4–33,630.9 | 17,986.3–18,578.7 |
| standard / i128-extra-list-wide-4096 / encode | 66,344.4–66,679.9 | 170,797.3–173,646.6 |
| standard / i128-extra-list-wide-4096 / decode | 85,056.0–85,409.0 | 136,975.9–138,615.2 |
| bulk / mixed-envelope-1024 / encode | 1,881.7–2,073.0 | 62,577.4–72,765.0 |
| bulk / mixed-envelope-1024 / decode | 3,825.4–4,080.7 | 128,586.6–239,291.8 |

Owned decode, opt-3 native instrumentation; each cell is Wire / Candid:

| Cohort / case | Fresh allocations | Reallocations | Peak live bytes |
| --- | ---: | ---: | ---: |
| standard / blob-random-1048576 | 1 / 25 | 0 / 0 | 1,048,576 / 1,049,380 |
| standard / page-blob-128 | 130 / 443 | 5 / 0 | 7,172 / 8,980 |
| standard / i128-extra-list-wide-4096 | 1 / 23 | 10 / 0 | 65,536 / 66,340 |
| bulk / mixed-envelope-1024 | 10 / 169 | 0 / 4 | 115,074 / 119,576 |
| bulk-100 / record_099 | 8 / 146 | 0 / 6 | 501 / 6,344 |

Fresh allocation counts, reallocations, net reallocated bytes, output capacity,
DHAT allocation traffic and peak live heap are separate measurements. In particular,
wide i128 vectors grow geometrically in Wire: fewer fresh allocations do not imply
less reallocation traffic. Native heap figures do not establish Wasm heap usage.

## Evidence and validation

All 21 cohorts completed: **84 Wasm artifacts**, **4,404 primary encode/decode
pairs**, **23,366 IC rows** and **75,328 native/instrumented rows**.
Every IC row has three identical instruction samples. Input/frame hashes agree
across native timing, allocation/heap, IC and optimization profiles. Matching
ordinary/packed controls preserve Candid input/frame identities. All 84 artifact
hashes, uncompressed sizes and code-section sizes were independently checked.

Expanded standard and generic-bulk tests pass semantic roundtrip, unique case IDs,
selected-case parity and damaged/trailing/truncated-frame rejection. Changed
fixtures pass strict Clippy with IC features and the Rust 1.91 Wasm check.
Separate standard/bulk roundtrip and damaged-frame checks also pass against the
current Platform 0.6.14 lock; performance and artifact figures remain frozen-0.6.12.
Runner portability checks use substituted Cargo for spaces, adverse CDPATH,
missing output parents, relative target paths and overwrite refusal. The frozen
initial orchestrator is archived. Editing that outer script during execution
caused a tail-parsing error after all 21 cohort commands completed; its exit was 2.
Frozen measurement helpers, codec inputs and loops were unchanged, and every
individual dataset completed and passed the checks above. The corrected runner
passes Bash syntax and all 21 dispatches/final exit with substituted measurements
in a disposable Git fixture; there was no second full measurement run. No broad CI, installed consumer,
real big-endian or TypeScript-vs-Candid performance claim is made.

- [Primary comparisons](evidence/full-comparison-0613/comparison.csv): all inputs,
  both profiles, frame bytes, IC instructions, native medians and memory metrics.
- [Every labelled lane/operation](evidence/full-comparison-0613/all-lanes.csv):
  includes bounded/borrowed/packed, reuse and rejection results. Raw IC loop
  controls remain in the archive; they are not subtracted from operation costs.
- [All Wasm artifacts](evidence/full-comparison-0613/artifacts.csv),
  [instruction win counts](evidence/full-comparison-0613/instruction-wins.csv) and
  [source/compiler identity](evidence/full-comparison-0613/identity.json).
- [Raw archive](evidence/full-comparison-0613/raw.json.gz): frozen source/lock,
  cohort plan, executed runner, analysis scripts, every sample and run identity,
  plus artifact metadata. Wasm binaries remain in the local run directory;
  reproduction requires rebuilding them. Compression is for evidence storage only.

Protocol unchanged. Consumer storage, endpoint semantics and adoption remain
consumer-owned; publication and release remain maintainer-owned.
