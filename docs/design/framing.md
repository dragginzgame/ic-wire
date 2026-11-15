# Portable framing contract

Frames have an 11-byte header: ASCII `IW`, a little-endian u64 method ID, then
request direction `0` or response direction `1`. Payload bytes follow directly.
`Encoding::Frame` selects identity and direction for the root encode/decode APIs.
`Encoding::Payload` selects a bare Postcard value.

`method_id(name)` hashes the UTF-8 domain `ic-wire:method:` and exact name using
FNV-1a-64. This is routing identity, not authentication or a schema hash.
`validate_request_header(id, &[u8; 11])` checks magic, method and request direction;
it returns `Error::Identity` on a mismatch. It does not validate payload or budgets.
Full decode precedence and header allowance transformations are specified in the
[admission ledger](admission.md).

## Qualification

Literal headers cover zero, maximum and asymmetric IDs, plus name-derived `ping`.
Every differing byte at each header offset is checked as an identity refusal.
Method vectors cover empty, ASCII, case-sensitive, embedded-NUL and multibyte
UTF-8 names. Expected IDs were independently calculated with arbitrary-precision
arithmetic. Run `cargo test -p ic-wire --test framing_tests --locked --offline`.
The [standalone consumer](../../qualification/ic-wire-consumer/README.md) independently
checks complete request/response literals and malformed-input boundaries.

## Source provenance

Protocol constants and error vocabulary were reviewed from source revision
`2dc434b10bec2fca92345d940dbfedc05993e9d5`. The retained root licence applies.
Selected source hashes under `crates/ic-wire/src/`:

| Source | SHA-256 |
| --- | --- |
| `native/mod.rs` | `49f7dd52976426b38949683d7568c47b055e2baf9f32cb3271ce2c4750d0e8a8` |
| `contract/mod.rs` | `b6ad6ae29b28b8a723dbe2bdf260f46f91632619806292e74a77dbf7df63d7d3` |
| `lib.rs` | `54531fcb75c6a69e246722ea5b1487c76dc729981b41b5239960de91b551df0d` |

These identify selected inspected inputs, not an external runtime dependency.
