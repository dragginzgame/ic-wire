# Current handoff

## Released baseline and pending 0.7.8

The checkout is at release 0.7.7 (`6d1f8a1bbde6ad16d24d1d857b1a30292d5b26b0`).
Package versions remain 0.7.7. Pending 0.7.8 addresses host metadata blockers
[#23](https://github.com/dragginzgame/ic-wire/issues/23) and
[#24](https://github.com/dragginzgame/ic-wire/issues/24). Publication is part of
both acceptance criteria and remains outstanding. No issue was closed.
Staging, commits, versions, publication and pushes remain maintainer-owned.
No sibling repository was changed.

## Serde projection additions

The sealed serde_bytes field qualification now carries its projected layout.
Options retain their tag and project to options of byte buffers rather than
ordinary numeric lists. Aliases, references and qualified generic fields use
trait resolution, without token-name guesses. None / Some(empty) / Some(nonempty)
remain distinct; Option<Vec<u16>> remains rejected.

Symmetric field/variant Serde renames project browser keys/tags while preserving
field order and declaration-order variant ordinals. Duplicate labels reject.
Asymmetric renames, rename_all, alias and custom serializers remain unsupported;
manual serializers still need explicitly verified metadata. Recursive registries
work with the renamed variants. No new codec or compatibility lane was added.

[Projection evidence](../evidence/serde-projection-078/README.md) records exact
frames, admission boundaries, focused validation and repeated host timing controls.
The full TypeScript gate passes 159 ordinary vectors, 33,560 mutations, 29 native
projection vectors, 16 admission shapes and 54 simulated-endian cases. Optional
byte adapters retain a measured Rust/TS node-charge difference; copying equal
budgets is not an acceptance-parity guarantee. Regenerate clients and preserve
consumer browser labels when replacing manual metadata. Runtime bytes, framing,
identity, admission and production codecs are unchanged. No IC/Wasm savings claim.

## Retained qualification

0.7.7 delivered named recursive SchemaRegistry collection, typed method_limits
projection and the incremental browser writer. See the packaged README for
consumer qualification and the existing evidence for
[host metadata](../evidence/native-registry-077/README.md) and
[writer allocation](../evidence/typescript-writer-077/README.md).
The writer starts at at most 64 bytes and grows within its directional ceiling;
outputs remain owned. Growth can increase cumulative allocations for tight limits.

The [parser cleanup](../evidence/parser-cleanup-077/README.md) removed the unused
input abstraction and preserved the selected codec. Keep separate packed
implementations after the [consolidation screen](../packed-consolidation-0611.md).
Historical evaluation remains archived rather than deleted.

## Production direction and tooling

One Serde/Postcard codec remains. Metadata follows Serde declaration order without
field IDs. Consumers assign identities and coordinate layouts. Collections/custom
Serde may normalize; retain original frames for retries and raw-byte signatures.
Use ByteBuf/Bytes for binary data and Packed<T> for suitable bulk fields, preserving
field bounds and small-varint controls. See the [runtime](../runtime.md) and
[bulk guide](../packed-fields.md). Real big-endian execution remains unqualified.
Full CI, fresh macOS execution, publication and consumer adoption were not performed
in this local batch. Sibling endpoint policy stays consumer-owned.

Build/Testkit and the release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling remains 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
The [evaluation guide](../evaluation.md) indexes maintained and historical evidence.
