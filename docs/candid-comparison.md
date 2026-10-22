# Candid comparison suite

`ic-wire-benchmarks` is unpublished. All codecs and profilers are dev dependencies;
Candid is consumed through IC Platform. Production IC Wire remains bounded
Postcard, with no Candid dependency or protocol change.

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
