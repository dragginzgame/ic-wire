# Research findings and retention

This directory holds compact historical topic summaries, not raw run archives.
The retained figures describe their recorded source, suppliers and settings;
they are not current performance or compatibility guarantees.

Raw logs, bulk CSV/JSON tables, source snapshots, patches, Wasm/build outputs and
compressed bundles were retired from the working tree during the Shared 0.30.4
adoption. Routine dependency-adoption and patch-release reports were retired too.
Their hashes and summaries cannot independently verify missing artifacts. The
maintainer also requested a fresh repository history; old experiment commits,
branches and tags are not a retained archive or a reproduction fallback.

New experimental output belongs under `target/evidence/` and is removable by an
explicit Cargo clean. Routine check results belong in the owning changelog,
current handoff or CI result; they need no new report directory. Update an
existing topic summary for useful new findings instead of preserving every run.
The [Shared retention contract](../../rules/engineering-details.md#changelogs-and-artifact-preservation)
sets the normal ceiling: one summary and at most two small data files per topic,
100 KiB combined per topic, 5 MiB for repository experimental evidence. A durable
raw-data exception needs a named claim, minimal payload, measured size, owner,
destination and review/retention period.

## Current commands and measurements

- [Maintained qualification and benchmark commands](../development.md)
- [Latest recorded Candid comparison](../candid-comparison.md)
- [Codec and resource contract](../runtime.md)
- [Evaluation scope and limitations](../evaluation.md)
- [Current work and supplier selections](../status/current.md)

Use those maintained runners for fresh measurements. Keep native time, IC
instructions, whole uncompressed Wasm and code-section bytes separate. Earlier
source and input configurations may no longer have a maintained reproducer.

## Selected topics

- [Owned byte/text cleanup](codec-cleanup.md) and [bound sharing](bound-sharing.md)
- [Account batches](account-batch.md) and [Account/Platform cut](account-platform-cut.md)
- [Primitive bulk research](primitive-bulk.md) and [ordinary-list hotspots](hotspots.md)
- [Marked-buffer investigation](marked-buffer.md)
- [Endpoint extraction](endpoints.md) and [complete endpoint costs](endpoint-matrix.md)
- [Input borrowing](input-borrowing.md) and [borrowed endpoints](borrowed-endpoints.md)
- [Macro ownership](endpoint-macro-owner.md) and [optional metadata derive](optional-metadata-derive.md)
- [Stable identities](stable-identities.md), [contract artifacts](contract-artifact.md)
  and [contract fingerprints](contract-fingerprint.md)
- [Schema generation](schema-generation.md) and [Platform schemas](platform-type-schemas.md)
- [Tripwire comparison](tripwire-comparison.md) and [follow-up](tripwire-followup.md)

Other `.md` files retain selected historical findings with explicit raw-data
retirement notices. Superseded candidates are not production modes or promises.

## Release and recovery records

Small [release receipts](../../releases/receipts/README.md) retain their separate
release-provenance role. They are historical observations, not evidence that a
retired log or binary is still available. Git-side release journals, active
deployment/recovery state and current tool/build artifacts were not part of this
documentation cleanup. No new archive or remote copy was created for retired runs.
