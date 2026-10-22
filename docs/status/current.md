# Current handoff

## Current batch: one production Serde codec

0.5.22 is released at `9a1c06e`; pending 0.6.0 is a maintainer-approved hard cut.
The minor release reflects breaking API/layout changes. Evidence retains its
original 0.5.23 candidate labels and paths; package versions remain release-owned.
Keep one production Serde/Postcard codec, exposed through `ic_wire::native`.
The former serializer-generating derive, finite Wire trait, Shape/static admission, Method and typed retained
frames, root finite functions and finite TypeScript generator are removed.
No parallel codec, compatibility shim, migration reader or caller-selected mode.
Alternate serializers/derive designs remain isolated research only.

Consumers use Serialize/Deserialize, explicit method IDs and runtime Limits.
Bytes/Text/List, borrowed views and finite floats remain ordinary Serde fields.
Wire supplies host-only metadata for concrete generics without field IDs; the
unreleased NativeSchema name is removed. Use Serialize, Deserialize and Wire.
Existing native frames and envelope bytes remain unchanged. Former finite records
now follow declaration order; enums use zero-based indices instead of wire IDs.
Coordinate affected consumers, regenerate metadata and retain exact same-release
retry bytes. The packaged [guide](../../crates/ic-wire/README.md) owns migration,
conservative traversal allowances and custom-serializer constraints.

## Canic unblock

Canic #613 records Wire #17's native generic metadata as its supplier blocker.
The pending Wire implementation covers Page<T>, ordinary records/enums,
transparent wrappers and verified explicit custom scalar metadata. It generates
no serializer and needs no deployment-side TypeScript/JSON feature. Unsupported
Serde attributes and recursive inline schemas reject; custom implementations
must describe actual serialized bytes and validate their domains.

Publication and Canic adoption are not complete. The maintainer owns package
versions, staging, commits, publication and releases. Canic still owns endpoint/
caller conversion, metadata/export identity, generated clients and installed
acceptance. No sibling files were edited and no release effect was executed.

## Current evidence

The [production comparison](../candid-comparison.md#production-serde-hard-cut--pending-0523)
measures 114 standard cases and 100 nominal record types against Candid, both
profiles, with 4,958 stable IC rows plus input/frame hashes and uncompressed Wasm.
Opt-3 100-type Wire total/code is 426,480/391,954 bytes versus Candid's
2,079,944/2,010,763; all those record inputs encode/decode faster than Candid.
Whole artifacts with sixteen repeated structural widths do not predict app size.

A universal within-3x goal is not met: large u64-list encoding is 18–28x Candid,
and generic Vec<u8> encoding is 37x. Byte-oriented 1 MiB encode is 0.50x Candid;
owned decode is 2.37x. Consumers should use byte-oriented opaque fields and
qualify large integer/hash-array workloads. No native-time/heap rerun for this cut.
Host metadata now supports serde_bytes::ByteBuf and borrowed Bytes directly,
projecting to Uint8Array through ordinary Wire derives. This exposes the existing
bulk byte path without custom serializers or automatic derive rewriting.
Vec<u8> frame bytes remain identical, but client types and traversal accounting
change; regenerate bindings and qualify limits. Runtime code is unchanged, so
the existing instruction evidence applies; no new runtime savings are claimed.
The retired finite blob and short-write savings are historical evidence only.

Runtime, borrowing, canonical bulk/custom-boundary tests, corpus roundtrips,
strict lint, Rustdoc, native/Wasm MSRV and nine compiler diagnostics pass.
TypeScript checks cover 146 vectors, 29,536 mutations, 1 MiB buffer boundaries,
and thirteen generic/custom-scalar/byte projection vectors. Allocation/packed-field checks
and all consumer benchmark cohorts pass. Evidence is retained under
`docs/evidence/serde-production-0523/`; logs: `target/serde-hardcut-0523/validation/`.
No full CI, macOS, live canister or consumer installed suite was run.
The byte-buffer metadata example is feature-gated; default-feature and
typescript-feature doctests both pass after correcting the CI documentation failure.

## Preserved tooling and research

Earlier IC Tooling 0.64.1 and Platform 0.6.8 work is preserved. Shared Tooling is
0.26.6 at `4cb9bc9`, unmodified. Candid/PocketIC remain unpublished benchmark dev
dependencies through IC Platform/Testkit; the runtime is SDK/host-independent.
Consumers own transport, endpoints, auth, retries, storage and deployment policy.

Finite writer inlining was rejected (40.38% instruction regression for a tiny
Wasm saving). The reserve cleanup was measured but its finite writer is now
retired. Those sources/results, earlier finite guarded verification and the
Serde-with-finite-admission prototype remain explicitly research/historical
material in the comparison report. Do not ship or cite them as this codec.
