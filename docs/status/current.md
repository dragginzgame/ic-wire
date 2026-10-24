# Current handoff

## Released baseline and pending 0.7.6

0.7.5 is released at `e79140671c739acbd6a5a0b7039d89999b333fa4`; package
versions remain 0.7.5. This batch began with a clean checkout. Pending 0.7.6
strengthens adversarial native decode tests and documents consumer integration
boundaries. Runtime code, dependencies, build profiles and protocol are unchanged.
Staging, commits, versions, publication and pushes remain maintainer-owned.
No sibling repository was changed.

## Adversarial qualification

The native adversarial module verifies request/response size and envelope refusal
before consumer deserialization, and valid decoding after envelope/payload failures.
Zero-byte collections visit each ordinary Serde element once for 0–128 elements.
Forged counts of 1,024, 1,048,576 and usize::MAX hit the node allowance with the
same bounded visitor count at each tested allowance (8, 16, 32). Later calls get
fresh allowances. Custom Serde work outside codec traversal remains trusted.

Reproduce with `cargo test --locked --offline -p ic-wire --test native` (23 tests)
and `cargo clippy --locked --offline -p ic-wire --test native -- -D warnings`.
These are work-count regression tests, not native timing or IC instruction
benchmarks. No runtime/generator cost changed, so no performance or Wasm saving
is claimed. Full IC and browser matrices are not rerun for this tests/docs batch.

The [runtime contract](../runtime.md) now states the consumer integration boundary.
Wire cannot establish transport pre-copy rejection, handler admission, rollback or
retry effect deduplication. Those require consumer-owned installed tests.

## Production direction

One Serde/Postcard codec remains. Native metadata uses Serde declaration order,
without field IDs. Consumers assign endpoint identities and coordinate layout
changes. Decode validates primitives in one pass; collection ordering/uniqueness
and custom Serde normalization are consumer-owned. Retain original frames for
exact retries and raw-byte signatures/hashes. Decoding does not guarantee
byte-identical re-encoding. No legacy reader or migration shim.

Use ByteBuf/Bytes for binary data and Packed<T> for supported bulk integer/hash
fields, including i8. The optional host bulk lint reports ordinary primitive
lists and supports reasoned exact-path acknowledgements with stale detection.
It does not rewrite fields. Tiny fields/small varints can favor ordinary vectors.

0.7.5 supplies host-only Cow<str> metadata for transparent consumer identifiers;
normal Serde decoding still produces Cow::Owned. The read-only Canic handoff now
selects 0.7.5 and records installed Blob workflow qualification. Its remaining
endpoint and deployment work stays consumer-owned.

## Tooling and retained evidence

Build/Testkit and the release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling remains 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
Keep one codec optimized for total uncompressed Wasm and code-section pressure;
gzip and symbol attribution are not savings criteria. Consumer opt levels remain
test conditions, not a library-controlled fix.

The retained [0.7.4 one-byte shortcut evidence](../evidence/short-varint-074/sizes.json)
and [instruction pairs](../evidence/short-varint-074/instructions.csv) cover 6,246
matched rows, including small regressions. Rejected experiments and complete
qualification narratives remain in Git at the released baseline above; their
evidence artifacts remain under docs/evidence, unchanged.

The [0.7.1 bulk qualification](../evidence/bulk-advice-071/README.md),
[one-pass study](../evidence/one-pass-070/README.md) and
[0.6.13 full matrix](../full-comparison-0613.md) retain their historical scope.
See the [bulk guide](../packed-fields.md). Real big-endian execution remains
unqualified; installed-consumer integration and storage contracts stay outside
Wire's qualification claims. Native macOS coverage comes from hosted CI.
