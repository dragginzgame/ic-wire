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
not a second codec. It supports six fixed-width integer types (`i16`, `u16`,
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
Int16Array, Uint16Array, Int32Array, Uint32Array, BigInt64Array or BigUint64Array.
Hash fields use one flat Uint8Array whose length must be divisible by 32.
Existing bounded `{k:'packed-i16',max:N}` metadata retains its limit.

The runtime writes little-endian bytes regardless of host endianness and copies
incoming bytes before exposing a typed array. Decoded values do not alias the
input. Byte and traversal budgets remain enforced; they do not bound JavaScript
engine heap. Big-endian TypeScript swap paths pass simulated typed-array storage checks, and
Rust append helpers pass local unit tests. Real big-endian Rust serialization
and JavaScript-engine execution remain unqualified. See [development](development.md).

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
