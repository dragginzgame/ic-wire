# Current handoff

## Current batch

Released 0.5.1 (`cdd47c3`) is the baseline. Pending 0.5.2 refines initial
reservation for finite schema ceilings of at most 128 bytes. Larger schemas keep
the released growth policy. Historical benchmarks retain their own measured
dependency selections; no dependency or package versions change in this batch.
Merging, tags, releases and publication remain maintainer-owned.

Finite encoding serializes once into fallibly growing output. In 0.5.1 requests
start at up to 64 bytes, grow within the admitted ceiling, and reserve the remainder
when a growth exceeds a quarter of that ceiling. Allocation failures remain
`Error::Allocation`; schema/serializer refusal identities, canonical framing,
method/direction binding and conservative admission are unchanged. Native
sizing/reuse remains unchanged. No new public API or codec mode was added. The 0.5.2 refinement requests the full
small-schema ceiling on first growth, avoiding another reservation for control
replies that cross 64 bytes; tiny values may initially request up to 64 more bytes.

## Evidence and tradeoffs

The 44 allocation fixtures plus six Relay-shaped batches have identical frame
hashes before/after under opt-z and opt-3. Tiny Relay batches use 88–96% fewer IC
encode instructions and retain 64 rather than 196,783 output bytes. Full Relay
cases cost at most 0.61% more instructions. The full array manifest changes by
+0.19%/-12.32% across the profiles; the full packed manifest costs up to 2.14% more.
Native fresh 4 KiB blob encoding adds about 38 ns (45%) from another reservation.
Combined probe Wasm grows 931/537 bytes, gzip 590/323 bytes. These are codec
measurements, not installed Relay latency or peak memory guarantees.
See [methodology and complete results](../evaluation.md#bounded-finite-output-growth).
Source snapshots, raw samples, locks and probes are retained at `target/output-growth/`.
No sibling checkout was modified. Consumer integration remains consumer-owned.

## Retained 0.5 contract

Wire supplies serialization, finite bounds and schema metadata; finite decode
adds `Deserialize<'de>`. BytesRef/TextRef and lifetime-aware derives support
borrowed bounded contracts with the same owned-form bytes and TypeScript output.
Use `into_bytes()` when retries must outlive a borrowed method's source value.
See the [migration guide](../../crates/ic-wire/README.md#borrowed-bounded-contracts-and-the-05-api-cut).
The [earlier borrowed measurements](../evaluation.md#final-05-borrowed-contract-measurements)
remain source-bound evidence for the released redesign.

## Qualification

This batch passes 36 library/runtime/native/borrowed tests on Rust 1.91, strict
all-target/all-feature runtime Clippy, Rust 1.91 Wasm compilation, formatting,
documentation links and diff checks. The new small-schema allocation regression
fails on 0.5.1 and passes with the refinement. Retained 0.5.1 TypeScript evidence
covers 90 vectors, 17,524 mutations and 1 MiB boundaries; this batch does not
regenerate its unchanged schemas. These are focused Linux checks, not full CI
or native macOS qualification.

## Code generation preference investigation

The public-API adapter was measured and then rejected: mixed opt-3 saved 1.8%
linked and 4.2% compressed Wasm, but opt-z linked size grew 3.9%. These gains did
not justify the Serde forwarding code, tests and permanent API. The adapter,
optimization attribute, root setting and dedicated tests have been removed.
No per-type preference is supported. The output-growth fix remains intact.
See [retained experimental evidence](../evaluation.md#public-api-code-generation-adapter).
Source snapshots, locks, raw samples and probes remain at `target/codegen-adapter/`.
