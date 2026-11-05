# Current handoff

## Repository history boundary

The maintainer requested a quick replacement with one clean main history root,
purging old branches/tags and obsolete Git-side release state. The current source
includes the accepted adoption/cleanup work below. Existing consumers should
fresh-clone rather than merge the discarded history. Registry versions/archives
are unchanged; old commit IDs in release receipts record original publication,
not refs to recreate. No package publication or version bump is part of this reset.

## Pending Core 0.17.3

Shared **0.30.4 / 2aa41c7** is exported canonically: **139** payloads, independently
verified against the committed source. The new Candid source check is part of
`check-boundary`: all **113** crate/example Rust files, with **34** exact named
comparison declarations in ci/candid-boundary.json. It is lexical, not macro
expansion/type resolution. Owned application and unreviewed benchmark DTO probes
refuse; no whole-file allowances. Private sparse/wide reference structs have
unambiguous names; blob type projections use the imported trait normally.

[Research retention](../evidence/README.md) now follows Shared's compact-result
rules. The former **266,099,732-byte / 1,997-file** collection is replaced by
**57** flat Markdown files (about **305 KB**); no bulk tables, snapshots, patches,
logs, Wasm or compressed bundles remain in it. All **131** small release records
moved unchanged to [release receipts](../../releases/receipts/README.md).
The 343 old Git-side records were unchanged during that documentation cleanup;
the separately requested history replacement retires their obsolete state.
Raw retirement is explicit; summaries/hashes do not verify missing artifacts.

Benchmark output defaults now use `target/evidence/`. One documentation checker
covers maintained Markdown plus the Shared selection without duplicate lists.
Four affected benchmark corpora pass **27 tests**, strict default benchmark
Clippy and Wasm compile checks. Snapshot, source/dependency boundaries, formatting,
175-document link checks, actual CI/Make routing and broken-link refusal pass.
No full CI, native macOS, installed-server or new performance matrix was run.
Protocol bytes, fields, schemas, identities, limits, budgets and rollback are
unchanged. No performance or Wasm-saving claim. Package versions remain Core
**0.17.2**, Endpoint **0.7.1**; notes are pending **0.17.3**. The source is included
in the history replacement; no new registry release was performed.

## Platform and compiler selections

The pending benchmark selection is published Platform facade **0.23.0**;
Testkit **0.86.0** retains native facade **0.22.3**. Types **0.18.1** and
Runtime/macros/exact guard **0.19.2** are unchanged. Prior focused facade
qualification passed six corpus tests, strict all-target lint, guest Wasm,
production graphs, pins and formatting; its routine raw report was retired.
Existing Build/Host **0.74.4**, Candid/derive **0.10.40**, mime **0.3.18** and
winnow **1.0.5** updates were preserved; wider Host adoption is outside those
focused checks. The selected release metadata executable remains **0.74.0**.

Rust **1.99.1** remains minimum and pinned. The approved fuzz-only exception is
`nightly-2026-11-05` (Rust **1.101.0-nightly**), previously qualified with 112 seeds
and 10,000 ASAN runs, leak checking disabled. Only scripts/ci/fuzz-runtime.sh
owns that special pin. No old compiler lane or new runtime supplier patch.

Core stays independent of Runtime/SDKs. Optional schemas select canonical Types;
Endpoint owns execution integration with one Runtime guard. Candid/facades and
native qualification tools remain outside production. Testkit's optional Wire
integration is off. The roster retains **20** full stages and **11** eligible
Endpoint-only Linux stages; shared/dependency/uncertain inputs broaden to full.
Stage reuse and final-candidate execution remain disabled.

## Delivered and recovery state

Core/derive **0.17.2**, Endpoint/macros **0.7.1** are already published:
**46d7d08 / core-v0.17.2**, **da5ed1e / endpoint-v0.7.1**. Their
[compact record](../../releases/receipts/published-core0172-endpoint071/delivery.json)
retains registry/tag/source identities. The two fresh 20-stage gates and
registry-only/installed consumers passed at delivery. Bulk proof bundles were
retired; those observations are not a current independent reproduction claim.
No package upload is outstanding for these immutable versions.

The earlier authorized removal of 255 legacy validation logs preserved their
plans/receipts. Its optional 7,162,157-byte backup remains under
target/deploy-0172-0071/legacy-validation-0711-0720.tar.gz, subject to Cargo clean.
Failed/cancelled hosted results are not relabelled as passes. Old history-scan
fingerprint allowances are retired with their discarded commits.

## Performance and protocol cautions

The [latest recorded matrix](../candid-comparison.md) uses an older supplier
cohort; figures are historical, not current dependency regression deltas.
Ordinary integer/float/bool and late-truncation gaps remain. Packed never
silently rewrites Serde fields; private float/bool bulk fields remain research.
The [byte](../evidence/codec-cleanup.md) and [list](../evidence/bound-sharing.md)
studies retain empty-byte opt-z +10.1% and one-u64 opt-3 +15.3% instruction costs.
No full Toko or paired-cost gate pass is claimed. Keep heavy mechanics shared;
avoid per-type codec variants and projected savings from stress fixtures.

Production retains explicit method IDs and name defaults; fingerprints and
no-manual-ID macros remain research. The [0.16 Account cut](../../releases/core/0.16.md)
changed present subaccounts `01 + 32 bytes` to `01 20 + 32 bytes`; None stays `00`.
Consumers own retained-data adoption. There is one codec, no legacy reader or
automatic migration; retained retry bytes and encoder hash inputs stay exact.
