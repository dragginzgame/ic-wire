# Current performance qualification

The latest local packed-word conversion fix measures **45,273 B opt-z / 46,150 B
opt-3**, eight bytes smaller at opt-z than the hard cut below. All four standalone
Wasm workloads pass. A fresh PocketIC run reproduces every instruction count in
the table below, including its existing encoding losses; the strict instruction
criterion still fails. This bounded-i16 consumer does not measure the new native
i32 hotspot directly. The evidence's `packed_word_conversion` section records
current hashes, native results, Wasm artifacts and IC counters.

The subsequent local hard cut consolidates packed types into `Packed<T, N>`.
Fresh size qualification passes at **45,281 B opt-z / 46,150 B opt-3**: unchanged
at opt-z and four bytes smaller at opt-3 than the checkpoint below. Both code
section payload sizes are unchanged. The report's `hard_cut_size` section records
the new source and artifact hashes. IC counters and host timing below describe
the pushed checkpoint, not a rerun of the hard cut.

Measured 15 November 2026 after pushing 0.23.8 at `dd6da23` to main.
Reference is published Core 0.22.4. No product code changed during measurement.
[Source hashes, artifact identities and measurements](performance-0.23.8.json)
identify this checkpoint rather than extending the older reports' claims.

The unchanged experiments are specified in [Wasm size](consumer-measurement.md),
[installed instructions](consumer-instructions.md) and
[host schema timing](schema-measurement.md). Rust 1.100.0, locked suppliers,
the independent 34/14-byte frame oracle and existing thresholds remain unchanged.
The hard cut removes legacy compatibility requirements, not the need to measure
costs fairly on identical workloads. No performance threshold was relaxed.

## Total uncompressed Wasm

| Profile | Candidate | Reference | Difference |
| --- | ---: | ---: | ---: |
| opt-z | 45,281 B | 46,138 B | -857 B (-1.86%) |
| opt-3 | 46,154 B | 47,026 B | -872 B (-1.85%) |

Both size criteria pass. All four SDK-free artifacts execute their assertions
under Node without host imports. Code-section payloads are 38,322/39,552 bytes
for candidate opt-z/opt-3 and 39,180/40,426 for reference. This is identical to
the earlier packed-refactor size run: subsequent host schema/lint changes add
no size to this workload. This does not measure unused domain adapters.

## Installed IC instructions

| Operation | Candidate opt-z | Reference opt-z | Candidate opt-3 | Reference opt-3 |
| --- | ---: | ---: | ---: | ---: |
| Request encode | 5,224 | 4,961 | 2,674 | 2,443 |
| Request decode | 5,413 | 5,634 | 2,969 | 3,294 |
| Response encode | 1,725 | 1,744 | 696 | 690 |
| Response decode | 590 | 712 | 389 | 425 |
| Bad header | 189 | 197 | 106 | 132 |
| Truncated request | 4,546 | 4,779 | 2,438 | 2,740 |

Counts are median warmed batches divided by eight, including destruction and
loop/counter overhead; loop controls are retained without subtraction. All three
samples agree for every operation. All 12 queries return the independent literal
frames. PocketIC 16.1.0 runs the four artifacts with successful canister cleanup.
The sandbox initially prevented the loopback server bind; the prepared driver
then ran with loopback permission against the same already-built artifacts.

The instruction criterion fails: request encoding is +5.30%/+9.46% at opt-z/opt-3,
and response encoding is +0.87% at opt-3. The driver intentionally returns failure
after retaining these results. Faster decoding does not erase encoding losses.
Relative to 0.23.7, request decode and truncation cost 23 more instructions at
opt-z and 47/45 fewer respectively at opt-3; encode counts are unchanged.

## Host schema cost

Collection, finalization and JSON export totals, median microseconds:

| Shape | Public roots | Candidate | Reference | Difference |
| --- | ---: | ---: | ---: | ---: |
| Scalar | 1 | 1.252 | 0.972 | +28.81% |
| Recursive | 1 | 8.032 | 6.740 | +19.17% |
| Scalar | 128 | 99.930 | 108.403 | -7.82% |
| Recursive | 128 | 193.491 | 201.142 | -3.80% |
| Scalar | 1,024 | 850.779 | 1,045.833 | -18.65% |
| Recursive | 1,024 | 1,738.006 | 1,880.670 | -7.59% |

All output-equivalence assertions pass. Each case alternates execution order,
uses three warmup pairs and 21 measured pairs. These shared-host observations
reproduce the small-registry finding; they are not statistical speed guarantees.
The probe excludes client rendering, automatic bulk lint and custom callbacks.

## Remaining qualification

The [expanded native investigation](codec-benchmarks.md#expanded-integer-investigation--15-november-2026)
now covers integer boundaries and nested requests. It confirms ordinary integer
encoding losses and exposes a packed-i32 decode loss at native opt-z that reverses
at opt-3. Temporary inlining experiments were removed. The subsequent checked-word
conversion simplification removes the large native opt-z i32 decode loss;
no IC instruction improvement is claimed. The intended eventual release is 0.24.0.

The current checkpoint is smaller for this consumer, but has not passed the
instruction or single-root host criteria. Next investigate generic encoding
and small-registry graph/export overhead, measuring size and instructions together.
Keep Postcard delegation and admission unless a measured design change justifies
otherwise; do not import legacy fast paths merely to match its implementation.

The five newly retained domain adapters need separate matched workloads, including
ordinary-list and small-value controls, before any broad performance claim.
Full client generation with mandatory lint also needs its own host-cost probe;
the collection/export figures above cannot stand in for it. Allocation admission
tests pass, but are not a new total-heap performance measurement. Actual browser
execution and distributed-package qualification remain separate work.
