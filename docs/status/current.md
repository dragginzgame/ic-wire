# Current handoff

## Current batch: measured small-u64 specialization

0.6.4 is released at `0300e81`; pending changes are under 0.6.5.
The private Serde adapter now emits u64 values below 128 through Postcard's
equivalent u8 writer. Original u64 resource charges and exact bytes are preserved.
Large small-u64 lists save 40.60% encode and 15.82% decode instructions at opt-3;
standard and 100-type probes each grow 146 total/code Wasm bytes. Opt-z wide-list
encoding regresses 1.70%; large integer lists still lose to Candid. See the
[measured study](../small-u64-study.md) for all 9,916 rows and source identity.
Runtime/boundary/borrowing tests, TypeScript/lint and native/Wasm MSRV pass.
No new native-time, heap or installed-consumer qualification is claimed.

Five short-write/inlining candidates were measured against the current Serde
codec with the same incoming dirty lockfile. None is retained: up to 40.26%
opt-3 small-u64-list savings trade against opt-z generic-byte regressions and
larger opt-3 Wasm. See the [experiment](../serde-short-write-study.md) for all
17,748 rows, source identity, controls and rejected implementations. Those trials
changed no production code or dependency selection. Native timing,
heap and 100-type scaling were not rerun for the rejected candidates.

The obsolete scripts/benchmarks/protocol runner and its native-view helper are
removed from the active tree. They depended on the retired finite Method/Wire
API. Historical studies link to the unchanged 0.5.22 source revision
`9a1c06e6829da053190d997cdeed92a861addedb`; retained evidence is untouched.
Production benchmarks remain in ic-wire-benchmarks and ic-wire-qualification.
That earlier tooling retirement changed no runtime, protocol or generated client.
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

A universal within-3x goal is not met: large u64-list encoding is 18–28x Candid,
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

Shared Tooling remains 0.26.6 at `4cb9bc9`, with Tooling 0.64.1 and Platform 0.6.8
lock selections. Candid/PocketIC stay in unpublished benchmark dev dependencies;
the runtime remains SDK/host-independent. Sibling checkouts are read-only.
