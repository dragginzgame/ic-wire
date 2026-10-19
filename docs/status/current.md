# Current handoff

## Scope and release boundary

The maintainer authorized the 0.5.0 borrowed bounded-contract redesign. Source
started clean at released 0.4.10 (`67aced4`). Pending notes select 0.5.0 because
generic callers can no longer infer owned deserialization from `T: Wire`.
Package versions, staging, commits, tags, pushes and publication remain maintainer
owned; this task performs none of them.

IC Wire remains one bounded canonical Postcard message codec. Transport, auth,
retry decisions, persistence and recovery belong to consumers. Native dynamic
Serde admission remains available. The workspace has no endpoint SDK/simulator
integration; installed consumer qualification stays with its owner.

## Implemented 0.5 contract

`Wire` supplies serialization, finite bounds and schema metadata. Finite decode
adds `Deserialize<'de>` at the input boundary. `BytesRef<'a, N>` and
`TextRef<'a, N>` borrow bounded input; owned Bytes/Text expose `as_borrowed`.
Record and enum derives support lifetime parameters and inline outlives bounds.
Type/const parameters, where clauses and recursive derives remain unsupported.

Owned and borrowed forms preserve field IDs, frame bytes and TypeScript output.
Views have no owned payload heap; nested List/Box allocations remain accounted
for. Encoded frames still own their output. Use `into_bytes()` when retained
retry bytes must outlive a lifetime-parameterized method's source value.
Consumers needing owned generic decode add `DeserializeOwned`; borrowed generic
decode uses an input-lifetime bound. See the [migration guide](../../crates/ic-wire/README.md#borrowed-bounded-contracts-and-the-05-api-cut).

The new consumer-shaped test covers 0/1/32/4,096/1,048,576-byte chunks, multiple
lifetimes, enums and nested collections, exact retries, UTF-8 and malformed input.
An isolated native allocation probe observes zero decode allocation requests for
borrowed chunks, versus two requests totaling 1,048,581 bytes for an owned 1 MiB
chunk plus its five-byte name. Input/fixture construction is excluded. This is
allocation-request evidence, not peak heap, latency or deployed Blob performance.
Final derived-contract probes now also measure native timing, IC instructions
and separately linked Wasm size under opt-z and opt-3. For 1 MiB decode plus a
full scan, borrowing reduces IC instructions by 17–25% and native median time
by 10–17% relative to the owned form of the same candidate. Frame hashes match.
See [methodology and limits](../evaluation.md#final-05-borrowed-contract-measurements).
These are isolated codec measurements; no sibling repository was changed.

## Qualification

Runtime/all-feature tests and doctests, strict runtime/derive/evaluation Clippy,
16 compiler-diagnostic cases with a passing control, evaluation tests and Rust
1.91 checks pass. The borrowed integration tests also run on Rust 1.91; the
consumer-shaped borrowed allocation probe checks for Wasm on that floor.
Rust/TypeScript conformance retains 90 vectors and 17,524 differential mutations;
new assertions prove generated owned/borrowed codecs are identical.
Formatting, pins, dependency boundaries and documentation links are checked.
Full CI, native macOS and installed consumer suites were not run.

Source snapshots, logs, allocation-probe source/lock and its compact CSV are
retained locally at `target/borrowed-contracts-0.5/`. Runtime dependencies are
unchanged; the previously adopted IC Build 0.22/Host 0.30 remain tooling-only.
The reviewed Shared Tooling snapshot remains 0.24.2 and is unchanged.

## Earlier evidence

See the [allocation comparison](../allocation-comparison.md),
[bounded allocation study](../bounded-allocation-study.md),
[Wasm optimization report](../runtime-optimization.md) and
[MessagePack comparison](../messagepack-comparison.md) for source-bound prior
measurements. Their isolated borrowed prototypes describe those historical
experiments; the current API is now implemented in the runtime and derive crate.
The reusable native buffers and Wasm same-slice canonical comparison are retained.
