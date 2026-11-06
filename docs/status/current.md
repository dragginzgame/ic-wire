# Current handoff

## Delivered: Core 0.17.5 through a frozen batch

Core **0.17.5** is delivered to origin/main and crates.io: **ic-wire** and
**ic-wire-derive**, tag **core-v0.17.5**, release commit **c789f96**. The reviewed
source was **4b064a6**. Endpoint/macros remain **0.7.2**; no separate Endpoint
release was needed. The normal main branch has been fast-forwarded to delivery.
The [delivery receipt](../../releases/receipts/published-core0175/delivery.json)
records package hashes, source identity, suppliers and qualification boundaries.

The maintainer explicitly selected frozen delivery. Shared **0.30.8 / dc9df70**
captured the complete affected roster, **core**, once in **batch-YDNNU8bS**.
The full 20-stage Linux gate passed in the managed checkout, together with real
all-feature archive verification and installed Endpoint admission/rollback on
PocketIC **16.1.0**. Publication verified both exact-version archives. Fresh Cargo
registry readback and downloaded archive hashes agree with the upload artifacts;
both archives identify the release commit. Completed-batch resume did not upload
again. No new performance study or native macOS qualification is implied.

Recovery state is retained under `.git/release-state/frozen/batch-YDNNU8bS/`;
the managed checkout and raw logs remain under `target/release-frozen/` and
`target/core0175-*`. The complete phase and delivered commit are recorded in
`intent.json`. Do not replay publication or replace retained state. Normal-checkout
reconciliation preserved the concurrent dependency edit described below.

## Deferred inputs and qualified graph

Delivery used Platform facade **0.23.4**, Types **0.18.2**, Runtime/macros/guard
**0.19.3**, Testkit **0.89.1**, Build **0.75.0**, Host **0.74.4** and Rust/Cargo
**1.99.1**. Its source lock hash and release lock hash are in the receipt; version
preparation changed only the five Core-group package versions in Cargo.lock.

A concurrent process advanced the normal checkout to Runtime/macros/guard
**0.19.4** during validation. The frozen source retained **0.19.3**. The 0.19.4
edit was preserved through a temporary Git stash, reapplied after fast-forward,
and checked byte-for-byte apart from the five expected Core version bumps.
Only that deferred Cargo.lock edit remains uncommitted. It is not qualified by
this delivery; inspect and qualify it with the next accepted batch. A dependency
patch alone does not require another package release.

## Current findings and retained contracts

The [6 November comparison](../candid-comparison.md) covers **28 cohorts**, **1,575**
overlapping case entries, **6,300** primary comparisons and **112** fresh Wasm
modules. Production representations win **5,360/5,924** comparisons; the private
research cohort wins **369/376**. Wire is smaller in total uncompressed Wasm and
code-section bytes in all **56** cohort/profile pairs. These are codec fixtures,
not a full Toko build or a traffic-weighted result.

That matrix used Platform **0.23.2**, Runtime **0.19.2**, Testkit **0.89.0** and its
own recorded lock. Do not attribute performance changes to the later release
graph. Raw measurements and frozen experiment sources remain under ignored
`target/evidence/codec-matrix-20261106-current/`; no bulk archive was added to Git.

Ordinary-list encode hotspots remain up to **121.39× / 49.67×** Candid at opt-z /
opt-3. Generic Packed wins all **436** primary comparisons. Default schema advice
covers the measured ≥10× families when supported metadata is generated; a real
15-family derive test checks paths, quiet byte/Packed alternatives,
acknowledgements and bounded display. Codec-only/custom layouts remain outside
that advice. Serde representations are never silently rewritten.

The benchmark reporter now ignores numbered native-run companion log files while
still requiring valid run directories. Its six tests and affected lint passed;
the matrix's reporter-only recovery is documented. Signed-i32 writer sharing was
rejected: it added 3/88 uncompressed Wasm bytes at opt-z/opt-3 in the bulk fixture.
Only its byte/budget boundary test was retained; no production optimization or
per-type codec variant was added. Parser-tail sharing remains rejected.

Bytes, schemas, identities, limits and acceptance are unchanged: `[128_u16]`
remains `01 80 01`; `64_i32` remains `80 01`. Account's [0.16 cut](../../releases/core/0.16.md)
remains `01 20 + 32 bytes` for present subaccounts and `00` for absent ones.
Consumers own collection/custom-Serde normalization and retain original retry
and hash bytes. Method IDs/name defaults remain production behavior; fingerprints
and no-manual-ID macros remain research. There is one production Serde/Postcard
codec; Candid and Testkit stay outside published packages.

## Workflow boundaries

Use the [documented explicit frozen Core command](../development.md) for future
authorized delivery. Core retains all 20 stages; qualified Endpoint-only Linux
changes use 11. Stage reuse/final-candidate execution remain disabled. A future
combined Core/Endpoint batch still needs complete supplier-first archive
qualification; its disposable hook fixture alone is not installed qualification.
Shared's underscore-ID recovery fix and Wire's target/roster admission are adopted.
Rust **1.99.1** remains both floor and pin; the separate nightly is fuzz-only.

The maintainer replaced older repository history with a clean root. Fresh-clone
rather than merging discarded refs; historical registry archives and receipt
identities are unchanged. Never re-upload an existing version to repair history.
