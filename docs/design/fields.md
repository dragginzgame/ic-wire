# Bounded bytes, text and ordinary lists

Scope: `Bytes<N>`, `Text<N>`, `List<T, N>`, `BytesRef<'a, N>` and
`TextRef<'a, N>`. These preserve the corresponding reference field contracts.
They do not automatically rewrite consumer Serde layouts. The subsequent
[packed i16 field](packed.md) is a separate opt-in representation.

Reference: `2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/bounded/mod.rs` and `bounded/borrowed/mod.rs`.
SHA-256 of the inspected committed sources:

| Source | SHA-256 |
| --- | --- |
| `bounded/mod.rs` | `7be5af1d71fb91dca14400d2afa6fcf73c1bd6f68a0a4f0ec066640ef13ea4a1` |
| `bounded/borrowed/mod.rs` | `7f44fa97365da377131702165a97587b90968c87c26ca7da4a83befe6b2c8753` |

Destination: `fields/mod.rs` and `fields/borrowed.rs`. Reuse is limited to these
five field adapters and their Serde visitors. Preserve root licensing. Retain
their visitor result types and delegation so type-size charges do not change.
Remove the bounded-list special-case dispatch and inline attributes; one list
reader handles every bound. This is a consolidation candidate, with no measured
size/performance claim. No packed code or host metadata is imported.

| Contract | Required behavior |
| --- | --- |
| Owned constructors | Reject excess length with Limit, otherwise transfer ownership without copying |
| Bytes and text bounds | N counts bytes; text must be UTF-8 |
| List bound | N counts elements, including zero-sized elements |
| Ordinary list bytes | Element count followed by ordinary Serde element representations |
| Bulk bytes | Byte count followed by the byte span; no per-element visits |
| Field rejection during decode | Serde custom error maps to Malformed, unless a traversal refusal takes precedence as Limit |
| Bytes decode | Admit span and bound before fallible owned-output reservation/copy |
| Text decode | Preserve reference byte-buffer admission, then validate UTF-8 on owned bytes; no claim of UTF-8 validation before allocation |
| List decode | Suppress input size hints at codec boundary; bounded amortized growth after admitted element decoding |
| Excess list element | Reject with a seed that never calls the excess element's decoder; seed admission can itself fail first |
| Borrowed decode | Accept only borrowed byte/text visits; no owned or transient fallback |

Owned values keep private storage and expose immutable views plus consuming
extraction. Borrowed views retain their input lifetime. These adapters constrain
their own lengths, not arbitrary allocation/work in consumer Serde code.

Qualification: literal encodings, empty/exact/excess bounds, UTF-8 byte lengths,
borrowed pointers and transient-input rejection, ordinary-vector controls,
constructor ownership, excess-element decoder suppression, and a record with
bounded bytes/text/list and an enum/option through every encoding destination.
The subsequent packed-field tests cover the full native record shape; Rust/TS
exchange remains pending.
Allocation instrumentation, reference differential comparison and opt-z/opt-3
measurements remain pending; this is not completed replacement qualification.
