# Evaluation scope

The maintained qualification exercises the selected Postcard codec and its
generated TypeScript implementation. Endpoint adapters, simulators, browser SDKs
and installed canister drivers are outside this repository.

## Maintained checks

| Target | Evidence |
| --- | --- |
| `test` | Runtime/native bounds, canonical bytes, typed failures and retry buffers |
| `check-typed-diagnostics` | Actual compile-fail subjects and a passing control |
| `check-typescript` | Generated types, Rust vectors and differential malformed-input checks |
| `check-evaluation` | Native roundtrips against private raw-codec controls and strict linting |
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.91 floor |

The maintainer-requested [MessagePack comparison](messagepack-comparison.md) adds
an optional raw-codec experiment with retained native, external IC instruction
and linked Wasm size evidence. `make evaluate-messagepack` reruns its native
portion; external installed probes remain outside the maintained workspace.

Private raw Postcard, MessagePack, bitcode and wincode controls have different admission and
canonicality guarantees. They are not production codec modes. Optional
[native timings](performance.md) must retain compiler, target, profile, inputs,
lock selection and codec parameters. Native nanoseconds do not establish IC
instruction counts, deployment size or end-to-end latency.

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
