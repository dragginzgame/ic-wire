# Public Packed fields: promotion qualification

`ic_wire::Packed<T>` retains the selected research prototype's IC instruction
counts, wire bytes, allocation counts, peak heap and uncompressed Wasm sizes.
All 716 paired encode/decode comparisons favor Wire over Candid: 68 field/manifest
cases and 1/10/100-record cohorts, in both optimization profiles. This is the
packed-field matrix, not a claim that ordinary vectors have been optimized.
The [consumer guide](packed-fields.md) documents supported types and migration.

## Wasm and IC results

Every total/code-section size below exactly matches the prototype (0% change).
These are complete fixture artifacts, not estimates of an installed application.
Candid uses ordinary integer vectors and nested hash arrays; this study does not
substitute a newly flattened Candid hash blob.

| Cohort | Profile | Wire total / code bytes | Candid total / code bytes |
| --- | --- | ---: | ---: |
| fields | opt-z | 69,599 / 59,154 | 376,381 / 342,985 |
| fields | opt-3 | 80,985 / 71,143 | 577,487 / 545,426 |
| scale-1 | opt-z | 72,995 / 61,023 | 365,689 / 331,589 |
| scale-1 | opt-3 | 104,979 / 77,291 | 531,930 / 482,712 |
| scale-10 | opt-z | 113,514 / 100,791 | 502,633 / 466,594 |
| scale-10 | opt-3 | 155,051 / 126,687 | 776,659 / 725,476 |
| scale-100 | opt-z | 460,922 / 441,443 | 1,793,974 / 1,739,556 |
| scale-100 | opt-3 | 655,798 / 620,658 | 2,639,766 / 2,570,257 |

At opt-3, packed fields retain the measured 16,775-byte saving versus ordinary
vectors in the field fixture and 19,234-byte saving with 100 records. These are
representation savings, not additional savings from making the type public.

| Opt-3 input | Wire encode / decode instructions | Candid encode / decode instructions | Wire / Candid frame bytes |
| --- | ---: | ---: | ---: |
| u32-small-0 | 654.25 / 572.50 | 13,577.25 / 14,987.50 | 12 / 10 |
| u32-small-1 | 767.25 / 1,098.50 | 12,897.00 / 15,097.50 | 16 / 14 |
| i16-wide-16384 | 33,689.25 / 80,362.50 | 80,492.25 / 1,408,282.50 | 32,782 / 32,780 |
| i64-wide-16384 | 132,173.00 / 317,074.00 | 277,284.00 / 1,277,361.00 | 131,086 / 131,084 |
| manifest-1024 | 35,175.25 / 82,935.50 | 156,158.25 / 4,233,662.50 | 32,831 / 33,883 |

All 3,580 IC rows have three identical samples. The 2,864 matched operation lanes
retain exact input/frame hashes, IC counts, allocation/reallocation counts and
peak heap. Candid controls also match. Wasm hashes differ; sizes matching does not
mean byte-identical artifacts. All 16 recorded artifact hashes were verified.

## Native time and regressions

Native measurements are Linux x86_64 host time, not IC instructions. Two unpinned
runs per profile use eleven calibrated samples each. Keep both run medians:
JIT/OS/compiler placement and host load prevent interpreting every difference as
a stable optimization. No compiler or benchmark was deliberately run alongside
these native timing samples.

| Opt-3 input / operation | Prototype ns, runs 1 / 2 | Public ns, runs 1 / 2 | Change %, runs 1 / 2 |
| --- | ---: | ---: | ---: |
| u32-small-0 / encode | 18.61 / 18.44 | 18.32 / 17.79 | -1.60 / -3.54 |
| u32-small-0 / decode | 57.52 / 57.50 | 57.05 / 58.61 | -0.82 / +1.92 |
| u32-small-1 / encode | 20.23 / 18.74 | 18.44 / 18.57 | -8.84 / -0.95 |
| u32-small-1 / decode | 31.26 / 32.71 | 30.15 / 31.07 | -3.55 / -5.00 |
| i16-wide-16384 / encode | 539.09 / 534.24 | 554.31 / 535.09 | +2.82 / +0.16 |
| i16-wide-16384 / decode | 976.39 / 971.38 | 1,043.02 / 976.09 | +6.82 / +0.48 |
| i64-wide-16384 / encode | 2,056.34 / 1,987.65 | 1,960.47 / 1,999.96 | -4.66 / +0.62 |
| i64-wide-16384 / decode | 3,798.55 / 3,772.75 | 3,767.02 / 3,784.03 | -0.83 / +0.30 |
| manifest-1024 / encode | 627.47 / 564.01 | 560.75 / 550.09 | -10.63 / -2.47 |
| manifest-1024 / decode | 1,139.95 / 1,164.00 | 1,011.43 / 1,014.02 | -11.27 / -12.88 |

A repeatable native opt-z regression remains: 16,384 small u16s decode in
7,614/7,554 ns, versus 4,873/4,763 ns (mean of run medians +57.40%). Wide u16s
similarly regress 56.16%. Both remain faster than Candid's roughly 29–33 µs;
IC counts and Wasm sizes are unchanged. Do not claim universal native parity.
The largest opt-3 mean regression is u64-wide-1024 encode: 103/103 ns becomes
107/255 ns (+75.72%); decode changes 199/186 to 198/470 ns (+73.44%). The second
run moved sharply, so this evidence does not establish a stable slowdown there.
All per-run results and regressions remain in the CSV.

The original representation tradeoffs also remain: small u64 payloads can grow
eightfold, and empty u32 decode costs 39.13% more IC instructions than ordinary
Wire. Promotion adds no further IC regression.

## TypeScript host runtime

The existing bounded packed-i16 schema now shares the generic typed-array path.
Node 24.21.0, two serial runs with reversed before/after order, 1,000 warmup calls,
eleven samples and >=3 ms calibrated batches measure identical frames at lengths
0, 1, 128 and 16,384. These are JavaScript host timings, not Wasm or IC savings.
The empty path avoids constructing an unnecessary byte-copy/view pair.

| i16 length / operation | Before ns, runs 1 / 2 | After ns, runs 1 / 2 | Change %, runs 1 / 2 |
| --- | ---: | ---: | ---: |
| 0 / encode | 642.3 / 341.6 | 396.0 / 386.4 | -38.3 / 13.1 |
| 0 / decode | 257.8 / 260.7 | 193.8 / 189.4 | -24.8 / -27.4 |
| 1 / encode | 412.4 / 365.8 | 443.7 / 423.3 | 7.6 / 15.7 |
| 1 / decode | 216.5 / 207.1 | 351.0 / 361.6 | 62.1 / 74.6 |
| 128 / encode | 1,129.6 / 1,216.3 | 983.7 / 975.3 | -12.9 / -19.8 |
| 128 / decode | 628.1 / 776.3 | 495.0 / 668.2 | -21.2 / -13.9 |
| 16384 / encode | 48,711.9 / 48,954.7 | 4,527.8 / 6,645.7 | -90.7 / -86.4 |
| 16384 / decode | 12,422.0 / 12,134.4 | 3,446.1 / 3,483.1 | -72.3 / -71.3 |

Tiny nonempty arrays regress: one-word decode adds about 135–155 ns. Large encode
improves 86–91% and decode 71–72%. Empty encode is noisy; no uniform speedup is
claimed. This benchmark checks the changed existing schema; generic word widths
are qualified by cross-language conformance, not separate JavaScript speed claims.

## Qualification and reproduction

Before: archived `runtime_source` and `*-runtime` cohorts in the
[research evidence](evidence/bulk-fields-0611/raw.json.gz), based on `31bcef2` plus
pending 0.6.11 work. After: the promoted source captured in this study's archive.
The final Rust packed module exactly matches the measured source. Later benchmark
import gating and documentation/test changes do not change measured runtime code.

Both sides use Rust 1.99.1 / LLVM 23.1.1, Serde 1.0.229, Postcard 1.1.3, Candid
0.10.39, Platform 0.6.10 and Testkit 0.66.0, with the identical archived Cargo.lock.
Profiles are opt-z/opt-3, LTO, one codegen unit, abort and stripped Wasm. PocketIC
16.1.0 reports three samples, four repetitions through 64 KiB and one above it;
loop overhead is included, not subtracted. Input, compiler, host, binary, source,
lock and codec identities are captured per run. Allocation and heap counters
come from separate instrumented builds. Gzip is only archive storage.

Extract `source` and `runner` from the raw archive to reproduce with isolated
build directories. `RUNS=2 SAMPLES=11 BATCH_MS=2`, a prepared PocketIC binary, and
`CORPUS_FEATURE=bulk`, `bulk-scale,derive-scale-1`, `bulk-scale,derive-scale-10`,
then `bulk-scale` select the four cohorts through `scripts/ci/evaluate-codecs.sh`.
The archive also includes the exact TypeScript before/after sources, benchmark,
compiled JavaScript and all samples; compile with the repository's pinned tsc,
ES2022, strict/exact optional properties/no unchecked indexed access, then run
`node bench.mjs`. Analysis scripts are archived with the evidence.

The first public candidate omitted inline hints on concrete word readers. Large
opt-3 i16 decode rose from 80,362.5 to 440,699.5 instructions (5.48x), and field
Wasm grew 755 bytes. Adding ordinary inline hints restored every measured IC
count and Wasm size; that rejected source and its measurements are archived.
A preliminary trial with newer Platform/Testkit pins was discarded, not mixed
into this matched comparison.

Focused current-checkout checks use the newer incoming Platform 0.6.11/Testkit
0.66.2 dependencies. Native admission tests, default/TypeScript Rustdoc, packed
and ordinary benchmark roundtrips, strict Clippy, Rust 1.91 native/Wasm checks and
Rust/TypeScript conformance pass. Conformance covers 157 vectors, 32,156 mutations,
wide signed/unsigned extrema, hash lengths, unaligned inputs, owned output and
canonical rejection. Native tests cover all 65,536 i16 values and typed malformed
failures. An incidental smallvec lock update was removed; final checks retain its
incoming pin. No full CI, big-endian execution or installed-consumer claim.

- [Full comparison](evidence/packed-promotion-0611/comparison.csv)
- [Matched before/after](evidence/packed-promotion-0611/before-after.csv)
- [All operation lanes](evidence/packed-promotion-0611/all-lanes.csv)
- [Wasm artifacts](evidence/packed-promotion-0611/artifacts.csv)
- [Identity](evidence/packed-promotion-0611/identity.json)
- [Sources, runners, samples and checks](evidence/packed-promotion-0611/raw.json.gz)
