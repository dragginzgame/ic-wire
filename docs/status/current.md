# Current handoff

## Released baseline and pending 0.7.0

0.6.13 is released at `1003261d481a65cf3dd87b5f1e249bc82103ddbc`; package versions
remain 0.6.13. The maintainer explicitly approved removing the post-decode
reserialization check after reviewing normalization examples. The open changelog
is 0.7.0 because acceptance rules and the public error enum change. Staging,
versions, commits, publication and delivery remain maintainer-owned. No sibling
was changed. Incoming research coverage and evidence were preserved.

## Implemented contract

One production Serde/Postcard codec remains. Decode requires only Deserialize,
uses a private attributed Postcard 1.1.3 reader with strict varint/char checks,
and retains framing, identity, trailing-byte and resource checks. Re-encoding
and the encoder traversal budget are no longer part of decode. Compare, its
helper/tests and Error::Noncanonical are retired; malformed primitives now fail
as Malformed. No dependency fork, alternative mode or compatibility reader exists.

Consumers own map/set ordering, uniqueness and custom Serde normalization.
Map bytes `02 02 14 01 0a` now decode and re-encode as `02 01 0a 02 14`.
Retain original immutable frames for exact retries and raw-byte signatures/hashes;
decode success does not guarantee identical re-encoding. Review exhaustive error
matches and configure encode/decode allowances independently. Emitted bytes,
method IDs, hash algorithms and the eleven-byte envelope are unchanged.

## Matched measurements and validation

The [production qualification](../evidence/one-pass-070/README.md) preserves
frozen sources, scripts, raw samples, identities and artifacts. All 1,904 matched
IC rows retain input/frame identity and three equal samples. Opt-3 1 MiB blob
decode falls 2,524,535 to 1,049,510 instructions (-58.43%); Candid is 1,065,023.
Encoding counters are unchanged and all successful decode counters improve in
the 87-case corpus. Truncated wide-u16 rejection regresses 16.28%.

Total/code Wasm drops 8,835/8,812 bytes for the hot spots and 25,467/25,432 bytes
for 100 ordinary records at opt-3 (6.00% less total record Wasm). The 100-record
probe measures size only. Native runs use two orders and eleven samples per row;
large-blob decode improves about 46%, with noisy host encoding and rejection
regressions recorded separately. No installed-canister size claim is made.

Runtime/native/borrowed tests, strict Clippy, default doctests and package file
listing pass. TypeScript qualification passes 157 vectors, 32,156 mutations,
20 native-projection vectors and 48 simulated-endian cases. No_std/default/schema
Wasm consumers and Rust 1.91 pass. ASan fuzzing passes 100,000 inputs with leak
detection disabled for sandbox constraints. Document links, format and diff
checks pass. No full 21-cohort rerun, macOS, real big-endian or installed-consumer
performance qualification is claimed.

## Retained direction and limitations

Use ByteBuf/Bytes/BytesRef for binary data and Packed<T> for supported homogeneous
integer/hash fields. Ordinary vectors remain available for compact small values
and general collections, with known large encoding gaps. This decoder change
does not provide implicit bulk encoding. Wire generates native host metadata;
Serde owns layouts with declaration order and no field IDs. Consumers coordinate
layout changes. See the [runtime guide](../runtime.md) and
[field guide](../packed-fields.md).

The [0.6.13 full matrix](../full-comparison-0613.md) remains historical evidence
for 21 cohorts. The [earlier one-pass study](../evidence/strict-decoding-0614/README.md)
is superseded by production qualification, not another shipping mode. Consumer
storage, endpoint IDs, installed acceptance and retries remain consumer-owned;
required external principals remain on Candid boundaries. Siblings are read-only.

## Tooling selection

Shared Tooling is pinned at `4c0f198e`, recorded version 0.26.10. IC Build/Host/
Testkit are 0.67.0 and Platform 0.6.14. Do not infer policy from mutable siblings
or patch snapshot-owned files. Ordinary work does not run releases.
