# Evaluation scope

The maintained qualification exercises the selected Postcard codec, its
generated TypeScript implementation and an isolated Candid comparison suite.
Endpoint adapters and browser SDKs remain consumer-owned. The unpublished codec
benchmark package may use PocketIC for isolated IC instruction measurements;
it does not qualify consumer endpoint integrations.

The released 0.6.0 Serde hard cut removed the finite Wire API. Sections below recording
finite admission and WireSchema experiments are historical research; the
[current production comparisons](candid-comparison.md)
owns shipped-codec IC, frame and Wasm measurements. The retired protocol-study
runner is available only in the archived finite revision linked by those studies;
it is no longer maintained alongside the production Serde suite.

## Maintained checks

| Target | Evidence |
| --- | --- |
| `test` | Runtime/native bounds, canonical bytes, typed failures and retry buffers |
| `check-typed-diagnostics` | Actual compile-fail subjects and a passing control |
| `check-typescript` | Generated types, Rust vectors and differential malformed-input checks |
| `check-benchmarks` | Matched Candid/Wire corpus, malformed-frame checks, lint and MSRV |
| `check-consumer-benchmarks` | Toko-shaped and 1/10/100-type derive corpus, lint and MSRV checks |
| `evaluate-codecs` | Repeated native timing, allocator counts, peak live heap and Wasm footprint |
| `check-qualification` | Allocation/borrowing fixture tests, strict linting and compiler-floor checks |
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.91 floor |

The unpublished `ic-wire-qualification` package owns the maintained allocation,
borrowing and buffer-reuse fixtures plus the IC Build compiler-diagnostic runner.
Earlier alternative-codec suites were retired in 0.5.19; their reports and raw
evidence remain historical. The new [Candid suite](candid-comparison.md) is owned
by the unpublished `ic-wire-benchmarks` package. See [retired comparisons](performance.md#retired-alternative-codec-comparisons)
for the last source revision and reproduction boundary. Raw Postcard controls in
the retained protocol probes omit Wire admission/canonicality and are not runtime
modes. Native timings, IC instructions and artifact sizes remain distinct evidence.

Run focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` scaffold gate is reserved for an explicit local request
or the maintainer's release flow. Wire does not qualify sibling endpoints,
authentication, storage formats or delivery policy.

See [borrowing, buffers and bounded layouts](allocation-comparison.md) for the
44-case native/IC experiment and its consumer-schema tradeoffs.

The 0.5 borrowed-contract [allocation probe](evidence/borrowed-contracts-0.5/allocations.csv)
compares owned and borrowed finite decoding of the same name/chunk frames. On
Linux x86_64 with Rust 1.99.1, opt-z/LTO/one codegen unit/panic abort, a single-thread
counting allocator observes zero requests for borrowed views. Owned decoding of
a 1 MiB chunk plus the five-byte name requests 1,048,581 bytes in two allocations.
Construction of the source/input frame is outside the count. These are allocation
request counts/bytes, not peak heap or timing measurements. Source, lock, compiler
identity and logs are retained at `target/borrowed-contracts-0.5/`; no consumer
repository or deployed endpoint is qualified by this probe.

## Final 0.5 borrowed contract measurements

The final lifetime-aware derive and finite decode API were measured against the
owned form of the same contract on 2026-10-19. These are within-candidate
owned/borrowed comparisons, not a 0.4-to-0.5 release speedup. Both forms use field
IDs 1/2, a five-byte `chunk` name, and 0/32/4,096/1,048,576 bytes of value 42.
Bounds are Text 64/Bytes 1 MiB; method ID 1 has limits of 2 MiB frame, 4 MiB
allocation, 16 nodes and depth 8. Frame hashes match between both IC probes.

| 1 MiB operation | opt-z owned → borrowed | opt-3 owned → borrowed |
| --- | --- | --- |
| Native decode/drop | 34.28 → 10.03 µs | 33.92 → 10.02 µs |
| Native decode/full scan/drop | 285.45 → 257.77 µs | 144.77 → 120.71 µs |
| IC decode/drop instructions | 3,281,082.5 → 2,005.5 | 2,754,987 → 966 |
| IC decode/full scan/drop instructions | 19,009,864.5 → 15,730,787.5 | 10,881,549 → 8,127,528 |
| Linked probe Wasm bytes | 41,383 → 40,232 | 32,331 → 31,380 |
| gzip-9 probe bytes | 14,638 → 14,468 | 12,466 → 12,167 |

[All timing/instruction samples summarized by case](evidence/borrowed-contracts-0.5/performance.csv)
and [linked/compressed sizes](evidence/borrowed-contracts-0.5/wasm-size.csv) retain
both profiles. Native results are medians of 11 alternating owned/borrowed
batches; repetition counts double until borrowed batches take at least 2 ms or
reach 1,048,576 iterations. Min/max batch means are included to expose noise.
Input construction is outside measurement; decode includes canonical admission,
black-boxed values and owned destruction. The full scan sums every payload/name
byte. Neither operation measures encoding, transport or persistence.

Rust 1.99.1/LLVM 23.1.1 targets Linux x86_64 and wasm32-unknown-unknown with LTO,
one codegen unit, panic abort and stripped debug information. `release` uses
opt-z; `speed` uses opt-3. PocketIC 16.1.0 runs each separately linked decoder in
five queries of four decodes, using `ic0.performance_counter(0)` around the loop;
the table divides the median counter delta by four without overhead subtraction.
Both Wasm probes include the same owned input encoder/setup and reply handler;
sizes describe these complete probes, not isolated decoder symbol attribution.
Gzip uses level 9 and mtime 0. No production dependency was added.

Borrowed opaque payload admission avoids a copy and, on Wasm, the existing
same-slice canonical comparison avoids scanning those bytes again. UTF-8 text
still needs validation. The full-scan results show the smaller benefit when the
consumer actually reads every byte. Borrowed views retain their input frame;
zero allocation requests do not mean zero resident input memory. Native and IC
numbers are distinct, and neither establishes installed Blob endpoint latency.
Source, lockfiles, compiler identity, raw batches, frame hashes and Wasm files
are retained locally at `target/borrowed-contracts-0.5/final-performance/`.

## Bounded finite output growth

Relay's 64 KiB-per-frame, three-frame batch contract exposed maximum-schema output
reservation costs even for empty messages. The 0.5.1 candidate changes finite
encoding to one-pass fallible growth: start at up to 64 bytes, grow geometrically
or to fit an emitted chunk, and reserve the remaining schema ceiling once a
growth exceeds one quarter of it. Bounds are checked before reservation; schema
admission and bytes remain unchanged. Native sizing/reuse APIs are unchanged.

Matched before/after probes use released `fe7ab95` runtime source versus this
candidate, with the same incoming working-tree Cargo.lock selection. The 44
existing allocation fixtures are supplemented by six copied Relay-shaped cases
from Relay `06bd5bf`: 0 frames, 1×3 bytes, 3×4 bytes, 1×4 KiB, 1×64 KiB and
3×64 KiB. Sender/receiver are single-byte values 10/20, lane 0, incarnation 1,
sequences start at 0 and payload bytes are 42. The copied contract retains Relay's
IDs, bounds and directional frame identity. This is an isolated encoder/decode
probe, not a rerun of Relay's installed endpoint or its historical instruction
numbers. Fixture construction, transport and storage are outside measurement.

| Encode case | opt-z IC instructions, before → after | opt-3 IC instructions, before → after |
| --- | ---: | ---: |
| Relay empty batch | 43,115.25 → 3,081.5 | 41,630.25 → 1,638.5 |
| Relay one three-byte frame | 43,972.25 → 3,914.5 | 42,176.25 → 2,171.5 |
| Relay three four-byte frames | 45,689.25 → 5,583.5 | 43,271.25 → 3,240.5 |
| Relay three maximum frames | 262,477.25 → 263,208.25 | 259,969.25 → 260,676.25 |
| 1,024-hash array manifest | 1,979,819.5 → 1,983,565.5 | 1,441,443.75 → 1,263,855.75 |

Small Relay batches retain 64 bytes of output capacity instead of 196,783. All
50 cases have identical before/after frame hashes in both profiles. Full Relay
cases increase encode instructions by at most 0.61%; the full packed manifest
increases by 2.14%/1.56%. This avoids the earlier universal sizing-pass candidate's
37% full array-manifest regression. Allocation handling stays out of the per-byte
hot path; initial candidates with inlined growth handling were rejected.

Native opt-z measurements expose a tradeoff: fresh 4 KiB blob encoding rises from
83.65 to 121.47 ns (45%, about 38 ns) because growth needs another reservation.
The 1 MiB blob stays near 21 µs; full array-manifest encoding falls from 46.69 to
23.06 µs in this run. These are calibrated 11-batch native medians, not IC costs
or latency guarantees. Growth may transiently require both old and new blocks;
observed capacity does not establish peak memory or allocation counts.

The combined 50-case probe grows by 931/537 linked Wasm bytes and 590/323 gzip
bytes under opt-z/opt-3. This is whole-probe size, not symbol-attributed savings.
[IC results and frame hashes](evidence/output-growth/ic.csv),
[native medians](evidence/output-growth/native.csv) and
[linked/compressed sizes](evidence/output-growth/size.csv) retain the complete
comparison. Rust 1.99.1/LLVM 23.1.1 targets x86_64 Linux and wasm32-unknown-unknown;
profiles use LTO, one codegen unit, panic abort and stripped debug information.
PocketIC 16.1.0 records five queries of four repetitions using performance counter
0; medians are divided by four without subtracting loop/counter overhead.
Native batches calibrate the faster operation to at least 2 ms or 1,048,576
repetitions, then rotate all cases over 11 batches. Encoding includes allocation
and result drop. Decoding retains each fixture's checksum/drop behavior; Relay
cases decode and drop their owned envelope. gzip uses level 9 and mtime 0.

Before/after source snapshots, locks, copied probe source, compiler identities,
raw samples and rejected candidates are retained at
`target/output-growth/`. Focused tests cover single serialization, tiny output
under a large schema, growth boundaries, spare capacity, and deterministic
capacity-overflow refusal. No sibling source or production dependency changed.

### Small finite schema reservation

PR #15 is refreshed against released 0.5.8 (`1894d0a`). Fresh encoding requests
the full admitted frame ceiling when it is at most 128 bytes. Larger ceilings retain the 64-byte start and existing growth
policy. This must happen in the direct header initialization paths introduced
since the original PR; changing only the cold growth routine is insufficient.
There is no new setting, wire identity or serialization pass.

The new regression fails on 0.5.8 and passes with the change. It checks fresh
initialization and warmed scratch reuse for Bytes ceilings 53, 80 and 107
(frame ceilings 74, 101 and 128), stable scratch capacity/pointer across payload growth, and exact bytes.
Existing tests retain typed allocation failure, logical limits despite spare
capacity, single serialization and large-schema tiny-output checks.

[IC instruction results](evidence/small-schema-reservation/ic.csv) cover 64
cases in opt-z and opt-3. For fresh Bytes<80>/Bytes<107> payloads crossing 64
frame bytes, encoding saves 322 instructions (18.1–19.0%) in opt-z and 274
(24.8–26.2%) in opt-3. Every other measured encode case and every decode case
retains its baseline instruction count. All frame lengths and SHA-256 identities
match. This qualifies fresh encoding only: an initial prototype also changing
scratch reservation regressed warmed reuse by 5–8%, so scratch policy is unchanged.

[Native allocation/timing results](evidence/small-schema-reservation/native.csv)
show affected fresh frames dropping from two allocation requests to one. Tiny
frames can request 101 or 128 bytes instead of 64; this is a bounded retained
capacity tradeoff, not a peak-memory guarantee. Native timings are supporting
observations, not IC measurements or universal speedup claims.
[Whole-probe sizes](evidence/small-schema-reservation/size.csv) change from
42,499 to 42,536 linked bytes in opt-z (+37), with gzip-9 +2 bytes. Opt-3 linked
size remains 48,303 bytes, with gzip-9 +27 bytes. These are complete linked probe
comparisons, not symbol-attributed savings or installed-consumer measurements.

Both builds use Rust 1.99.1 (`58cf8f9c3`, LLVM 23.1.1), Postcard 1.1.3, LTO,
one codegen unit and panic abort.
IC builds target wasm32-unknown-unknown, with opt-z/opt-3 and stripped debuginfo;
the external PocketIC 16.1.0 driver takes five queries of four repetitions per
case using ic0.performance_counter(0), reporting median instructions per encode
or decode. Input setup, frame identity and capacity reporting are outside the
timed loops. Warm reuse prepares an exact-length buffer before timing.
IC cases are seven byte bounds (43, 44, 80, 107, 108, 4,096, 1,048,576), four
payload lengths (0, 32, min(53,bound), bound), and both directions, plus eight
warm-reuse Bytes<80> controls. Inputs are bytes filled with 42, method ID 1,
frame allowance bound+21, allocation allowance 2*bound+21, one node/depth.

Native builds target x86_64 Linux on a Threadripper 7970X, opt-3, pinned to CPU 0.
They use the same seven bounds and directions with lengths 0/32/52/53/bound
(excluding lengths above the bound), 1,000 warmups and medians of nine batches
of 100,000 encodes (1,000 for the 1 MiB bound). Allocation counting runs separately
from timing; the counter-disabled allocator wrapper remains in both timed builds.
Two baseline runs and a final candidate run are retained in the compact table.
Local scripts, sources, dependency locks, raw samples, rejected scratch results
and logs remain at `/tmp/ic-wire-pr15-probe/`. No runtime dependencies or sibling
sources were added or changed for the external qualification.

Probe source SHA-256 identities:

- Native: `b2b4fdc6beb151023182dcd5cc8593146889083bb11d8e00f330b0f88d0b0096`.
- IC: `dd69b54aae1de5e70a7e7a0514cd95d14e24b2c5e065a8569115f68f7082cfff`.
- IC probe lock: `91615bb5d30d5826138dfd8d03b6525985ff04c872756eb08d8f86f675094266`.

## Per-type code generation preference experiment

A requested speed/size selector was prototyped outside the maintained runtime.
The experiment starts from the 0.5.1 output-growth working tree, including its
incoming lockfile selection. Proposed `#[wire(optimize = "speed" | "size")]`
syntax is experimental only: it is not accepted by the shipped derive.

The specialized prototype forces generated Serialize/Deserialize methods inline.
The shared prototype keeps those methods out of line and routes Postcard output,
canonical comparison and input access through non-inlined routines shared across
contract types. These are the same Postcard encoding and admission rules. A
mixed canister specializes Relay/blob types and shares manifest processing.
Inlining-only controls remove the shared wrappers while retaining the hints.
No runtime branching on a user option is involved: the prototype selects paths
through a per-type associated constant. Only root DTOs select a preference;
this is not an independently composable policy on every nested field.

| Strategy | opt-z linked / gzip bytes | opt-3 linked / gzip bytes |
| --- | ---: | ---: |
| Existing behavior (`default`) | 76,896 / 21,662 | 52,290 / 19,088 |
| Specialized (`fast`) | 74,488 / 21,580 | 52,290 / 19,089 |
| Shared (`compact`) | 78,459 / 21,637 | 57,764 / 18,797 |
| Specialized/shared (`mixed`) | 75,712 / 21,742 | 55,784 / 19,585 |
| Out-of-line hints only (`hints-compact`) | 76,896 / 21,666 | 54,454 / 18,629 |
| Mixed hints only (`hints-mixed`) | 75,309 / 21,562 | 52,430 / 18,952 |

For the full 1,024-hash manifest under opt-3, shared encoding uses 1,530,148.25
instructions versus 1,247,188.25 specialized; decoding uses 3,462,146 versus
1,889,124.5. The inlining-only compact control still needs 2,520,595 decode
instructions. The shared prototype pays more instructions without shrinking
linked Wasm. Its small opt-3 gzip benefit in the all-shared probe disappears
when combined with specialized paths. Specialized inlining under opt-z instead
reduces both instructions and size in these fixtures; that is not evidence for
a two-level speed/size tradeoff. These results reject these implementations,
not the possibility of a different design providing useful sharing.

No public preference, extra production codec path or dependency is adopted.
A future selector must demonstrate an actual linked-size/instruction tradeoff
on multiple contract families and in mixed deployments, with exact byte/error
parity. A schema interpreter or direct specialized codec would be a larger
implementation requiring its own evidence; it is not implied by an inline hint.
The promising opt-z inlining result also needs broader qualification before
changing default derive code generation.

[Complete IC instruction results](evidence/codegen-preferences/ic.csv) and
[linked/compressed sizes](evidence/codegen-preferences/size.csv) cover all six
strategies under both profiles. Each probe includes the same ten cases: Relay
0 frames, 1×3, 3×4 and 3×64 KiB; manifests with 0/32/1,024 hashes; and named blobs
of 0/4 KiB/1 MiB. Relay uses the prior experiment's bindings/payloads. Manifest
fields are upload u128::MAX−1, root [149;32] and hashes [index as u8;32]. Blob
fields are name `chunk` and repeated byte 42. All methods use ID 7, a 2 MiB byte
ceiling, 16 MiB allocation ceiling, 100,000 nodes and depth 16. Schemas and input
values remain identical between strategies; encoded hashes match for every case.

Rust 1.99.1/LLVM 23.1.1, wasm32-unknown-unknown, LTO, one codegen unit, panic abort
and stripped debug information are fixed. `release` is opt-z; `speed` is opt-3.
PocketIC 16.1.0 measures five queries of four encode/decode repetitions with
performance counter 0. Medians are divided by four without subtracting counter
or loop costs. Input construction and reply transmission are excluded; output
allocation and owned result destruction are included. Decoding does not add an
application full-byte scan. Gzip uses level 9 and mtime 0. No native timing,
installed endpoint throughput or peak memory claim is made by this experiment.

Native differential validation of the default, specialized, shared and mixed
prototypes matches acceptance/error results for 2,886 truncation/byte-mutation
cases; every accepted input re-encodes identically. Separate checks reject
trailing bytes and verify a borrowed 1 MiB blob remains input-backed and preserves
its frame. These checks do not qualify the prototype as a production API.
Source copies, locks, raw samples, validation output and Wasm artifacts are
retained locally at `target/codegen-preferences/`. The existing 0.5.1 encoder fix
and all incoming working-tree edits remain unchanged.

## Follow-up: sharing scalar routines

A deeper isolated prototype finds a useful opt-3 tradeoff. It shares whole
Postcard integer routines while allowing generated record traversal to inline.
This differs from the earlier rejected byte-level wrappers and out-of-line
record methods. It is still an experiment, not an available Wire attribute.

The prototype patches a private copy of Postcard 1.1.3: serialization flavors
can select outlined integer writers, and deserialization flavors can select a
shared checked varint reader. The reader combines u16/u32/u64/u128 handling in
one u128 accumulator routine with explicit maximum byte count and final-byte
bounds. Signed decoding continues using Postcard's existing zigzag conversion.
The specialized path retains original routines. Input/output flavor forwarding
and root record traversal inline; the integer helper is the sharing boundary.
Framing, allocation policy, bounded DTO visitors and canonical re-encoding remain
unchanged. No registry cache, sibling checkout or production dependency is edited.

The final probe contains the earlier ten Relay/manifest/blob cases plus eight
small admin-shaped contracts: page cursor/limit, status counters, configuration,
acknowledgement, four counters, object inspection, cursor window and scheduling.
These are illustrative consumer-shaped records, not installed consumer endpoints.
Their exact fields and values are retained with the probe source. The mixed
strategy specializes Relay/blob and shares manifest/admin scalar routines.

| Final 18-case strategy | opt-z linked / gzip bytes | opt-3 linked / gzip bytes |
| --- | ---: | ---: |
| All specialized | 105,689 / 25,474 | 71,088 / 23,402 |
| All scalar-shared | 105,384 / 24,726 | 61,600 / 19,326 |
| Mixed hot/cold | 107,406 / 25,292 | 67,597 / 21,263 |

Under opt-3 the mixed probe saves 3,491 linked bytes (4.9%) and 2,139 gzip bytes
(9.1%). Every measured Relay/blob hot-path encode and decode instruction count
is identical to the all-specialized probe. Admin decode instructions rise
29.5–72.5%; admin encoding changes −0.9% to +5.6%. For example page decoding
rises from 841 to 1,166 instructions, and object inspection from 1,649.5 to
2,846. The full hash manifest decode changes from 1,889,202.5 to 1,890,408
instructions (+0.064%). All-shared saves 13.3% linked and 17.4% compressed bytes,
but also exposes hot types to the shared-path costs.

The result depends on the compiler profile and linked contract set. Under opt-z,
mixed adds 1,717 linked bytes (1.6%) while saving only 182 gzip bytes (0.7%).
Its full manifest encode costs 11.6% more instructions, despite decode costing
4.2% less. There is no universal level where one choice always shrinks Wasm or
always reduces instructions. Linked Wasm and compressed deployment size are
separate objectives. Do not extrapolate from the ten-case or eighteen-case
probe to a complete application, or compare them as a controlled size delta.

The intermediate experiments are retained too: merely outlining width-specific
integer routines still enlarged linked Wasm; unifying the decoder reduced that
cost, and allowing record traversal to inline removed substantial overhead.
The combined change is what establishes the final mixed opt-3 benefit.
[Complete final instruction results](evidence/scalar-sharing/ic.csv) and
[linked/compressed sizes](evidence/scalar-sharing/size.csv) retain both profiles.
All final frame hashes match across strategies. Native differential validation
matches 5,290 mutation/truncation cases plus twelve signed/unsigned boundary
reference frames. Accepted mutated frames re-encode identically. Additional
checks cover trailing bytes and input-backed borrowing of a 1 MiB blob. The
mixed prototype also checks for wasm32-unknown-unknown on Rust 1.91.

Compiler, LTO, codegen units, panic/strip settings, PocketIC version, five-query /
four-repetition counter method and gzip parameters match the preceding experiment.
These are codec IC instructions and whole-probe sizes, not native timing, peak
memory or installed endpoint measurements. Raw results, intermediate artifacts,
source copies, locks, validation output and the experimental Postcard source are
retained at `target/scalar-sharing/`.

Production adoption remains unresolved: this result depends on an unshipped
Postcard policy hook. A supported upstream hook or a measured adapter around
public Postcard APIs is needed before adding a Wire preference. Do not vendor
this experimental dependency fork or reproduce varint logic in the maintained
Wire runtime on the strength of these measurements. The proposed setting remains
unsupported; the 0.5.1 output-growth fix is unchanged.

## Public-API code generation adapter

Rejected after measurement: the modest profile-dependent savings did not justify
the Serde forwarding code, test burden and permanent public API. The adapter,
attribute and runtime selection were removed. No preference setting is supported.
The independent finite output-growth optimization remains.

The reverted experiment supported `#[wire(optimize = "speed")]` and
`#[wire(optimize = "size")]` on root records/enums. Manual contracts use
`Wire::OPTIMIZATION`. The default preserves existing specialization and inlining
behavior. Explicit preferences inline generated record traversal; size selection
shares integer serializer/deserializer calls through public Serde/Postcard APIs.
Compound visitors wrap their children so the root preference covers one traversal.
There is no dependency patch, copied varint implementation, new allocation,
alternate frame format or per-field runtime dispatch. The selected path remains
subject to the same admission, output growth and canonical re-encoding checks.

The experimental implementation was measured with the same eighteen-case probe and
input parameters as the scalar-sharing study. The registry Postcard 1.1.3 checksum
and lockfile are retained. `fast` means all roots select speed; `compact` means all
select size; `mixed` selects speed for Relay/blob and size for manifest/admin.
These are host fixture labels, not additional public API names.

| Final implementation | opt-z linked / gzip bytes | opt-3 linked / gzip bytes |
| --- | ---: | ---: |
| All speed | 110,180 / 25,938 | 69,437 / 22,576 |
| All size | 113,544 / 25,448 | 67,009 / 21,313 |
| Mixed | 114,473 / 25,784 | 68,207 / 21,631 |

Compared with all speed, final mixed opt-3 saves 1,230 linked bytes (1.8%) and
945 gzip bytes (4.2%). Relay/blob hot-path encode/decode instruction samples are
identical. Admin encodes cost 1.5–6.2% more instructions; decodes cost 3.5–12.4%
more. Full manifest encode/decode differences are below 0.01%. Under opt-z,
mixed linked size grows 4,293 bytes (3.9%) while gzip shrinks 154 bytes (0.6%);
admin encode/decode changes remain within +2.3%/+1.6%. The final specialized
control also differs from the isolated prototype: use this matched comparison,
not earlier prototype percentages, for this rejected experiment.

[All final instruction samples summarized by case](evidence/codegen-adapter/ic.csv)
and [linked/compressed sizes](evidence/codegen-adapter/size.csv) retain both
profiles and identical frame hashes for all eighteen cases. Compiler and counter
method match the prior study: Rust 1.99.1/LLVM 23.1.1, wasm32-unknown-unknown, LTO,
one codegen unit, panic abort, stripped debug information, PocketIC 16.1.0,
five queries of four repetitions, median counter delta divided by four without
overhead subtraction. `release` is opt-z, `speed` is opt-3. gzip uses level 9 and
mtime 0. Construction, argument copying, reply delivery and storage are outside
the clock; encode/decode and result destruction are included. There is no added
application byte scan. Native timing, peak memory and installed endpoint behavior
are not established by these measurements.

Experimental tests exercised integer boundaries, malformed/truncated frames,
noncanonical refusal, schema admission, request/response direction, retained byte
identity, finite floats, options/results/lists/boxes, derived and manual enums,
borrowed fields and unchanged TypeScript metadata. Compiler diagnostics rejected
unknown, duplicate and misplaced preferences. The selected traversal also checked
on Rust 1.91 native/Wasm. Source snapshots, raw samples, locks and probe artifacts
are retained at `target/codegen-adapter/`. No sibling source or production
dependency selection was changed; incoming lockfile edits were preserved.

## Direct header initialization

Against released 0.5.1 (`cdd47c3`), initialize the finite encoder's header using
`try_reserve_exact(min(schema_ceiling, 64))` and a direct copy. This is the same
initial reservation chosen by the general growth routine for the 11-byte header.
Admission proves it fits. Payload growth, allocation failures, canonical bytes
and public APIs remain unchanged; no new helper or traversal is introduced.

The matched 50-case probe covers 44 allocation-study lanes and six Relay batches.
All frame hashes, lengths and retained output capacities match across both
profiles. Eighteen finite encode cases improve; the other 32 cases and every
decode instruction sample are unchanged. Tiny Relay cases (empty, 1×3, 3×4 bytes)
save 1.4–2.6% encode instructions under opt-z and 5.8–11.5% under opt-3. Full Relay
batches save 80 instructions under opt-z and 189 under opt-3, a much smaller
relative gain. No measured encode case regresses.

| Profile | Baseline linked / gzip bytes | Direct header linked / gzip bytes |
| --- | ---: | ---: |
| opt-z | 117,800 / 36,067 | 117,660 / 35,965 |
| opt-3 | 130,786 / 40,967 | 129,890 / 40,663 |

[Per-case instruction results](evidence/header-initialization/ic.csv) and
[whole-probe size results](evidence/header-initialization/size.csv) retain the
matched controls. Rust 1.99.1/LLVM 23.1.1, wasm32-unknown-unknown, Postcard 1.1.3,
LTO, one codegen unit, panic abort and stripped debug information were used.
The probe profiles are `evaluation` (opt-z) and `speed` (opt-3). PocketIC 16.1.0
runs five queries of four repetitions; values are median counters divided by
four, without subtracting harness overhead. gzip uses level 9 and mtime 0.
Allocation-study decoding includes its existing application byte checksum;
Relay decoding includes result destruction without that scan. Input construction,
transport and storage are outside the clock. These are IC instruction and
whole-probe size measurements, not native timing or installed endpoint latency.

An initial cached host driver only exercised 18 cases; those samples were
superseded by the corrected 50-case runs. Source archives, exact locks, corrected
host/probe sources, binaries and raw final samples are retained at
`target/header-initialization/`. Existing tests cover single-pass encoding,
bounded growth, capacity overflow and limits despite spare allocator capacity.

## Native borrowed-pointer equality experiment

Rejected: removing the Wasm-only condition on the canonical comparator's
same-pointer shortcut improves the 1 MiB borrowed blob's native decode time by
5.4% and 6.2% in two paired runs, but regresses the 1,024-hash borrowed packed
manifest by 11.9% and 13.0%. Small packed manifests also regress. The change
was reverted; the existing Wasm shortcut and pending header optimization remain.
These results do not establish the cause of the regression.

[All 44 cases in both pairs](evidence/native-pointer-identity/native.csv) retain
encode/decode medians. Baseline is the task-entry working tree after header
initialization and Shared Tooling adoption; the candidate removes only the target
condition on pointer equality. Both use Rust 1.99.1/LLVM 23.1.1, x86_64 Linux,
Postcard 1.1.3, opt-z, LTO, one codegen unit, panic abort and stripped debug info.
The existing allocation driver calibrates batches to at least 2 ms (or its
repetition cap), then runs 11 rotating samples per case. Runs alternate baseline
and candidate twice, pinned to CPU 0; reported values are median nanoseconds per
operation. Decoding includes the fixture's application checksum and destruction.
Frame hashes, lengths, capacities and reuse history match. Native binary file
sizes are both 843,064 bytes. No new IC instruction, Wasm-size or installed
endpoint measurements were made. Raw runs, driver/fixture sources, lockfile and
both comparator sources are retained at `target/native-pointer-identity/`.

## Finite reuse, text validation and scan separation

This 2026-10-21 investigation uses released 0.5.4 (`1e1ea56`) plus the incoming
IC Build/Host 0.62.1 lockfile selection. Production runtime source was unchanged in this initial experiment.
An isolated finite encoder prototype reuses caller storage with the existing
bounded growth policy. It clears prior bytes before admission, maps serializer
failure to `Malformed`, and may lose capacity on serialization failure, like the
native reuse API. It was not yet a supported public API. Separate checks cover size
changes, borrowed input, frozen retry copies, schema failure and oversized output
with spare capacity. No consumer checkout was changed.

The 60-case probe combines blob lengths 0, 32, 4,096 and 1,048,576 with three
encoding modes (fresh, warmed reuse, reuse plus a retained copy), owned/borrowed
decoding and codec-only/full-scan decoding. Twelve text cases cover lengths 32,
4,096 and 1,048,576, using ASCII, multibyte UTF-8 and invalid first/last bytes.

| Blob bytes | Fresh encode, opt-3 | Warm reuse | Reuse plus retained copy |
| --- | ---: | ---: | ---: |
| 0 | 644 | 420 | 819 |
| 32 | 786 | 562 | 993 |
| 4,096 | 5,186 | 4,643 | 9,133 |
| 1,048,576 | 1,370,143.75 | 1,049,140 | 2,418,239.75 |

These are IC instructions per operation. Warm reuse saves 10.5–34.8% under opt-3
and 15.2–26.3% under opt-z. Reuse plus a frozen copy is slower than fresh encoding
at every tested size. Initial buffer preparation is outside the timed interval;
the result does not establish cold-buffer, peak-memory or installed-endpoint
savings. Large retained capacity also survives later small messages. The useful
next consumer qualification is a scratch-buffer path that does not need to keep
a separate frame after reuse; this evidence does not justify replacing retained
request bytes with mutable storage. The follow-up below qualifies finite scratch-buffer encoding.

For the same isolated 1 MiB borrowed blob, opt-3 codec-only decoding measures
553.75 instructions, versus 8,127,053.75 with the application checksum. The codec
validates framing, bounds and canonical encoding; a byte payload has no UTF-8 or
application-content validation. This illustrates different work, not an additional
runtime speedup. Both timings include result destruction and harness overhead.

Two text prototypes validate UTF-8 before allocating: one validates byte input
explicitly, the other delegates string validation to Postcard. They reduce
invalid-first-byte rejection of a 1 MiB input by over 99.9%, and invalid-last-byte
rejection by about 26.7%. However, 32-byte valid ASCII/multibyte text costs about
10.1%/6.5% more instructions under opt-3 for the Postcard-string candidate; the
explicit-byte candidate has similar regressions. Opt-z improves small valid text,
but both candidates enlarge linked Wasm in both profiles. Both were rejected;
production `Text<N>` decoding remains unchanged.

[All IC cases](evidence/reuse-text-scan/ic.csv) and
[linked/compressed probe sizes](evidence/reuse-text-scan/size.csv) distinguish
`baseline`, explicit-byte `candidate`, and Postcard `string` variants. Frame
hashes, lengths and output capacities match for every variant/profile. The reuse
prototype is linked into all three controls, so sizes do not measure the cost of
adding a production reuse API. Rust 1.99.1/LLVM 23.1.1, wasm32-unknown-unknown,
Postcard 1.1.3, LTO, one codegen unit, panic abort and stripped debug information
are fixed. Profiles are opt-z (`release`) and opt-3 (`speed`). PocketIC 16.1.0
runs five queries of four repetitions; medians are divided by four, with no
harness subtraction. gzip uses level 9 and mtime 0. Input setup, transport and
storage are outside the clock. Encode includes allocation/drop or the explicitly
labelled warm reuse/copy work. These results establish no native timing claims.

The maintained allocation CLI now exposes `--codec-only`, preserving the default
full-scan mode. It independently calibrates encode/decode and labels format-2
reports with separate repetition counts. Both modes ran all 44 maintained cases
with identical frames/capacities. An initial isolated native attempt using one
shared repetition count was stopped because cheap rejection caused excessive
large-buffer encoding; its partial output is not comparative evidence. Source
snapshots, raw IC runs, corrected native reports and isolated prototypes are
retained at `target/reuse-text-scan/`. No runtime changes were adopted by that initial experiment.

## Finite scratch-buffer follow-up

The 64-case follow-up keeps the earlier 60 case identities in order and adds
cases 60–63: a six-message chunk cycle with sequence metadata and borrowed
payload lengths 0, 32, 4,096, 1,048,576, 32 and 0. The modes are fresh encoding,
reuse starting empty each cycle, warmed reuse, and warmed reuse plus a copy of
each frame. Results below are IC instructions **per six-message cycle**.

| Profile | Fresh | Initially empty reuse | Warm reuse | Reuse plus copies |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 1,388,226.5 | 1,065,207.5 | 1,063,645.25 | 2,440,585.5 |
| opt-3 | 1,379,863.25 | 1,057,564.25 | 1,056,583 | 2,432,077.25 |

Reuse saves about 23% in this cycle even when starting empty. Copying every frame
loses the gain. The scratch high-water capacity is 1,048,607 bytes and remains
allocated after returning to small messages. No installed consumer was changed;
transport, durable storage and application scans are outside this measurement.
This qualifies optional finite request/response scratch APIs, with caller-owned
retained-memory and retry policy. It does not justify changing immutable retries.

Sharing fresh encoding through the mutable-buffer helper regressed small fresh
messages by 6–14%; forced inlining still regressed opt-z by about 4–5%. The kept
implementation leaves the fresh path intact and uses a separate scratch helper.
All 16 fresh blob cases and the fresh mixed cycle have identical instruction
counts to the matched baseline in both profiles. Frame lengths and hashes match
for all 64 cases. Request/response, borrowed inputs, schema rejection, serializer
errors with spare capacity, retained capacity and frozen retries have focused tests.

[Full instruction results](evidence/finite-scratch/ic.csv) and
[linked and compressed sizes](evidence/finite-scratch/size.csv) use the same
compiler, Postcard, target, profiles and five-by-four PocketIC protocol as the
preceding experiment. The baseline already links the isolated reuse prototype;
size differences are whole-probe differences, not the marginal cost of adding
an API to a consumer. Final linked/gzip bytes are 75,540/24,897 for opt-z and
58,223/22,010 for opt-3. Mixed-case frame hashes cover concatenated frames and
reported capacity is the scratch high-water preparation value in every mode,
including fresh. Each initially-empty cycle drops the preceding scratch buffer
inside the timed interval. Native time and peak memory were not measured.
Sources, locks, raw runs, rejected revisions and logs are retained under
`target/finite-scratch/`; the baseline source contains only the isolated prototype.


## TypeScript bounded-growth experiment

Against released 0.5.8 (`1894d0a`), an isolated writer started with at most 64
bytes and doubled capacity as needed, capped at the method's byte allowance.
Bulk writes could grow directly to their required size. Growth copied only the
written prefix and refreshed the float DataView. Text admission used the full
allowance, independent of current capacity; the final independent slice stayed.
**This prototype was rejected; the maintained writer is unchanged.**

[Matched native results](evidence/typescript-buffer-growth/native.csv) cover
20 cases in two runs. With a 2 MiB allowance, a 13-byte scalar frame went from
2,097,165 requested buffer bytes to 77, and median encode time fell about 98.5%.
With tight allowances, 4 KiB blobs slowed 48–59%, 1,024 f64 values slowed 20–67%,
and 1,024 u64 values slowed 10–11%. The tight f64 case requested 32,726 buffer
bytes instead of 16,406 due to intermediate growth and copying. These regressions
rule out adopting this implementation as a general speed improvement. The 1 MiB
blob timings varied substantially; do not infer a stable large-blob speedup.

Measurement used Node 24.21.0 / V8 13.6.233.17-node.53, TypeScript 7.0.2,
ES2022 CommonJS with strict checking, Linux x86_64, AMD Threadripper 7970X.
No CPU affinity was set. Each case warmed 100 encodes, calibrated each variant
independently to at least 5 ms (16,384-call cap), then took the median of 11
batches. Case order rotated and variant order alternated. Explicit GC ran before
each sample outside timing; allocation and incidental GC remained inside timing.
Setup and decode verification were outside timing. These are native Node results,
not browser, IC-instruction, peak-memory or Wasm/deployment-size measurements.
Requested buffer bytes count explicit numeric Uint8Array construction plus the
final slice, separately from timing; they do not measure total engine allocation.

Inputs were scalar u32 128; zero/32/4,096/1,048,576 bytes filled with 42;
32 ASCII `a` characters; 2,048 `é` characters; 4/1,024 f64 values `i + 0.5`;
and 1,024 u64 values `i`. Each used method ID 1, node allowance 4,096, depth 16,
and equal request/response schemas. Wide frame allowances were 2,097,152 bytes;
tight controls used exact frame length (scalar: 16 bytes). Byte/text maxima were
1,048,576 and fixed-array maxima matched element counts. Frames and decoded
values matched for every timed case. Another 450 comparisons covered both
directions, growth thresholds, Unicode, floats and insufficient/exact allowances,
preserving successful bytes and typed errors.

Local sources, compiled variants, scripts and raw samples are retained under
`target/typescript-buffer-growth/`. Both matched runs used source SHA-256:

- Baseline: `caf41dabcfdcc401dc2cb90bd4ce6e92843b404c6131af2462c46b74a808f2a8`.
- Candidate: `20e74baf919a4a922ef7d57cadfd094a1a510e34402777c8724449156c3c5b7b`.

Benchmark script SHA-256: `e4e65a41c07ef227651b1efb63317415c177a7038c7be3e6ca4a8205cd5220b5`.

## TypeScript small-integer and text encoding study

Against released 0.5.11 (`9190e54`), the generated codec handles 8/16/32-bit integer
payloads using exact number arithmetic. Unsigned shifts are used only inside the
admitted 32-bit writer; the reader accumulates with multiplication before checking
width and minimality, avoiding signed bitwise truncation. Signed ZigZag includes
i32::MIN. Wide integers, method IDs and length prefixes keep bigint handling.
Small-integer helpers keep the main dispatch functions compact. Rust production
encoding and all protocol bytes are unchanged.

[Selected native results](evidence/typescript-small-integers/native.csv) compare
the final integer-only change with baseline in two runs. Each selected schema
runs in a separate Node process, with both variants loaded in that process.
The u64 array is an unchanged-path control. These are native Node measurements,
not browser results, IC instructions, network latency or Wasm deployment size.
Small-scalar and control timings vary; this does not establish a universal speedup.

Text encoding is unchanged. Two alternatives were tested and rejected:

- A JavaScript UTF-8 length/surrogate scan followed by direct frame encoding
  increased large-text encode times roughly 95–251% in its first run.
- Direct frame encoding followed by an in-place shift for the length prefix
  removed one UTF-8 scratch buffer per field and passed 8,600 byte/error boundary
  comparisons. Isolated runs improved, but repeated mixed-workload runs were
  inconsistent: one regressed 4 KiB text by up to 37% and 1 MiB text by 54%.
  This was not retained as a general optimization. UTF-8 roundtrip validation
  and final independent frame copying remain in the maintained implementation.

[Rejected text results](evidence/typescript-small-integers/text-rejected.csv)
retain the scan and both mixed-workload shift runs. Raw rejected candidates,
isolated controls, source variants, scripts and samples are under
`target/typescript-small-integers/` and `/tmp/ic-wire-0512/`.

Environment: Node 24.21.0, V8 13.6.233.17-node.53, TypeScript 7.0.2, strict ES2022
CommonJS output, Linux x86_64, AMD Threadripper 7970X, pinned to CPU 0. Each lane
warms 1,000 operations, calibrates independently to at least 5 ms (65,536-call
cap), then reports the median of 11 batches. Case order rotates; baseline and
candidate order alternate. Explicit GC is outside timed samples; allocation and
incidental GC remain inside. Input setup, equality assertions and hashing are
outside timing. Encode and decode are measured separately.

Selected integer inputs are u16::MAX/u32::MAX/u64::MAX fixed arrays of 1,024 values
and a u8::MAX scalar. Frame allowances are 16,384 for arrays and 32 for the scalar.
Broader prototypes also exercised signed extrema and u32 value 1. Rejected text
inputs are empty, 32 ASCII characters, 16 `é` characters, 16 crab emoji, 4,096
ASCII characters, 2,048 `é` characters and 1 MiB ASCII. Text bounds are 1 MiB;
frame allowance is UTF-8 length plus 16. All cases use method ID 1, equal request/
response schemas, 4,096 nodes and depth 16. Every timed frame matches baseline
bytes and decoded values. No peak-memory claim is made from constructor counts.

Qualification adds every small-integer varint-width transition and signed
ZigZag boundary to the Rust-generated corpus, plus malformed/overflowing prefixes.
The expanded gate passes 145 vectors and 29,356 Rust/TypeScript differential
mutations. Separate tests cover nested node/depth refusal, text prefix growth,
exact/short frames, buffer independence and owned/borrowed Rust error parity.
An external probe checks all 131,072 u16/i16 values against the original codec.
The text prototype comparison includes unpaired surrogates and exact error codes.

In both selected runs, u16/u32 array encoding improves 69–75%; u32 array
decoding improves 44–45%. u16 decode ranges from 22–54% faster. The u64 control
ranges from 14% faster to 5% slower, so no wide-integer improvement is claimed.

Unminified CommonJS runtime size rises from 15,512 to 16,959 bytes; gzip-9
rises from 3,610 to 3,831. This is generated JavaScript size, not linked Wasm.

Retained probe SHA-256 identities:

- `baseline.ts`: `caf41dabcfdcc401dc2cb90bd4ce6e92843b404c6131af2462c46b74a808f2a8`.
- `selected.ts`: `3e41cfee7cf7a3f1d0cc1137fce6108d80e01f8b64b6972df7d9a31899f90b7a`.
- `bench-selected.cjs`: `6fb8829cb09f8171688351a9adbd81a69edefe5f39d6ffa61ef3772e5853b84a`.

## TypeScript enum tag arithmetic

Against released 0.5.12 (`8562529`), enum tags reuse the existing number varint
reader/writer and compare numeric IDs directly. Generated schema IDs are u32;
number arithmetic preserves all of them exactly. Wider payloads, length prefixes
and method identities retain bigint handling. No protocol or public API changes
are required, and no helper or lookup cache is added.

[Native results](evidence/typescript-enum-tags/native.csv) cover two runs, each
with separate Node processes for each case and both variants in each process.
Fixed arrays contain 1,024 unit enums with selected ID 0, 128, 16,384 or u32::MAX.
Descriptors list `Other` (ID 3) before `Chosen` (the selected ID); every value is
`{tag:'Chosen',value:null}`. The control is 1,024 u64::MAX values. All cases use
method ID 1, 16,384-byte frame allowances, 4,096 nodes, depth 16 and identical
request/response schemas. Setup verifies equal bytes and decoded values; the CSV
records frame hashes, lengths and median nanoseconds per complete operation.

Environment: Node 24.21.0 / V8 13.6.233.17-node.53, TypeScript 7.0.2, strict ES2022
CommonJS, Linux x86_64, AMD Threadripper 7970X, CPU 0 affinity. Each lane warms
1,000 operations, independently calibrates to at least 5 ms (65,536-call cap),
and measures 11 batches with alternating variant order. Explicit GC is outside
timing; incidental GC and allocations remain inside. Encode and decode are
separate; input setup, hashing and equality checks are outside timing.

Enum-array encoding improves 5–35% and decoding 31–71% across these runs.
The unchanged u64 control encodes 4–10% faster and decodes 2–5% slower, illustrating
measurement variation. These are native Node enum-heavy workloads, not universal
application speedups, browser results, IC instruction or Wasm size measurements.
The compiled prototype is 4 bytes smaller uncompressed and 2 bytes larger with
gzip-9; this is effectively neutral generated JavaScript size.

Qualification adds a Rust-derived maximum-ID variant and independent bigint
byte oracles for every u32 tag width transition, both directions, unknown tags,
truncation, nonminimal representations and overflow. The focused gate passes
146 Rust vectors and 29,560 differential mutations, plus existing 1 MiB checks.

Raw samples, script and compiled/source variants are in `/tmp/ic-wire-0513/`
and `target/typescript-enum-tags/`. The final source also clarifies a helper
comment; measured executable logic is identical. Probe SHA-256 identities:

- `baseline.ts`: `3e41cfee7cf7a3f1d0cc1137fce6108d80e01f8b64b6972df7d9a31899f90b7a`.
- `selected.ts`: `e359c4e2ec92636650cd48bc7a1e4bfd78de88cbb0414b6beafbcf4822ab1491`.
- `bench.cjs`: `c7bb3a0651b73f57df51435f20369be08e19ab9149c49d561c031cc96da9cc2c`.

## TypeScript short-length decoding experiment

Against 0.5.13 (`752a308`), a rejected prototype made `Reader.length` peek at the
next byte and return a checked number for prefixes below 128. Other prefixes
used the unchanged u64 bigint decoder. This preserved successful bytes/values and
typed errors, including nonminimal, truncated and overflowing prefixes. It is
not retained: longer-text controls repeatedly regressed despite short-value wins.

[All native results](evidence/typescript-length-prefixes/rejected.csv) include
four runs. Runs 1–2 use 1,000 warm-up operations, independent calibration to at
least 5 ms (65,536-call cap), and medians of 11 batches. Runs 3–4 increase warm-up
to 10,000, calibration to 20 ms with the same cap, and samples to 21 batches.
Each case runs in its own process with both variants; their measurement order
alternates and explicit GC occurs outside timing. Allocation and incidental GC
remain inside timing. Setup, hashing and equality checks are outside timing.

Environment: Node 24.21.0 / V8 13.6.233.17-node.53, TypeScript 7.0.2 strict ES2022
CommonJS, Linux x86_64, AMD Threadripper 7970X, CPU 0 affinity. Only request decode
is timed. Byte inputs contain 42; text inputs contain ASCII `a`. Both use lengths
0, 32, 127, 128 and 4,096, schema maximum 4,096 and frame allowance length + 32.
The nested-list case is a fixed array of 128 lists, each `[1,2,3,4]` with u8
items, list maximum 8 and frame allowance 1,024. Every case uses method ID 1,
4,096 nodes, depth 16 and identical request/response schemas. CSV frame hashes
identify inputs. Runs 3–4 repeat bytes-32, text-32/128/4096 and nested lists.

In longer runs, 32-byte blobs improve 17–25%, 32-byte text 18–19% and nested lists
28–33%. However, 4 KiB text regresses 13–32%; 128-byte text ranges from 1% faster
to 5% slower. This does not justify changing the default decoder. These are native
Node timings, not browser, IC instruction, Wasm size or application-wide claims.

The maintained checks cover every one-byte length for bytes/text/lists, exact
and exceeded schema bounds in both directions, and malformed u64 prefixes.
An external deterministic probe additionally compares 150,000 inputs across the
prototype and baseline, including typed errors (LCG seed 1, multiplier 1664525,
increment 1013904223; high bytes, lengths below 32; maxima 0/1/127/128/4096).
The retained runtime remains identical to 0.5.13. Raw scripts, sources, compiled
variants and samples are in `/tmp/ic-wire-0514/` and
`target/typescript-length-prefixes/`. SHA-256 identities:

- `baseline.ts`: `8f8e8fea4e26d40b1efdfb153bc35382be9c8162776ace3d72db58381f34c945`.
- `selected.ts` (rejected): `42f3a0867a492a9e607347d9b53b3bb3085c397d2ca59db702f0884299dd49fe`.
- `bench.cjs`: `ad23ccf27e80275e8113f6505d96ad791e9eebd4604a3e2719c49fb6477fa961`.
- `bench-long.cjs`: `f1edce0e2351215d2bded6ada75f6b87a38516cb8c07f258239f4da86fcd73e3`.
- `parity.cjs`: `92a8c1202392bb80ecc83fe0f147ba0d8668de7a2eda93c0009d2f8df86d1dfb`.

## TypeScript byte ownership correction

The 0.5.13 decoder used `input.raw(length).slice()`, which aliases input storage
when a Node Buffer is supplied: Buffer overrides `slice` with view semantics.
The retained correction constructs `new Uint8Array(input.raw(length))` instead.
This explicitly copies byte elements into owned storage without adding a runtime
mode or changing wire bytes. Existing bounds are checked before copying.

The conformance fixture now decodes every Rust vector from an offset Buffer view
and overwrites the source before checking decoded values, in both directions.
Separate checks mutate decoded bytes and confirm the source frame is unchanged.
The aliasing failure was reproduced against 0.5.13 and is absent with the fix.
Focused qualification passes 146 vectors, 29,560 differential mutations and the
existing 1 MiB frame checks. Node imports remain confined to the test fixture.

[Native measurements](evidence/typescript-byte-ownership/native.csv) compare two
runs against 0.5.13 (`752a308`). Inputs are Uint8Array blobs filled with 42 at
lengths 0, 32, 4,096 and 1,048,576, with ASCII text controls of 32 and 4,096 bytes.
Schema maxima are 1 MiB; frame allowances are length + 32, method ID 1, 4,096
nodes and depth 16. Request/response schemas are identical; request decode alone
is timed. These measurements use ordinary Uint8Array inputs; Buffer inputs are
covered by correctness checks, not speed claims.

Environment: Node 24.21.0 / V8 13.6.233.17-node.53, TypeScript 7.0.2 strict ES2022
CommonJS, Linux x86_64, AMD Threadripper 7970X, CPU 0 affinity. Each case runs in a
separate process with both variants, 10,000 warm-up operations, independent 20 ms
calibration (65,536-call cap), and median of 21 batches with alternating variant
order. Explicit GC is outside timing; incidental GC and allocations are inside.
Setup, equality checks and hashes are outside timing. CSV records frame identity.

The 4 KiB blob improves about 24% and 1 MiB improves 6–12%; 32-byte blobs regress
2–6%. Text controls improve 1–9%, showing timing variation. This is a correctness
fix with a small-copy tradeoff, not a universal speedup, browser result, IC
instruction or Wasm size claim. An earlier direct-slice prototype still inherited
Buffer aliasing and was discarded; its raw data remain in `/tmp/ic-wire-blob-copy/`.

Retained-fix sources, script and raw samples are in `/tmp/ic-wire-blob-owned/`
and `target/typescript-byte-ownership/`. SHA-256 identities:

- `baseline.ts`: `8f8e8fea4e26d40b1efdfb153bc35382be9c8162776ace3d72db58381f34c945`.
- `selected.ts`: `ea8e64c5b84b1ba8429acdd0bdc2e64122f06da8a37a05e237e7479e3f296a15`.
- `bench-long.cjs`: `bc2dfed5fffad041468f1399fb90a305f0512a7904c1d91e724454257d2822c7`.

## Native Serde metadata projection (pending 0.5.23)

`NativeSchema` adds host metadata generation without changing serializers, frame
bytes or codec admission. The maintained TypeScript conformance runner compares
handwritten and derived `Page<u64>` metadata through the same registry renderer,
asserting identical schemas and output. This measures schema construction plus
rendering, not proc-macro compilation, deployed instructions or codec speed.

| Named schemas | Handwritten median | Derived median | Change |
| --- | ---: | ---: | ---: |
| 1 | 3.412 µs | 3.567 µs | +4.54% |
| 100 | 317.314 µs | 347.103 µs | +9.39% |

Seven alternating samples of 100 iterations follow warmup, with black-boxed
outputs. One-schema min/max are 2.837–3.597 µs handwritten and 3.095–3.804 µs
derived; 100-schema ranges are 312.895–326.612 and 343.939–350.668 µs. These are
unpinned Linux x86_64 observations using Rust 1.99.1/LLVM 23.1.1, opt-3/LTO,
one codegen unit, panic abort and stripped symbols. The 100 names repeat one
concrete shape; this is not a 100-distinct-derive compilation benchmark.
Reproduce with `cargo run --locked -p ic-wire --features typescript --profile
benchmark-speed --example typescript-conformance -- target/native-schema-measured`.
The output directory includes `native-schema-host-benchmark.json`.

Separately built deployed Wire fixtures are byte-identical before/after:

| Profile | Wasm code-section bytes, before → after | Total Wasm bytes, before → after |
| --- | ---: | ---: |
| opt-z | 43,345 → 43,345 | 54,378 → 54,378 |
| opt-3 | 51,525 → 51,525 | 78,338 → 78,338 |

Both use `codec-wasm`, features `wire,derive-scale-1`, target
`wasm32-unknown-unknown`, and the evaluation/benchmark-speed profiles. Baseline
is `9a1c06e` with the candidate Cargo.toml/Cargo.lock (Tooling 0.64.1, Platform
0.6.8); candidate adds the metadata implementation. Separate target directories
prevent cross-checkout artifact reuse. Build each with `cargo build --locked
-p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown
--profile PROFILE --features wire,derive-scale-1`; `codec-report --code-size`
measures the code section. No IC instruction rerun is needed for identical
binaries; this does not requalify every consumer or the full Candid suite.

[Samples](evidence/native-schema-0523/host.json),
[source/compiler identity](evidence/native-schema-0523/identity.json) and
[artifact sizes/hashes](evidence/native-schema-0523/artifacts.json) retain the
measurement boundary. Existing 146 vectors and 29,560 mutation comparisons pass,
plus ten derived-native exact-frame fixtures in both directions. Custom hex
projections reject invalid strings and missing validators. Runtime implementations
and the TypeScript codec are unchanged; this adds a schema supplier for them.

## Standard Serde derives with finite admission (0.5.23 experiment)

An isolated, unpublished prototype replaces the benchmark records/enums' generated
Wire serializers with ordinary Serde Serialize/Deserialize derives. An experimental
`WireSchema` derive retains the existing Shape, schema and canonical hooks, assigning
positions from declaration order without field-ID annotations. Runtime encoding,
budgets and verification are unchanged. This is a derive-only experiment, not a
switch to `native` admission. Bounded Bytes/Text/List and finite field constraints
remain; unrestricted Vec/String contracts are not qualified by these results.

Matched opt-3/LTO Wasm results (bytes):

| Fixture | Code section, existing → Serde | Total Wasm, existing → Serde |
| --- | ---: | ---: |
| One record | 51,525 → 51,527 | 78,338 → 78,340 |
| 100 records | 355,288 → 345,103 | 389,823 → 379,595 |
| Standard corpus | 264,914 → 264,810 | 314,728 → 314,624 |

The 100-record fixture saves 10,228 total bytes (2.62%) and 10,185 code bytes
(2.87%). At opt-z it saves 4,604 total bytes and 4,608 code bytes. These are linked
fixtures, not symbol-attributed savings; the 100 nominal types use sixteen array
widths. The one-type opt-z artifact has equal size but a different hash; its opt-3 artifact grows
by two bytes. Both profiles, all artifact hashes and the standard corpus are in
[artifact evidence](evidence/serde-derive-0523/artifacts.json).

For all 100 records at opt-3, IC encode instructions fall 2.42–5.53% and decode
instructions fall 1.85–4.15%. The standard corpus retains the 1 MiB owned blob's
1,049,786 decode instructions; the ordinary native control remains 2,524,535.
Empty relay batch decode falls from 526.5 to 450.5. There are small regressions:
unit-enum encode rises from 500.25 to 505.25 (+1.00%); at opt-z one scale-record
decode rises by one instruction, and the worst scale truncation rejection rises
from 3,482.5 to 3,541.5 (+1.69%). No blanket speedup is claimed.

All 2,708 paired IC rows have unchanged input/frame hashes and three identical
samples. Each short operation repeats four times per query, or once above 64 KiB,
with no loop/counter subtraction. Both artifacts decode against the same host
oracle; standard and 100-type roundtrip/damaged-frame tests also pass. The
[instruction table](evidence/serde-derive-0523/instructions.csv) retains unchanged
controls and error-path costs as well as improvements.

This is evidence for separating Serde derives from Wire metadata/admission, not
approval to expose the prototype as a public API. It assumes reviewed ordinary
Serde implementations and has no general custom-serializer attribute validation.
A production derive must establish that contract before granting canonical
shortcuts, support the intended generic/bound policy and qualify TypeScript and
malformed-input behavior. Automatically assigned metadata IDs differ from the
old explicit IDs even where fixture bytes match; consumers hashing schemas would
need regenerated identities. No production codec/API or existing work was changed.

Native opt-3 timing uses two runs per variant, eleven calibrated batches of at
least 2 ms (cap 1,048,576 iterations), ordered baseline/candidate/candidate/baseline.
Empty relay decode moves from 23.98–24.23 to 21.82–22.07 ns; 1 MiB owned blob
remains approximately 17 µs (17.29–17.42 versus 17.13–17.35). Results are mixed:
32-hash manifest decode moves from 1.233–1.252 to 1.238–1.275 µs, and record-variant
decode from 70.39–74.95 to 72.91–75.93 ns. All 2,316 native row pairs preserve
input/frame hashes. These unpinned Linux timings include result destruction and
are separate from IC costs; allocation/heap measurements were not rerun.

[Native results](evidence/serde-derive-0523/native.csv),
[compiler/source identity and parameters](evidence/serde-derive-0523/identity.json)
and [raw samples, source snapshots, prototype patch and commands](evidence/serde-derive-0523/raw.json.gz)
retain reproduction evidence. Both targets use Rust 1.99.1/LLVM 23.1.1 and matched
locks/profiles; PocketIC is 16.1.0. Sources live in isolated baseline/candidate
copies of `9a1c06e` with the same updated Cargo selections. No new Candid IC run,
full CI, macOS or production migration was performed.
