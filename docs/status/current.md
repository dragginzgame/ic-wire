# Current handoff

## Released 0.7.17; pending 0.7.18 upstream adoption

Local HEAD `85eea63acddbf34f0ccbff5458788b36ca071a91` includes the authorized
merges of [#27](https://github.com/dragginzgame/ic-wire/pull/27) and
[#28](https://github.com/dragginzgame/ic-wire/pull/28). #27 was reconciled with the
already-adopted common workflow before merging; its obsolete local exception
was not restored. #28 adds the current root changelog's emoji Summary section.
The checkout was fast-forwarded without changing the incoming lockfile edits.

The current uncommitted adoption selects Shared Tooling 0.26.21 at
`66c91c4412856baa147482e648fc80428db8b179` through the canonical exporter.
IC Build/Testkit minimums and the managed editor select 0.68.8; Platform selects
0.6.19. The incoming lockfile already selected these versions, including its
OpenSSL probe update, and is preserved byte-for-byte. Own package versions remain
0.7.17; no release, tag or publication was performed. Leave adoption commits and
release delivery to the maintainer.

## Runtime and qualification

There is one production Serde/Postcard codec. This adoption changes no runtime
or generator source, protocol bytes, identities, admission rules or typed errors.
The production encoder retains two-pass admission/sizing; primitive decoding
remains one pass. Consumers own collection/custom-Serde normalization and must
retain original frames for exact retries and signatures.

The [runtime study](../evidence/runtime-0716/README.md) owns the released u16/i16
optimization's IC, native-time and Wasm tradeoffs, including small-input
regressions. It uses frozen Build/Testkit/host 0.68.5 and Platform 0.6.16. The
[encoder study](../evidence/encoder-followup-0716/README.md) uses 0.68.6/0.6.17;
those results do not qualify the newer dependency graph. Later temporary
experiments retained no runtime change. No new runtime speed or Wasm saving is
claimed for this tooling adoption.

Issues #20–#26 were closed after checking their delivered Wire implementations,
published source and complete release CI. Consumer rollout and installed endpoint
acceptance remain consumer-owned.

## Current validation and boundaries

Focused Linux checks pass: snapshot integrity, dependency pins/boundaries, actual
Make/hook routing, release-editor recovery, compiler diagnostics, benchmark corpus
roundtrips and strict Clippy for the IC Build and Testkit consumers. The 0.68.8
editor is installed and admitted. The existing PocketIC admission probe passes
144 valid/truncated and budget cases with three samples each, exercising owned
startup/cleanup. Local evidence: `target/tooling-adoption.O8cpIY/results`.

The PR merges were authorized with no enforced branch checks. #28's hosted CI was
still running at merge; no hosted pass is claimed for that run or this local
adoption. Full CI, the complete Candid performance matrix and fresh macOS
execution were not run locally. Use the maintainer's release gate for delivery.

Consumers choose byte buffers/Packed for bulk fields and bind schemas to deployed
identities. See the [runtime contract](../runtime.md) and
[field adoption guide](../packed-fields.md#adopting-an-existing-field).
