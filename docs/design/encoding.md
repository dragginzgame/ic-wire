# Shared encoding preflight and output custody

Destination contract: the proposed `encoded_len`, `encode`, `encode_into` and
`encode_to_slice` APIs in DESIGN.md, each taking one Encoding selection.
They replace payload/request/response wrapper multiplication only in this
unpublished evaluation. Bytes, limits and failure categories remain frozen.

Reference: `2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/native/encode/mod.rs` and `native/mod.rs`.
The encode adapter's source SHA-256 is
`6314ad42b620168133a95038e1539697ca7965f63f7137d2a0a9fc092a0df035`.
Reuse is limited to the bounded Serialize/Serializer/compound adapter structure
and admission visits. Preserve the IcyDB attribution and root license. Remove
all private primitive and enum-tag writers: Postcard owns these operations.
This is an investigational supplier-delegation candidate, not a measured win.
Exact vectors qualify bytes and budgets; opt-z/opt-3 and instruction comparisons
are still required before selecting this over the reference's specializations.

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
Allocator instrumentation and full reference differential qualification remain
pending; capacity observations alone do not prove process-wide allocation bounds.
