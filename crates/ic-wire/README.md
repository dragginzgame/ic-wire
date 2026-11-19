# IC Wire

Bounded Serde/Postcard encoding for controlled Internet Computer contracts, with
optional schema collection, bulk-field lint and TypeScript client generation.
The codec is `no_std` with `alloc`, Wasm-safe and independent of IC SDKs.
Rust 1.100.0 or later is required.

```toml
[dependencies]
ic-wire = "0.26.0"
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
| Schema limits every unsigned value to 0–127 or every signed value to −64–63 | Keep the ordinary list for payload size | Each element uses one varint byte throughout that range. |
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

### Floating-point lists

Keep an ordinary list when NaN or infinity is valid. For finite-only fields,
the bulk adapters preserve the bits of finite values, including negative zero.
Both layouts use four bytes per f32 or eight per f64. Their prefixes count
different units, so packing first costs an extra byte at 32 f32s or 16 f64s.
At 128 elements the ordinary count also needs two bytes and the sizes match again.

### Flags and fixed-size hashes

Ordinary boolean lists and `serde_packed::bools` both use one byte per flag
plus the count prefix. Choose the adapter for bulk traversal and boolean-byte
validation. If payload size matters, compare a bounded flag count and a
length-prefixed bitmap. In the measured Wire layout the bitmap costs one extra
byte at one flag, ties at two, and first saves a byte at three. At 128 flags
it uses 19 bytes versus 130. Specify bit order and enforce the flag bound and
exactly `ceil(count / 8)` bytes when decoding.

For a list of 16- or 32-byte hashes, `Packed<[u8; W], N>`
counts payload bytes in its prefix, while an ordinary list counts hashes.
At a field bound of seven 16-byte hashes or three 32-byte hashes, both prefixes
always fit in one byte. Packing then reduces traversal at the same payload size.
The first extra prefix byte appears at eight or four hashes respectively.

### Sparse readings and repeated labels

For mostly empty readings, compare a list of optional values with a slot count
and ordered index/value pairs. Include the slot count so trailing empty slots
survive decoding. Measure both typical and dense inputs, where the index for
each reading can outweigh the savings from omitting empty slots.

For repeated labels, compare inline text with a per-message table and a list of
indices. Include the table in the measured payload and try messages with all
distinct labels. The choice depends on label length and repetition, which the
schema alone cannot tell you.

For consecutive repeats of the same enum state, compare a list of tags with
ordered `(state, run_length)` pairs. For 256 states with runs of eight, pairs
used 65 Wire bytes versus 258 for tags. When the state changed every entry,
pairs used 514 bytes. Measure both patterns and bound the sum of run lengths
before expanding the list.

## Optional schema features

| Feature | Purpose |
| --- | --- |
| `typescript` | Schema collection, validated graph, lint and generated clients |
| `derive` | Also provides `ic_wire::typescript::Wire` derive |
| `platform-types` | Also projects published Platform Types Principal/Account |

For a crate that owns shared DTOs, enable `derive` once on its Cargo dependency:

```toml
[dependencies]
ic-wire = { version = "0.26.0", features = ["derive"] }
```

Use the same declaration on native and Wasm:

```rust
use ic_wire::typescript::Wire;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Wire)]
struct Request { sequence: u64 }
```

With `derive` enabled, the same schema trait implementation is available on both
targets, including `ValidatedRegistry::from_type::<Request>`. A crate that explicitly offers a codec-only
build can instead forward `ic-wire/derive` from its own optional feature and
apply `cfg_attr` for that feature alone.

## TypeScript delivery

Register application roots with `SchemaRegistry`, finish into `ValidatedRegistry`,
then call `payload_module` or `method_module` with explicit `PayloadLimits`, an
acknowledgement map and a diagnostic handler. Display the bulk report or reject
it with `is_clean()`; acknowledgements require exact paths and review reasons.
Warnings include the declared maximum element count when available. Empty-only
collections skip element advice; lists that can contain zero-byte elements still
receive traversal guidance.

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
