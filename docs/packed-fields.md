# Packed bulk fields

`ic_wire::Packed<T>` is an owned field in the production Serde/Postcard codec,
not a second codec. It supports six fixed-width integer types (`i16`, `u16`,
`i32`, `u32`, `i64`, `u64`) and 32-byte hashes (`[u8; 32]`). Floats, arbitrary
structs and platform-sized integers are excluded by a sealed element trait.
Use ordinary Serde derives and `ic_wire::Wire` metadata as usual. For byte data,
keep using `Bytes<N>`, `serde_bytes::ByteBuf` or the qualified `serde_bytes` adapter.

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
installed instruction/Wasm measurement should precede another canonical-comparison
rewrite. Wire's exact canonical verification remains required.
