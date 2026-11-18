# IC Wire

Bounded Serde/Postcard encoding for controlled Internet Computer contracts, with
optional host schema collection, bulk-field lint and TypeScript client generation.
The codec is `no_std` with `alloc`, Wasm-safe and independent of IC SDKs.
Rust 1.100.0 or later is required.

```toml
[dependencies]
ic-wire = "0.24.6"
serde = { version = "1", features = ["derive"] }
```

```rust
use ic_wire::{decode, encode, Encoding, Limits};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Reply { accepted: bool, revision: u64 }

let limits = Limits { bytes: 64, allocation: 4096, nodes: 32, depth: 8 };
let value = Reply { accepted: true, revision: 129 };
let bytes = encode(&value, Encoding::Payload, limits)?;
let decoded: Reply = decode(&bytes, Encoding::Payload, limits)?;
assert_eq!(decoded, value);
# Ok::<(), ic_wire::Error>(())
```

Use `Encoding::Frame { method, direction }` for request/response framing and
`method_id(name)` for name-derived identities. `encoded_len`, `encode_into` and
`encode_to_slice` share admission with owned encoding. Decode rejects trailing
bytes. Header identity is not authentication.

## Fields and limits

`Bytes`, `Text` and `List` provide explicit field bounds; `BytesRef` and `TextRef`
support borrowed decoding. `ByteBuf`/`ByteArray`, `Packed<T, N>`, finite
float wrappers and `serde_packed` adapters express intentional bulk layouts.
An ordinary array and `ByteArray` have different length-prefix representations.
Selecting an adapter is a consumer schema decision, never an automatic rewrite.

`Packed<T, N>` bounds the element count for any supported packed word type;
`Packed<T>` omits the field bound. Both use the fallible `new(Vec<T>)` constructor.
Use `Packed<i16, N>` in place of the removed `PackedI16<N>`; there is no old-name
alias. Browser metadata uses one `packed` kind with a word and optional maximum.

Limits govern bytes, conservative allocation charges, traversal nodes and depth.
Allocation allowance is not measured process heap usage. Encode preflights before
codec output allocation; custom Serde code remains trusted and must serialize
deterministically across passes. Reusable output is empty after errors; slice
output may be modified on errors other than an undersized destination. Retain
original input bytes when signatures, retries or hashes depend on them: decoding
and re-encoding can normalize collections or accepted noncanonical forms.

### Choosing an integer list layout

For signed and unsigned 16-, 32- and 64-bit lists, use the field's element
limit for `N` in either layout.

| What matters for this field? | Starting choice | How to confirm |
| --- | --- | --- |
| Typical values are small, or their distribution is unknown | `List<T, N>` for ordinary varints | Compare `encoded_len` for both layouts on representative inputs. |
| Most values require many varint bytes | `Packed<T, N>` for fixed-width words | Check typical and largest allowed messages against your byte budget. |
| Encoding or decoding a large list dominates IC instructions | Try `Packed<T, N>`, even for small integers | Measure encode and decode on the target canister and check the larger small-value payload fits. |

For short lists, include typical lengths in the decision. On the measured opt-z
canister, a single small `u32` used 2 bytes and 1,075 encode instructions with
`List`, versus 5 bytes and 1,121 instructions with `Packed`. At 16 such values,
`Packed` used fewer instructions for both encoding and decoding, while `List`
still produced the shorter payload.

For example, 4,096 copies of `1_u32` use 4,098 ordinary payload bytes or
16,387 packed bytes. With 4,096 copies of `u32::MAX`, the ordinary payload
grows to 20,482 bytes and the packed payload stays at 16,387. If both layouts
meet your budgets and you already ship one, keep the existing layout. Select
once per schema: switching layouts per message changes how the receiver must
decode the field.

### Flags and fixed-size hashes

Ordinary boolean lists and `serde_packed::bools` both use one byte per flag
plus the count prefix. Choose the adapter for bulk traversal and boolean-byte
validation. For a list of 16- or 32-byte hashes, `Packed<[u8; W], N>`
counts payload bytes in its prefix, while an ordinary list counts hashes.
Packing can therefore cost a prefix byte while substantially reducing node
charges. Choose the layout against both the payload and traversal budgets.

## Optional host features

| Feature | Purpose |
| --- | --- |
| `typescript` | Schema collection, validated graph, lint and generated clients |
| `derive` | Also provides `ic_wire::typescript::Wire` derive |
| `platform-types` | Also projects published Platform Types Principal/Account |

There are no default features. Host modules and their optional dependencies are
excluded on Wasm even when features are forwarded. Gate derives on native builds:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(all(feature = "schema", not(target_family = "wasm")),
    derive(ic_wire::typescript::Wire))]
struct Request { sequence: u64 }
```

Here `schema` is a consumer feature forwarding `ic-wire/derive`. The derive
supports explicit supported Serde layouts and rejects unsupported attributes;
it never generates or changes Rust serialization.

## TypeScript delivery

Register application roots with `SchemaRegistry`, finish into `ValidatedRegistry`,
then call `payload_module` or `method_module` with explicit `PayloadLimits`, an
acknowledgement map and a diagnostic handler. Display the bulk report or reject
it with `is_clean()`; acknowledgements require exact paths and review reasons.

Write `ic_wire::typescript::RUNTIME` to `runtime.ts` beside the generated modules
and select `./runtime.ts` as their runtime import. Compile them together with a
TypeScript toolchain supporting BigInt and typed arrays. For plain browser ES
modules, TypeScript 5.9's `rewriteRelativeImportExtensions` emits `.js` imports.
The runtime is bundled in this crate; there is no separate npm runtime to fetch.
Use the generated TypeScript shapes: wide integers are BigInt and bulk fields
use the corresponding typed arrays. Transport, authentication and retries belong
to the application.

Scalar bounds are enforced by the browser runtime. Named custom checks can be
supplied through a consumer runtime wrapper forwarding the factory's optional
third `Checks` argument. Checks are trusted synchronous functions which throw
to reject; their work is outside traversal budgets. Consumer Serde owns matching
Rust validation.

## Upgrade from 0.22

This is a hard cut. Payload/request/response functions under `native` become
root-level operations with `Encoding`; schema generation uses a validated graph;
derive is a separate feature and lives under `typescript`. Generation requires
an explicit diagnostic handler. Regenerate clients and runtime together. No
legacy compatibility lane, fallback reader or storage migration is provided.

See the [repository](https://github.com/dragginzgame/ic-wire) for qualification,
design and measured tradeoffs. Smaller Wasm in the sample workload does not imply
universally faster encoding. Licensed under the included Toko Studios Network
Licence; see `LICENSE.md` for the controlling terms.
