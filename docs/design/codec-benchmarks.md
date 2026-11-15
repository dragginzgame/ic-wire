# Native codec hotspot investigation

This private benchmark answers which workloads and public operations deserve
profiling. It does not replace installed IC instruction measurements, total Wasm
size qualification, or correctness tests. No benchmark dependency or instrumentation
enters either product crate. There is no publication step.

## Run a bounded comparison

```sh
# Full default cohort: 52 families, 85 applicable representations, 4 sizes,
# 12 common operations plus applicable borrowed/domain operations, 2 suppliers.
make bench-codec

# Narrow subsequent investigation to one workload.
make bench-codec CODEC_ARGS='--case mixed --size 4096 --representation bulk'
make bench-codec CODEC_ARGS='--case i16-small --size 4096 --operation preflight'
make bench-codec CODEC_ARGS='--case u32-boundaries --size boundaries'
make bench-codec CODEC_ARGS='--case text-unicode --operation decode-borrowed-text'
make bench-codec CODEC_ARGS='--case finite-f32 --operation invalid-last'

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
`--size boundaries` selects 0, 1, 2, 7, 8, 15, 16, 63, 64, 127, 128, 129,
255, 256 and 4096. The correctness matrix covers 0, 1, 2, 7, 8, 127, 128 and
129 independently of the default timing sizes. Empty operation selections and
bulk-only requests for ordinary-only types are rejected rather than silently timed.
The mixed record uses size bytes plus at most 4096 samples and fixed labels/cursor.
Large ordinary ID lists can exhaust logical traversal allowances before memory.
An admission failure stops the benchmark; it is not converted into a timing row.

The families cover signed/unsigned integer magnitudes and selected varint boundaries
through 128 bits, 16/32-byte IDs, bytes, ASCII/Unicode/escaped text, options,
results, tuples, integer/text/byte-key maps, sets, deques, linked lists, record
lists, nested lists, zero-sized values, Platform Principals/Accounts, mixed/nested
requests, boolean distributions, finite normal/subnormal floats, nonzero i64 and
heap orderings. `--list` is the exact catalog. The nested request contains two enum
variants, optional revision/cursor fields, a string and size i32 samples; it
exercises compound traversal around ordinary or packed samples. Boundary cases
include the values immediately around selected Postcard varint-width transitions
and integer extrema. These are synthetic requests, not production traffic.
Ordinary controls retain their Serde representations; bulk controls explicitly select packed words,
serde_bytes or the corresponding domain adapter. They represent the same logical
fixture but can have different bytes and domain validation costs. Never aggregate
them into a single claim that one representation is universally faster.
Text size counts repeated fixture units (Unicode and escaped units contain
multiple bytes); container size counts entries, and record-list entries contain
fixed small fields. These units must not be treated as equal byte throughput.
Nineteen families have only an ordinary representation: no artificial bulk DTO
or duplicate timing row is introduced for them.

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
| `decode-borrowed-text` / `decode-borrowed-bytes` | Borrow text/bytes directly from retained input; applicable cases only |
| `bad-header` | Reject an identity-corrupted request |
| `short-header` / `wrong-method` / `wrong-direction` | Reject each envelope failure separately |
| `truncated` | Decode/reject a request missing its final byte |
| `trailing` | Decode/reject an otherwise valid frame with an extra byte |
| `invalid-first` / `invalid-middle` / `invalid-last` | Reject a nonfinite float, invalid boolean or zero nonzero-word at that position; nonempty bulk domain cases only |
| `loop` | Harness dispatch/black-box control; reported, never subtracted |

Reuse and slice encoding still include preflight; they are not private write-only
paths. Do not subtract independently measured medians and call the difference an
exact phase cost. The harness performs indirect closure dispatch per operation;
the loop control makes its scale visible for tiny operations.
Malformed buffers are prepared and checked for rejection outside timing.
Float tests additionally cover infinities, quiet/signaling NaNs, partial words,
and bitwise preservation of signed zero, subnormals and extrema. Borrowed tests
check that returned slices point inside the original frame. These tests do not
substitute for the separate preallocation-admission checks.

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

The broader matrix additionally reviews the pinned reference's `fixtures/shapes`,
`fixtures/followup/containers`, `fixtures/followup/words`, `fixtures/byte_keys`,
`input_borrowing`, `admission` and `unsigned_magnitude` studies. Coverage is
re-expressed as small fixtures in this runner rather than importing its mutually
exclusive Cargo-feature cohorts or endpoint lifecycle code.

| Legacy study | Destination coverage or remaining boundary |
| --- | --- |
| Scalar magnitudes, signed32, hotspots | Integer boundary/small/negative distributions, ordinary and supported packed words |
| Shapes and containers | Options, results, tuples, records, maps, sets, deque/list, nested/zero-sized collections, actual Platform identities |
| Bulk and byte keys | 16/32-byte IDs, byte-key maps, ordinary/bulk controls and caller storage |
| Input borrowing | Native borrowed text/bytes timing, allocations and pointer provenance; endpoint lifetime remains downstream |
| Domain adapters | Normal/subnormal/extreme float values, boolean and heap distributions, malformed first/middle/last positions |
| Admission | Existing separate allocation/admission contracts; this timing matrix still uses one fixed generous policy |
| Candid/Nat conversions | Not imported: optional comparator research and consumer-owned arbitrary-precision normalization remain separate |
| Derive scale, Wasm and IC instruction matrices | Existing independent consumer/schema measurements retained; the expanded native matrix is not a full port of these studies |

The native suite is substantially broader, but neither it nor a legacy fixture
import proves full browser, schema/lint or deployed-consumer coverage. Request
framing is the timed format; payload/response contracts remain in the independent
consumer and core tests. No Candid or native tooling enters the product graph.

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

Compiler/host/source identities and selected results are retained in the
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
Source/artifact hashes and selected results are in `packed_word_conversion`
in the compact evidence. Ordinary integer encoding remains a separate hotspot.

A fresh installed-IC run also completes all operations and reproduces the prior
instruction counts exactly. Its strict zero-regression criterion still fails on
the previously disclosed encoding losses versus reference. This is successful
measurement with a failing performance criterion, not an IC speedup. The current
consumer's i16 fields do not qualify the i32-specific native gain on IC.

## Finite-float follow-up after 0.23.9

After pushing `c9ae8e3`, repeated native opt-z measurements at 4,096 bulk words
tested the same checked-conversion simplification in both finite adapters.
Each operation uses 21 paired samples and a 5 ms calibration target. Candidate
medians, microseconds:

| Case | Operation | Pushed baseline | Temporary conversion change |
| --- | --- | ---: | ---: |
| f32 | Preflight | 0.876 | 0.895 |
| f32 | Owned encode | 1.993 | 1.951 |
| f32 | Decode | 2.742 | 2.697 |
| f64 | Preflight | 1.411 | 1.420 |
| f64 | Owned encode | 3.341 | 3.363 |
| f64 | Decode | 3.251 | 3.306 |

No material benefit was demonstrated, and the product file was restored exactly
to the pushed version. Baseline assembly already has direct exponent checks in
the validation loops, without a per-word `expect` call. The pinned reference
uses unrolled scanning, including paired exponent arithmetic for f32. Its
baseline owned-encode medians were 1.106 µs f32 and 3.041 µs f64; this narrower
comparison does not establish that importing those specializations is worthwhile.
No new native speedup, Wasm size or IC instruction claim follows from this
rejected experiment. `finite_conversion_investigation` retains source identities
and selected results in the evidence JSON.

The next priority remains ordinary integer encoding. Any future float scanner
proposal needs small inputs and malformed-position controls as well as large
all-finite buffers, since early refusal before allocation is part of its value.

## Expanded matrix results — 15 November 2026

The 52-family expansion completes **8,380 rows per build**: native opt-z,
native opt-3, and separate allocation instrumentation. Timing screening uses
five paired samples and a 1 ms calibration target; retain the longer selected
runs for optimization decisions. Both suppliers pass the premeasurement byte,
roundtrip, output-destination and applicable rejection assertions for every row.

The five focused test functions pass, including 680 value/representation pairs
across both suppliers, malformed positions, all truncated prefixes of selected
records, and borrowed pointer provenance. Running the same native executable in
check mode with `--size boundaries` passes 1,275 value/representation pairs.
Selected one-million-element byte and u64 cases (ordinary/bulk), and Unicode text
with one million repeated units, also pass. These are native checks, not Wasm or
browser execution. Both ordinary and allocator-instrumented Clippy checks pass.

Allocation observations across both suppliers:

- All 2,040 preflight/reuse/slice rows have zero allocations and reallocations.
- All 40 borrowed-text/bytes rows have zero allocations and reallocations.
- All 162 invalid-position bool/finite-float rows refuse without allocation.
- The 18 invalid-position nonzero-i64 rows allocate once on both suppliers,
  consistent with the existing allocate-then-domain-check contract. This is not
  a claim that all domain validation happens before allocation.

Selected opt-z medians at 4,096 elements, microseconds:

| Case/operation | Candidate | Reference |
| --- | ---: | ---: |
| Ordinary small i16 encode | 50.991 | 21.800 |
| Ordinary small u32 encode | 55.920 | 23.908 |
| Ordinary nested-list encode | 261.436 | 165.105 |
| Packed i32 decode | 1.110 | 1.120 |
| Bulk finite-f32 encode | 2.027 | 1.141 |
| Bulk finite-f32 invalid first | 0.015 | 0.016 |
| Bulk finite-f32 invalid last | 0.929 | 0.479 |

The expanded suite reinforces ordinary-integer/compound encoding and finite
scanning as investigation targets. It does not establish a single overall winner;
fixture composition, compiler profile and larger-binary code layout matter.
The source hashes, catalog, allocation summary and selected results are retained
in [matrix evidence](codec-matrix-0.23.9.json); full reports remain in the recorded
`target/` paths. No production code changed during this matrix expansion.

## Measured integer and finite-float improvements

Release 0.24.0 retains two small changes: a shared one-byte integer shortcut
inside the existing Postcard adapter, and a two-word finite-f32 validity scan.
Admission, frame/payload bytes and output custody remain unchanged. Wider
varints still use Postcard; f64 retains its scalar scan. No new dependency or
public representation was introduced.

Native timing in microseconds at 4,096 elements (paired suppliers; baseline and
changed builds measured separately, so shared-host/layout effects remain):

| Operation | Profile | Before | Changed | Reference with changed build |
| --- | --- | ---: | ---: | ---: |
| Ordinary small i16 owned encode | opt-z | 50.629 | 22.309 | 20.516 |
| Bulk finite-f32 owned encode | opt-z | 2.027 | 1.259 | 1.147 |
| Bulk finite-f32 owned encode | opt-3 | 3.153 | 1.541 | 0.673 |
| Bulk finite-f32 decode | opt-3 | 1.185 | 1.144 | 0.508 |

The final combined changes also screened owned encoding and decoding across all
52 families and four default sizes in both profiles (2,720 rows). This is a short
five-sample screening run, not a universal speed guarantee. Wider integers and
finite decoding still have substantial losses against reference. Longer float
runs cover sizes 0/1/128/4096 and malformed positions with 21 samples. The float
encode gain does not establish equivalent decode or installed IC gains.

The upload consumer grows by 129/181 B to **45,402/46,331 B** at opt-z/opt-3,
still 736/695 B smaller than reference. Installed request encoding improves to
**4,953/2,435 instructions** (reference 4,961/2,443). Opt-z response rises to
1,751 (reference 1,744); opt-3 response is 689 (reference 690). Decoding and error
counts are unchanged. The strict no-regression instruction criterion still
fails on opt-z response encoding; it was not relaxed.

A separate finite-f32 consumer now measures the affected code rather than
attributing upload results to an unused adapter:

```sh
node qualification/consumer/measure-wasm.mjs wire-float-consumer
```

It executes sizes 0/1/7/128/4096, checks exact roundtrip bits (including signed
zero and subnormals), and rejects a final infinity. Both suppliers/profiles run
without host imports. The float scan grows this workload by **113/210 B** to
**21,260/22,011 B**: 122 B smaller than reference at opt-z and equal at opt-3.
This added size is retained for the measured native encode improvement; no
float-specific IC instruction measurement is claimed.

The exhaustive integer vectors, width boundaries, output/admission tests,
allocation checks and 262,144 exponent-lane cases pass. Invalid float spans
still refuse before output allocation; payload sizes are unchanged. Source and
artifact identities plus selected results are in
[optimization evidence](codec-optimizations-0.23.10.json).

## Shared-admission inlining investigation

Opt-z assembly shows ordinary i16 sizing calling the admission helper and
Postcard's varint writer per element. Postcard builds a temporary varint buffer
even when the flavor only needs the resulting length. This helps explain why
removing allocation from the benchmark does not close the encoding gap. The
supplier remains unmodified; no second sizing codec was introduced.

A temporary one-line change forced `Budget::enter_many` inline on native targets
as well as Wasm. The accounting and error checks were unchanged. Selected
21-sample opt-z runs at 4,096 elements initially looked promising:

| Ordinary owned encoding | Baseline candidate µs | Experimental candidate µs |
| --- | ---: | ---: |
| Small i16 | 50.629 | 45.966 |
| Small u32 | 52.118 | 47.142 |
| Nested lists | 255.189 | 226.131 |

Opt-3 nested-list encoding was essentially unchanged (96.807 → 96.926 µs).
Opt-3 i16 encoding rose 12.268 → 12.904 µs (+5.2%), while the reference rose
9.846 → 10.213 µs (+3.7%). These shared-host observations do not establish a
uniform improvement.

The expanded matrix then screened owned encoding and decoding across all 52
families and four default sizes in both profiles: **2,720 additional timing
rows**. It exposed bulk regressions absent from the narrow starting cases.
Longer 31-sample repeats confirmed packed-u16 opt-z decode at **1.816 µs with
the change versus 1.076 µs after restoring it**; reference timings were stable
at 1.089/1.080 µs. The approximately 69% loss outweighs the modest encoding gains.
The broad screen's apparent opt-3 packed-i32 encoding loss did not reproduce:
selected changed/restored measurements were 0.178/0.183 µs, so it is not counted
as a confirmed regression.

The experiment was rejected and `meter.rs` restored exactly to pushed 0.23.9.
No new production optimization, Wasm size or IC instruction result is claimed.
The next encoding proposal must preserve bulk/decode controls as well as improve
ordinary lists. Raw selected samples and identities are retained in the matrix
evidence's `admission_inline_investigation` section.
