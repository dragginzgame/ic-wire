# Runtime contract

The maintained [runtime guide](../crates/ic-wire/README.md) ships with the crate
and supplies its Rustdoc, quickstart, limits and wire contract.

## Explicit packed domain fields

`Packed<[u8; 16]>` carries opaque 16-byte identifiers as one byte buffer. The
`hash16` browser schema uses a flat `Uint8Array` and rejects partial identifiers.
Adopting one ID changes `01 + 16 bytes` to `10 + 16 bytes`; eight IDs change
`08 + 128 bytes` to `80 01 + 128 bytes`. Byte order is preserved. Coordinate
identities and clients, retain retry/hash bytes, and requalify buffer budgets
and field bounds. Existing fields keep their bytes, limits and acceptance.

`serde_packed::bools` explicitly selects byte-buffer admission for `Vec<bool>`.
Valid bytes match ordinary bool lists (`[false, true]`: `02 00 01`); Rust and
generated `boolean[]` clients validate 00/01 before allocating decoded values.
No bit packing or automatic Serde change is introduced. Switching fields still
requires regenerated clients, preserved field bounds and budget review: fewer
nodes and different conservative allocation charges are consumed. Existing
codec bytes, method identities, limits and acceptance remain unchanged for
consumers that do not select the adapter. Packed integer/hash words retain their
existing all-bit-patterns-valid invariant; `Packed<bool>` remains unsupported.

`serde_packed::nonzero_i64` and `serde_packed::heap_u64` reuse the existing Packed
byte codec for `Vec<NonZeroI64>` and `BinaryHeap<u64>`. Selection is explicit in a
Serde `with` attribute. Nonzero decode rejects zero after bounded word decoding;
heap decode accepts arbitrary order and duplicates and rebuilds the heap. Browser
metadata enforces nonzero values automatically, but does not normalize heap order.

These adapters do not change existing lists or Packed readers. Consumer adoption
from an ordinary `[-1]` nonzero list changes `01 01` to
`08 ff ff ff ff ff ff ff ff`; heap `[1]` changes `01 01` to
`08 01 00 00 00 00 00 00 00`. Coordinate identities and consumers, regenerate
clients and requalify limits. Valid bytes already represented by Packed match;
adapter record-field encoding charges can differ because Serde passes a smaller
reference wrapper. Admission is still a conservative traversal allowance, not a
physical-allocation guarantee. Preserve count bounds and original retry/hash
bytes; never infer canonical heap bytes from reconstruction.
