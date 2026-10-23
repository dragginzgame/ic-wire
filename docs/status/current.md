# Current handoff

## Current batch: full expanded Candid comparison

Source `066b618` includes the pending 0.6.7 i64 optimization; workspace package
versions still read 0.6.6. This batch adds sixty benchmark cases, taking the
standard corpus to 214, plus ten Toko-shaped and 1/10/100-record cohorts.
The [full comparison](../full-comparison-067.md) owns current results and source
identity: native time, allocation counts, peak heap, IC instructions, wire bytes,
and total/code-section uncompressed Wasm. Gzip measurement is removed.
All five cohorts complete in both profiles: 1,340 paired encode/decode rows,
7,598 IC rows and 24,672 native/instrumented rows. Cross-mode input/frame identities
match; every IC row has three identical samples. Focused benchmark roundtrips,
lint, Rust 1.91 checking and runner/format checks pass.

Production runtime, generator logic, protocol and acceptance rules are unchanged
by this benchmark batch. One production Serde/Postcard codec remains.
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

At opt-3 in the expanded standard corpus, Wire wins 161/214 encodes and 155/214
decodes by IC instructions. The whole Wasm fixture is 435,212 bytes versus
Candid's 1,053,914. These case counts are not a production traffic weighting.
Candid remains substantially faster for large primitive vectors: wide i16
encoding is about 50x Wire/Candid, generic byte encoding 37x, and wide i64 decode
9.55x. Owned 1 MiB blob decode is 2.37x; byte-oriented encoding is 0.50x.

At opt-3, all ten Toko-shaped decode cases favor Wire; the 50-card projection encode is
1.9% more expensive than Candid. Native timings vary between runs; the report
keeps both medians and separates them from deterministic IC counts. Fixture
sizes do not predict an installed application's footprint.

The [integer study](../integer-shortcut-study.md#067-share-the-i64-writer) retains
the preceding matched optimization experiment and its opt-z tradeoffs.
This expanded suite is a fresh comparison, not a before/after speedup claim.
No full CI, macOS, installed-consumer or application-latency claim is made.

Shared Tooling is 0.26.7 at `3e8d7cd`, with Build/Testkit/Host 0.65.0 and
Platform 0.6.9 lock selections. Candid/PocketIC remain unpublished benchmark
dev dependencies; the runtime stays SDK/host-independent. Siblings are read-only.
