# Runtime contract

The maintained [runtime guide](../crates/ic-wire/README.md) ships with the crate
and supplies its Rustdoc, quickstart, limits and wire contract.

## Explicit packed domain fields

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
