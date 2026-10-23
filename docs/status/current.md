# Current handoff

## Current batch: integer optimization and tooling adoption

0.6.5 is released at `e10196e`; pending changes are under 0.6.6.
The private Serde adapter now also forwards u16/u32 values below 128 through
Postcard's equivalent u8 writer. Original admission and exact bytes are preserved.
Large small-value u16/u32 vectors save 61–63% opt-3 encode instructions; the
expanded fixture grows 601 total Wasm bytes. Worst regression is 3.89% on opt-z
wide-u16 encoding. See the [study](../integer-shortcut-study.md) for matched
instructions, sizes, controls, rejected candidates and reproducible source.

Signed shortcuts, streamed wide-u64 encoding and 128-byte canonical comparison
remain research: their wide-value/small-blob regressions do not justify adoption.
One production codec remains. No wire or acceptance-rule change is introduced.
Runtime/boundary/borrowing, TypeScript/lint, native/Wasm MSRV and benchmark
roundtrips pass. No new native timing, heap or installed-consumer qualification.
Versions, staging, commits, publication and releases remain maintainer-owned.

## Production contract

Keep one production Serde/Postcard codec, exposed through ic_wire::native.
Consumers use Serialize/Deserialize, explicit method IDs and runtime Limits.
Bytes/Text/List, borrowed views and finite floats are ordinary Serde field types.
Wire generates host-only metadata; it does not generate serializers.
The finite Wire trait, Shape/static admission, Method, typed retained frames and
finite TypeScript generator are removed. No legacy reader, compatibility shim,
automatic migration or alternate production codec is maintained.

The 0.6.0 cut changed former finite records to Serde declaration order and enums
to zero-based indices; existing native frames and envelope bytes were unchanged.
Affected consumers must regenerate metadata and coordinate layouts. Preserve
exact same-release retry bytes. The packaged [guide](../../crates/ic-wire/README.md)
owns migration, conservative traversal allowances and custom-serializer rules.

## Metadata and Canic integration

Released metadata supports concrete generics, associated field types, transparent
wrappers, borrowed records/newtype variants, slices and boxed slices. Recursive
inline schemas return Complexity; recursive fields retain conservative bounds.
Unsupported custom Serde layouts require explicit verified metadata. Standard
serde_bytes field adapters and deny_unknown_fields are supported; qualified byte
inputs project to Uint8Array without changing their existing Postcard bytes.

Canic #17 supplied these requirements. Canic owns endpoint/caller conversion,
IDs, budgets, compiled role selection, generated clients and installed acceptance.
The maintainer keeps ExternalPrincipal on the Candid side; no principal metadata
or platform dependency was added. Do not infer that all consumer integration
acceptance is complete from Wire's standalone conformance checks.

## Evidence and remaining costs

The [production comparison](../candid-comparison.md#production-serde-hard-cut--pending-0523)
measures 114 standard cases and 100 nominal record types against Candid, both
profiles, with 4,958 stable IC rows plus input/frame hashes and uncompressed Wasm.
Opt-3 100-type Wire total/code is 426,480/391,954 bytes versus Candid's
2,079,944/2,010,763; all those record inputs encode/decode faster than Candid.
Sixteen repeated structural widths do not predict installed application size.
Historical evidence keeps its original 0.5.23 candidate labels and paths.

A universal within-3x goal is not met: large small-u64-list opt-3 encoding
is still 10.46x Candid after the [0.6.5 optimization](../small-u64-study.md),
and generic Vec<u8> encoding is 37x. Byte-oriented 1 MiB encode is 0.50x Candid;
owned decode is 2.37x. Qualify integer/hash-array workloads and use byte-oriented
opaque fields. Field representation changes can change budgets and client types.

Prior canonical-comparison experiments remain rejected: skipping comparison is
invalid for custom Serde; unrolling saved 4.54% on opt-3 1 MiB decode but added
570 Wasm bytes and regressed a small opt-z case by 11.04%. Retired finite blob and
writer savings are historical only. No new performance claim accompanies cleanup.

Focused runtime, borrowing, derive, diagnostics, strict lint, native/Wasm MSRV and
default/TypeScript doctests passed for the preceding implementation batches.
TypeScript checks cover 146 vectors, 29,536 mutations, 1 MiB buffer boundaries
and seventeen generic/custom-scalar/byte/slice projection vectors. Full CI,
macOS and installed consumer suites were not run in these audits.

Shared Tooling is 0.26.7 at `3e8d7cd`, with Build/Testkit/Host 0.65.0 and
Platform 0.6.9 lock selections. Setup, snapshot, dependency-boundary/pin and
disposable Make/release adapter checks pass; PocketIC startup verifies the new
Testkit integration. Candid/PocketIC stay in unpublished benchmark dev dependencies;
the runtime remains SDK/host-independent. Sibling checkouts are read-only.
