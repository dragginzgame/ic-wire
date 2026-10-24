# Serde runtime and Canic integration

IC Wire has one production codec: bounded Serde traversal with Postcard encoding.
Use `Serialize` / `Deserialize` on consumer DTOs and the `native` module for
framing, resource admission and canonical validation. `Wire` generates
host-only metadata; it never generates a serializer. Alternative codec and
derive designs are research, not selectable runtime modes.

## Hard cut from the finite API

The former serializer-generating `Wire` derive is replaced by metadata-only
`Wire`, used alongside Serde derives. The finite `Wire` trait, `Shape`, `Method`, typed retained-byte wrappers,
root finite encode/decode functions and `typescript::generate::<Method>()` are
removed. Use ordinary Serde derives, explicit method IDs and `Limits`, retain
returned immutable frame bytes for retries, and generate TypeScript through
`Wire` and `generate_registry`. No compatibility shim or fallback exists.

This changes layouts previously controlled by `wire(id)`: record fields now
follow declaration order and enum tags use zero-based declaration indices.
Postcard primitives and the eleven-byte `IW` + little-endian u64 method ID +
direction header remain unchanged (request 0, response 1). Existing native Serde
frames are unchanged. Coordinate affected Rust/TypeScript clients and regenerate
metadata; do not replay old finite frames against a changed layout.

`Bytes<N>`, `BytesRef`, `Text<N>`, `TextRef`, `List<T,N>` and finite float wrappers
remain ordinary Serde field types, not another admission strategy. Their local
bounds supplement runtime budgets. `FiniteF32`/`FiniteF64` reject NaN/infinity and
preserve every finite bit, including negative zero. Raw Serde floats retain their
Serde behavior; consumers own domain validation.

## Native Serde payloads

`ic_wire::native` handles consumer-owned Serde types, including String, Vec,
tuples, maps and recursive enums, using the same Postcard encoding and envelope.
It does not infer static maxima or TypeScript metadata during codec execution.
Consumers own declaration order, compatible method IDs, deterministic serializers
and domain validation (including finite floats where required). Unordered maps
are unsuitable unless their serializer establishes deterministic ordering.

### Choose fields for bulk data

Use these defaults when declaring a new contract:

| Data | Default field | TypeScript shape |
| --- | --- | --- |
| Binary data with a field limit | `Bytes<N>` | `Uint8Array` |
| Binary data with operation-level limits | `ic_wire::ByteBuf` or `Vec<u8>` with `#[serde(with = "serde_bytes")]` | `Uint8Array` |
| Large homogeneous integer arrays | `Packed<T>` with `i16`, `u16`, `i32`, `u32`, `i64` or `u64` | Corresponding integer typed array |
| Lists of 32-byte hashes | `Packed<[u8; 32]>` | Flat `Uint8Array` |
| Records, general collections or small-value varints | Ordinary `Vec<T>` or bounded `List<T,N>` | Array of element values |

The bulk fields use Serde's byte-buffer hook within the same Postcard codec.
Ordinary sequences visit each element; the metadata derive does not rewrite
them. A large list of small integers may still favor ordinary varints for payload
size. `PackedI16<N>` remains available when a signed-16-bit field requires an
element bound; generic `Packed<T>` has no per-field maximum.
Choose one representation per field for all values, including empty and small
inputs; do not switch layouts according to the current value's length.

This complete DTO combines the recommended fields with an ordinary collection:

```rust
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
struct BulkTransfer {
    payload: ic_wire::Bytes<4096>,
    samples: ic_wire::Packed<i16>,
    chunk_hashes: ic_wire::Packed<[u8; 32]>,
    labels: Vec<String>,
}
let transfer = BulkTransfer {
    payload: ic_wire::Bytes::new(vec![42; 1024])?,
    samples: ic_wire::Packed::new(vec![i16::MAX; 1024]),
    chunk_hashes: ic_wire::Packed::new(vec![[0xab; 32]; 32]),
    labels: vec!["sensor".into()],
};
let limits = ic_wire::Limits {
    bytes: 16 * 1024, allocation: 256 * 1024, nodes: 256, depth: 32,
};
let frame = ic_wire::native::encode_request(7, &transfer, limits)?;
let decoded: BulkTransfer = ic_wire::native::decode_request(7, &frame, limits)?;
assert_eq!(decoded, transfer);
# Ok::<(), ic_wire::Error>(())
```

Operation budgets bound codec admission. Consumers additionally validate their
own sample/hash counts and semantics before encoding and after decoding. Keep
existing field-local bounds when migrating a declaration. Switching an ordinary
integer/hash list to packed data changes bytes and generated client types;
coordinate both ends and resolve retained old-layout frames before switching.
The details and byte examples below apply to that explicit adoption.

### Representation and adoption details

For opaque binary fields, use byte-oriented serialization. Byte-oriented field wrappers include
`Bytes<N>` and `BytesRef<'_, N>`. A native DTO can retain its `Vec<u8>` field and
use `#[serde(with = "serde_bytes")]` with a consumer dependency on `serde_bytes`.
Ordinary `Vec<u8>` serialization visits and admits every element; the byte path
admits the buffer in bulk. Both produce the same Postcard byte-vector payload
(for example, `[0, 127, 255]` becomes `03 00 7f ff`), but their traversal and
allocation-budget accounting differ. Qualify the chosen representation against
consumer limits. The benchmark corpus checks exact frame equivalence at 4 KiB
and 1 MiB; the byte path requires no new runtime API or encoding mode.

For bulk integer vectors or hash lists, `Packed<T>` owns a `Vec<T>` and serializes
it as a single byte buffer. Supported elements are `i16`, `u16`, `i32`, `u32`,
`i64`, `u64` and `[u8; 32]`. Runtime `Limits` bound admission; there is no per-field
const bound. Use `new`, `as_slice` and `into_vec` to wrap, inspect and recover data.

```rust
let hashes = ic_wire::Packed::new(vec![[0xab_u8; 32]]);
assert_eq!(hashes.as_slice().len(), 1);
```

This is still the same Serde/Postcard codec. Selecting this field changes ordinary
list bytes to a byte count followed by fixed little-endian words. For example,
`[32767_i16]` changes from `01 fe ff 03` to `02 ff 7f`. Coordinate both endpoint
schemas and retain exact retry bytes. Small integers can produce larger payloads.
The TypeScript feature maps integers to the corresponding typed arrays (including
`BigInt64Array`/`BigUint64Array`) and hashes to flat `Uint8Array` bytes, with a length
that must be divisible by 32. Ordinary vectors retain their current encoding.

For bulk signed 16-bit values, `PackedI16<N>` owns a bounded `Vec<i16>` and
serializes it as bytes: a byte count followed by fixed-width little-endian words.
`new` admits an existing vector without copying; `as_slice` borrows its words and
`into_vec` returns ownership. `Wire` metadata projects it to `Int16Array`, with
`N` counting elements. Rust and TypeScript decode into owned storage and reject
odd byte lengths and excess elements before allocating.

```rust
let samples = ic_wire::PackedI16::<1024>::new(vec![-32768, 0, 32767])?;
assert_eq!(samples.as_slice(), &[-32768, 0, 32767]);
# Ok::<(), ic_wire::Error>(())
```

This is an explicit field layout, not an optimization of ordinary `Vec<i16>` or
`List<i16,N>`. For `[32767]`, their payload `01 fe ff 03` becomes `02 ff 7f`;
for `[0]`, `01 00` becomes `02 00 00`. Wide values shrink, small values grow.
Switch both Rust and TypeScript schemas together, regenerate clients, and resolve
retained old frames before switching a deployed field. Framing, method identity
and exact retry/canonical-byte requirements do not change; no fallback reader exists.
The native budget admits the packed byte span in bulk, rather than charging one
Serde node per integer. Qualify limits for this representation. Little-endian
hosts and Wasm use a safe byte view without a packing allocation; big-endian
serialization uses a bounded temporary buffer and is not performance-qualified.

For owned byte fields, use `ic_wire::ByteBuf` (or `ic_wire::Bytes<N>` for a
field-local bound). It re-exports `serde_bytes::ByteBuf` directly, with alloc-only
runtime support on native and Wasm targets. No extra dependency or Serde field
annotation is needed:

```rust
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
struct Upload {
    data: ic_wire::ByteBuf,
}
let upload = Upload { data: vec![0, 127, 255].into() };
let limits = ic_wire::Limits { bytes: 64, allocation: 1024, nodes: 16, depth: 8 };
let frame = ic_wire::native::encode_request(7, &upload, limits)?;
assert_eq!(ic_wire::native::decode_request::<Upload>(7, &frame, limits)?, upload);
# Ok::<(), ic_wire::Error>(())
```

Enable IC Wire's `typescript` feature in the host metadata generator.
`ByteBuf` owns its bytes; `BytesRef<'_, N>` supports bounded borrowing.
The direct `serde_bytes` adapter and unbounded `&serde_bytes::Bytes` remain
available with a consumer dependency on `serde_bytes`. These byte fields project to
`Uint8Array`, whereas plain `Vec<u8>` projects to `number[]`. Regenerate bindings
and qualify traversal budgets when changing that field. `Wire` does not rewrite
Serde derives. Existing `#[serde(with = "serde_bytes")]` fields also derive
byte metadata, without changing the field type or serializer. Supported inputs
are `Vec<u8>`, `[u8]`, `serde_bytes::ByteBuf`/`Bytes` and references to those types;
aliases of these Rust types work too. Fixed arrays and arbitrary custom adapters
still require explicit verified metadata. Use the standard dependency under its
`serde_bytes` name (the absolute `::serde_bytes` path is also recognized).
This uses the already-measured byte path, not a new runtime optimization: at
1 MiB, retained opt-3 measurements show 1,049,715 encode instructions versus
78,644,290 for a generic vector and 2,115,305 for Candid's byte representation.
Owned byte decoding still costs 2.37× Candid; this does not fix integer lists or
hash arrays. The repository comparison report retains inputs and build identity.

```rust
use ic_wire::{Limits, native};
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Request { text: String, values: Vec<u64> }
let limits = Limits { bytes: 4096, allocation: 65536, nodes: 4096, depth: 64 };
let request = Request { text: "hello".into(), values: vec![1, 2] };
let bytes = native::encode_request(7, &request, limits)?;
assert_eq!(native::encoded_frame_len(&request, limits)?, bytes.len());
assert_eq!(native::decode_request::<Request>(7, &bytes, limits)?, request);
# Ok::<(), ic_wire::Error>(())
```

The two small values above deliberately use ordinary varints. Large homogeneous
integer fields should start with `Packed<T>` and measure any payload-size tradeoff.

`encode`/`decode` and `encoded_len` operate on payloads without headers.
`encode_request`/`encode_response` accept borrowed Serialize values without an
owned Deserialize requirement. Native decode accepts `Deserialize<'de>` values:
strings and explicit byte views can borrow directly from the input. Ordinary
owned callers remain supported. Borrowing
does not skip resource admission or canonical verification, and does not make
arbitrary integer sequences borrowable. The decoded value cannot outlive its input.
`encoded_frame_len` includes the 11-byte header for either direction. Retain the
returned bytes unchanged for any consumer-authorized retry; IDs alone do not
establish idempotency. Sizing does not retain a serialization for later delivery.

`encode_into`, `encode_request_into` and `encode_response_into` write into a
caller-owned `Vec<u8>` and return `Result<(), Error>`. They clear previous contents
on entry and leave the buffer empty on error. Successful calls retain capacity;
serialization failures may discard capacity. They preserve the two-pass native
admission/sizing contract and the existing frame bytes. Existing capacity never
relaxes byte limits. Oversized retained capacity is caller-owned storage, not a
measurement of this operation's allocation allowance; shrink or drop it according
to the caller's memory policy. Keep separate immutable bytes for retries before
reusing a buffer.

```rust
use ic_wire::{Limits, native};
let limits = Limits { bytes: 4096, allocation: 65536, nodes: 4096, depth: 32 };
let mut output = Vec::new();
native::encode_request_into(7, &"hello", limits, &mut output)?;
let text: &str = native::decode_request(7, &output, limits)?;
assert_eq!(text, "hello");
// Once the borrowed view is no longer used, the buffer can be reused.
native::encode_response_into(7, &"world", limits, &mut output)?;
# Ok::<(), ic_wire::Error>(())
```

```compile_fail
use ic_wire::{Limits, native};
let limits = Limits { bytes: 64, allocation: 1024, nodes: 64, depth: 16 };
let text: &str = {
    let frame = native::encode_request(7, &"hello", limits).unwrap();
    native::decode_request(7, &frame, limits).unwrap()
}; // The input storage cannot be dropped while its decoded view is used.
println!("{text}");
```

Limits are explicit per operation. Native `bytes` bounds the payload or full
frame, respectively. `allocation` reserves that entire byte allowance for output
scratch before admitting value storage. The remaining allowance charges eight
times visited value sizes and text/byte lengths; node and depth accounting follows
Serde traversal, including seeds and enum wrappers. It is deliberately conservative,
not an allocator/peak-memory measurement. It follows actual values. Different borrowed/owned representations may require
different traversal allowances. Native decoding suppresses collection size hints
and admits elements before their visitors can allocate. Canonical checking streams
serialization against the input without allocating a second output buffer.

Custom Serde code is trusted: work and allocations outside these visited boundaries
must be bounded by its owner. Standard/custom Deserialize allocations may abort on
allocator exhaustion; the adapter does not make arbitrary Serde allocation fallible.
The codec's own output reserve is fallible. Resource refusals are `Error::Limit`;
invalid payloads, trailing bytes, identity and noncanonical encodings retain typed
errors. Caller input custody is excluded. Native depth allowances must fit the
consumer's stack budget. No schema-independent performance claim is made.

The traversal adapters are adapted from IcyDB's native admission implementation;
its MIT notice is retained in `src/native/LICENSE-MIT`. IcyDB's numeric policies,
SQL semantics, endpoint roster and storage encodings are not imported.

## Concrete native TypeScript metadata

`typescript::RUNTIME` accepts trusted concrete metadata through
`createCodec(method, types, checks)`, including named recursive `ref` nodes,
`tuple` items and transparent `newtype` wrappers. Consumers supply byte, node and
depth allowances for both directions. Dynamic lengths are checked before copying
or descending. Missing/cyclic aliases and sparse arrays reject. Concrete nodes
and named references can name trusted scalar checks that run before encoding and
after decoding; every check along an alias chain applies. Float metadata can
supply finite numeric `min`/`max` bounds, inclusive in both directions; the
generator rejects malformed or inverted bounds. Unbounded finite floats retain
their existing representation, including negative zero. Integer metadata bounds
must be integral numbers within ±9,007,199,254,740,991; use named consumer checks
for wider bigint constraints. This restriction applies to metadata bounds, not
the supported 64/128-bit integer payload range.

Ordinary native Serde metadata can be derived as described below. Domain validation
and endpoint identities remain consumer-owned.
Host generators can pass that normalized registry and method list to
`typescript::generate_registry(&types, &methods, "./native-runtime.js")`. It
emits named TypeScript types and typed query/update codec objects. The selected module must export
`createCodec`; a consumer adapter can supply its scalar validators to the shared
runtime. Custom serialization projections and domain-specific constraints stay
with the consumer.
Generation checks names, references, supported type shapes and bounded metadata
traversal; it preserves supplied wire metadata and never infers native limits.
Registry names such as `__proto__` retain their ordinary own-property meaning.
Generation does not infer codec allowances or JavaScript heap bounds. The shared runtime compiles with strict
TypeScript, `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.

### Deriving native metadata without IDs

Keep the existing Serde derives and add `ic_wire::Wire` in the host
generator build. For shared contract packages, gate it with a local feature:
`wire-schema = ["ic-wire/typescript"]`. Enable that feature in the generator.
On Wasm targets, forwarding it keeps the runtime codec but omits Wire's metadata
module, metadata trait implementations and optional JSON dependency.
The alloc-only serde_bytes dependency remains available for runtime ByteBuf use.
`ic_wire::Wire` emits metadata implementations only for native targets. Gate calls
to `ic_wire::typescript` and consumer-owned metadata implementations with
`#[cfg(not(target_family = "wasm"))]`. A direct consumer dependency such as
`serde_json` still needs its own native target gate; Wire cannot gate it for you.
Serde implementations, frames and runtime limits are independent of this feature.

```rust
# #[cfg(feature = "typescript")]
# fn example() -> Result<(), ic_wire::typescript::GenerationError> {
#[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
struct Page<T> {
    entries: Vec<T>,
    total: u64,
}
let schema = ic_wire::typescript::native_schema::<Page<String>>()?;
// Put `schema` in a named registry entry or a method's request/response field,
// then call the existing generate_registry with explicit method budgets.
# let _ = schema;
# Ok(())
# }
```

In a shared DTO package, replace the unconditional metadata derive above with
`#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]`.
The derive generates only metadata; it does not implement or replace Serde,
require `wire(id)` annotations, or claim finite static resource bounds.
Generic declarations are projected after instantiation, such as `Page<String>`.
Metadata bounds follow field types, including associated types such as `T::Item`;
the provider `T` need not implement `Wire` unless it is itself a serialized field.
Recursive fields retain conservative parameter bounds and inline recursive
metadata remains unsupported. Serde still owns the type's Serialize requirements.
Field-level `#[serde(borrow)]` and `#[serde(borrow = "'a")]` work with `Wire`,
including bounded `BytesRef`/`TextRef` DTOs. Serde validates the lifetime names;
borrowing changes ownership, not the generated client shape or payload bytes.
These borrowing annotations also work on newtype enum variants; their tags and
payload layouts are unchanged.
Rust field declaration order and zero-based enum variant order describe the
existing Serde bytes. For example, `{ a: u8, b: u16 }` with values 1/128 emits
payload `01 80 01`; reversing the declarations emits `80 01 01`. Reordering or
changing fields/variants is a protocol change requiring coordinated consumers.
There is no Candid-style schema evolution or compatibility fallback.

Supported shapes are named/unit/tuple structs, externally tagged enums (unit,
newtype, tuple and named variants), single-field `#[serde(transparent)]` structs,
generic types/where clauses, fixed-width integers, bool, unit, String/str,
Vec, slices, Option, Result, Box (including boxed slices), references, serializable fixed arrays and tuples of
one through four elements, bounded byte/text/list wrappers and finite float
wrappers. Raw floats, pointer-sized integers, maps and other
types require an explicit supported projection; they are not guessed. Apart
from container-level `transparent`/`deny_unknown_fields`, field/newtype-variant
`borrow` and the qualified field-level `serde_bytes` adapter, Serde attributes
are rejected, including rename, skip, flatten, tagging and custom serializers. Rust raw
identifiers retain their unescaped names in the TypeScript object.
Slices use the same length-prefixed sequence metadata as Vec, including `&[u8]`.
Use `serde_bytes::Bytes` for byte-oriented `Uint8Array` metadata; slice metadata
does not make arbitrary typed slices borrowable during Serde deserialization.

The first version inlines concrete schemas and rejects recursive types or schemas
exceeding 64 levels/65,536 visited nodes with `GenerationError::Complexity`.
The existing explicit registry API still supports named recursive references.
Metadata is trusted host configuration, not reflection over arbitrary serializers.
A derive cannot inspect a handwritten Serialize/Deserialize implementation in
another block: use it only alongside ordinary Serde derives on that declaration.

For custom serialization, implement `typescript::Wire` explicitly and
qualify it against both Serde directions. A domain ID stored as `[u8; 32]` but
serialized as lowercase hex must describe text, for example
`{"k":"text","max":64,"check":"networkHex"}`, not an array. The generator's
runtime adapter supplies `networkHex`; it must **throw** on anything except
64 lowercase hexadecimal characters (returning false is not a rejection).
Missing named checks fail closed. Do not omit domain validation or add an
unchecked fallback. The maintained native schema conformance fixture demonstrates
this full Rust/TypeScript contract, including invalid and missing checks.

Native byte/allocation/node/depth budgets and method/direction binding are
unchanged. Metadata does not infer allowances. JavaScript enforces its existing
byte/node/depth limits; these are not a measurement of JavaScript heap allocation
or a promise of identical traversal charges to Rust's Serde visitor.

## Package and license

Runtime and derive crates use the MIT License. Candid and simulator dependencies
are confined to unpublished benchmarks; production remains platform-independent.
