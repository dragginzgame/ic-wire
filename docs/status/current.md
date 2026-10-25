# Current handoff

## Released 0.7.9; pending 0.7.10

The checkout baseline is released 0.7.9
(`00b1686a80cd7a3068be0a24aaee0dc8239009c7`); own versions remain 0.7.9.
The current batch implements the requested derive cleanup and registry traversal
audit. Staging, commits, version changes and publication remain maintainer-owned.
No sibling checkout, remote issue or release state was changed in this batch.

## Host simplification

Field attributes are parsed once into the existing Projection and reused for
metadata labels and generic bounds. The later independent field collection is
removed. No new public API, helper type or parsing mode was introduced.

Registry generation serializes the borrowed map directly instead of converting
it into another owned JSON value tree. Generated tokens, JSON/source escaping and
client contracts are unchanged. No protocol/runtime/admission change is included.

The traversal audit retained projection conflict checks, alias-cycle validation,
public-input validation and rooted-lint expansion limits. Each protects a distinct
boundary; type_name and function pointers are not safe unique-type substitutes.
A parallel validation path or cache would add complexity without a demonstrated
need. [Evidence](../evidence/host-cleanup-0710/README.md) records the audit and
reproducible before/after measurements, including small controls and noisy cases.

Host expansion of an attributed 100-field record measures 444.681 → 352.476 µs
(−20.74%); generic 100-field records measure −14.63%. Plain records are essentially
unchanged. Registry generation measures 5.072 → 3.015 µs for one recursive named
record and 323.461 → 120.342 µs for 100; all paired generated output is identical.
These are host operation measurements, not whole-build, codec, IC or Wasm savings.

The full TypeScript gate passes 159 vectors, 33,560 mutations, 29 native projection
vectors, 16 admission shapes and 54 simulated-endian cases, strict Clippy,
Rust 1.91 and native/Wasm schema forwarding. Native tests pass 35 cases; derive
refusal tests and nine compiler diagnostics plus a passing control pass. Formatting,
links and whitespace checks pass. Full CI and fresh macOS execution were not run.

## Retained adoption context

0.7.9 delivered rooted bulk review for
[#25](https://github.com/dragginzgame/ic-wire/issues/25). Consumer root labels and
primitive-word suffixes keep acknowledgements independent across interned fields.
The definition-level lint keeps its existing semantics. See
[rooted lint evidence](../evidence/rooted-bulk-lint-079/README.md).
IcyDB has its own active adoption work; this batch does not qualify or replace it.
Issue closeout/hosted CI was not reinspected as part of this focused cleanup.

0.7.8 delivered [optional byte/rename projection](../evidence/serde-projection-078/README.md).
Keep existing browser labels and optional byte tags when replacing manual metadata;
custom serializers still require explicitly verified metadata.
0.7.7 delivered [recursive metadata](../evidence/native-registry-077/README.md),
[incremental browser allocation](../evidence/typescript-writer-077/README.md) and
[parser cleanup](../evidence/parser-cleanup-077/README.md). Historical evidence is
retained; no compatibility codec or fallback was introduced.

## Production direction and tooling

One Serde/Postcard codec remains. Metadata follows declaration order without field
IDs. Consumers own identities and coordinated layout changes. Keep original frames
for retries and raw-byte signatures; collection/custom Serde normalization remains
consumer-owned. Use ByteBuf/Bytes for binary fields and Packed<T> where measured,
retaining bounds and small-varint controls. See [runtime](../runtime.md) and
[bulk guidance](../packed-fields.md). Real big-endian execution remains unqualified.

Build/Testkit and release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling is 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
