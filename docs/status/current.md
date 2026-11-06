# Current handoff

## Current work: frozen Core 0.17.5 delivery

The accepted comparison, advice audit and shared-code investigation are complete.
Core **0.17.5** is the pending compatible patch: Shared/native qualification
adoption, a benchmark reporter fix and meaningful regression coverage. Package
versions remain at delivered Core **0.17.4** / Endpoint **0.7.2** while preparing
the authorized Core release. The maintainer selected a frozen `core` batch from
this checkout's `main` to `origin/main`, tag `core-v0.17.5`, and crates.io.
Endpoint has no package changes requiring a separate release. Archive checks
and the full 20-stage gate must pass against the selected inputs before delivery.

Shared **0.30.8 / dc9df70** is adopted through the canonical exporter at the
verified pushed revision, with 143 payloads. It fixes resume admission for generated
batch IDs containing underscores without changing recovery paths. Frozen release batches remain
off by default. Make now removes only its own target export at the outer frozen
launcher and rejects invalid roster/order, group mismatch and caller target
overrides. `test-local-make.sh --case release-routing` passes both ordinary and
frozen dispatch checks. `test-local-release.sh core --frozen` exercises actual
two-group metadata/Git/publication hooks with gates/uploads substituted; it tests
deferred lock isolation and uncertain/completed publication recovery, and passes.
Both fixture dispatchers also reject deliberately contaminated candidate locks.
Core's real all-feature archive verification now passes. The earlier two-group
probe could not resolve unpublished Core from Endpoint offline; it is not
Endpoint archive qualification. This delivery selects only Core and must complete
its full gate in the managed checkout. Default-feature Core, schema-host
Clippy and Wasm compilation retain their distinct obligations; no validation
reuse or elapsed-time saving is claimed.
Snapshot/source verification, both-group release routing, existing CI-routing,
script syntax and local documentation references pass. No full CI or real
checkout/registry release effects ran; earlier work is preserved.

The [6 November full comparison](../candid-comparison.md) covers 28 cohorts,
1,575 overlapping case entries, 6,300 primary comparisons and 112 fresh Wasm
modules. Production representations win 5,360/5,924 comparisons; the private
research cohort wins 369/376. Wire is smaller in both total uncompressed Wasm
and code-section bytes in all 56 cohort/profile pairs. These are codec fixtures,
not a full Toko build or a traffic-weighted result.

Ordinary-list hotspots remain: encode reaches 121.39× / 49.67× Candid at
opt-z / opt-3. Generic Packed wins all 436 primary comparisons. The advice audit
finds existing categories cover the ≥10× families when supported metadata is
generated. A real-derived 15-family test checks default paths, quiet byte/Packed
alternatives, acknowledgements and bounded display. Codec-only users and custom
layouts remain outside automatic schema advice; no Serde representation changes.
Both changed native tests and formatting passed in the preceding codec batch.

The reporter now distinguishes numbered native-run directories from companion
`.log` files. The original matrix stopped at that reporting defect; it resumed
with retained samples and a recorded reporter-only repair. All runtime, fixture,
manifest and lock inputs stayed frozen. Six reporter tests and strict affected
Clippy pass; invalid required run directories still fail.

The isolated signed-i32 writer-reuse candidate passes focused codec and exact
byte/budget tests but grows ordinary-bulk Wasm by 3 bytes opt-z / 88 bytes opt-3
in both total/code metrics. Four Toko/small-control modules are bit-identical.
It is rejected before IC/native performance qualification; production source is
unchanged. The i32 boundary test is retained. Parser-tail sharing remains
rejected on prior size/instruction evidence; no new performance claim is made.

## Evidence and supplier identity

The full run uses Rust/Cargo **1.99.1**, LLVM **23.1.1**, PocketIC **16.1.0**,
Candid **0.10.40**, Postcard **1.1.3**, Platform facade **0.23.2**, Types
**0.18.2**, Runtime/macros/guard **0.19.2**, Testkit **0.89.0** and Build **0.75.0**.
The report records the complete lock hash and profile recipes. All 33,480 IC
operation rows have three equal fresh-caller samples; native/IC input and frame
identities, guest freshness and physical Wasm/code sizes verify. Native time,
allocation and heap probes remain separate; the host was not CPU-isolated.

Raw output and frozen sources are under ignored
`target/evidence/codec-matrix-20261106-current/` and
`target/evidence/signed32-20261106/`. The latter's `size-screen-qualified/`
contains the completed six-module screen.
No raw experiment archive is added to Git. Historical retired artifacts cannot
be verified from retained summaries alone.

Concurrent lock updates are preserved: the checkout now selects Platform facade
**0.23.4**, Runtime/macros/guard **0.19.3** and Testkit **0.89.1**. These are not
the matrix's measured graph. Do not describe this refresh as qualification of
those updates or as a matched regression test against earlier released matrices.
The authorized release gate must qualify these supplier boundaries. Later
normal-checkout updates belong to the next batch once this source is frozen.

Testkit 0.89.0 removes implicit logging setup; Wire has no removed-API callers and
adds no logging dependency. Its 0.89.0 adoption passed strict affected-driver
Clippy, installed Endpoint admission/replies/rollback, native-dev/Wasm isolation,
boundaries, pins and links; logs remain in `target/testkit089/`. This does not
qualify the later concurrent patches. The existing flate2 1.1.11 update is also
preserved. Removing native logging dependencies is not a production Wasm saving.

## Delivered boundary and retained contracts

Core 0.17.4 and Endpoint 0.7.2 were delivered to origin/main and crates.io;
annotated tags resolve to **fd31693** and **b6a462c**. Both full 20-stage gates
passed and registry readback confirmed all four packages. Small
[delivery receipts](../../releases/receipts/README.md) own publication provenance.
No full repository CI, macOS or consumer application build was run for this batch.

Rust **1.99.1** remains both minimum and pinned compiler; only the opt-in sanitizer fuzzer uses
the approved dated nightly. CI/release routing remains one roster: 20 full
stages, 11 for qualified Endpoint-only Linux changes; stage reuse/final-candidate
execution stay disabled. Production Core is SDK-free; optional schemas select
Types, Endpoint alone selects Runtime execution, Candid/Testkit remain test-only.

Bytes, schemas, identities, limits, primitive acceptance and rollback are
unchanged: `[128_u16]` remains `01 80 01`; `64_i32` remains `80 01`. No migration,
legacy reader, new codec mode or per-type codec variant is introduced. Account's
[0.16 cut](../../releases/core/0.16.md) remains `01 20 + 32 bytes` for present
subaccounts and `00` for absent ones. Consumers own normalization and retain
original retry/hash bytes. Method IDs/name defaults remain production behavior;
fingerprints and no-manual-ID macros remain research.

The maintainer replaced repository history with one clean root. Fresh-clone
rather than merging discarded refs; registry archives and historical receipt
provenance remain unchanged. See [focused development commands](../development.md)
and the current report for reproduction and measurement limits.
