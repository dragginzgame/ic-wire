# Required feature parity

Local 0.23.8 work following the [pushed 0.23.7 comparison](comparison.md).
Reference remains Core 0.22.4, revision
`2dc434b10bec2fca92345d940dbfedc05993e9d5`. Legacy compatibility is not required: the user authorized a hard cut. The
reference supplies comparison evidence; it does not dictate the new API or format.
Actual production deployment remains outside this local work.

## Contracts and disposition

| Surface | Evidence / required contract | Current disposition |
| --- | --- | --- |
| Result, arrays, tuples | Blob protocol uses Result replies, `Root = [u8;32]`, header pairs and multi-field variants | Implemented; fixed containers omit lengths, Result tags are u32 varints |
| Slices and Cow text | Reference native_schema public projections | Implemented; sequence/string bytes retained |
| ByteBuf/ByteArray and serde_bytes | DB rollback fixtures and entropy probe use ByteBuf; Account uses optional serde_bytes | Published types reused; sealed adapter projection only |
| Platform Principal/Account | Blob protocol identity fields; published Types 0.20.4 contract | Host-only integration; supplier owns Serde and identity |
| Finite scalar floats | Reference public finite wrappers | Implemented; finite bits preserved, raw float host projection remains excluded |
| Generic packed integers/IDs | Reference public Packed contract | Implemented for i8/i16/u16/i32/u32/i64/u64 and 16/32-byte IDs |
| Domain packed adapters | Candid controls expose large ordinary-list costs; explicit adapters retain validation/reconstruction | Implemented bool, finite f32/f64, nonzero i64 and heap u64; matched performance remains outstanding |
| Scalar bounds and custom domain checks | Advice must not discard constraints; typed arrays alone do not enforce consumer invariants | Implemented scalar bounds and named synchronous browser checks |
| Rooted bulk lint and generation diagnostics | User explicitly prioritizes lint; Candid comparisons demonstrate representation cliffs | Implemented on the admitted graph; payload/method generation requires a handler |
| Scoped scratch/fallback output | Design requires a named caller needing drop-before-callback | Caller requirement unresolved; existing owned/reused/slice outputs retained |
| Endpoint/executor, transport/retry | Explicitly assigned downstream by design | Excluded from Wire parity |

Local consumer observations are read-only evidence, not consumer migration:
`../ic-blob-storage/crates/ic-blob-storage/src/protocol.rs`, `src/local/types.rs`
in that package, and `../ic-blob-storage/crates/ic-blob-storage-canister/src/lib.rs`;
DB rollback and entropy probe qualification remain outside this repository.
Absence of call sites does not retire any public feature. The user requested an
evidence-based retention decision using the reference's Candid comparisons and
explicitly prioritized linting. The review below resolves the data-adapter and
advice scope. Implementation and qualification are recorded below.

## Candid evidence and retention decision

Reviewed the clean sibling at the pinned revision above, specifically
[`docs/benchmarks/candid-comparison.md`](../../../ic-wire/docs/benchmarks/candid-comparison.md),
[`docs/evidence/primitive-bulk.md`](../../../ic-wire/docs/evidence/primitive-bulk.md),
[`docs/evidence/rooted-bulk-lint.md`](../../../ic-wire/docs/evidence/rooted-bulk-lint.md),
[`docs/evidence/default-bulk-advice.md`](../../../ic-wire/docs/evidence/default-bulk-advice.md),
and the actual bulk-lint implementation and composition tests. These are historical
source-specific measurements. The matrix's raw artifacts were removed on
12 November; summaries justify retained requirements, not verified performance
claims for this candidate. The latest table uses Rust 1.99.1 and different supplier
versions from this checkout. Do not mix numbers from successive studies.

| Historical observation | Retention consequence |
| --- | --- |
| Core 0.21 ordinary 4,096-ID decode costs 128.2× Candid; explicit packed decode costs 0.74× | Keep packed IDs and warnings for ordinary ID lists, including aliases |
| Core 0.21 wide-u16 encode costs 110.7× Candid; packed costs 0.40× | Keep integer packing and ordinary small-value controls; warnings recommend review, not automatic conversion |
| Bool study: 1 MiB encode falls from 97,519,445 to 1,050,620 instructions; Candid 2,118,112 | Keep checked bool adapter even though valid byte sequences match ordinary bool lists; accounting differs |
| Core 0.20 nonzero-i64 adapter encode/decode costs 0.40×/0.33× Candid; heap adapter 0.40×/0.50× | Keep domain adapters, including nonzero validation and heap reconstruction; Candid's vector controls do less semantic work |
| Later finite study reports 2.29×/1.95× Candid encode and 0.47×/0.51× decode for f32/f64 | Keep explicit finite adapters, signed-zero/subnormal preservation and nonfinite rejection; do not adopt the faster private all-IEEE prototype |
| Packed/small controls can regress; mixed-width tuple encode still costs 2.63× opt-z and 6.68× opt-3 Candid | Preserve workload-specific advice and bounds; neither packing nor linting fixes every aggregate-encoding cost |

### Required lint behavior

The destination is a traversal of the already admitted graph, not another JSON
validator and not a new codec. Preserve these reference responsibilities:

- Run advice automatically during client generation, with an explicit required
  diagnostic handler. A standalone lint API is also useful, but cannot be the
  only way warnings appear. Keep canister/runtime dependencies unchanged.
- Review public request/response use sites independently, following references
  without exposing generated names in paths. Shared definitions must not share
  acknowledgements across unrelated fields or methods. Use deterministic paths,
  JSON-pointer escaping, container positions and primitive-width suffixes.
- Accept only exact-path acknowledgements with nonempty reasons. Report removed
  or changed candidates as stale acknowledgements. No wildcard/global suppression;
  CI must be able to reject both warnings and stale acknowledgements.
- Recognize byte/integer/ID lists, bool and finite-float candidates, unsupported
  i128/u128 packing, and zero-width unit/empty-record/empty-tuple/empty-array lists.
  Explicit byte/packed fields are not candidates. Zero wire bytes do not imply
  zero traversal work. Do not recommend nonexistent `Packed<f32>` or `Packed<u128>`.
- Retain bounded work, active-path recursion detection, per-root and aggregate
  ceilings, and deterministic bounded display (reference: eight entries plus an
  omitted-count summary). Never silently return a clean or incomplete report
  after complexity exhaustion. The reference's composed-service tests exercise
  594 public roots and 15,925 distinct warnings, beyond a single-root workload.
- Preserve field bounds, scalar/domain checks, and original Serde declarations.
  Advice is not a schema rewrite or a performance guarantee. Acknowledgements
  do not authorize dropping constraints or normalizing retained bytes.

Automatic advice has a measured host cost: the historical 1,000-list-field
generator rose from roughly 349 to 676 microseconds. Keep visibility and measure
the new graph implementation; do not hide this cost inside codec or Wasm claims.
Qualification must compare warning paths, acknowledgements, recursive/shared
graphs, refusal bounds and generated source independence, then exercise the
actual standalone consumer generator with a visible handler.

Implementation order: rooted lint and mandatory generation diagnostics; domain
adapters plus matching schema/runtime validation; focused matched measurements
including ordinary/small controls. No broad legacy benchmark tree needs copying.
The scoped scratch helper remains conditional on a named caller: its separate
study improves some operation counts but adds 722/2,490 total Wasm bytes at
opt-z/opt-3. That does not establish a required replacement for existing slice
and reusable-buffer APIs. Endpoint execution and alternative codecs stay excluded.

### Implemented rooted advice

`ValidatedRegistry::lint_bulk_roots` accepts public labels mapped to admitted type
names and exact-path acknowledgements. It traverses private nodes directly: no
JSON export/reparse, second schema validator or parallel representation authority.
References preserve use-site paths; newtype/container positions retain `/item`.
The source contracts and traversal fixtures come from the pinned reference's
`typescript/bulk_lint/mod.rs` and its composition tests; implementation uses the
destination graph under the retained repository license.

`payload_module` and `method_module` now require an acknowledgement map and a
diagnostic callback. The callback receives one complete report after successful
generation, including when clean; it is not called on failure. Payload roots use
`/types/NAME`; method roots use `/methods/EXPORT/request` and `/response`.
Only selected public roots are reviewed. Callers should display the report or
enforce `is_clean()` in CI. Each invocation's acknowledgement map is scoped to
its roots. Raw JSON export and declaration-only rendering remain graph projections;
standalone lint is available to their callers without client generation.

Reference ceilings of 65,536 visits/root, 1,000,000/report and depth below 64 are
retained. The candidate additionally rejects paths above 4 KiB, retained diagnostic
paths above 16 MiB and more than 1,000,000 acknowledgement entries. These are explicit
host diagnostic resource limits, not runtime budgets or wire acceptance changes.
They return Complexity without a partial report. Display shows at most eight
entries and an omitted-count summary; the full admitted report remains available.
Bool/float warnings now name the implemented explicit adapters while preserving
bound/admission review and excluding unsupported generic float packing.

Five new qualification tests compare rooted paths and stale acknowledgements
against published Core 0.22.4, cover malformed acknowledgements, public-label
escaping, shared recursive graphs, zero-width/unsupported scalar lists, exact work
ceilings and 594 composed roots yielding 15,925 warnings. They verify callback
counts and source independence. All 18 TypeScript/host qualification tests pass,
along with host/derive tests, strict Clippy, documentation and forwarded-feature
Wasm checks. No host-cost improvement is claimed; timing remains future work.

The standalone consumer forwards the required handler and checks both tags-list
warning paths. Its native generator prints diagnostics; `--deny-warnings` fails
before creating output when advice is unreviewed. Both ordinary success and deny
failure were executed, and the generator passes strict Clippy. These changes are
host-only; the earlier size report retains its original source hashes and has not
been relabelled as a new measurement.

### Implemented domain adapters

The public `serde_packed` modules now cover bools, finite_f32, finite_f64,
nonzero_i64 and heap_u64. They explicitly serialize a byte buffer through the
existing Serde/Postcard path. Nonzero and heap encoding reuse the packed word
serializer; decoding reuses owned packed words before enforcing nonzero values or
reconstructing the heap. Bool decoding uses a safe checked Bytemuck view. Both
finite widths share one source pattern for validation, endian conversion and
owned decoding. The reference's unrolled/exponent-carry scanner optimizations
were not imported without a new matched measurement.

Reference `serde_packed/*/mod.rs` supplies the selected contracts and minimal
adapter algorithms under the retained license. Bool and finite spans reject
invalid values before output allocation. Nonzero validation follows packed-word
allocation, matching the reference; it does not claim pre-allocation rejection.
Heap decoding accepts duplicates and unordered input and may change re-encoded
order. Neither Rust nor browser adapters provide canonical retained bytes.

The strict derive recognizes the exact adapter path in the selected Wire crate
(including a renamed dependency and leading `::`). Sealed input traits admit the
actual Vec<bool/f32/f64/NonZeroI64> or BinaryHeap<u64> type, including aliases, not
similar spellings or an arbitrary serializer. These scalar field projections stay
inline in named registries rather than inventing additional adapter type names.
Final graph admission retains accounting for every projected node.

The single browser packed branch now handles checked boolean arrays, finite
Float32Array/Float64Array and nonzero BigInt64Array. Metadata admits `nonzero:true`
only for packed i64; false or constraints on other words reject. Ordinary lists,
raw IEEE-float Serde and existing packed integer fields retain their semantics.

`domain_cases` qualifies independent payload literals, both frame directions,
selected unaligned input views, owned decoding, signed zero/subnormal bits,
nonfinite/partial-word/bool/zero failures and heap normalization against published
Core 0.22.4 and both TS runtimes. Native sweeps compare all four admission axes
for each adapter. Named schema and compile-fail type checks qualify projection.
The ninth isolated allocation group checks malformed pre-allocation refusal,
successful controls and nonzero's post-allocation validation boundary. Alternate
endian float helpers pass literal tests; actual big-endian execution is not claimed.
New domain workloads have not yet received matched instruction/size measurements.

## Implementation and provenance

The destination remains one Serde/Postcard codec and one admitted host graph.
Grammar/representation sources are reference `typescript/native_schema/mod.rs`,
`typescript/metadata/mod.rs`, `typescript/runtime.ts` and derive projection.
The new graph nodes use the existing importer, bounded writers and interpreter.
Fixed array length checks, tuple child counts and Result tags retain reference
semantics, including redundant u32 Result tags. Only lists emit count prefixes.
Standard `serde_bytes` projection is sealed to qualified byte inputs; arbitrary
custom serializer paths reject. It never changes the consumer's Serde declaration.

`finite/mod.rs`, `packed/mod.rs` and `platform_types/mod.rs` supply the selected
public contracts and minimal adapter patterns under the retained license. Packed
word serialization and owned visitor allocation now share the bounded-i16 path.
No raw-float or bool arbitrary-bit packed cast is exposed. Published serde_bytes
0.11.19 owns byte types; optional host-only Platform Types 0.20.4 owns identities.
Account equality normalizes absent/zero, but qualification explicitly inspects
the stored optional field and exact encoded bytes rather than relying on equality.

## Qualification and limits

`qualification/compatibility/tests/standard_cases/` contains independent literal
vectors, Rust round trips and exchanges with both candidate and published
reference TS runtimes. Cases cover exact prefixes/order/tags, byte/node/depth
boundaries, strict object shapes, owned selected-view decoding, finite signed zero
and subnormals, invalid values, Principal boundaries and Account absent/zero.
Generic packed vectors also compare native reference encoding. Existing bounded
i16 allocation/admission checks remain required after sharing its visitor.
Host tests reject invalid new metadata and check exact graph-node charging.

Feature parity is not yet complete. Browser-engine and distributed-package proof,
specialized features and consumer migration decisions remain separate. Existing
0.23.7 size/instruction reports describe their recorded source hashes only:
new packed code has a fresh matched size run below. IC instruction counts have
not been rerun for 0.23.8. No failed performance threshold has been relaxed.

### 0.23.8 verification

Product tests, host and derive tests, all 13 candidate/reference TypeScript tests
with strict type checking, eight native compatibility tests and eight allocation
groups pass. All-feature native Clippy, compile-fail documentation and the Wasm
build pass; the Wasm normal/build graph excludes Platform Types, serde_json and
Wire derive. Generic packed tests additionally sweep each allowance independently
against the reference for both encoding and decoding. The compatibility manifest
now disables automatic test discovery: its helper modules belong to the three
explicit test binaries, not additional standalone test crates.

Fresh isolated builds through `make measure-consumer-wasm` execute all four
artifacts without host imports. [Recorded hashes and results](parity-wasm.json)
use the existing unchanged workload and thresholds:

| Profile | Candidate | Reference | Delta |
| --- | ---: | ---: | ---: |
| opt-z | 45,281 B | 46,138 B | -857 B |
| opt-3 | 46,154 B | 47,026 B | -872 B |

Both reference-relative gates pass. Compared with the 0.23.7 candidate, this
increment costs 2 B at opt-z and 35 B at opt-3. These are total uncompressed Wasm
measurements for the standalone workload, not a claim that every new packed type
has equivalent instruction cost. The earlier failed instruction gates remain open.

## Scalar validation and hard-cut decision

Scalar min/max and named browser checks now round-trip through the single typed
graph and execute in payload and framed clients. Validation tests cover integer
and float boundaries, callback ordering and failure categories, escaped names,
unsafe bounds and consumer-owned runtime injection. The strict TypeScript suite
contains 23 tests. Rust Serde domain validation remains consumer-owned.

The user authorized a hard cut: legacy API, schema, wire and budget compatibility
are not release gates. One unversioned schema/runtime interface replaces the
proposed v2 marker; no old-runtime fixture or compatibility branch is retained.
Reference tests remain comparative evidence for deliberately retained behavior.
Browser/distribution and matched performance qualification remain outstanding.
