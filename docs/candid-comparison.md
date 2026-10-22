# Candid comparison suite

`ic-wire-benchmarks` is unpublished. All codecs and profilers are dev dependencies;
Candid is consumed through IC Platform. Production IC Wire remains bounded
Postcard, with no Candid dependency or protocol change.

## Guarded finite verification — pending 0.5.22

The retained follow-up removes the redundant owned-byte scan for direct finite
`Bytes<N>` and eligible generated records, without replacing Postcard or adding a
codec option. These measurements supersede the standard and 100-type results in
the earlier full rerun below. Toko and the 1/10-type cohorts were not rerun here.

| opt-3 owned blob, 1 MiB | Wire before | Wire after | Candid |
| --- | ---: | ---: | ---: |
| Frame bytes | 1,048,590 | 1,048,590 | 1,048,588 |
| Decode IC instructions | 2,524,473 | 1,049,786 | 1,065,406 |

Wire finite decode falls 58.42%, about 1.47% below Candid for this input. This
does **not** speed up ordinary native Serde blob decode, which remains 2,524,535
instructions. Borrowed Wire remains 824 instructions with different ownership.
At opt-z finite owned decode falls from 2,574,508 to 1,050,451 (−59.20%);
Candid uses 1,073,875. All encoded frame hashes remain unchanged.

| Fixture/profile | Wire code before → after | Wire Wasm before → after | Candid code / Wasm |
| --- | ---: | ---: | ---: |
| Standard / opt-3 | 264,599 → 264,914 | 314,412 → 314,728 | 788,971 / 860,566 |
| 100 types / opt-3 | 363,691 → 355,288 | 398,228 → 389,823 | 2,010,763 / 2,079,944 |
| Standard / opt-z | 186,739 → 186,895 | 221,815 → 221,969 | 506,669 / 563,600 |
| 100 types / opt-z | 286,569 → 285,724 | 305,391 → 304,550 | 1,362,761 / 1,416,775 |

All sizes are uncompressed bytes of complete fixture canisters. The opt-3
100-type code reduction is 8,403 bytes (2.31%); this is not isolated derive-symbol
attribution or a promise for a consumer application. Small-input costs remain:
one-byte finite blob decode rises from 868.5 to 885.5 instructions (+1.96%), and
an empty finite manifest rises from 2,853.5 to 2,901.5 (+1.68%). The 100-type
record-000/015 decode counts fall from 2,883.5/3,800.5 to 2,858.5/3,347.5.
Some opt-3 bad-header checks in that fixture cost three extra instructions.
At opt-z, record-001 decode rises from 5,241.5 to 5,278.5 (+0.71%); the
100-type decode range is −2.26% to +0.71%. At opt-3 all 100 improve by 0.36–11.92%.

Two opt-3 native runs put finite 1 MiB owned decode at 17.34–17.42 µs, versus
33.53–33.69 µs in the baseline and Candid's rerun 18.20–18.38 µs. Ordinary
native Serde remains 33.70–34.20 µs. These are unpinned Linux host measurements,
not IC latency guarantees. All 1,844 fresh encode/decode allocation and peak-heap
summaries match baseline exactly; finite blob decode still allocates once with
1,048,576 bytes peak live value heap.

[Full before/after results](evidence/canonical-fields-0522/summary.csv),
[Wasm sizes](evidence/canonical-fields-0522/artifacts.csv),
[identities](evidence/canonical-fields-0522/identity.json) and
[raw samples, source patches and regression evidence](evidence/canonical-fields-0522/raw.json.gz)
cover both profiles, 114 standard cases and 100 derived types. All 4,838 IC rows
have three agreeing samples; all input/frame hashes match baseline and native
runs, with unchanged encoded lengths. All eight final artifacts are byte-identical
to those used for the guarded PocketIC runs, including Candid controls. Native
timing has two runs per profile, eleven calibrated batches per operation, minimum
2 ms batches and a 1,048,576-repetition cap. Allocation and heap passes are separate.
IC sampling, fixture shapes and compiler settings match the full rerun below:
Rust 1.99.1/LLVM 23.1.1, locked dependencies, opt-z/opt-3, LTO, one codegen unit,
panic abort, stripped symbols and PocketIC 16.1.0. IC setup and reply transport
are excluded; destruction and loop overhead are included without subtraction.

Reproduce with `scripts/ci/evaluate-codecs.sh NEW_OUTPUT`, once with the default
corpus and once with `CORPUS_FEATURE=derive-scale`, setting `POCKET_IC_BIN` to
the prepared server to include IC measurements. Full source snapshots and outputs
are in `target/canonical-fields-0522-final-{standard,scale}`. The initial guarded
IC artifacts and runs are under `target/canonical-fields-0522-gated-*`; exact
artifact equality permits reusing those counts for the final source cleanup.
Runtime tests, strict lint, native/Wasm MSRV, 16 compile-fail diagnostics with
passing control and documentation-link checks pass. Full CI, macOS, TypeScript
conformance and other consumer cohorts were not rerun for this change.

### Why the whole-type guard matters

The default verification hook still compares full serialized output. Supplied
bytes can skip their unchanged copied payload, and generated records delegate
only when **all** field types preserve decoded values and field boundaries.
Manual implementations default to no guarantee. Their containing records retain
whole-value comparison. The hidden hooks are reserved for runtime/derive code;
they are not user-selectable trust or optimization settings. Ordinary Serde,
enums and container verification remain on exact comparison. No protocol bytes,
canonical rejection rules, resource limits, allocation policy or ownership change.

A first unguarded prototype passed existing tests but was rejected after a new
counterexample. One custom field consumes two bytes but serializes one; another
consumes one but serializes two. With a copied blob between them, payload
`07 01 01 2a 09` decodes but re-encodes as `07 01 2a 2a 09`. Local comparisons
can look equal at shifted boundaries while skipping the wrong blob region.
The whole-type guard preserves `Noncanonical` for this input. The regression
fails on the unguarded prototype and passes on the retained implementation.
Additional tests cover copied-record length/scalar prefix minimality, every
truncation, payload changes and zero/word/varint/4 KiB boundaries.

## Complete pending 0.5.22 rerun

This complete rerun includes shared header validation and symbol stripping for
both codecs in all five cohorts. It supersedes the earlier five-cohort run for
current measurements; that run remains in [pre-stripping evidence](evidence/final-comparison-0522/identity.json).

This is the full comparison before guarded finite verification, after shared finite output setup, shared
header validation, symbol stripping and removal of per-type debug formatting
from Wasm assertions. Other measurement sections below retain historical results. Production has one Postcard encoding and no size/speed
selector. `wire-native`, `wire-finite` and `wire-borrowed` label existing admission
and ownership APIs, not alternate codecs. They remain separately measured;
this rerun does not remove or merge their public contracts. Compiler opt-z and
opt-3 are build profiles of the same implementation, not selectable Wire modes.

Uncompressed Wasm and the code section drive footprint decisions. Gzip has no
role in candidate selection or the headline comparison. Sizes below cover whole
fixture canisters including input construction, identity and dispatch; they do
not isolate derive code or predict savings in a deployed consumer.

| opt-3 fixture | Wire code section | Candid code section | Wire total Wasm | Candid total Wasm |
| --- | ---: | ---: | ---: | ---: |
| Standard 114 cases | 264,599 | 788,971 | 314,412 | 860,566 |
| Toko-shaped 10 cases | 103,529 | 559,369 | 114,105 | 592,130 |
| 1 derived type | 51,371 | 385,608 | 78,183 | 433,476 |
| 10 derived types | 106,408 | 628,148 | 134,030 | 678,550 |
| 100 derived types | 363,691 | 2,010,763 | 398,228 | 2,079,944 |

All sizes are bytes. From 1 to 100 types, code grows 312,320 bytes for Wire and
1,625,155 for Candid. The cohort has 100 nominal records but sixteen structural
width patterns, so this is not extrapolatable to arbitrary application schemas.
At opt-z the 100-type code sections are 286,569/1,362,761 bytes, with total Wasm
305,391/1,416,775 bytes. Full results for both profiles are retained below.

Selected opt-3 results retain each operation's admission/canonicality work:

| Input / API | Wire / Candid payload bytes | Wire / Candid encode instructions | Wire / Candid decode instructions |
| --- | ---: | ---: | ---: |
| Billing / ordinary Serde | 77 / 171 | 5,408.25 / 71,447.25 | 9,386.5 / 70,250.5 |
| 1,000-token filter / ordinary Serde | 19,617 / 19,663 | 170,638.25 / 172,445.25 | 2,039,087.5 / 2,925,759.5 |
| 50-card projection / ordinary Serde | 10,798 / 11,004 | 219,873.25 / 213,439.75 | 743,614.5 / 1,845,558.5 |
| Derived record 000 in 100-type cohort | 43 / 99 | 1,636.25 / 57,479.5 | 2,883.5 / 54,304.5 |
| 1 MiB byte-oriented blob / finite owned | 1,048,590 / 1,048,588 | 1,050,174 / 2,115,305 | 2,524,473 / 1,065,406 |
| 1 MiB generic byte vector / ordinary Serde | 1,048,590 / 1,048,588 | 50,332,657 / 2,112,185 | 68,687,385 / 119,553,449 |
| 1,024-hash manifest / finite | 32,830 / 33,883 | 1,270,583.25 / 155,989.25 | 2,097,685.5 / 4,233,898.5 |

Across all 100 small derived records, Candid/Wire instruction ratios range from
25.51–35.13 for encoding and 14.74–18.83 for decoding at opt-3. This does not
mean Wire wins every workload. Candid uses 2.93% fewer encode instructions for
the card projection, about 2.37× fewer instructions for owned 1 MiB blob decode,
23.83× fewer for generic 1 MiB byte-vector encoding, and 8.13× fewer for finite
manifest encoding. The generic-vector and blob fixtures use different Serde
representations; applications should not treat their costs as interchangeable.

Native time is a separate metric. The two opt-3 native run medians for billing
encode are 121.8–124.0 ns Wire versus 2,186.6–2,233.2 ns Candid; decode is
175.0–179.9 ns versus 1,669.2–1,674.5 ns. Projection encode is 3.69–3.72 µs
versus 4.93–4.94 µs, despite Candid's IC encode advantage. Owned finite 1 MiB blob
decode is 33.53–33.69 µs versus 18.18–18.49 µs, favoring Candid on both metrics.
Host results are unpinned/unisolated Linux observations, not latency guarantees.

Allocation and peak-live-heap passes are separate from timing. Billing decode
uses no Wire allocations versus Candid's 67 allocations plus 4 reallocations.
The card projection uses 351+4 versus 864+3 requests, but its Wire peak live heap
is higher: 24,866 versus 22,613 bytes. Borrowed Wire 1 MiB blob decode takes
824 IC instructions with no value allocation, but is not an ownership-equivalent
comparison to Candid's owned result; Candid borrowing was not measured.

### Rerun evidence and reproduction

The rerun uses Rust 1.99.1, locked dependencies (Candid 0.10.39/Postcard 1.1.3),
Linux x86_64 native and wasm32-unknown-unknown IC builds, LTO, one codegen unit,
panic abort and stripped symbols (`strip = "symbols"`) for both codecs.
Both opt-z/opt-3 profiles have two native runs with eleven calibrated samples (minimum 2 ms batches), separate allocation
and DHAT heap passes, and three IC samples per operation through PocketIC 16.1.0.
IC counts divide by four repetitions through 64 KiB or one above it, without
subtracting counter/loop overhead. Input construction is outside timed loops.
The [consumer study](candid-consumer-comparison.md) owns Toko provenance, ULID
representation, preformatted decimal and projection limitations.

All 5,278 IC rows have three identical samples. Timing, allocation, heap and IC
passes have matching input/frame identities, and all five cohorts use the same runtime source.
The source is released `91244e9` plus the pending serialization inlining, shared
output setup, shared header validation and benchmark corrections, not an unmodified
released package.
No runtime or protocol changes were made during this rerun. The benchmark itself
checks roundtrips and malformed frames; full CI and macOS were not rerun.

- [Payload, IC, native, allocation and heap summary](evidence/stripped-full-comparison-0522/summary.csv).
- [Uncompressed artifact sizes, code sections, hashes and observed build times](evidence/stripped-full-comparison-0522/artifacts.csv).
- [Compiler/source identities and parameters](evidence/stripped-full-comparison-0522/identity.json).
- [Complete raw samples and identities](evidence/stripped-full-comparison-0522/raw.json.gz).

Build times are single dependency-warm observations, not isolated macro-expansion
or clean-build benchmarks. The raw sample archive is compressed for storage;
that compression is unrelated to the Wasm optimization criteria.
Full source snapshots, logs and Wasm files remain in `target/stripped-full-0522-*`.
To reproduce with fresh output directories and an explicitly prepared server:

```sh
POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh target/new-standard
CORPUS_FEATURE=toko POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh target/new-toko
CORPUS_FEATURE=derive-scale-1 POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh target/new-derive-1
CORPUS_FEATURE=derive-scale-10 POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh target/new-derive-10
CORPUS_FEATURE=derive-scale POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh target/new-derive-100
```

## Owned blob comparison-cost experiment — pending 0.5.22

**No runtime change retained.** Three probes against the complete stripped rerun
above isolate bulk canonical comparison and test two exact-equality alternatives.
The baseline remains the selected implementation; neither an alternate codec nor
a relaxed canonicality mode was added. Protocol bytes and acceptance are unchanged.

For the same 1 MiB random blob, finite-owned decode instructions are:

| Implementation | opt-z | opt-3 | opt-3 change |
| --- | ---: | ---: | ---: |
| Selected Wire baseline | 2,574,508 | 2,524,473 | — |
| Skip bulk comparison: invalid diagnostic control | 1,050,656 | 1,049,794 | −58.42% |
| Exact 256-byte unrolled comparison: rejected | 2,422,952 | 2,409,790 | −4.54% |
| Exact short-circuit word comparison: rejected | 2,787,494 | 2,786,614 | +10.38% |
| Candid owned decode | 1,073,875 | 1,065,406 | — |

Wire/Candid frame sizes remain 1,048,590/1,048,588 bytes. The diagnostic control
replaces only the shared bulk equality helper with `true`; it still allocates,
copies and performs the surrounding decode/serialization checks. Its native unit
test fails at length 64, mismatch position 0. It is not a usable fast decoder or
a performance promise for a future implementation. The measured delta attributes
about 58% of current opt-3 owned-blob decode cost to that bulk comparison path,
including its code-generation effects.

Checking a canonical length prefix alone cannot replace this comparison under
the existing custom-Serde contract. The maintained canonical-bytes regression
contains a decoder that normalizes blob contents: structurally valid bytes can
deserialize successfully yet serialize differently. A structural Postcard validator
would miss this. Safely removing the scan requires a stronger roundtrip guarantee
for the admitted type implementations, or a deliberate change to that contract;
schema metadata alone supplies no such guarantee. This experiment does not implement
or qualify such a redesign.

The 256-byte probe grows whole-fixture Wasm/code by 477 bytes at opt-z and 570
at opt-3. It also regresses opt-z 128-byte native blob decode from 2,047.5 to
2,273.5 instructions (+11.04%). The short-circuit probe grows those sizes by
15/17 bytes at opt-z/opt-3 and regresses large-blob decode by 8.27%/10.38%.
Both exact candidates pass the existing equality, runtime, native and borrowed
checks; neither warrants adoption. The restored baseline passes all 43 focused
tests. No new tests, dependencies or runtime symbols were needed.

[Instruction results](evidence/owned-blob-0522/instructions.csv),
[artifact sizes and hashes](evidence/owned-blob-0522/artifacts.csv) and
[raw samples, source patches and test logs](evidence/owned-blob-0522/raw.json.gz)
retain the experiment. The archive includes baseline/candidate source hashes,
compiler identity and patches against the full rerun's source snapshot.
All 11,352 rows across baseline and three probes have three agreeing IC samples;
input/frame hashes, frame lengths and repetition counts match for every key.
Candid artifacts are reused byte-for-byte from the baseline. Sizes describe the
complete standard 114-case fixture, including its harness, not isolated helpers.

Reproduction uses the full rerun's `target/stripped-full-0522-standard/source`,
applying one archived patch at a time. Build `codec-wasm` with `--locked`,
`--features wire`, `--target wasm32-unknown-unknown` and each of the `evaluation`
and `benchmark-speed` profiles; copy the resulting artifacts alongside baseline
Candid artifacts under the driver's expected filenames. Run the unchanged standard
`codec-instructions` driver with the prepared PocketIC 16.1.0 server, as below.
Rust 1.99.1, LLVM 23.1.1, LTO, one codegen unit, panic abort and symbol stripping
match the baseline. The driver measures three samples, four operations per sample
through 64 KiB and one above, without overhead subtraction. Local artifacts live
under `target/owned-blob-0522-{ablation,unroll,shortcircuit}`. Native timings,
allocator passes, consumer cohorts, full CI and macOS were not rerun for these
rejected Wasm-only probes. Production source is restored exactly to its starting
dirty-tree content; earlier pending changes are preserved.

## Run and reproduce

- `make check-benchmarks`: semantic roundtrips, stable re-encoding, damaged-header,
  truncation and trailing-byte rejection, buffer-reuse equivalence, lint and MSRV.
- `make evaluate-codecs`: two independent native runs per profile, allocator and
  peak-heap passes, separate linked Wasm artifacts and deterministic gzip sizes.
- `POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-codecs`: also measure
  actual IC instruction costs through the benchmark-only PocketIC driver.
- `make evaluate-codec-instructions`: run that driver against the existing default
  evidence directory (requires `POCKET_IC_BIN` and no previous instruction output).
- `bash scripts/ci/evaluate-codecs.sh target/another-run`: choose a new directory.
  `RUNS=2 SAMPLES=11 BATCH_MS=2` are the defaults. Output is never overwritten.
  Wasm artifacts are selected from Cargo's JSON build messages, honoring configured
  target directories. Missing/ambiguous artifacts fail instead of using a guessed path.
  Reports reject empty runs, duplicate rows and missing Wire/Candid partners before
  creating the CSV. Wire-only reuse rows require a corresponding fresh encode row.

The two profiles use LTO, one codegen unit, aborting panics and stripped debug
information. `evaluation` selects opt-z; `benchmark-speed` selects opt-3. Keep
profiles separate. Prepare Rust 1.99.1 and `wasm32-unknown-unknown`; `gzip`, `shasum`,
Bash and the ordinary Cargo tools must already be available. Linux measurements
are not evidence of macOS execution.

## Corpus and comparison boundary

The deterministic 114-case corpus covers fixed-width signed/unsigned integer
families and varint boundaries, 128-bit extremes, finite float edge values, booleans,
unit, tuples, options, nested options, results, principals, fixed hashes, UTF-8 and
ASCII text, random/zero blobs through 1 MiB, numeric vectors, sorted maps, text
pairs, relay batches, array/packed blob manifests, variants and recursive records.

`wire-native` measures complete framed `native::encode_request` and
`native::decode_request` calls, including runtime budgets and canonical validation.
`candid` measures `encode_one` and IC Platform's bounded `decode_exactly_one`.
Both represent the same Rust values. Byte-oriented Candid fixtures call its
standard `serialize_blob` primitive; generic byte vectors are separate cases.
The explicit blob implementation fixes the intended API choice without changing
Candid's encoded format. The pinned Candid also optimizes fixed-primitive vectors
on little-endian targets; generic-vector cases retain that upstream optimization. Candid carries its type information; Wire
carries its method/frame header and depends on the agreed schema. Neither framing
cost is removed. Their compatibility and resource-limit semantics differ.

Additional `wire-finite` text/blob, relay-batch, manifest and variant lanes measure
the derived bounded contract API.
`wire-borrowed` blob lanes check input aliasing and measure borrowed decoding.
`encode-reuse` is an additional Wire capability, compared separately from fresh
encoding. These lanes are not relabelled as equivalent Candid ownership/reuse APIs.
Candid borrowing and alternative output-buffer APIs are not measured here.

Each lane first asserts decoded semantic equality. Finite and borrowed cases also
check exact equality with the native Wire frame. Input construction, equality,
file I/O and reporting remain outside timing and memory measurement. Encoding
includes output destruction; owned decoding includes decoded-value destruction.
The timed loop includes black-box barriers, indirect calls and result checks.

Both decoders admit at most 2 MiB. Wire permits 256 MiB allocation budget, 8 Mi
nodes and depth 64. Candid permits 128 Mi decoding/skipping work units, 4096 type
entries and 64 KiB headers, with compact errors. These are different kinds of
limits, not equivalent sandbox guarantees. Wire's conservative allocation allowance
is an admission budget, not a measurement of actual heap use. All values and complete frames are
saved so inputs can be independently inspected.

## Metrics and their limits

- Encoded bytes, fresh output capacity and frame SHA-256, paired by input SHA-256.
- Native nanoseconds/op: calibrated batches of at least 2 ms (up to 1,048,576
  repetitions), 11 samples, minimum/median/maximum and p95 of batch means.
  The p95 is not request-tail latency. Throughput uses encoded bytes/second.
- Fresh encode, owned/labelled-borrowed decode, warm Wire output reuse, invalid
  header rejection and truncated-frame rejection measured separately.
- Allocation/deallocation/reallocation calls and requested bytes, measured with
  `stats_alloc` in a separate executable from the timing run.
- Peak live blocks/bytes, total allocation and retained bytes, measured using DHAT
  in another executable. Peaks are additional live allocations within one warmed
  operation; pre-existing input/fixture/reuse-buffer storage is excluded. These
  numbers are not RSS, Wasm linear-memory peaks or allocator usable-size totals.
- Native executable bytes; complete linked Wasm bytes and `gzip -n -9` deployment
  bytes, with separate Wire/Candid artifacts for both profiles. Whole fixtures
  include workload construction, validation and the measurement entrypoint. They
  are not isolated codec symbols or an application's incremental deployment cost.

`comparison.csv` reports each matched Wire/Candid case, labelled by Wire lane,
without a misleading cross-workload average. Negative change percentages favor Wire. Speed ratio is
Candid median time divided by Wire median time. Memory results and all raw timing
samples remain in `samples.jsonl`; IC counts also have `instructions/summary.csv`.
Identity files record compiler/host, lock/source hashes, executable identity,
budgets, flags and dirty checkout status.

The Wasm example exports a raw `benchmark` query accepting three little-endian
u32 values: case index, codec-local lane index and repetitions (1–1024). It returns
six u64 instruction deltas (fresh encode, decode, reuse, bad header, truncation,
loop control), one u64 encoded length, and the complete frame. Unavailable reuse
is marked `u64::MAX`. The driver takes three samples (four repetitions through 64 KiB, one above that
size to stay within query instruction limits), checks
every returned frame against the native corpus, and retains all raw counts. The
counter is `ic0.performance_counter(0)`; loop controls are reported, not subtracted.
Setup, validation and reply transport are outside the counter.

PocketIC and IC Testkit are native dev dependencies only. Testkit owns explicit
server compatibility admission and lifetime; the driver never downloads a server.
Use Testkit's `ic-testkit-server setup` and `check` tooling to prepare/verify a
server, then pass its absolute path as `POCKET_IC_BIN`. Execution requires local
process/socket access. Evidence records the server version and executable hash,
both codec artifact hashes, source/lock hashes and repetitions. These measurements
are IC instruction counts, not estimates from native time or Wasm fuel.

PocketIC 16.1.0 pins `thiserror` 2.0.18. Cargo therefore resolves the shared lockfile
from 2.0.21 to 2.0.18, including Postcard's transitive error dependency. Runtime
regressions are checked with that resolution. No published crate gains a Candid,
PocketIC, profiler or Testkit dependency; published library consumers resolve their
own compatible dependency versions.

Cold process/type-cache startup, end-to-end canister latency, concurrency, cycles,
peak Wasm memory, browser performance and build-time attribution are not measured.
No universal winner or deployment claim follows from this microbenchmark suite.

## Measured baseline results — 2026-10-22 (before Wasm optimization)

Measured on Linux x86-64, AMD Ryzen Threadripper 7970X, Rust 1.99.1 / LLVM 23.1.1,
Postcard 1.1.3, Candid 0.10.39, IC Platform 0.6.7 and PocketIC server 16.1.0.
The host was not isolated or affinity-pinned. These are current dirty-worktree
measurements, not a released-version comparison. Base commit: `83f90a4779aa3cdaf016484dc096322708cd0cdf`.
The fixture SHA-256 is `cf10e2a23c68044a54af50853d20eb78e065c20f6ec9466c49ab66d239880947`.
The lock SHA-256 is `bd334989af69c3e8a5099ba584ed402d5fc163733e7fd0461ff39e35dfbf7e75`.

Raw evidence is retained at `target/codec-comparison-2026-10-22-v1/`: four native
runs, four separate memory passes, four Wasm artifacts, 2,838 IC result rows,
identities, complete frames and a source snapshot. Earlier directories are pilot
runs and are not used below. Each native run has 11 samples; the following opt-3
figures pool the 22 batch means from both runs and report their median in µs/op.

| Workload | Wire native encode | Candid encode | Wire native decode | Candid decode |
| --- | ---: | ---: | ---: | ---: |
| Unit | 0.028 | 0.191 | 0.005 | 0.176 |
| Scalar record | 0.080 | 1.488 | 0.073 | 0.977 |
| 1 MiB bulk blob | 17.562 | 756.325 | 34.104 | 18.290 |
| 1 MiB generic byte vector | 1,526.566 | 774.749 | 4,863.915 | 2,116.519 |
| 128-message relay batch | 3.434 | 3.186 | 14.314 | 26.162 |
| 1,024-hash array manifest | 78.248 | 7.913 | 105.988 | 99.573 |
| 1,024-hash packed manifest | 0.584 | 2.561 | 1.066 | 1.566 |

Actual opt-3 IC instructions/op (three agreeing samples for every operation):

| Workload | Wire native encode | Candid encode | Wire native decode | Candid decode |
| --- | ---: | ---: | ---: | ---: |
| Unit | 385.25 | 6,016.75 | 175.50 | 6,325.00 |
| Scalar record | 2,555.25 | 52,647.00 | 2,947.50 | 37,600.50 |
| 1 MiB bulk blob | 1,049,885.00 | 2,115,305.00 | 2,753,963.00 | 1,065,406.00 |
| 1 MiB generic byte vector | 50,332,657.00 | 2,112,185.00 | 69,211,642.00 | 119,553,449.00 |
| 128-message relay batch | 225,610.25 | 178,577.50 | 496,474.50 | 986,233.50 |
| 1,024-hash array manifest | 3,143,287.25 | 155,989.25 | 6,209,665.50 | 4,233,898.50 |
| 1,024-hash packed manifest | 35,636.25 | 112,280.25 | 90,219.50 | 73,259.50 |

The finite Wire path matters: the 128-message relay batch costs 158,377.25 encode
and 436,236.50 decode instructions, versus native Wire's 225,610.25 / 496,474.50.
The finite array manifest costs 1,270,559.25 / 2,097,658.50, improving on native Wire
but still losing to Candid on encoding. These lanes are retained in the raw results.

For the 1 MiB bulk blob, both owned Wire decoders allocate one 1,048,576-byte block;
Candid allocates 25 blocks totalling 1,050,661 requested bytes, with a 1,049,380-byte
peak. Borrowed Wire decoding allocates zero bytes and costs 806 IC instructions;
it does not scan or otherwise consume the application payload. Owned Wire decoding
costs more instructions than Candid despite fewer allocations. Native time also
favors Wire on packed-manifest decoding while IC instructions favor Candid there.
Use the metric matching the deployment target.

Complete linked fixture sizes (not application/library incremental sizes):

| Profile | Codec | Wasm bytes | gzip bytes |
| --- | --- | ---: | ---: |
| evaluation | wire | 404,519 | 112,569 |
| evaluation | candid | 783,009 | 219,766 |
| benchmark-speed | wire | 486,892 | 135,643 |
| benchmark-speed | candid | 1,067,801 | 279,653 |

Payload size is also workload-dependent: unit is 11 Wire bytes versus 7 Candid;
the scalar record is 48 versus 100; the 1 MiB blob is 1,048,590 versus 1,048,588;
the array manifest is 32,830 versus 33,883; the packed manifest is 32,831 versus
32,858. Do not substitute aggregate speed or size averages for those tradeoffs.

The measurements identify generic byte/array traversal and owned large-blob decode
as worthwhile Wire optimization candidates. They do not establish a safe runtime
change; this batch changes no encoding, admission, canonicalization or hashing rule.

## Wasm comparison optimization — pending 0.5.20

The retained change compares 32-byte bulk blocks instead of 16-byte blocks, then
compares the remainder in 8-byte words before its final byte tail. The small-write
branch and native slice comparison stay unchanged. This preserves exact canonical
bytes, limits, typed failures, hashing and retries; consumers need no migration.

Actual IC instructions for owned `wire-native` decoding, using the same corpus,
compiler, lockfile, limits and profiles as the baseline above:

| Profile / workload | Before | After | Change |
| --- | ---: | ---: | ---: |
| opt-z / 127-byte blob | 2,274.5 | 2,098.5 | −7.74% |
| opt-z / 4 KiB blob | 14,505.5 | 12,743.5 | −12.15% |
| opt-z / 1 MiB blob | 3,278,792 | 2,820,070 | −13.99% |
| opt-z / packed manifest, 1,024 hashes | 111,263.5 | 96,957.5 | −12.86% |
| opt-3 / 127-byte blob | 1,801.5 | 1,609.5 | −10.66% |
| opt-3 / 4 KiB blob | 11,974.5 | 11,729.5 | −2.05% |
| opt-3 / 1 MiB blob | 2,753,963 | 2,688,438 | −2.38% |
| opt-3 / packed manifest, 1,024 hashes | 90,219.5 | 88,182.5 | −2.26% |

| Whole Wire fixture | Before | After | Delta |
| --- | ---: | ---: | ---: |
| opt-z Wasm bytes | 404,519 | 404,648 | +129 (+0.032%) |
| opt-z gzip bytes | 112,569 | 112,617 | +48 (+0.043%) |
| opt-3 Wasm bytes | 486,892 | 486,917 | +25 (+0.005%) |
| opt-3 gzip bytes | 135,643 | 135,643 | 0 |

All 2,838 result rows match input/frame hashes; each operation has three agreeing
instruction samples. Of these, 67 improve, seven regress and 2,764 are unchanged.
The seven regressions are opt-3 owned decodes, each adding exactly three instructions
(at most 0.22%, on a 128-byte finite blob). Scalar records, generic byte vectors,
borrowed blob decoding and all Candid controls are unchanged. The owned 1 MiB
blob remains more expensive than Candid: 2,688,438 versus 1,065,406 opt-3 instructions.
No native timing improvement is claimed: rebuilt native benchmark executables
are byte-identical to the baseline under both profiles, so native timing was not
rerun. These are codec microbenchmarks,
not end-to-end latency, constant-time guarantees or application footprint estimates.

An early-exit candidate saved at most 101 Wasm bytes but increased large owned-blob
decode instructions by 12–26%; it was rejected. A 32-byte candidate with a bytewise
tail regressed small inputs, and sharing its word tail with the small-write branch
also regressed small controls. The retained version keeps that branch separate.

[Changed rows and unchanged controls](evidence/wasm-comparison/instructions.csv)
retain absolute counts and all three samples. [Artifact sizes](evidence/wasm-comparison/wasm.csv),
[source/artifact identities](evidence/wasm-comparison/identity.json) and the
[baseline-to-candidate patch](evidence/wasm-comparison/candidate.patch) identify the
comparison. Full raw data and source snapshots remain in
`target/codec-comparison-2026-10-22-v1/` and `target/wasm-compare-final-2026-10-22/`.
The final Wasm binaries are byte-identical to the measured `wasm-compare-split`
experiment; its instruction output is copied unchanged into the final directory.

To reproduce from this batch's source, make two disposable copies with the same
Cargo.lock; reverse the retained patch in the baseline copy only. In each copy,
run `POCKET_IC_BIN=/absolute/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh
/absolute/path/to/new-evidence` (one shell command). Compare matching profile,
case, lane and operation; use `gzip -n -9` for deployment sizes. The snapshots
retain the incoming dirty work, so bare base commit `83f90a4` alone is insufficient.
Runtime tests cover each mismatch position through 161 bytes with an unaligned
input start; runtime tests, strict Clippy, native/Wasm Rust 1.91 checks and the
benchmark corpus pass. No full CI or macOS execution is claimed.

## Verified rerun after benchmark-tooling fixes

The full suite completed again at `target/codec-comparison-2026-10-22-rerun/`:
two native runs per profile (11 samples each), separate allocation/peak-heap passes,
four Wasm artifacts and 2,838 IC result rows. The validated report contains 2,352
matched comparisons. `CARGO_BUILD_TARGET_DIR` selected
`/tmp/ic-wire-bench-configured-target`, a symlink to the existing build cache;
retained Cargo JSON messages show the artifact paths actually used. No artifact
location was inferred from the default `target/` directory.

Every IC instruction count and input/frame hash matches the retained optimized
run; all three samples agree. Both profiles' allocator/peak-heap results match the
original suite. Native executables are byte-identical to the original baseline;
new timing variation is not attributed to these reporting-only fixes.
Compiler, dependencies, corpus and codec limits remain those recorded above.

| Profile / codec | Linked Wasm bytes | gzip bytes |
| --- | ---: | ---: |
| opt-z Wire | 404,648 | 112,617 |
| opt-z Candid | 783,005 | 219,770 |
| opt-3 Wire | 486,917 | 135,643 |
| opt-3 Candid | 1,067,801 | 279,653 |

The fresh opt-z Candid artifact differs from the original by −4 Wasm / +4 gzip
bytes; its measured instruction counts are unchanged. This tiny artifact change
is not a claimed codec optimization. Wire artifacts match the retained optimized
hashes exactly. These remain whole-fixture sizes, not incremental library costs.

Selected opt-3 native rerun medians (22 batch means pooled, µs/op):

| Workload | Wire encode | Candid encode | Wire owned decode | Candid decode |
| --- | ---: | ---: | ---: | ---: |
| Scalar record | 0.081 | 1.517 | 0.074 | 0.994 |
| 1 MiB bulk blob | 18.435 | 918.458 | 37.803 | 21.098 |
| 1 MiB generic byte vector | 1,607.167 | 813.745 | 4,955.215 | 2,167.213 |
| 1,024-hash array manifest | 82.117 | 8.010 | 111.447 | 101.459 |
| 1,024-hash packed manifest | 0.592 | 2.456 | 1.079 | 1.653 |

The host was not isolated or pinned. IC instructions remain the stronger evidence
for the Wasm comparator optimization: opt-3 owned 1 MiB blob decode remains
2,688,438 for Wire versus 1,065,406 for Candid; borrowing remains 806 for Wire,
without an application payload scan. No universal speed winner is established.

`make check-benchmarks` now runs report/artifact regression tests as well as the
corpus, lint and MSRV checks. An incomplete report reproducer now exits nonzero
with a missing-partner error and creates no CSV. No protocol or runtime change
was made by these tooling fixes.

## Owned blob follow-up — pending 0.5.21

The retained candidate moves the Wasm bulk comparison into a non-inlined helper
and widens its blocks from 32 to 64 bytes. The eight-byte tail, scalar path,
exact-input-region borrowing shortcut and native slice equality remain. It still
copies owned payloads and validates their complete canonical serialization:
custom Serde visitors can transform data, so successful decoding alone cannot
replace that validation. Wire bytes, limits, error precedence and retry/hash
semantics do not change; no consumer migration is needed.

Baseline is released 0.5.20, commit `a5b80e6`, rebuilt with the same lockfile,
compiler, corpus and profiles as every candidate. Both profiles use LTO, one
codegen unit and stripped debug information. The existing PocketIC driver takes
three samples per operation and checks every reply frame against native fixtures.
All 2,838 rows match input/frame hashes and have agreeing repeated counts.

| Owned Wire decode | Before instructions | After instructions | Change |
| --- | ---: | ---: | ---: |
| opt-z / 127-byte blob | 2,098.5 | 2,171.5 | +3.48% |
| opt-z / 4 KiB blob | 12,743.5 | 11,821.5 | −7.24% |
| opt-z / 1 MiB blob | 2,820,070 | 2,574,348 | −8.71% |
| opt-z / packed manifest, 1,024 hashes | 96,957.5 | 89,315.5 | −7.88% |
| opt-3 / 127-byte blob | 1,609.5 | 1,650.5 | +2.55% |
| opt-3 / 4 KiB blob | 11,729.5 | 11,143.5 | −5.00% |
| opt-3 / 1 MiB blob | 2,688,438 | 2,524,652 | −6.09% |
| opt-3 / packed manifest, 1,024 hashes | 88,182.5 | 83,114.5 | −5.75% |

| Whole Wire fixture | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| opt-z Wasm | 404,487 | 404,687 | +200 |
| opt-z gzip | 112,534 | 112,643 | +109 |
| opt-3 Wasm | 486,916 | 487,243 | +327 |
| opt-3 gzip | 135,371 | 135,572 | +201 |

Across all rows, 75 improve, 34 regress and 2,729 are unchanged. The worst
percentage regression is the opt-z 127-byte blob (+73 instructions / 3.48%).
These small-input costs and additional code are the tradeoff for the larger
payload gains. Candid still wins owned 1 MiB decoding at opt-3: 1,065,406
instructions versus Wire's 2,524,652. No universal codec speedup is claimed.

Two alternatives were rejected. An inline 64-byte scan improves the 1 MiB case
8.7%/2.4% but adds 67/618 Wasm bytes at opt-z/opt-3 and regresses common controls
by up to 6.6%. A 32-byte scan using a u64 accumulator worsens the 1 MiB case
1.2%/9.8%, with −13/+418 Wasm bytes. Merely using a narrower Rust integer did
not produce a faster Wasm loop; no instruction-level cause is asserted.

[Changed rows and small controls](evidence/owned-blob-0521/instructions.csv)
retain all three counts and frame hashes; [artifact sizes](evidence/owned-blob-0521/wasm.csv)
cover the baseline and all candidates. Retained patches reconstruct the
[selected helper](evidence/owned-blob-0521/selected.patch), rejected
[inline 64-byte loop](evidence/owned-blob-0521/inline64.patch) and rejected
[u64 reduction](evidence/owned-blob-0521/u64.patch). Use disposable checkouts of
`a5b80e6`, apply one patch to the candidate only, and run the existing
`evaluate-codecs.sh` with an explicit `POCKET_IC_BIN` in each checkout and distinct
output directories. No dependency or compiler-setting change is required.
Full sources, artifacts and samples remain in `target/owned-blob-0521-baseline/`,
`target/owned-blob-0521-64/`, `target/owned-blob-0521-u64/` and
`target/owned-blob-0521-helper/`. The compiler and driver identities accompany them.

[Source/compiler/artifact identities](evidence/owned-blob-0521/identity.json)
include both native executable hashes. Rebuilding native benchmarks under both
profiles yields byte-identical executables before/after, so native timing was
not rerun and no native speedup is claimed. Runtime, corpus, Clippy and Rust 1.91
native/Wasm checks pass. Tests cover every mismatch position through 257 bytes
from an unaligned input start and preserve later allowance errors after a bulk
mismatch. Full CI and macOS were not run.
Final sources and rebuilt artifacts are also retained at
`target/owned-blob-0521-final/`. All four Wasm binaries are byte-identical to the
measured helper run; its instruction output is copied there unchanged.

## Allocation budget units — pending 0.5.21

The native Serde budget now stores whole eight-byte charge units. Previously,
each visit multiplied its charge by eight before subtracting from a byte budget.
Dividing the initial allowance once preserves admission, overflow refusal and
unused remainders while removing repeated checked multiplication. Protocol bytes,
canonical validation, consumer limits and typed errors are unchanged. An independent
regression test compares the old arithmetic through fractional budgets, `usize::MAX`
and sticky refusal. No consumer migration is required.

This comparison starts **after** the owned-blob helper above: release `a5b80e6`
plus its selected patch. Both profiles use the same compiler, lockfile, corpus,
LTO and single codegen unit. Rust 1.99.1 / LLVM 23.1.1, PocketIC 16.1.0 and the
114-case fixture are retained in [measurement identities](evidence/budget-units-0521/identity.json).
All 2,838 IC rows preserve input/frame hashes and have three agreeing samples.
419 improve, two regress and 2,417 are unchanged; Candid artifacts are unchanged.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| opt-z Wire Wasm bytes | 404,687 | 404,361 | −326 (−0.08%) |
| opt-z Wire gzip bytes | 112,643 | 112,554 | −89 (−0.08%) |
| opt-3 Wire Wasm bytes | 487,243 | 486,447 | −796 (−0.16%) |
| opt-3 Wire gzip bytes | 135,572 | 135,350 | −222 (−0.16%) |
| opt-3 encode 128 text pairs, IC instructions | 134,609.25 | 130,513.25 | −3.04% |
| opt-3 unit encode, IC instructions | 385.25 | 385.25 | unchanged |
| opt-3 reject truncated 16-entry map, IC instructions | 41,585.75 | 41,595.75 | +0.024% |
| opt-3 reject truncated 128-entry map, IC instructions | 586,208.75 | 586,330.75 | +0.021% |

Allocation and heap measurements are identical in all 1,158 rows per profile
and measurement mode. Native timing ran twice per profile, with 11 samples per
operation per run on an unpinned Threadripper 7970X. Median candidate/baseline
ratios across Wire-native rows were 0.915/0.920 at opt-z and 0.977/0.989 at opt-3.
However, unchanged Candid controls also measured about 4% faster in both profiles:
these native timings do not establish a causal or universal speedup.
[Absolute native controls and repeated samples](evidence/budget-units-0521/native-controls.csv)
include small values, text pairs and both blob representations.

The generic byte-vector gap remains: opt-3 1 MiB encoding still costs 50,332,657
IC instructions, versus 1,049,868 using byte-oriented Serde encoding (about 48×
less). These are distinct representations in the suite, not a before/after runtime
speedup. Generic and byte-oriented fixtures now assert identical frames at 4 KiB
and 1 MiB. Consumers can select `serde_bytes` for native blob fields or finite
`Bytes`/`BytesRef`; limits must be qualified because traversal accounting differs.
Candid still wins some cases, including owned large-blob decoding.

[Changed IC rows and controls](evidence/budget-units-0521/instructions.csv),
[Wasm sizes](evidence/budget-units-0521/wasm.csv) and the
[selected runtime patch](evidence/budget-units-0521/selected.patch) retain the
comparison. To reproduce, create two disposable checkouts of `a5b80e6`, apply the
owned-blob selected patch to both and this patch to the candidate only. Run
`POCKET_IC_BIN=/path/to/pocket-ic bash scripts/ci/evaluate-codecs.sh OUTPUT`
in each with distinct output directories and identical tooling. Use the retained
`target/budget-units-0521-baseline/evaluation-native-1/Cargo.lock` in both:
the working lockfile differs from release 0.5.20 and its hash is in the identities.
Full samples remain
in `target/budget-units-0521-baseline/` and `target/budget-units-0521-candidate/`.
Baseline IC samples were reused from `owned-blob-0521-final` only after all four
rebuilt Wasm hashes matched; candidate IC measurements are fresh.

`make fmt test lint check-msrv check-benchmarks` passes, including the arithmetic
regression and frame equivalence assertions. Full CI and macOS were not run.

## Wasm serialization inlining — pending 0.5.22

Two Wasm-only inlining annotations expose native value admission and compound
field forwarding to the caller's optimizer. The selected change keeps both;
there is no new serialization mode. Wire bytes, node/depth/allocation charges,
canonical validation, errors and retry/hash semantics remain unchanged. No
consumer migration is required. Native targets do not receive either annotation.

Baseline is release 0.5.21 (`91244e9`), freshly rebuilt with its lockfile. All
variants use Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown, LTO, one codegen
unit, panic abort and stripped debug information. `evaluation` selects opt-z;
`benchmark-speed` selects opt-3. PocketIC 16.1.0 measures the same 114-case corpus,
three agreeing samples per operation, four repetitions through 64 KiB and one
above it. Limits and codec parameters are those of the suite described above.
All 2,838 rows retain identical input/frame hashes. Candid binaries and counts
are identical between variants. This is IC instruction evidence, not native time.

| opt-3 IC instructions | Wire before | Wire selected | Candid |
| --- | ---: | ---: | ---: |
| 1,024-hash array manifest encode | 3,143,270.25 | 2,513,359.25 (−20.04%) | 155,989.25 |
| 1,024-hash array manifest decode | 6,090,868.50 | 4,944,774.50 (−18.82%) | 4,233,898.50 |
| 128-message relay encode | 221,386.25 | 220,234.25 (−0.52%) | 178,577.50 |
| 128-message relay decode | 494,042.50 | 487,387.50 (−1.35%) | 986,233.50 |
| 1 MiB generic byte-vector encode | 50,332,657 | 50,332,657 (unchanged) | 2,112,185 |

| Whole Wire fixture | Before bytes | Selected bytes | Delta |
| --- | ---: | ---: | ---: |
| opt-z Wasm | 404,522 | 399,073 | −5,449 (−1.35%) |
| opt-z gzip | 112,658 | 112,368 | −290 (−0.26%) |
| opt-3 Wasm | 486,448 | 484,879 | −1,569 (−0.32%) |
| opt-3 gzip | 135,554 | 135,164 | −390 (−0.29%) |

Across all rows, 464 improve, 200 regress and 2,174 are unchanged. The worst
percentage regression is opt-z empty array-manifest decoding: 7,338.5 → 7,491.5
instructions (+153 / 2.08%). The largest absolute regression is opt-z relay
encoding/reuse (+768 instructions, about 0.19%). Opt-z large-manifest encoding
improves 4.42% and decoding 2.43%. This is a measured tradeoff, not universal
improvement or a claim that Wire always beats Candid.

Two narrower variants were rejected. Inlining value admission alone shrinks
Wasm 6,270/2,694 bytes at opt-z/opt-3, but loses the opt-3 manifest encode gain
and regresses its decoding 1.85%. Inlining compound forwarding alone changes
Wasm by −3,246/+52 bytes and leaves opt-3 large-manifest costs unchanged.
The combination gives the useful manifest improvement with smaller artifacts.

[Instruction changes and controls](evidence/wasm-inlining-0522/instructions.csv)
retain all changed selected rows, small controls, Candid controls and repeated
samples. [Sizes](evidence/wasm-inlining-0522/wasm.csv) and
[identities](evidence/wasm-inlining-0522/identity.json) cover all four builds.
Apply the [selected patch](evidence/wasm-inlining-0522/candidate.patch),
[value-only patch](evidence/wasm-inlining-0522/bounded.patch) or
[compound-only patch](evidence/wasm-inlining-0522/compound.patch) independently
to disposable `91244e9` checkouts. Run `evaluate-codecs.sh` with the same explicit
`POCKET_IC_BIN` and distinct output directories to reproduce the standard suite.
This experiment rebuilt both-profile Wasm artifacts and ran `codec-instructions`
for each variant; it did not rerun native timing, allocations or peak heap, and
makes no new claims for those metrics. Full source snapshots, Cargo artifact
messages and all samples remain in `target/inline-0522-{baseline,candidate,bounded,compound}/`.

Focused runtime tests, strict Clippy, Rust 1.91 native/Wasm checks and the benchmark
roundtrip corpus pass. Full CI and macOS were not run. Generic primitive-vector
encoding and owned large-blob decoding remain important Candid wins; byte-oriented
schemas help some consumers but do not remove those comparison cases.

## Consumer-shaped and derive-scaling cohorts

The [Toko-shaped and 1/10/100-type study](candid-consumer-comparison.md) adds
separately selected corpora, preserving the standard suite and distinguishing
actual DTO shapes, a token-card projection and synthetic derive scaling.
