# Current handoff

## Current batch: 0.6.11 packed fields

0.6.10 is released at `31bcef2`; pending work is recorded under 0.6.11.
Earlier pending work shares PackedI16 decoding across bounds while retaining
checks before allocation. Bytes, metadata, canonical verification and retries
are unchanged. The [paired study](../packed-sharing-0611.md) records 100-DTO
savings of 4,354 total opt-z bytes / 6,886 opt-3 bytes, plus small decode overhead.
All 33,952 typed admission outcomes match. That change is preserved.

## Public bulk fields

The measured runtime-limited prototype is promoted to `ic_wire::Packed<T>` for
six integer widths and 32-byte hashes. It remains a field in the one production
Serde/Postcard codec. The benchmark now uses the public implementation; its
private duplicate is removed. The sealed element trait excludes arbitrary types.
The [migration guide](../packed-fields.md) explains bytes, limits and typed arrays.
Existing PackedI16<N> retains its tighter element bound and exact wire contract.

Rust/TypeScript conformance passes 157 vectors and 32,156 differential mutations,
including wide signed/unsigned extrema, flat hashes, unaligned input, owned output,
malformed lengths and canonical prefixes. Native tests include all 65,536 i16s.
Generic TypeScript descriptors use packed/word; bounded packed-i16 remains supported.
Both use a shared typed-array path with explicit little-endian conversion.

The [bulk study](../bulk-fields-0611.md) measured all 68 field/manifest and 1/10/100
record cases ahead of Candid in both profiles. At opt-3, a 1,024-hash manifest
falls from 2.52M/5.04M to 35,175/82,936 encode/decode instructions, growing one byte.
100 records shrink 19,234 total / 19,209 code Wasm bytes. Small integers can grow
up to eight times in payload; empty u32 decode regresses 39.13% versus ordinary Wire.
Per-field const-bound and extra entry-helper prototypes were rejected for Wasm growth.
The [promotion qualification](../packed-promotion-0611.md) confirms all 3,580 IC
rows and 16 total/code Wasm sizes match the prototype, with unchanged allocations,
peak heap and frames. Concrete word readers need inline hints across crate
boundaries; the first public candidate regressed large i16 decode 5.48x and was
rejected. Native opt-z large u16 decode still regresses 57% versus the prototype.
TypeScript bulk copies improve large arrays but one-word decode adds 135–155 ns.
Current-dependency Rustdoc, native/benchmark tests, Clippy, TypeScript and MSRV
checks pass; consumer adoption and big-endian execution remain unqualified.

## Consumer boundary and consolidation

The [Wire-side review](../packed-consolidation-0611.md) qualifies Blob's manifest
shape with 0/1/4/1,024 hashes, exact headers, owned roundtrip, frame limits and
host metadata. Application bounds/root validation and Candid storage stay owned
by Blob. The maintainer explicitly selected Wire-side work only: no sibling
mutation, build or installed-endpoint claim. Adoption remains consumer-owned.
Sharing packed visitors added 29 opt-z bytes at 100 bounded DTOs; sharing only
serialization added four. Generic packed artifacts were identical. Both are
rejected screens, with no new runtime code or protocol changes. Native timing
and IC counters were not rerun for these rejected candidates. All 16 native
tests, focused Clippy and documentation links pass. Comparison navigation is
now a current index plus preserved historical reports. The inspected manifest
has no whole-content bytes, so owned-blob optimization is not established as its
next bottleneck; actual consumer adoption/measurement comes first.

## Codec-cost investigation

The [cost breakdown](../cost-breakdown-0611.md) preserves six isolated variants,
22 inputs, 528 primary pairs and 24 artifacts. Pure Postcard AllocVec still costs
40.3x Candid for wide-i16 encode and 18.8x for wide-i64. Budget accounting alone
saves only 3.3% on i16 encode. Removing canonical verification improves owned blob
decode but violates the contract; removing preflight causes reallocations. No
checks were removed. Rejected variants and original source remain in the archive.

## Previous qualification and remaining costs

The [packed-i16 qualification](../packed-i16-production.md) owns the original
bounded-field measurements and migration contract. Its implementation is preserved.
The [full candidate matrix](../full-comparison-0611.md) covers 597 overlapping
case entries, thirteen cohorts, two profiles, 52 Wasms and 2,388 paired comparisons.
All cross-mode input/frame hashes match and each IC row has three equal samples.
Wire Wasm is smaller in every cohort/profile. Opt-3 standard encode/decode wins
are 161/214 and 155/214. Ordinary wide i16 encode remains 49.9x Candid, generic
byte-vector encode 37.2x, wide i64 decode 9.55x and owned 1 MiB blob decode 2.37x.
Packed is an explicit field choice, not a fix to ordinary list representations.

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

## Tooling and qualification limits

Shared Tooling is 0.26.7 at `3e8d7cd`. Incoming dependency updates select Platform
0.6.11 and Testkit 0.66.2; these are preserved. Matched promotion measurements use
the archived study's Platform 0.6.10/Testkit 0.66.0 pins in an isolated source tree.
Current-dependency focused checks are separate. Candid/PocketIC remain benchmark
dev dependencies; runtime is SDK/host-independent. Siblings remain read-only.
No full CI, big-endian execution, installed-consumer or application-latency claim.
Versions, staging, commits, pushes and releases remain maintainer-owned.
