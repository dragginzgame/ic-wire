# Shared encoding preflight and output custody

Destination contract: the proposed `encoded_len`, `encode`, `encode_into` and
`encode_to_slice` APIs in DESIGN.md, each taking one Encoding selection.
They replace payload/request/response wrapper multiplication only in this
unpublished evaluation. Bytes, limits and failure categories remain frozen.

Reference: `2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/native/encode/mod.rs` and `native/mod.rs`.
The encode adapter's source SHA-256 is
`6314ad42b620168133a95038e1539697ca7965f63f7137d2a0a9fc092a0df035`.
The initial reuse was limited to the bounded Serialize/Serializer/compound adapter
structure and admission visits, retaining the IcyDB attribution and root license.
The measured integer decision below adds only a one-byte shortcut; wider varints
and enum tags remain delegated to Postcard.

## Measured one-byte integer shortcut

The expanded matrix exposed approximately 2.3× native opt-z encoding losses for
small integer lists. Assembly shows Postcard constructing a varint buffer even
during sizing. One shared macro now handles signed/unsigned 16/32/64-bit values:
signed values use the standard ZigZag mapping, unsigned values below 128 go
through Postcard's raw-u8 method, and all larger values use Postcard's unsigned
writer. Raw 8-bit values, 128-bit values, lengths and enum tags are unchanged.
The private adapter is constructed only around Postcard serializers, where this
one-byte identity holds; it is not a general-purpose Serde representation change.

The shortcut does not write continuation bytes or maintain a second sizing
serializer. The original value is admitted before this dispatch, so narrowing
the internal write method changes neither visits, charges, depth nor allocation
ordering. Exact i16/u16 exhaustive vectors, wider boundaries, all output
destinations, inconsistent-Serde cases and budget sweeps pass.

This selected one-byte technique is informed by the pinned reference encoder
above; its wider unrolled writers are not imported. The independently measured
request workload falls from 5,224/2,674 to 4,953/2,435 IC instructions at opt-z/
opt-3. The representative Wasm grows by 129/181 bytes, while remaining 736/695
bytes smaller than reference. Opt-z response encoding rises to 1,751 instructions
(reference 1,744); the small loss is retained explicitly alongside the larger
request improvement. See the codec benchmark guide for native controls and
the final measured source identities. This is a measured exception to pure
primitive delegation, not permission to recreate all legacy specializations.

## Contract

- One sizing preflight uses Postcard's serialized_size and fresh admission.
  A frame subtracts 11 from both byte and allocation allowances before sizing.
  This preserves the available traversal allowance. The completed length adds
  the header back and cannot overflow because sizing is bounded by bytes.
- Every value pays four nodes and twice its Rust size in charge units and
  requires depth + 2; children increment depth by one. Text/byte spans pay their
  lengths in additional units. The shared meter retains decoder accounting.
- Writing gets a fresh meter with the original allowances. It is bounded by
  preflight length, not allocator capacity or caller buffer length.
- Owned output reserves fallibly only after preflight. Reused Vec output is
  empty on every error; retained capacity on failure is not promised.
- A short slice fails with Limit before any write. Other failures can alter
  its contents; only the successful returned prefix is publishable.
- Custom Serialize must be deterministic across passes and is trusted for
  work outside visited values. A longer second pass cannot overrun preflight;
  without a budget refusal its supplier error maps to Malformed. As in the
  reference, a shorter successful second pass can return fewer bytes.
- Sizing/serialization failures map to Limit after budget refusal, otherwise
  Malformed. Fallible output reservation failure maps to Allocation.

No prepared-encoding token, new sink abstraction, scratch callback, executor,
schema graph or new codec is introduced. Tests use independent literal payload
and frame vectors plus exact and insufficient allowances, unchanged short
buffers, cleared reused output, and deliberately inconsistent serializers.
The [qualification workspace](../../qualification/README.md) now checks selected
allocation ordering and reference comparisons. These focused cases do not prove
universal compatibility or process-wide allocation bounds.
