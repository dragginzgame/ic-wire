# Native codec hotspot investigation

This private benchmark answers which workloads and public operations deserve
profiling. It does not replace installed IC instruction measurements, total Wasm
size qualification, or correctness tests. No benchmark dependency or instrumentation
enters either product crate. There is no publication step.

## Run a bounded comparison

```sh
# Full default cohort: 16 families × 2 representations × 4 sizes × 8 operations
# × 2 suppliers. Compiler work precedes measurement.
make bench-codec

# Narrow subsequent investigation to one workload.
make bench-codec CODEC_ARGS='--case mixed --size 4096 --representation bulk'
make bench-codec CODEC_ARGS='--case i16-small --size 4096 --operation preflight'

# Compile with allocator instrumentation separately; no timings are emitted.
make bench-codec-allocations CODEC_ARGS='--case mixed --size 4096'

# Secondary native opt-3 investigation, explicitly selected.
CODEC_PROFILE=codec-profile-speed make bench-codec CODEC_ARGS='--case mixed --size 4096'

make bench-codec CODEC_ARGS='--list'
```

`CODEC_ARGS` selects `--case`, `--representation ordinary|bulk|both`,
`--size`, `--operation` and `--supplier candidate|reference|both`.
Default sizes are 0, 1, 128 and 4096. A selected size may be up to 1,048,576,
but the fixed published benchmark allowances must still admit the workload.
The mixed record uses size bytes plus at most 4096 samples and fixed labels/cursor.
Large ordinary ID lists can exhaust logical traversal allowances before memory.
An admission failure stops the benchmark; it is not converted into a timing row.

The families are small/wide/negative/mixed i16, boundary-distributed i32/u64,
wide u64, 32-byte IDs, bytes, mixed blob and nested request records, booleans,
finite f32/f64, nonzero i64 and heap u64. The nested request contains two enum
variants, optional revision/cursor fields, a string and size i32 samples; it
exercises compound traversal around ordinary or packed samples. Boundary cases
include the values immediately around selected Postcard varint-width transitions
and integer extrema. These are synthetic requests, not production traffic.
Ordinary
controls retain their Serde lists; bulk controls explicitly select packed words,
serde_bytes or the corresponding domain adapter. They represent the same logical
fixture but can have different bytes and domain validation costs. Never aggregate
them into a single claim that one representation is universally faster.

Each supplier/representation is checked before measuring: request bytes agree
between suppliers, lengths agree with preflight, decode matches the logical input,
reused/slice outputs match owned encoding, and bad headers/truncated frames reject.
Heap equality compares sorted values outside the measured region. Candidate and
reference use their own domain adapters; the legacy source checkout is not linked.

## What the rows mean

| Operation | Measured work |
| --- | --- |
| `preflight` | Complete request sizing and admission via the public length API |
| `encode-owned` | Preflight, encoding, output allocation and destruction |
| `encode-reuse` | Preflight and encoding into a preallocated, warmed Vec |
| `encode-slice` | Preflight and encoding into a preallocated exact-size slice |
| `decode` | Decode and destruction of owned output |
| `bad-header` | Reject an identity-corrupted request |
| `truncated` | Decode/reject a request missing its final byte |
| `loop` | Harness dispatch/black-box control; reported, never subtracted |

Reuse and slice encoding still include preflight; they are not private write-only
paths. Do not subtract independently measured medians and call the difference an
exact phase cost. The harness performs indirect closure dispatch per operation;
the loop control makes its scale visible for tiny operations.

Three warmup operations precede power-of-two calibration to a 2 ms batch target,
capped at 1,048,576 repetitions. Nine samples per supplier follow, alternating
candidate/reference order each round. `--samples` permits 3–51 and `--batch-ms`
1–100. Results retain every batch mean and min/median/max in nanoseconds per
operation; these are not individual-call latency percentiles. They are shared-host
observations, without CPU affinity/frequency isolation or a CI timing threshold.

Allocation mode uses the already selected stats_alloc dependency, in a separately
compiled executable. It warms buffers before measuring a single operation and
reports allocation/deallocation/reallocation counts and bytes. Fixture creation,
correctness checks, buffer reservation and JSON reporting are outside the region;
returned-value destruction is inside. Counts are not peak live heap and are not
the codec's logical allocation allowance.

Native `codec-profile` uses opt-z, full LTO, one codegen unit and debug symbols;
`codec-profile-speed` changes optimization to 3. These are native investigation
profiles, not Wasm deployment profiles. The existing size and IC commands remain
the decision checks after an optimization. Native speed can rank differently.

Each invocation creates `qualification/target/codec-*/` with build output, raw
rows and a report recording compiler, CPU/OS, profile, executable hash, source/
lock hashes, allowances and command arguments. Failed runs retain failure state;
only completed measurements have `completed: true`. Raw reports stay out of Git.

## Function-level sampling

```sh
make profile-codec CODEC_ARGS='--case mixed --size 4096 --representation bulk --supplier candidate --operation encode-owned --seconds 10'
```

Profiling requires exactly one case, representation, supplier, size and operation;
duration is 1–120 seconds. The same symbol-bearing executable repeats the selected
operation under `perf record`, using userspace CPU-clock events and DWARF call
stacks. `hotspots.txt` reports sampled self cost; `perf.data` also supports an
interactive `perf report --input /path/to/perf.data`. Profile startup includes
fixture checking/calibration: use a sufficiently long selected run to dominate it.
Allocation-instrumented builds cannot enter profiling mode.

The current host has `perf_event_paranoid=4` and denies even task-clock events.
Native call stacks therefore cannot be collected here. This preparation does not
change host security settings or install another profiler. On an allowed Linux
host, the command above is the profiling entrypoint; elsewhere the same executable
and `--mode profile` loop can be launched under the platform's native profiler.
Phase timings and allocation counts remain usable on the current host.

## Scope and provenance

Reviewed the read-only legacy benchmark suite at
`2dc434b10bec2fca92345d940dbfedc05993e9d5`, especially
`qualification/ic-wire-benchmarks/benches/fixtures/hotspots/mod.rs`,
`fixtures/packed_i16/mod.rs`, `caller_storage/mod.rs`, `operations/mod.rs` and
`suite.rs`. Selected fixture distributions, the mixed-record shape, separate
allocation/timing builds and calibrated-batch methodology inform this destination
under the retained licence. The broad feature matrix, Candid lane, report system,
release orchestration and alternative codecs are not transplanted.

The runner lives in the existing private compatibility package, with focused
fixture/selection tests and strict Clippy in both instrumented and ordinary builds.
It intentionally does not add per-type fast paths to production merely to improve
the charts. Investigate an observed cost first, then measure a small proposed change
against both native observations and the independent Wasm/IC evidence.

## First observations — 15 November 2026

The default native opt-z run completed all 1,664 timing rows, and the separate
allocator build completed the same 1,664 rows. All 624 preflight/reuse/slice rows
reported zero allocations and reallocations. The fixture/selection tests and
both Clippy builds pass. A selected two-second profiling loop completed 7,618,560
operations; `perf` itself refused recording, so no function-level samples exist.

Selected medians at 4,096 elements, in microseconds:

| Native opt-z workload | Operation | Candidate | Reference |
| --- | --- | ---: | ---: |
| Ordinary small i16 | Preflight | 18.405 | 7.210 |
| Ordinary small i16 | Owned encode | 46.175 | 18.111 |
| Ordinary small i16 | Reused encode | 45.772 | 17.743 |
| Ordinary small i16 | Slice encode | 36.941 | 13.357 |
| Ordinary small i16 | Decode | 25.869 | 27.168 |
| Bulk finite f32 | Preflight | 0.915 | 0.468 |
| Bulk finite f32 | Owned encode | 1.993 | 1.118 |
| Bulk finite f32 | Decode | 2.001 | 1.431 |

For ordinary i16 both suppliers allocate once during owned encoding, zero times
for reused/slice encoding, and once plus ten reallocations during decoding.
Packed i16 decoding allocates once with no reallocations. The encode gap remains
when output allocation is removed, directing investigation toward traversal and
serialization rather than output allocation alone. Finite scanning is another
candidate for focused investigation; these observations do not identify sampled
function costs or justify importing legacy specializations.

A selected native opt-3 run of the same i16 case completed 32 rows. Preflight
was **1.642 vs 4.440 µs**, but owned encoding remained **13.240 vs 9.933 µs**.
The different phase ranking reinforces the need to check profiles independently.
Do not infer equivalent IC regressions from either native profile.

Compiler/host/source identities and selected raw samples are retained in the
`native_hotspots` section of [the compact evidence](performance-0.23.8.json).
Full rows and failure diagnostics remain in the printed `target/` directories.

## Expanded integer investigation — 15 November 2026

Three added families (i32/u64 varint boundaries and a nested request) completed
384 native opt-z timing rows, with 21 paired samples and 5 ms calibration targets.
The fixture test checks all 16 families at 0, 1 and 128 elements in both
representations, including byte equality, decoding, output destinations and
malformed inputs. Ordinary and allocator-instrumented Clippy builds pass.

Selected medians, in microseconds (candidate / reference):

| Workload | Elements | Operation | opt-z | opt-3 |
| --- | ---: | --- | ---: | ---: |
| Ordinary i32 boundaries | 4096 | Owned encode | 56.154 / 26.030 | 20.429 / 12.626 |
| Packed i32 boundaries | 4096 | Decode | 5.334 / 1.119 | 0.274 / 0.348 |
| Ordinary u64 boundaries | 4096 | Owned encode | 63.775 / 53.892 | Not measured |
| Ordinary nested request | 1 | Owned encode | 0.196 / 0.191 | Not measured |
| Ordinary nested request | 128 | Owned encode | 1.826 / 0.909 | Not measured |
| Packed nested request | 4096 | Decode | 5.703 / 1.307 | Not measured |

The 32-row opt-3 i32 run reverses the packed-decode ranking while ordinary
encoding still loses. This makes code generation under opt-z a specific
investigation target, not evidence of an inherently expensive packed format.
Small nested requests are close; larger ordinary collections remain expensive.

Three temporary attribute experiments were removed:

- Forcing the generic encoding admission wrapper inline left i16 preflight and
  reused encoding essentially unchanged (18.109 → 18.208 µs and
  45.390 → 45.300 µs). Ratios alone were misleading because reference timings moved.
- Inlining packed word readers left i32 decode essentially unchanged
  (5.334 → 5.390 µs).
- Keeping the packed visitor out of line reduced that case to 4.360 µs, still
  substantially behind the reference. This observation is not a qualified change
  across workloads, Wasm size or IC instructions, so the attribute was not retained.

Product code remains unchanged by this investigation. The next targeted step is
inspection of generated packed-i32 loops at opt-z versus opt-3, followed by a
generic change only if supported by size and instruction evidence. No handwritten
primitive codec or per-type fast path was introduced. Selected source identities
and raw samples are retained in the evidence JSON's `boundary_investigation`,
`inline_investigation` and `packed_inlining_investigation` sections.

## Packed-word conversion fix — 15 November 2026

Disassembly of the previous non-inlined-visitor experiment identified a
`Result<[u8; 4], TryFromSliceError>::expect` call inside every candidate i32
iteration at opt-z. The reference loop had direct loads/stores. Replacing the
custom `expect` message with checked `unwrap` in the sealed word readers removes
that helper call; the resulting i32 visitor loop uses direct loads/stores. This
is an assembly observation and controlled source experiment, not a sampled
profile. No custom varint encoding, unsafe conversion or type-specific path was
introduced. Partial words and element limits still reject before allocation.

Native decode medians for 4,096 words, microseconds:

| Case/profile | Before candidate | After candidate | After reference |
| --- | ---: | ---: | ---: |
| Packed i32 / opt-z | 5.334 | 1.174 | 1.122 |
| Nested packed i32 / opt-z | 5.703 | 1.267 | 1.276 |
| Packed i32 / opt-3 | 0.274 | 0.260 | 0.197 |
| Packed 32-byte IDs / opt-z | — | 1.902 | 1.988 |

An initial narrow opt-z i32 run measured 1.106 / 1.100 µs, and the subsequent
four-size run above confirms the large candidate improvement. Opt-3 candidate
time remains similar; reference time moved from 0.348 to 0.197 µs, so the earlier
claim that candidate opt-3 wins does not generalize across these binaries/runs.
Shared-host timings and code layout remain relevant. The four-size i32 runs
cover zero/single/small/large buffers in both profiles; nested and ID cases check
the selected larger shapes. Existing fixture correctness checks cover all 16
families and both representations. Seven packed contract tests, nine allocation
groups and core Clippy also pass.

The independently built representative Wasm consumer passes all four artifact
executions. Candidate total size is **45,273 B opt-z / 46,150 B opt-3**, eight
bytes smaller at opt-z and unchanged at opt-3 versus the pre-fix hard cut. This
consumer uses bounded i16, so it does not establish i32-specific Wasm savings.
Source/artifact hashes and selected raw samples are in `packed_word_conversion`
in the compact evidence. Ordinary integer encoding remains a separate hotspot.

A fresh installed-IC run also completes all operations and reproduces the prior
instruction counts exactly. Its strict zero-regression criterion still fails on
the previously disclosed encoding losses versus reference. This is successful
measurement with a failing performance criterion, not an IC speedup. The current
consumer's i16 fields do not qualify the i32-specific native gain on IC.
