# Bounded packed i16 workload field

Destination: `PackedI16<N>`, the explicit packed field in the first native
representative record. N counts elements; the Postcard prefix counts bytes.
Ordinary `Vec<i16>` and `List<i16, N>` retain element counts and ZigZag varints.
Selecting a packed field is a consumer representation decision, never an
automatic optimization. Small-magnitude controls can be larger when packed.

Reference: `2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/bounded/packed_i16/mod.rs` and `packed/mod.rs`.
Their committed SHA-256 values are respectively
`5b61334310fe72e15b3cf562909c62efa45a35bde5582331bf8a90abfab61835` and
`8f56da054856e3ba877f046bb894d3fce2c703a0aebb2ec8745ef8dbbd13a091`.
Retain the constructor/accessor/visitor contract and the minimal safe bulk
serialization/decoding algorithm, under the root license. Do not import the
generic Packed API, word trait hierarchy, metadata or per-type specialization.
One implemented word representation does not require a generic abstraction;
additional word types must share the bulk mechanism when a real caller arrives.

## Byte, identity and admission contract

- Constructor rejects more than N words with Limit and otherwise transfers
  ownership without a copy. Storage stays private and immutable through views.
- Encode emits one byte buffer: checked byte width, then little-endian i16 words.
  On little-endian targets a safe Bytemuck byte view avoids temporary allocation;
  use the reference's published Bytemuck 1.25.2 with `must_cast`, no defaults.
  Its registry checksum matches the reference lock:
  `95832e849adfb21180ccb6826a99da14e5d266ae5c2e668e1602cf234f153797`.
  Big-endian conversion reserves fallibly and writes each `to_le_bytes` value.
  This conversion is adapter-owned temporary work, not codec output reservation;
  the reference also performs it during serialization/preflight.
- Deserialize requests a byte span, charging the existing Vec<i16> visitor
  size and full span. Reject odd lengths and more than N words before typed
  output reservation/copy. Read individual words with `from_le_bytes`, so
  unaligned input is supported without borrowed typed casts or unsafe code.
- Field and adapter allocation failures become Serde errors (Malformed through
  the codec unless its meter refused first). No new Allocation classification
  or acceptance rule is introduced. Trailing input remains Trailing.
- The field retains owned decoded storage. Borrowed byte projections point into
  original input; packed words do not. No method identity or header changes.

## Qualification

Literal vectors cover empty, zero, signed extrema, negative and small values;
compare ordinary lists explicitly. Check exact and insufficient field/runtime
bounds, malformed/truncated/odd spans, unaligned input and owned custody. Exercise
the big-endian conversion helper on the native test host against literal bytes;
this is not native big-endian target execution.

The representative record includes enum/option, bounded bytes, packed i16 and
ordinary bounded i16 controls. Freeze literal payload/request/response bytes and
exercise owned/reused/slice outputs plus a borrowed-byte projection.
Rust/TypeScript exchange, reference differential comparison, allocator
instrumentation, native macOS execution and opt-z/opt-3 measurements remain
pending. Native correctness and Wasm compilation are not a replacement decision.
