# Evaluation scope

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The maintained qualification exercises the selected Postcard codec, its
generated TypeScript implementation and an isolated Candid comparison suite.
The optional typed endpoint integration has a standalone Platform/Testkit fixture;
framework policies and browser SDKs remain consumer-owned. The unpublished codec
benchmark package may use PocketIC for isolated IC instruction measurements;
it does not qualify consumer endpoint integrations.

The released 0.6.0 Serde hard cut removed the finite Wire API. Archived sections recording
finite admission and WireSchema experiments are historical research; the
[current production comparisons](candid-comparison.md)
owns shipped-codec IC, frame and Wasm measurements. The retired protocol-study
runner is available only in the archived finite revision linked by those studies;
it is no longer maintained alongside the production Serde suite.

The [4 November full matrix](candid-comparison.md) now
measures its complete frozen 26-cohort roster, including the formerly skipped hotspot cohort:
1,387 overlapping case entries, 5,548 primary comparisons and 104 fresh modules.
It includes the complete 391-case standard corpus and the Platform/Testkit
selection recorded in that report, not the current lockfile. [Coverage gaps](candid-comparison.md) and
[ranked hotspots](candid-comparison.md) distinguish
supported field choices from proposed research and missing comparison controls.
The maintained roster now adds `coverage-shapes` and paired `research-bulk`:
[separate follow-up evidence](evidence/primitive-bulk.md) keeps these new
controls distinct from that frozen full run. Candid DTOs remain test-only; Wire
serializes owned application representations. Private float/bool fields do not
extend the production Packed element set or generated client contract.

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
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.99.1 floor |

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

The focused `byte-keys` and `byte-keys-control` cohorts compare standard
`ByteArray<32>` keys against unchanged `[u8;32]` map keys at 0/1/16/128 entries.
Both keep identical Candid maps and logical inputs. The
[matched study](evidence/byte-array.md) retains repeated native time,
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
[0.13.4 investigation](evidence/cost-experiments.md) records five
independent candidates, rejected tradeoffs and the selected admission cleanup.
It includes repeated native samples, IC counters, unchanged allocation/heap
observations and total uncompressed Wasm/code-section measurements. The large
unit-list encode gap at opt-z remains; native and IC changes are different.

The [follow-up parser screen](evidence/parser-screen.md) rejects
three independent rewrites using rebuilt 100-type raw/code modules. None improves
size in either profile, so they do not proceed to instruction/timing qualification.

`check-codec-costs` compares completed, matched runs with unchanged Candid
controls. It rejects growth in either raw Wasm metric and instruction increases
above baseline × 1.05 + 64 per operation. The absolute margin is explicit for
tiny inputs; this is a screening policy, not a universal performance promise.
It starts no server, never updates a baseline and does not replace native-time
review or the complete comparison matrix. Ordinary CI runs its cheap corruption
fixtures through `check-benchmark-cli`; measurements remain explicitly invoked.
See [commands and evidence requirements](development.md#paired-cost-regression-check).

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

The [Account and fragmented-blob study](evidence/hotspots.md) retains
real-supplier before/after measurements, the approved coordinated Account cut,
matching schema/byte/budget proofs and rejected encoder experiments. The earlier isolated preparation is historical; the coordinated cut shipped
with Core 0.16 / canonical Types 0.18. Current selection and requalification are
recorded in the [handoff](status/current.md).

For late failures in blob batches, callers can deserialize bounded `BytesRef`
views and copy into owned `Bytes` only after the full frame has been accepted.
The [compiled qualification example](../crates/ic-wire-qualification/tests/deferred_blob_ownership/mod.rs)
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
[marked-buffer investigation](evidence/marked-buffer.md) reproduces
approximately 15 µs versus 1 µs for identical bytes at different offsets, including
raw copies with all decoding removed. This is native attribution evidence;
IC costs and Wasm footprint require their own measurements.

See [borrowing, buffers and bounded layouts](allocation-comparison.md) for the
historical 44-case experiment; its finite API/canonicalization claims are historical.

## Historical evidence

The [historical experiments](history/evaluation.md) preserve earlier measurements,
source revisions and rejected designs. They are not supported runtime APIs.
