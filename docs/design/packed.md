# Packed fields

`Packed<T, N>` is the single packed field type. N counts elements and defaults to
usize::MAX. `new(Vec<T>)` returns Result, rejecting more than N words with Limit
without copying. `as_slice` borrows immutable words; `into_vec` returns storage.
Supported sealed words are i8/i16/u16/i32/u32/i64/u64 and 16/32-byte opaque IDs.
Domain bool/finite/nonzero/heap adapters remain explicit Serde field adapters.

The pre-publication hard cut removes `PackedI16<N>`, its duplicate wrapper methods
and Serde implementations, the `PackedI16` graph node, and the `packed-i16` schema
kind. No alias or product compatibility conversion remains. Use `Packed<i16, N>`
instead. All word types share the same constructor, serializer and owned visitor.

Host metadata uses `{k:"packed",word:"i16",max:N}`, omitting max for unbounded
fields. Browser max counts words, including opaque IDs, rather than backing-array
bytes. Host admission rejects maxima whose byte width exceeds exact JavaScript
integer range. Runtime bounds apply to bool, finite and nonzero metadata as well.
Consumer Rust Serde remains responsible for domain constraints on manual schemas.

## Byte and allocation contract

The Postcard prefix counts bytes, followed by fixed-width little-endian words.
Ordinary lists keep element counts and Postcard varints; small values can be
larger when packed. Choosing a packed field never rewrites Serde automatically.
On little-endian targets a checked-width safe Bytemuck view avoids temporary
allocation. Big-endian conversion reserves fallibly and appends little-endian
words. That adapter work is separate from codec-owned output reservation.

Decoding obtains a byte span, then rejects partial words and excessive element
counts before reserving the owned typed output. Unaligned input is supported by
individual word reads; output never borrows typed storage from the input.
The visitor returns Vec, keeping traversal accounting independent of the wrapper.
Serde field failures map to Malformed unless the codec meter refused first.
The browser reports an exceeded element maximum as `limit`; these are separate
Rust and TypeScript error contracts, both tested explicitly.
Trailing bytes remain Trailing. The consolidation changes neither frame identity
nor encoded payloads for previously admitted values.

## Provenance and qualification

Selected serializer/visitor patterns come from reference commit
`2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/bounded/packed_i16/mod.rs` and `packed/mod.rs`, under the
retained licence. Their source hashes are respectively
`5b61334310fe72e15b3cf562909c62efa45a35bde5582331bf8a90abfab61835` and
`8f56da054856e3ba877f046bb894d3fce2c703a0aebb2ec8745ef8dbbd13a091`.
The generic bound is a destination API decision, not a copied legacy API.

Native checks cover each supported word type, zero/exact/excess bounds, literal
prefixes, partial/truncated words, selected input spans and storage custody.
An isolated allocator check verifies excessive wider-word counts reject before
allocation. Browser checks exercise all integer/ID/domain packed metadata kinds,
including zero bounds and unsafe host maxima. Big-endian helpers are tested with
literal bytes without claiming execution on a big-endian machine.

The legacy interpreter remains unchanged as a private oracle. A test-only schema
adapter lowers bounded i16 to its legacy spelling; new bounded-word behavior is
checked independently because that interpreter does not enforce it. Historical
performance reports retain their original source hashes; refresh qualification
after production changes before attributing those results to the hard cut.

The packed reader uses a checked `try_into().unwrap()` on each exact-width chunk.
Its private caller establishes that width after rejecting partial words and
excess element counts. Replacing the custom `expect` message lets opt-z eliminate
a retained per-word panic-helper call; it does not remove the conversion check,
introduce unsafe reads or change public malformed-input behavior. See the
[codec investigation](codec-benchmarks.md) for native and Wasm qualification.
