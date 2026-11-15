# Comparison with IC Wire 0.22.4

Candidate: pushed `ic-wire-new` 0.23.7, commit
`2efbfb4` (2026-11-15). Reference: published `ic-wire` 0.22.4,
source revision `2dc434b10bec2fca92345d940dbfedc05993e9d5`.
The local read-only `../ic-wire` HEAD is exactly that revision and its working
tree is clean. Published reference library-source equivalence is recorded in
[qualification](../../qualification/README.md). This comparison reuses the
recorded matched experiments; it does not present fresh runs or compare a moving
legacy branch. Neither the push nor this report publishes to crates.io.

## Architecture and contract

| Area | Reference | Candidate | Assessment |
| --- | --- | --- | --- |
| Portable codec | Serde/Postcard with selected primitive fast paths | Serde/Postcard delegation | One upstream codec retained; fewer custom primitive paths, but encoding cost remains higher in the measured request |
| Payload/framing API | Payload and request/response entrypoints; shared prefixed internals | One Encoding selection across owned/reused/slice operations | Smaller public API; not a new discovery that framing internals can be shared |
| Admission | Preflight, weighted Serde traversal, bytes/allocation/nodes/depth | Same selected accounting/error/custody contracts | Differential/allocator evidence passes; custom Serde remains trusted |
| Host schema | JSON metadata interpreted by collection, validation, rendering and advice | Bounded projection into one sealed typed graph; renderers consume that graph | Concrete ownership consolidation, with conversion/finalization cost |
| Named Rust types | Shared/recursive/generic/borrowed projections | Preserves selected names and these capabilities; sticky collection failure | Existing reference capability preserved, not a new feature advantage |
| TypeScript | One metadata interpreter and generated types/wrappers | One subset interpreter; private metadata, frozen codec exports, format mismatch refusal | Functional end-to-end subset; narrower coverage |
| SDK/transport | Downstream-owned | Downstream-owned; optional counter SDK only in private qualification | No product endpoint/executor was restored |

The candidate is a breaking Rust API even where bytes agree. A package-version
change does not migrate persisted frames, client retention, consumer type identity
or runtime policy. There is no authorized production cutover.

## Coverage is not yet equivalent

The generic Rust codec accepts consumer Serde types, including types without Wire
metadata. The following gaps concern built-in adapters or host/TS projection;
they must not be misreported as an inability of Postcard to serialize the type.

| Reference surface | Candidate 0.23.7 |
| --- | --- |
| Bounded Bytes/Text/List, borrowed byte/text, PackedI16 | Implemented and qualified for selected cases |
| Common records/enums, integers, options, shared/recursive references | Implemented; strict derive rejects unsupported layouts |
| Result, fixed arrays, tuples, slices/Cow schema projections | Missing built-in host projection and/or schema/runtime kinds |
| Generic Packed and serde_packed adapters, finite float wrappers | Not implemented; PackedI16 is the selected packed surface |
| ByteBuf/ByteArray re-exports and qualified serde_bytes field projection | Not implemented as candidate-owned convenience/projection APIs |
| Platform Types Principal/Account schema integration | Missing; identity/Serde ownership remains with Platform Types |
| Bulk advice/lint and custom runtime domain validation | Missing |
| Scoped scratch/fallback output helper | Missing; owned, reused Vec and slice destinations exist |
| Published package/installed consumer evidence | Candidate remains unpublished; standalone local consumer is qualified |

Source scope: reference `src/lib.rs`, `native/mod.rs`, `typescript/native_schema/mod.rs`,
`typescript/metadata/mod.rs`, `typescript/bulk_lint/mod.rs`, `platform_types/mod.rs`
and derive sources; candidate public exports, native projections, graph importer,
renderers and runtime. Missing surfaces are unresolved requirements, not silently
retired APIs. A full legacy replacement needs their implementation/qualification
or an explicit consumer-backed retirement decision.

## Matched performance evidence

The standalone consumer owns one sample upload and receipt with distinct types,
fixed method ID, exact 34/14-byte frames, bounded bytes/text, packed i16 and ordinary
integer tags. It uses the same fixture for both suppliers. Numbers below do not
establish universal performance across the legacy workload corpus.

| Total uncompressed SDK-free Wasm | Candidate | Reference | Candidate difference |
| --- | ---: | ---: | ---: |
| opt-z | 45,279 B | 46,138 B | -859 B (-1.86%) |
| opt-3 | 46,119 B | 47,026 B | -907 B (-1.93%) |

Both size gates pass. Full LTO, one codegen unit, abort panics, symbol stripping,
compiler, input hashes and the initial opt-3 regression are documented in
[the size experiment](consumer-measurement.md). Inlining gives back some of the
initial candidate's opt-z saving to eliminate the opt-3 loss.

| IC instructions per operation¹ | Candidate opt-z | Reference opt-z | Candidate opt-3 | Reference opt-3 |
| --- | ---: | ---: | ---: | ---: |
| Request encode | 5224 | 4961 | 2674 | 2443 |
| Request decode | 5390 | 5634 | 3016 | 3294 |
| Response encode | 1725 | 1744 | 696 | 690 |
| Response decode | 590 | 712 | 389 | 425 |
| Bad request header | 189 | 197 | 106 | 132 |
| Truncated request | 4523 | 4779 | 2483 | 2740 |

¹Median warmed eight-operation batch divided by eight, including destruction and
loop/counter overhead. Three samples agree exactly; all installed replies match
the independent frame oracle. The same SDK wrapper is outside the measured codec
intervals. [Instruction experiment](consumer-instructions.md).

The instruction gate fails in three comparisons: request encoding +5.30% at
opt-z, +9.46% at opt-3, and response encoding +0.87% at opt-3. Faster decoding does
not accept these losses. No threshold was relaxed after measurement.

Host collection/finalization/JSON export totals are below reference at 128/1024
roots in the final recorded run, but remain higher at one root: scalar
1.172 vs 0.981 µs; recursive 8.082 vs 6.660 µs. The [host probe](schema-measurement.md)
contains scheduler noise and is not a statistically qualified speed guarantee.
JSON export remains slower than the reference map writer despite the span-append
improvement. Selected native allocation/custody checks pass; neither Rust logical
allowances nor JS traversal budgets are process-wide heap limits.

## Decision and next work

The candidate has demonstrated a smaller total Wasm artifact for one independent
consumer and a simpler schema authority. It has not demonstrated full feature
parity or a universally better replacement. The measured gates leave concrete
work, rather than a reason to expand generic test coverage indefinitely:

1. Investigate generic encode/admission costs while retaining upstream Postcard;
   any change must recheck size and IC instruction criteria together.
2. Resolve required legacy feature coverage, especially standard Result/array/tuple
   projections and Platform Types integration. Do not infer retirement from the
   standalone consumer's narrower needs.
3. Finish actual browser-engine and installed-package qualification for the
   retained scope, and resolve the independent small-registry performance finding.

Post-push review of 0.23.7 found no additional delivery-blocking codec change;
its known failed performance criteria are explicit. Qualification dependencies
remain private, and the native simulator lock is isolated from the codec lock.
Production migration and publication remain separate decisions.
