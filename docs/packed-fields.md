# Bulk field selection and adoption

Default new binary fields to `Bytes<N>`, `ic_wire::ByteBuf` or the qualified
`serde_bytes` adapter, and large homogeneous integer/hash arrays to `Packed<T>`.
Keep ordinary collections for records and for small integers when compact
varints matter more. This convention uses the existing Serde/Postcard codec;
field declarations select the representation explicitly. The shipped
[runtime guide](../crates/ic-wire/README.md#choose-fields-for-bulk-data) contains
a complete mixed DTO and roundtrip example, also exercised across Rust/TypeScript.
The choice is fixed for the field, including its empty/small values; it is never
selected dynamically from the current input length.

`ic_wire::Packed<T>` is an owned field in the production Serde/Postcard codec,
not a second codec. It supports seven fixed-width integer types (`i8`, `i16`, `u16`,
`i32`, `u32`, `i64`, `u64`) and 32-byte hashes (`[u8; 32]`). Floats, arbitrary
structs and platform-sized integers are excluded by a sealed element trait.
Use ordinary Serde derives and `ic_wire::Wire` metadata as usual. For byte data,
keep using `Bytes<N>`, `ic_wire::ByteBuf` or the qualified `serde_bytes` adapter.

```rust
#[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
struct Manifest {
    hashes: ic_wire::Packed<[u8; 32]>,
    samples: ic_wire::Packed<i16>,
}
// Wire metadata requires the host-only `typescript` feature.
```

Wrap a Vec with `Packed::new(values)` without copying. `as_slice()` borrows its
words; `into_vec()` recovers ownership. No per-field bound is encoded in the type.
Native `Limits` still check the encoded bytes and allocation allowance before the
visitor reserves element storage. The field is admitted as a byte buffer, so it
uses bulk accounting instead of visiting every list element. Direct use through
another Serde deserializer does not inherit native runtime limits.

## Bytes and migration

Ordinary lists retain their encoding. Packed fields encode a Postcard byte count
followed by fixed little-endian words, without per-element varints. Hash bytes
retain their order. Examples show payload bytes, excluding the unchanged envelope:

| Value | Ordinary Vec | Packed |
| --- | --- | --- |
| `[32767_i16]` | `01 fe ff 03` | `02 ff 7f` |
| `[1_u32]` | `01 01` | `04 01 00 00 00` |
| One 32-byte hash | `01` + 32 bytes | `20` + 32 bytes |
| Empty list | `00` | `00` |

Switch producers and consumers together, regenerate clients and keep original
retry bytes. There is no legacy lane or automatic format detection. Do not treat
same bytes for empty inputs as general compatibility. Decoding rejects partial
words, truncated buffers and noncanonical length prefixes. It supports unaligned
input and returns owned storage. Canonical hashing and envelope rules are unchanged.

`PackedI16<N>` remains the field with an explicit element limit. Its bytes match
`Packed<i16>`, but removing N would remove that constraint: do not substitute the
new type unless the consumer deliberately accepts runtime-only limits. It is not
a second codec strategy. This batch preserves its Rust implementation and bounds.

### Adopting an existing field

1. Select the field representation and preserve its constraints. Use `Bytes<N>`
   for a binary field maximum and `PackedI16<N>` for an existing signed-word
   element maximum. `Packed<T>` has no per-field maximum; operation budgets and
   consumer validation remain explicit. Measure empty/small/large values before
   trading compact varints for fixed-width words.
2. Update both endpoint declarations and regenerate clients. Numeric arrays become
   typed arrays; packed hashes become a flat `Uint8Array`; byte-oriented fields
   become `Uint8Array` instead of `number[]`. Review other Serde users, including
   JSON projections. Keep domain ordering, semantic count checks and application
   hashing rules with the consumer.
3. Qualify Rust/TypeScript frames, typed failures and budgets together, then
   coordinate the deployment and resolve retained old-layout frames before the
   switch. Keep exact bytes for retries within the selected layout. External
   Candid boundaries and persisted storage require their own decisions.

Ordinary `Vec<u8>` and the byte-buffer path both encode `[0,127,255]` as
`03 00 7f ff`, but bulk admission changes resource accounting and the generated
client type. This byte equivalence does not extend to integer/hash lists: the
table above shows their changed prefixes and words. There is no automatic field
rewrite or compatibility reader. The recommendation itself changes no existing
runtime bytes, schemas or acceptance rules.

## TypeScript

Generated metadata uses `{k:'packed',word:'i16'}` (or the other supported word
name; hashes use `hash32`). Integer fields use the corresponding typed array:
Int8Array, Int16Array, Uint16Array, Int32Array, Uint32Array, BigInt64Array or BigUint64Array.
Hash fields use one flat Uint8Array whose length must be divisible by 32.
Existing bounded `{k:'packed-i16',max:N}` metadata retains its limit.

The runtime writes little-endian bytes regardless of host endianness and copies
incoming bytes before exposing a typed array. Decoded values do not alias the
input. Byte and traversal budgets remain enforced; they do not bound JavaScript
engine heap. Big-endian TypeScript swap paths pass simulated typed-array storage checks, and
Rust append helpers pass local unit tests. Real big-endian Rust serialization
and JavaScript-engine execution remain unqualified. See [development](development.md).

For unsigned evidence beyond u128, use the documented [bounded unsigned magnitude](unsigned-magnitude.md)
convention over `Bytes<N>`, with lossless external Nat conversion and browser
round trips. Numeric canonicality remains a consumer boundary check.

## Selection and evidence

Choose packed fields for bulk wide integers and hash lists. Small-magnitude
integers can take substantially more bytes (eight times the data for small u64s),
and tiny fields do not always improve instruction cost. Keep ordinary vectors
where their compact varints matter more. The [research study](bulk-fields-0611.md)
records the matched ordinary-vector comparison, including regressions. The [promotion qualification](packed-promotion-0611.md) confirms matching IC
counts, Wasm sizes, frames and allocations, and records native/TypeScript timing
regressions as well as improvements.

The [full comparison](full-comparison-0613.md) retains paired ordinary/bulk
controls and 1/10/100-type probes. The [hotspot follow-up](hotspot-encoding-0613.md)
shows why ordinary-list micro-optimizations still leave large Candid gaps and
records the byte-buffer alternative with matching bytes. Keep those controls
visible when evaluating a consumer migration; this recommendation does not
relabel ordinary-vector losses as packed-field wins.

## Consumer boundary example: Blob Storage

Read-only inspection of `ic-blob-storage` at `dcbb8811` identifies
`UploadManifestDeclaration` in `crates/ic-blob-storage-contracts/src/dto/upload/manifest/mod.rs`
as a candidate: ordered `Vec<[u8; 32]>` chunks followed by a vector of exact
header names and values. The Wire-side `blob_manifest_boundary_preserves_order_and_header_bytes`
test qualifies this shape with 0/1/4/1,024 hashes. These are representation controls,
not a claim that every control is a semantically valid admitted Blob manifest.
The test checks owned roundtrip, exact unchanged hash/header suffixes and frame
budget refusal. It does not qualify an installed Blob endpoint.

A consumer adoption should change the Wire declaration and regenerated clients
together. Request and response types both contain this declaration. Preserve
`upload::manifests::bounds`: chunk count, header count and total header bytes remain
application constraints even though Wire now admits the hashes as one buffer.
Keep content-root reconstruction and exact header ordering/spelling validation.
Changing the wire representation must not change the application's content hash.

Blob also has a versioned Candid `UploadManifestRecord` with a Vec of hashes in
`crates/ic-blob-storage/src/model/service/upload/record/mod.rs`. Preserve that storage
representation unless the owner deliberately designs and qualifies a storage
migration. Convert through `as_slice()` or `into_vec()` at the boundary; do not
replace every hash vector across the repository. Review JSON/Serde consumers too:
Packed's byte-buffer representation is not an automatic JSON array-of-hashes
compatibility guarantee. Settle retained old-layout intents before the coordinated
cut; no fallback decoder or automatic replay conversion is provided by Wire.

The inspected upload declaration contains metadata and hashes, not complete
content bytes. Consequently, the synthetic owned-1-MiB-blob decoding hotspot is
not evidence of a bottleneck in this declaration. Manifest adoption and its
installed instruction/Wasm measurement remains consumer-owned. Since the 0.7 cut,
Wire validates primitives while reading but does not enforce byte-identical
reserialization. Consumers own normalization and retain exact retry frames.

## Budget selection

Set operation limits from the largest legitimate payload and its actual field
representations, then test valid maxima as well as malformed inputs. Node and
allocation budgets can stop ordinary-list traversal early, but also refuse large
valid lists. A byte-buffer field can carry many bytes using few traversal nodes;
allocation and frame limits still apply. Tight limits complement bulk fields.
They do not make a large ordinary vector cheap to accept.

The [budget study](evidence/tail-and-budgets-0712/README.md) measures these tradeoffs
with small valid controls and truncated large inputs. Its example limits are not
a universal endpoint recommendation or an IC instruction quota.

## Host-side bulk advice and CI

`generate_registry` now always runs bulk advice. Its host API takes an
acknowledgement map and a required diagnostic handler; the former silent
three-argument call is removed. This is a host API cut only: generated TypeScript,
wire bytes, identities and runtime admission do not change.

```rust,ignore
let source = generate_registry(&types, &methods, "./wire-runtime.js",
    &acknowledgements, |report| eprint!("{report}"))?;
```

The handler runs once on successful validation, even for a clean report. It must
display warnings in the consumer's normal build output. A `build.rs` integration
uses Cargo's warning channel:

```rust,ignore
|report| {
    for warning in &report.warnings {
        println!("cargo::warning={warning}");
    }
    for path in &report.stale_acknowledgements {
        println!("cargo::warning={path}: stale bulk acknowledgement");
    }
    // Optional CI enforcement:
    // assert!(report.is_clean(), "review bulk-field advice");
}
```

Advice explains per-element encoding and truncated-input rejection costs without
claiming a slowdown for every payload. Suppress a field by mapping its exact path
to a nonempty reason, for example
`/methods/upload/request/fields/data/list/u8` → `At most four bytes`.
Request/response and method use sites remain independent. A changed primitive
width or removed field leaves a stale acknowledgement that the handler reports.
With methods, advice follows their request/response shapes through references;
unreachable exported definitions are not separate roots. With no methods, every
exported type is reviewed under `/types/NAME`. Rooted traversal limits still apply;
invalid input or exhausted traversal returns an error without partial diagnostics.

Canic and other consumers must adopt the new call and connect the handler to their
build output. Wire does not modify sibling build pipelines. Plain Serde types
outside metadata generation remain outside this advice. The host TypeScript
feature remains useful for keeping JSON/generation out of pure Rust consumers;
there is no new opt-in feature for warnings and no canister logging code.

The standalone lint APIs remain useful for schema review without generation:

For field-specific review of `SchemaRegistry` output, use `lint_bulk_roots` with
an explicit map of public root labels to schemas/refs, the registry, and a map
of acknowledgement paths to nonempty reasons. For example, map `Reply` to the
ref returned by `registry.register::<Reply>("Reply")`. Diagnostics such as
`/Reply/fields/field_ids/list/u32` resolve references without exposing generated
type names. Acknowledging that path leaves `other_ids` visible even when both
fields share one interned `Vec<u32>`. Inline request/error roots work too.
Aliases add no path segments; cycles stop at references already active on the
current path, while siblings are inspected independently. Depth/work limits
refuse expansion with `Complexity`, not partial advice. The primitive-word suffix
requires fresh review after a word change; bounds/custom validators remain the
consumer's responsibility. When migrating from definition-level review, rerun
the rooted lint and explicitly review each new path instead of copying a
synthetic definition acknowledgement to every use site.

With the host-only `typescript` feature, `typescript::lint_bulk_fields` scans
resolved schemas for ordinary primitive lists and recommends byte buffers or
`Packed<T>`. Rust type aliases are resolved by `native_schema`; nested schemas
and named references are supported. Named references are reported at their
shared definition once, not at every referring endpoint. Include inline method
request/response schemas as named roots too. Types without Wire metadata are
outside this lint's coverage.

```rust
use std::collections::BTreeMap;
use ic_wire::typescript::{lint_bulk_fields, native_schema};

let schemas = BTreeMap::from([("Samples".into(), native_schema::<Vec<i64>>().unwrap())]);
let acknowledgements = BTreeMap::new();
let report = lint_bulk_fields(&schemas, &acknowledgements).unwrap();
for warning in &report.warnings {
    eprintln!("{}: consider {}", warning.path, warning.representation);
}
// Optional CI policy: assert!(report.is_clean());
```

Diagnostic paths escape `~` and `/` as `~0` and `~1`. For example,
`/Request/fields/samples/item` identifies a list inside an optional `samples`
field. Acknowledge an intentional compact vector with an exact path and a
nonempty reason, such as `{"/Samples":"At most four small values; retain varints"}`.
Stale acknowledgements make `is_clean()` false. There is no wildcard suppression.

For a JSON object of named schema roots, the repository command is:

```sh
cargo run -p ic-wire --features typescript --example bulk-lint -- schemas.json
cargo run -p ic-wire --features typescript --example bulk-lint -- schemas.json --acknowledgements acknowledged.json --deny-warnings
```

The first command prints advice and succeeds; the second fails on unacknowledged
candidates or stale acknowledgements. Invalid schemas or empty reasons fail in
both modes. This separate review command is optional; generation already runs
advice automatically. Neither changes canister code, emitted bytes or schemas.
It does not automatically rewrite fields or predict a slowdown:
short lists and small values may favor ordinary encoding. Preserve bounds,
newtype validation and other consumer constraints when adopting a suggestion.

`Packed<i8>` now supplies bulk signed-byte handling and projects to `Int8Array`.
For `[-128, 127]`, both `Vec<i8>` and `Packed<i8>` emit `02 80 7f`. Adoption still
changes traversal budgets, JSON projection and generated client types; coordinate
consumers and review limits. The exhaustive 256-value test confirms byte parity.
