# Evaluation scope

The maintained qualification exercises the selected Postcard codec, its
generated TypeScript implementation and an isolated Candid comparison suite.
The optional typed endpoint integration has a standalone Platform/Testkit fixture;
framework policies and browser SDKs remain consumer-owned. The unpublished codec
benchmark package may use PocketIC for isolated IC instruction measurements;
it does not qualify consumer endpoint integrations.

## Native qualification ownership

| Owner | Proof | Execution entrypoint |
| --- | --- | --- |
| Core | Codec bytes, malformed input, budgets, borrowing and generated schemas | `test`, `check-qualification`, `check-typescript` |
| Endpoint | Local codec plus generated exports, handler composition and rollback | `check-endpoint`; installed execution through `check-endpoint-ic` |
| Private benchmark harness | Codec/admission instruction measurements and the explicit Endpoint borrowing probe | `evaluate-codecs.sh`, `codec-instructions`, `admission-instructions`, `input-borrowing` |
| Testkit | PocketIC transport, server admission and lifecycle | Optional native `ic` dependency used through raw query/update calls |

Wire constructs and decodes frames itself. Its native harnesses deliberately do
not enable Testkit's optional `wire` adapter. Testing/adopting that upstream
adapter belongs to Tooling ([#349](https://github.com/dragginzgame/ic-tooling/issues/349));
it is not a reciprocal Wire release prerequisite. Published Core/Endpoint
normal/build graphs stay Testkit-free. The two harness packages remain private,
with Testkit selected only by the native `ic` feature and defaults disabled.

`make check-testkit-boundary` verifies ten product graphs and rejects Build,
Testkit, the Platform facade and Tokio anywhere in the product resolution.
`make check-integration-boundary` verifies twenty product/private graphs using
the separate qualification lock: default native harnesses exclude Testkit,
while IC codec/Endpoint selections and the Endpoint borrowing probe select it.
It rejects Testkit's Wire feature, registry/duplicate Wire identities, split
local Types/Runtime identities and missing execution guards. These are composed
consumer checks, not separate supplier qualification or reciprocal release waits.
Every local direct Types requirement must match Core. Published schema/Endpoint
graphs still require the canonical generation, and execution has one guard.
The report records actual
local Wire, Types and Runtime identities, source hashes and graph inputs under
`target/product-boundary.*/` and `target/integration-boundary.*/`; neither starts
PocketIC or rebuilds suppliers.

The existing `check-boundary` stage owns the product proof; `check-benchmarks`
owns the private integration proof and future-facade identity simulation.
Core retains 20 stages and qualified Endpoint-only changes retain 11. Reviewing
the group selector, release adapter and compatible-consumer checks found no
Testkit adapter release wait to remove. Required Wire product and installed
composition proofs stay with their existing owners; private fixture membership
does not make simulator infrastructure or Tooling's adapter a published Core API.
This enforcement resolves [Wire #57](https://github.com/dragginzgame/ic-wire/issues/57)
without a new crate, client wrapper, supplier pin change or codec change.

## Candid bigint type identity

The private benchmark package's only direct num-bigint uses construct upstream Candid
`Nat` values, including the timed bounded-magnitude conversion. Candid 0.10.40
exposes num-bigint 0.4's `BigUint` in that public tuple field. A 9 November probe
with num-bigint 0.5.1 fails with E0308 in `fixtures/followup::nat_case` and
`unsigned_magnitude::to_nat`; the two `BigUint` types are distinct.

The maintainer approved retaining 0.4.8 on 9 November 2026 solely as this private
Candid-comparison dev dependency. Revisit when the Candid supplier accepts the newer type or this
comparison is retired. Adding decimal/byte conversion solely to accommodate two
versions would change the measured conversion path. Published Wire packages do
not depend on num-bigint or Candid. Probe: `target/deps0374/bigint-coverage-probe.log`.

SHA-2 is independent: private evidence producers select 0.11.1 with defaults
disabled, and format digest bytes explicitly as lowercase two-digit hex. Existing
golden corpus hashes verify unchanged digest inputs and outputs. The older
transitive SHA-2 remains supplier-owned; it does not hold back this selection.

## Historical context

The released 0.6.0 Serde hard cut removed the finite Wire API. Archived sections recording
finite admission and WireSchema experiments are historical research; the
[current production comparisons](candid-comparison.md)
owns shipped-codec IC, frame and Wasm measurements. The retired protocol-study
runner is available only in the archived finite revision linked by those studies;
it is no longer maintained alongside the production Serde suite.

The [9 November full matrix](candid-comparison.md) measures its complete frozen
39-cohort roster: 2,194 overlapping case entries, 8,776 primary comparisons and
156 fresh modules. Its report separates 38 existing-field/control cohorts
from the mixed cohort containing the public bool adapter, private float bulk
prototypes and unchanged controls.
It includes the complete 391-case standard corpus and the frozen Platform/Testkit
selection recorded in that report. [Coverage gaps](candid-comparison.md) and
[ranked hotspots](candid-comparison.md) distinguish
supported field choices from proposed research and missing comparison controls.
The full run includes `coverage-shapes` and paired `research-bulk`; the
[original follow-up](../evidence/primitive-bulk.md) retains their design and
earlier measurements. Candid DTOs remain test-only; Wire
serializes owned application representations. Private float fields do not
extend the production Packed element set or generated client contract; the
explicit bool adapter is already public.

The maintained `id16` and `id16-control` cohorts each contain 20 workloads:
0/1/8/128/4,096 identifiers in lists and filter records, with owned and
borrowed-input encode paths. The control retains ordinary arrays; `id16` uses
public Packed fields. Candid receives already-flat blobs in both cohorts and
validates ID/cursor widths during decoding. Its DTO setup stays outside counters;
the Wire borrowed-input path explicitly includes copying. Both cohorts are
included in the current full matrix.

The `signed32` cohort adds 49 cases without changing any existing case indices:
17 zigzag boundary scalars plus ordinary lists with small, negative, wide and
mixed values; `VecDeque`, `BTreeSet`, tuple/option/result combinations; and Packed
controls at 0/1/128/4,096 values. Set names denote actual cardinality. Packed and
ordinary wide lists retain identical logical inputs and Candid frames; their
Wire layouts intentionally differ. Names and order have a frozen digest.
Tests reject every truncated scalar prefix and overflowing varint, including
the four-/five-byte transition beyond the timed scalar set. Redundant zero is
accepted under Core 0.20's upstream Postcard rule; encoding remains minimal.
Deque tests hold logical input fixed across every physical split at 128 values,
plus empty/singleton controls, and check exact encoder allowances and one-short
allocation/node/depth refusal while preserving list bytes in both codecs.
This is native Serde coverage, not new TypeScript schema or adapter support.
The maintained roster has 39 cohorts, all included in the current full matrix.

## Maintained checks

| Target | Evidence |
| --- | --- |
| `test` | Runtime/native bounds, canonical bytes, typed failures and retry buffers |
| `check-typed-diagnostics` | Actual compile-fail subjects and a passing control |
| `check-typescript` | Generated types, Rust vectors and differential malformed-input checks |
| `check-benchmarks` | Matched Candid/Wire corpus, malformed-frame checks, lint and MSRV |
| `check-consumer-benchmarks` | Toko-shaped and 1/10/100-type derive corpus, lint and MSRV checks |
| `evaluate-codecs` | Repeated native timing, allocator counts, peak live heap and Wasm footprint |
| `check-codec-costs` | Data-only matched cost regression check; completed baseline/candidate runs required |
| `check-qualification` | Allocation/borrowing fixtures, frozen contracts and conservative snapshot comparison |
| `check-endpoint` | Typed endpoint/native schema checks, compiler diagnostics, mixed-version consumers, lint, Wasm/MSRV and graph isolation |
| `check-endpoint-ic` | Explicit installed endpoint/rollback and async qualification using a prepared PocketIC server |
| `evaluate-endpoints` | Every exported fixture endpoint: repeated complete IC instructions, observed cycle debits, frame sizes and Wasm; explicitly prepared instrumented server |
| `check-wasm` | Runtime compilation for Wasm under the Rust 1.99.1 floor/pin |

The unpublished `ic-wire-qualification` package owns the maintained allocation,
borrowing and buffer-reuse fixtures plus the IC Build compiler-diagnostic runner.

The standard Wire/Candid corpus includes empty and singleton controls, zero-sized
unit/record collections, nested empty lists, binary-key maps, blob-valued results,
fixed optional arrays, boxed values, Unicode `Cow<str>` and tuple/record enum
variants. New workloads append to the corpus so existing IC case indices stay
fixed. Binary-key maps use ordered pairs for their JSON input identity only;
both measured codecs still serialize the map. These cases use the existing native,
IC instruction and Wasm-footprint runners; adding coverage is not a new performance
measurement. Historical reports retain their original corpus and source identity.

The standard corpus test freezes both its 391-case count and ordered-name digest;
review an intentional inventory update when adding cases, never silently bless
a smaller corpus or a reordered case index.

The `coverage-shapes` follow-up now contains 154 cases. Twenty appended cases
cover paginated records containing Packed words, mixed unit/newtype/record enum
batches with byte payloads, sparse optional blobs, maps of per-item results and
mixed-width integer tuples. Each uses 0, 1, 128 and 4,096 entries. Matched external
Candid values preserve the same logical inputs; DTO construction is untimed.
The existing 134 indices remain unchanged, and selected IC cases must match the
native inventory byte for byte. These additions enter all four follow-up cohorts;
the historical pre-optimization matrix contains the preceding 134 cases. The
Core 0.20 refresh includes all 154 in each of those four cohorts.

Ten finite-only
f32/f64 cases exercise the public field adapters (empty/singleton through 1 MiB)
against raw Candid vectors, preserving signed zeros, subnormals and finite maxima.
They are separate from the all-IEEE research controls. These additions have
focused correctness qualification; the preceding full matrix predates them.
The earlier 30 container cases
cover nested optional byte buffers (outer absent, inner absent and empty),
success/error results containing packed integers or blobs, byte-array versus
Packed hash lists, nonzero signed-integer lists and binary heaps. Collections
have empty, singleton, 128- and 4,096-element controls. The original 94 names and
indices are frozen separately. Tests check malformed option tags, every prefix
of small option payloads and rejection of a late zero in a nonzero list.
Candid hash-list controls use identical external byte-vector DTOs. Nonzero
admission and heap reconstruction are additional Wire-side semantics, not
identical invalid-value or reconstruction policies. These native Serde cases
do not add schema support or change production bytes, limits or acceptance.

The `borrowed-containers` selection uses the same 154 logical inputs with public
`serde_packed` field adapters for the nonzero-list and heap cases. The private
`packed-containers` control copies into Packed inside serialization; the original
one-copy DTO is also preserved in paired study snapshots. Candid's external
integer-vector DTOs remain unchanged. Do not attribute gains over the twice-visited
copying adapter to the more efficient original DTO control.

Core tests cover exact packed bytes, real admission transitions, zero/partial-word/
trailing rejection and unordered/duplicate heap reconstruction. Generated browser
vectors enforce nonzero constraints without callbacks. Standalone consumer tests
cover aliases, generic containing records, no_std host/Wasm and codec-only feature
isolation. Exact record-field encoding allowance may differ from an owned Packed
field; transparent-root equivalence alone is insufficient. Consumer adoption from
ordinary lists is a coordinated layout cut; existing consumers stay unchanged.

The `adapter-scale` corpus reuses the existing nominally distinct 1/10/100-record
roster with both public domain adapters. `adapter-scale-dto` selects matching
Packed fields as a footprint control; these DTOs are constructed outside timing,
so it does not represent domain-to-DTO conversion latency. Records retain their
varying fixed arrays and visible runtime dispatch. Both use one production codec;
this is a monomorphization stress fixture, not an application savings claim.
Select the documented rows in `scripts/ci/codec-cohorts.psv` through
`CORPUS_FEATURE=... scripts/ci/evaluate-codecs.sh /absolute/fresh/output`.

The focused `byte-keys` and `byte-keys-control` cohorts compare standard
`ByteArray<32>` keys against unchanged `[u8;32]` map keys at 0/1/16/128 entries.
Both keep identical Candid maps and logical inputs. The
[matched study](../evidence/byte-array.md) retains repeated native time,
IC instructions, frame bytes, allocation/heap probes and raw Wasm/code sizes.
This explicit representation improves nonempty-map instructions while increasing
Wasm and wire size; ordinary controls and existing corpus indices remain intact.

The standard corpus now appends 58 contract-shape cases (391 total), also
selectable alone with the private `contract-shapes` cohort. These cover actual
Platform Accounts with absent/zero/nonzero subaccounts, Principal-keyed maps,
Account sets, 8/32/128-field records, sparse optional records, 8/32/129-variant
enums and equal-byte-count/ragged blob batches. Candid uses explicit external
identity DTOs; conversion/setup is outside measurement. Account controls compare
raw owner/subaccount fields because its equality normalizes absent and zero.
Variant indices 127/128 and malformed variant 129 have direct byte controls.
The focused cohort measures its complete linked Wasm, not per-type attribution
or projected application savings. Existing workloads keep indices 0–332.

Rust-only native tests additionally qualify Serde's standard nonzero integers,
network addresses and ranges, including invalid values, every truncated frame
prefix and trailing-byte refusal. Raw floating-point checks compare IEEE bits,
including signed zero and NaN payloads, rather than float equality. These codec
checks do not imply generated TypeScript schema support for every Rust type.

The private `cost-smoke` cohort retains 20 scalar, blob, zero-sized collection,
ordinary wide-integer, record and byte-key-map controls. It uses the existing
codec runner, not another production strategy. The
[0.13.4 investigation](../evidence/cost-experiments.md) records five
independent candidates, rejected tradeoffs and the selected admission cleanup.
It includes repeated native samples, IC counters, unchanged allocation/heap
observations and total uncompressed Wasm/code-section measurements. The large
unit-list encode gap at opt-z remains; native and IC changes are different.

The [follow-up parser screen](../evidence/parser-screen.md) rejects
three independent rewrites using rebuilt 100-type raw/code modules. None improves
size in either profile, so they do not proceed to instruction/timing qualification.

`check-codec-costs` compares completed, matched runs with unchanged Candid
controls. The oracle policy and all follow-up/container fixture identities must
match, even when their reported counters and frames happen to agree. It rejects
growth in either raw Wasm metric and instruction increases
above baseline × 1.05 + 64 per operation. The absolute margin is explicit for
tiny inputs; this is a screening policy, not a universal performance promise.
Opt-z drives adoption decisions; opt-3 checks secondary regressions. A measured,
explicitly accepted instruction/size tradeoff may justify small Wasm growth;
retain the automated failure and the decision rather than claiming no growth.
It starts no server, never updates a baseline and does not replace native-time
review or the complete comparison matrix. Ordinary CI runs its cheap corruption
fixtures through `check-benchmark-cli`; measurements remain explicitly invoked.
See [commands and evidence requirements](../guides/development.md#paired-cost-regression-check).

Earlier alternative-codec suites were retired in 0.5.19. Their
[historical reports](../history/README.md) retain source-specific findings;
retired raw artifacts and old Git history are not reproduction guarantees.
The maintained [Candid suite](candid-comparison.md) belongs to the unpublished
`ic-wire-benchmarks` package. Raw Postcard controls omit Wire admission and
are not selectable runtime modes. Native timings, IC instructions and artifact
sizes remain distinct evidence.

Run focused checks under the shared [validation policy](../../TOKO-TOOLING.md#validation-and-evidence).
The complete `make ci` scaffold gate is reserved for an explicit local request
or the maintainer's release flow. Wire does not qualify sibling endpoints,
authentication, storage formats or delivery policy.

The [Account and fragmented-blob study](../evidence/hotspots.md) retains
real-supplier before/after measurements, the approved coordinated Account cut,
matching schema/byte/budget proofs and rejected encoder experiments. The earlier isolated preparation is historical; the coordinated cut shipped
with Core 0.16 / canonical Types 0.18. Current selection and requalification are
recorded in the [handoff](../status/current.md).

For late failures in blob batches, callers can deserialize bounded `BytesRef`
views and copy into owned `Bytes` only after the full frame has been accepted.
The [compiled qualification example](../../qualification/ic-wire-qualification/tests/deferred_blob_ownership/mod.rs)
checks borrowing, unchanged encoding, bounds and copy ordering. This uses the
existing Serde decoder. Borrowed traversal has its own admission charges;
subsequent ownership allocations need a consumer budget in addition to the
frame/field bounds. It is not a transparent replacement for owned decoding.
Ingress still copies incoming bytes into the frame first, and views cannot
outlive that frame; the synchronous Endpoint borrowing path can use such views.
Large late-failure savings have small-input and Wasm tradeoffs in the study.

For native large-buffer comparisons, retain a raw-copy control over the same
source and vary buffer placement as well as codec order. Repeated batches over
one allocation can repeat a libc copy-path cliff: the
[marked-buffer investigation](../evidence/marked-buffer.md) reproduces
approximately 15 µs versus 1 µs for identical bytes at different offsets, including
raw copies with all decoding removed. This is native attribution evidence;
IC costs and Wasm footprint require their own measurements.

See [borrowing, buffers and bounded layouts](../history/allocation-comparison.md) for the
historical 44-case experiment; its finite API/canonicalization claims are historical.

## Historical evidence

The [historical experiments](../history/evaluation.md) preserve earlier measurements,
source revisions and rejected designs. They are not supported runtime APIs.

## Host schema rendering suppression (8 November 2026)

Issue [53](https://github.com/dragginzgame/ic-wire/issues/53) is implemented using
one runtime rendering flag in the existing metadata walker. Standalone schema
validation, registry finish and bulk prevalidation visit the same nodes and
checks but suppress type-string construction. Generation enables rendering.
There is no second validator or per-type codec implementation. Bytes, schema
identities, limits, acceptance and generated output remain unchanged; no consumer
action or storage migration is required.

Matched sources use Rust 1.99.1, the same lock and native target, separate Cargo
targets, evaluation (opt-z) and benchmark-speed (opt-3) profiles. The maintained
private `schema-cost` example uses stats_alloc for both lanes. Inputs are built
outside measurement. Each profile has three before/after runs with 11 calibrated
batches each; below are medians of 33 per-operation samples. Allocator accounting
is enabled in timing runs, so these are instrumented native times, not IC costs.
Baked source hashes distinguish paired artifacts; input and generated-output
hashes and rejection results agree in every row.

| Validation shape | opt-z before → after | Allocations before → after | opt-3 time change |
| --- | --- | --- | --- |
| One-field record | 531 → 318 ns | 10 → 3 | −52.2% |
| 4,096-field record | 1,138,542 → 675,667 ns | 12,929 → 637 | −45.7% |
| 128 fields, 32 nested options each | 666,319 → 293,642 ns | 4,506 → 22 | −75.7% |
| Large registry with invalid unused definition | 1,160,866 → 689,655 ns | 12,926 → 635 | −44.8% |

Generation allocations remain equal. At opt-z, small generation adds 85 ns
(4.0%), large changes −0.3%, nested +0.2%, and invalid-unused +2.4%. At opt-3,
generation changes −2.9%/−3.1%/+1.0%/+2.2% respectively. These host regressions
are retained alongside the validation benefit. This code is host-only: no
production Wasm or IC instruction saving is claimed. Macro compilation time
and native executable size were not measured.

Reproduce using `cargo run --locked --manifest-path qualification/Cargo.toml -p ic-wire-qualification --example schema-cost
--features schema-cost --profile evaluation` on each frozen source, selecting
a separate CARGO_TARGET_DIR. Repeat with benchmark-speed. Local raw results,
compiler identity, source/lock hashes and summary: `target/schema-render-study/`.
Native tests (75 passed, three opt-in probes ignored), expanded payload rejection
cases, strict Clippy, Rust/TypeScript conformance, simulated big-endian checks
and dependency boundaries pass. Allocation decreases reflect this host fixture,
not application runtime savings.

## Postcard derive dependency qualification (8 November 2026)

Upstream already merged [Postcard PR 219](https://github.com/jamesmunns/postcard/pull/219)
(commit 9d14e69d5aef2aac6d6beb83bbcf486eae53562e). It replaces the error enum's
derived serialization with manual implementations and enables Serde derive only
for tests. Current upstream 7fdd14028b51a5ff7480c48843999f6209eee735 contains it.
The freshly fetched crates.io index still ends at 1.1.3, whose manifest enables
Serde derive on the normal dependency. A duplicate upstream contribution is
unnecessary; Wire adoption requires a published supplying release.

A disposable standalone Wire codec consumer patches Postcard to the pinned
upstream source for qualification only. A derive-control feature re-enables
serde/derive, isolating that feature's build cost with the same sources, lock and
compiler. This is not a complete published-1.1.3 versus upstream comparison.
Rust 1.99.1 native default dev-profile builds use fresh targets, incremental
compilation disabled, compiler wrappers unset, locked offline dependencies and
prepared download/page caches. Three sequential alternating clean builds give:

| Lane | Absolute build seconds | Median |
| --- | --- | --- |
| With derive control | 2.65, 2.53, 2.51 | 2.53 s |
| Without derive control | 1.89, 1.85, 1.83 | 1.85 s |

This fixture improves 26.9%. It removes serde_derive from the selected normal/build
graph; syn, quote and proc-macro2 remain through cobs/thiserror. Consumers already
selecting derive elsewhere should not expect this dependency-removal saving.
No linked Wasm size, IC instruction or codec throughput improvement is claimed.
The measurement is a native compilation study; opt-z/opt-3 runtime profiles are
not its objective. Source archive and fixture/lock hashes, compiler identity,
graphs and raw logs are under `target/postcard-derive-study/`.

Primitive exact bytes and malformed input pass, as do both directions of all
16 Postcard error variant byte indices (plus invalid index rejection), with and
without the derive control. The existing standalone schema fixture passes its
derive/metadata/byte test; codec-only and schema-enabled Wasm checks pass. This
is focused Wire consumer qualification, not a full upstream feature/test matrix
or a complete future release adoption. Production manifests/lock selections are
unchanged by this study; no downstream patch is required.
