# Current handoff

## Released 0.7.8; pending 0.7.9

The checkout is at released 0.7.8 (`dece7fccbf98f059994e71adf3b9bcc532057522`).
Both registry package caches identify that exact release commit. The hosted
[release CI](https://github.com/dragginzgame/ic-wire/actions/runs/40175224140)
passed Ubuntu and Apple Silicon macOS; Intel macOS remained in progress at the
last inspection. Issues #23/#24 remain open until the required matrix completes.
No release/publication was rerun, and no issue was closed during this batch.

The user authorized release closeout, IcyDB adoption and a consumer-gap audit.
IcyDB already has active uncommitted Wire adoption work, which was left untouched.
Its published-0.7.8 audit exposed [#25](https://github.com/dragginzgame/ic-wire/issues/25),
a prerequisite for replacing its reflection projection without weakening field
review. No isolated adoption PR was created over the active owner's work.

## Rooted bulk review

Pending `lint_bulk_roots` accepts explicit public root labels and traverses
registry refs per use site. Synthetic type names do not enter diagnostic identity.
A word suffix distinguishes changed primitive types; field removal/renaming or
conversion to bulk makes the old acknowledgement stale. Cycles stop on active
references; shared siblings remain independent. Expansion is depth/work bounded.
The existing definition-level `lint_bulk_fields` API retains its semantics;
both use the same walker and candidate detector. No protocol, runtime codec,
resource-admission or generated-TypeScript change is included.

[Evidence](../evidence/rooted-bulk-lint-079/README.md) retains source and repeated
host-cost controls. 35 native tests, strict all-target Clippy, doctests and
Rust 1.91 host checks pass. Legacy lint controls remain within ordinary timing
noise; rooted review has additional per-use traversal cost. No IC/Wasm savings
are claimed. Publication and consumer migration remain outstanding.

Consumers should select stable DTO or method/direction roots and explicitly
review rooted paths, for example `/Reply/fields/field_ids/list/u32`. Do not copy
one synthetic-container acknowledgement to all fields. Custom scalar constraints
and endpoint policy remain consumer-owned; paths are not full contract hashes.

## Retained qualification and production direction

0.7.8 delivered optional serde_bytes and symmetric Serde rename projection;
see [projection evidence](../evidence/serde-projection-078/README.md). Keep existing
browser labels and optional byte tags when replacing manual metadata. Unsupported
custom serializers still need explicitly verified metadata.
0.7.7 delivered [recursive metadata](../evidence/native-registry-077/README.md),
[incremental browser allocation](../evidence/typescript-writer-077/README.md) and
[parser cleanup](../evidence/parser-cleanup-077/README.md). Historical evidence is
retained; no compatibility codec or fallback was introduced.

One Serde/Postcard codec remains. Metadata follows declaration order without field
IDs. Consumers own identities and coordinated layout changes. Keep original frames
for retries and raw-byte signatures; collection/custom Serde normalization remains
consumer-owned. Use ByteBuf/Bytes for binary fields and Packed<T> where measured,
retaining bounds and small-varint controls. See [runtime](../runtime.md) and
[bulk guidance](../packed-fields.md). Real big-endian execution remains unqualified.

Build/Testkit and release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling is 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
Own versions remain 0.7.8. This ordinary follow-up leaves staging, commits and
publication of pending 0.7.9 to the maintainer. Full CI and consumer suites were
not run for this local change.
