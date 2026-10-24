# Codec-cost breakdown: 0.6.11 candidate

This is diagnostic research against released base `31bcef2` plus the pending
shared packed decoder, with package versions still 0.6.10. No production code,
protocol bytes, admission rules or canonical verification are changed by this
batch. Every ablation lives only in the evidence's disposable source snapshots.

## Findings

The large encoding outliers survive removal of Wire's traversal accounting,
sizing pass and canonical verification. With upstream Postcard's own AllocVec
writer, opt-3 wide-i16 encoding still costs 40.3x Candid, wide-i64 encoding 18.8x,
and generic byte-vector encoding 14.6x. These are the selected large cases, not
universal ratios for those types. Bulk representation remains the main opportunity.

Budget accounting alone explains little of wide-i16 cost: disabling it reduces
encode/decode instructions by 3.26%/3.32%. Removing the sizing preflight cuts its
encoding by 11.18%, but generic-byte encoding regresses 0.73% and incurs seventeen
reallocations instead of zero. Dropping preflight is not a justified production fix.

Canonical verification is substantial on decode. Omitting it cuts wide-i16 decode
69.48%, wide-i64 decode 53.70%, and owned 1 MiB blob decode 58.43%. The blob falls
from 2,524,404 to 1,049,385 instructions, close to Candid's 1,064,978. Removing
budget accounting alone saves only eight instructions on that blob. This isolates
an important cost, but removing the check would change accepted inputs and the
custom-Serde contract. Those diagnostic reductions are not shippable savings.

The output writer also matters: replacing the bounded writer with upstream
AllocVec in the otherwise stripped control reduces large generic-byte encoding
from 42,309,055 to 30,774,799 instructions (27.26%). It still loses badly to
Candid's 2,112,047. This comparison also changes allocation/reuse behavior; it
does not establish that one output-bound branch accounts for the whole difference.

## What Candid actually does

The exact Cargo.lock sources were inspected, rather than inferring implementation
from benchmark ratios. Source hashes are retained in the archive:

- Candid 0.10.39, `src/types/impls.rs`, `fixed_primitive_byte_size` and
  `CandidType for [T]`: on little-endian targets, recognize bool and fixed-width
  numeric primitives and offer their memory bytes to `try_write_raw_elements`.
  The fast path returns before the ordinary per-element loop. Its implementation
  uses unsafe slice conversion; IC Wire does not copy that unsafe implementation.
- Candid `src/ser.rs`: fixed-width integers write little-endian words. The bulk
  vector path is compatible with that representation. Ordinary Postcard integers
  have different bytes, so blindly copying it would change the protocol.
- Serde core 1.0.229, `src/ser/impls.rs`: ordinary slices/vectors use `collect_seq`.
  Postcard 1.1.3, `src/ser/serializer.rs`: signed integers use ZigZag and varints,
  then write each encoded element through the output flavor. Ordinary Serde
  vectors do not automatically select byte serialization.
- Candid `src/de.rs`, `deserialize_byte_buf`/`deserialize_bytes`: byte-oriented
  fields read a contiguous slice and copy or borrow it. Ordinary Serde Vec<T>
  decoding still loops through `SeqAccess` (`serde_core/src/de/impls.rs`). Candid's
  encoder bulk path should not be confused with a universal bulk decoder.
- Wire `native/mod.rs`: encoding sizes/admit-visits before reserving the output,
  then serializes with admission again. Decoding checks admission and trailing
  bytes, then reserializes into an exact comparison flavor for canonicality.

The practical direction is to retain the existing byte-oriented fields and
PackedI16 for known hot paths, investigate output-writer optimizations while
preserving bounds, and assess the same explicit packed representation for another
integer type only when consumer measurements justify it. Standard Serde derives
remain appropriate. No second codec, custom serializer derive, automatic type
specialization or relaxed canonicality is introduced here.

## Controlled variants

All six use the same 22 inputs and dependencies: i16 small/wide and i64 wide lists
at lengths 0/1/128/16384, plus generic and byte-oriented blobs at lengths
0/1/128/4096/1048576. Baseline is the current production runtime. Other variants:

| Variant | Difference from baseline | Important limitation |
| --- | --- | --- |
| no-budget | Budget `enter_many` and `bytes` return true | Traversal wrappers and other checks remain; resource admission is invalid |
| no-presizing | Skip sizing/admission preflight and exact reservation | Checked encoding remains, but output grows before complete admission |
| no-canonical | Skip reserialization/comparison after decode | Does not preserve canonical/custom-Serde acceptance |
| raw-postcard | Remove wrappers, preflight and canonical verification | Retains framing, input/trailing checks and bounded output flavor only |
| postcard-allocvec | Same stripped control with upstream AllocVec output | No output byte bound; always allocates fresh, even for the reuse probe |

These are independent ablations, not successive subtraction terms. Compiler
optimization and allocation changes interact. The smaller fixture also changes
optimizer decisions: its ordinary generic-byte baseline is 50.33M encoding
instructions, versus 78.64M in the full matrix. Compare variants within this
study; the [full matrix](full-comparison-0611.md) remains the full-fixture result.
Do not infer production traffic or installed application costs from either fixture.

## Method and reproduction

Rust 1.99.1 / LLVM 23.1.1, x86_64 Linux Threadripper 7970X native,
wasm32-unknown-unknown, Candid 0.10.39, Postcard 1.1.3, Serde 1.0.229,
IC Platform 0.6.10 and Host/Testkit 0.66.0. Profiles are evaluation (opt-z) and
benchmark-speed (opt-3), LTO, one codegen unit, panic abort, stripped symbols.
Both profiles and all absolute values/deltas are retained in the CSVs.

Two serial native runs/profile use eleven calibrated samples/run, minimum 2 ms
batches with a 1,048,576-operation cap. They are unpinned warm-cache timings,
including result destruction and loop overhead, excluding setup/roundtrip checks.
The reported range spans run medians, not request latency or a confidence interval.
Allocation and peak-heap runs are separately instrumented. IC performance counter
0 uses PocketIC 16.1.0, three samples, four repetitions through 64 KiB and one above;
setup/transport is excluded and loop overhead is not subtracted. Whole Wasm totals
include fixtures/harness; code sections are measured separately. No gzip objective.

Baseline was rebuilt in an existing target. Initial variant runs reused cached
custom-profile artifacts and were discarded. Every retained variant uses a fresh,
separate target directory; Candid artifacts remain byte-identical across variants.
The baseline IC run initially could not bind localhost in the sandbox; only that
step was retried with local-server access. These discarded attempts are not mixed
into the measurements.

Extract `variant_sources` from the raw JSON archive into `source-VARIANT`
directories. Its `reproduction_runner` runs all six serially with separate target
paths; change root/output paths and POCKET_IC_BIN to local fresh paths. Use the
archived lock and toolchain, RUNS=2 SAMPLES=11 BATCH_MS=2. The archived report,
tables and delta scripts reproduce summaries after adjusting root/output paths.
The initial preparer is provenance only; `variant_sources` is authoritative for
all six exact snapshots. The production sources are not replaced during research.

## IC instruction breakdown

Opt-3, absolute instructions per operation, rounded. Each diagnostic removes
guarantees and is not a production replacement. Ratios/deltas are comparisons
between separately compiled artifacts, not an additive decomposition.

| Case / operation | Baseline | No budget accounting | No presizing | No canonical check | Postcard + bounded writer | Postcard AllocVec | Candid |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| i16-wide-16384 / encode | 4,014,926 | 3,883,847 | 3,566,177 | 4,014,926 | 3,435,104 | 3,238,933 | 80,325 |
| i16-wide-16384 / decode | 5,919,297 | 5,722,687 | 5,919,297 | 1,806,650 | 1,704,507 | 1,704,507 | 1,408,091 |
| i64-wide-16384 / encode | 6,407,165 | 6,341,619 | 5,354,551 | 6,407,165 | 5,370,941 | 5,207,570 | 277,120 |
| i64-wide-16384 / decode | 10,801,672 | 10,473,990 | 10,801,672 | 5,001,476 | 4,702,941 | 4,702,941 | 1,277,171 |
| generic-bytes-1048576 / encode | 50,332,646 | 41,944,046 | 50,697,662 | 50,332,646 | 42,309,055 | 30,774,799 | 2,112,047 |
| generic-bytes-1048576 / decode | 68,687,387 | 61,871,634 | 68,687,387 | 42,472,688 | 34,603,741 | 34,603,741 | 78,659,096 |
| blob-1048576 / encode | 1,049,703 | 1,049,700 | 1,050,800 | 1,049,703 | 1,050,798 | 1,050,845 | 2,115,165 |
| blob-1048576 / decode | 2,524,404 | 2,524,396 | 2,524,404 | 1,049,385 | 1,049,388 | 1,049,388 | 1,064,978 |

## Small-input controls

Opt-3, Wire instructions. Empty and one-element inputs expose fixed costs.

| Case / operation | Baseline | No budget | No presizing | No canonical | Bounded writer | AllocVec |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| i16-wide-0 / encode | 779 | 777 | 1,050 | 779 | 1,053 | 1,071 |
| i16-wide-0 / decode | 421 | 419 | 421 | 303 | 322 | 322 |
| i16-wide-1 / encode | 1,040 | 1,023 | 1,266 | 1,040 | 1,261 | 1,274 |
| i16-wide-1 / decode | 1,111 | 1,097 | 1,111 | 740 | 641 | 641 |
| blob-0 / encode | 803 | 798 | 1,076 | 803 | 1,074 | 1,087 |
| blob-0 / decode | 508 | 505 | 508 | 310 | 305 | 305 |
| blob-1 / encode | 913 | 908 | 1,479 | 913 | 1,477 | 1,490 |
| blob-1 / decode | 805 | 797 | 805 | 627 | 630 | 630 |

## Artifact sizes

Whole selected fixture, uncompressed total / code-section bytes. These
diagnostic sizes include removed guarantees and are not shippable savings.

| Variant / profile | Wire total / code | Candid total / code |
| --- | ---: | ---: |
| baseline / evaluation | 56,111 / 46,491 | 307,292 / 275,245 |
| baseline / benchmark-speed | 66,559 / 57,616 | 427,992 / 397,581 |
| no-budget / evaluation | 55,581 / 45,961 | 307,292 / 275,245 |
| no-budget / benchmark-speed | 66,232 / 57,289 | 427,992 / 397,581 |
| no-canonical / evaluation | 53,809 / 44,195 | 307,292 / 275,245 |
| no-canonical / benchmark-speed | 63,036 / 54,097 | 427,992 / 397,581 |
| raw-postcard / evaluation | 51,949 / 42,355 | 307,292 / 275,245 |
| raw-postcard / benchmark-speed | 62,128 / 53,189 | 427,992 / 397,581 |
| no-presizing / evaluation | 55,192 / 45,575 | 307,292 / 275,245 |
| no-presizing / benchmark-speed | 65,791 / 56,848 | 427,992 / 397,581 |
| postcard-allocvec / evaluation | 51,158 / 41,565 | 307,292 / 275,245 |
| postcard-allocvec / benchmark-speed | 61,343 / 52,374 | 427,992 / 397,581 |

## Native and allocation examples

Opt-3, two native run medians shown as a range in microseconds. Allocation
instrumentation is separate; peak live bytes are native heap, not Wasm heap.

| Variant / case / operation | Native µs | Allocations / reallocations | Peak bytes |
| --- | ---: | ---: | ---: |
| baseline / i16-wide-16384 / encode | 89.189–90.294 | 1 / 0 | 49,166 |
| no-budget / i16-wide-16384 / encode | 85.427–85.593 | 1 / 0 | 49,166 |
| no-presizing / i16-wide-16384 / encode | 83.743–84.142 | 1 / 13 | 90,112 |
| no-canonical / i16-wide-16384 / encode | 89.164–90.222 | 1 / 0 | 49,166 |
| raw-postcard / i16-wide-16384 / encode | 79.365–80.281 | 1 / 13 | 90,112 |
| postcard-allocvec / i16-wide-16384 / encode | 79.518–79.605 | 1 / 13 | 90,112 |
| baseline / generic-bytes-1048576 / encode | 1215.457–1249.303 | 1 / 0 | 1,048,590 |
| no-budget / generic-bytes-1048576 / encode | 816.496–820.750 | 1 / 0 | 1,048,590 |
| no-presizing / generic-bytes-1048576 / encode | 1247.657–1288.883 | 1 / 17 | 1,441,792 |
| no-canonical / generic-bytes-1048576 / encode | 1253.621–1255.789 | 1 / 0 | 1,048,590 |
| raw-postcard / generic-bytes-1048576 / encode | 519.271–519.939 | 1 / 17 | 1,441,792 |
| postcard-allocvec / generic-bytes-1048576 / encode | 468.426–474.163 | 1 / 17 | 1,441,792 |
| baseline / blob-1048576 / decode | 32.556–33.113 | 1 / 0 | 1,048,576 |
| no-budget / blob-1048576 / decode | 32.234–32.880 | 1 / 0 | 1,048,576 |
| no-presizing / blob-1048576 / decode | 32.150–33.019 | 1 / 0 | 1,048,576 |
| no-canonical / blob-1048576 / decode | 17.286–17.327 | 1 / 0 | 1,048,576 |
| raw-postcard / blob-1048576 / decode | 17.275–17.578 | 1 / 0 | 1,048,576 |
| postcard-allocvec / blob-1048576 / decode | 17.958–18.545 | 1 / 0 | 1,048,576 |

## Evidence and checks

All six variants completed both profiles: 528 paired comparisons, 2,904 IC rows,
9,504 native/instrumented rows and 24 Wasm artifacts. Each IC row has three
identical samples. Input/frame hashes and lengths match across all runs and
variants. All Candid instruction controls match; Candid artifacts are identical.
All artifact hashes/sizes were independently checked. The runner checks semantic
roundtrips and IC frames against host output; malformed-header/truncation probes
pass. This does not qualify the ablations' changed admission/canonical behavior.
No new full CI, installed-consumer or big-endian execution is claimed.

- [All primary comparisons](evidence/cost-breakdown-0611/comparison.csv), including
  wire size, native timings, IC instructions, allocations and peak live heap.
- [Before/after deltas](evidence/cost-breakdown-0611/ablations.csv), with absolute
  values, percentage changes, small controls and regressions in both profiles.
- [All labelled operations](evidence/cost-breakdown-0611/all-lanes.csv), including
  reuse and rejection probes. AllocVec's reuse label does not imply retained storage.
- [Artifacts](evidence/cost-breakdown-0611/artifacts.csv) and
  [identity](evidence/cost-breakdown-0611/identity.json).
- [Raw archive](evidence/cost-breakdown-0611/raw.json.gz): sources, samples,
  identities, upstream source hashes, reproduction and analysis scripts. Archive
  compression is storage only, not an artifact-size optimization metric.

No production changes are retained from the diagnostic variants. The previous
pending shared-decoder change remains intact. Versions, staging and release
remain maintainer-owned.
