# Current handoff

## Core 0.17.14 delivery in progress

Keep the checked slice-copy path for existing Packed signed bytes and 16-/32-byte
hashes on little-endian targets. Wider words and big-endian readers stay intact.
Matched opt-z IC decode drops **112,047 → 67,100** for 4,096 IDs and
**394,599.5 → 17,879.5** for 16,384 signed bytes. Whole-fixture total/code Wasm
shrinks **30/19 bytes** (ID) and **70/28 bytes** (bulk) at opt-z/3. Singleton
opt-z decode can add up to **104 instructions**. All **1,032 operation pairs**
pass the cost policy; allocation counts/heap stay unchanged. Focused Packed
byte/admission/endian tests and Clippy pass. The [matched study](../evidence/primitive-bulk.md#byte-aligned-packed-copy-8-november)
records native time, regressions and reproduction under
`target/evidence/packed-copy-20261108/`. Testkit **0.93.1** is fixed on both sides;
its previously deferred lock update is included. No new full matrix or application
performance claim. Bytes, identities, limits and acceptance remain unchanged:
one packed ID is still `10 + 16 bytes`; no consumer action is required.
The maintainer authorized Core/derive delivery to origin/main and crates.io.
The full 20-stage frozen gate precedes version preparation/publication; Endpoint
stays **0.8.1**. Source preparation and delivery are in progress.

## Delivered: Endpoint 0.8.1

Shared **0.31.9 / d95f271** was exported from its committed revision. It scopes
formatting to workspace members, adds the shared hashing policy and replaces
the old LOC selector with `CLOC_DEPTH` / `cloc.sh -d`. It does not change the
downloader implicated in the recent hosted HTTP 504 bootstrap failures.

That release selected Build **0.79.0** for compiler diagnostics and the release
metadata CLI, Platform **0.26.0** for private benchmarks, and Runtime/macros/guard
**0.20.1** for Endpoint. The Runtime lock update was already present on entry.
Types **0.18.2**, Testkit **0.93.0** and Host **0.75.0** were unchanged. Testkit
retains native-only Platform **0.25.1** alongside the benchmark facade; execution
resolves one Runtime/guard. No removed Build batch or Platform hashing API is
used. Supplier preparation, offline release-editor admission, snapshot integrity,
formatting, hashing/dependency boundaries and both LOC depth modes pass. Build
passes 8 Core / 13 Endpoint diagnostic cases. Mixed-version host/Wasm consumers,
Endpoint fixture lint, installed typed calls/limits/transactions on PocketIC
16.1.0, and the native/Wasm Candid benchmark corpus pass. Schema-only Core stays
Runtime/Candid-free. Evidence is under `target/adoption-20261108/`.
Unrelated Windows dependency edges were preserved and final locked metadata
passes. No sibling checkout was modified. No new performance comparison or
native macOS qualification is claimed.

The maintainer authorized delivery. Only Endpoint **0.8.1** needs publication for
its changed Runtime requirement; Core stays **0.17.13**. Core archive comparison
finds identical product files, with only Git provenance and the corresponding
locally packaged derive checksum differing. Evidence is under
`target/endpoint081-delivery/`; this comparison is not a compiled archive check.
All 20 Linux gate stages passed. Endpoint/macros **0.8.1** are on crates.io and
origin/main; tag **endpoint-v0.8.1** selects **5bb495a**, from source **e0f42ca**
in frozen batch **batch-sLAg4biN**. Final all-feature archives passed and match
independent registry downloads, checksums and release provenance. The
[delivery receipt](../../releases/receipts/published-endpoint081/delivery.json)
owns exact identities. The normal checkout is reconciled. A concurrent Testkit
**0.93.1** lock update is preserved at **c134348** on
`deferred/endpoint081-testkit0931` and is now included in the Core candidate;
Endpoint's release qualified **0.93.0**. Hosted CI remains asynchronous.
Wire bytes, schemas, identities, limits, acceptance and rollback are unchanged.
Prior performance measurements below retain their original supplier identities.

## Delivered: Core 0.17.13

Core/derive **0.17.13** is on origin/main and crates.io. Tag **core-v0.17.13**
selects **c719ece**, from source **888be9b** in frozen batch **batch-Rkv2gVBa**.
All 20 Linux gate stages passed, as did final all-feature package verification.
Independent registry downloads match uploaded and all-feature archive checksums
and Git provenance. That delivery left Endpoint at **0.8.0**.
The [delivery receipt](../../releases/receipts/published-core01713/delivery.json)
owns exact identities and verification. Evidence: `target/core01713-delivery/`,
`target/core01713-deploy-3.log` and the retained frozen checkout. Hosted CI is
asynchronous; no native macOS or fresh installed application proof is claimed.
Failed Core batches **batch-mzOGyRZA** (source **89e1240**) and
**batch-Oup1NG9b** (source **e83ad0b**) had no delivery effects and are superseded;
retain their state without resuming. The delivery receipt records their repaired
Candid declaration and compatibility-fixture failures. Current/previous-Core/
published-Endpoint lanes pass; evidence: `target/compatible-consumers/run.MGmXha/`.

Public `Packed<[u8; 16]>` reuses the existing bulk codec, with `hash16` metadata,
flat `Uint8Array` clients, partial-ID rejection and default/suppressible list
advice. No arbitrary-width generic family, second codec or dependency was added.
Adopting one ID changes `01 + 16 bytes` to `10 + 16 bytes`; eight IDs change
`08 + 128 bytes` to `80 01 + 128 bytes`. Coordinate identities and clients,
retain original retry/hash bytes, and requalify buffer budgets and field bounds.
Existing field bytes, identities, limits and acceptance remain unchanged.

The [flat-Candid study](../evidence/primitive-bulk.md#public-16-byte-ids-and-flat-candid-8-november)
corrected the prior nested-vector interpretation; retain its ordinary controls and
borrowed-input copy costs. Its two opt-z large-ID decode losses are addressed by
the pending slice-copy change above. Original evidence remains under
`target/evidence/id16-flat-20261108/` and `target/id16-work/`. Fixture savings
are not application savings; native big-endian/application proofs remain absent.

## Suppliers and prior delivery

Rust/Cargo **1.99.1** remains floor and pin. The delivered release used Shared
**0.31.5 / b2026a1**, Build **0.78.0**, Host **0.75.0**, Platform **0.25.1**,
Types **0.18.2**, Runtime/macros/guard **0.20.0**, Testkit **0.93.0**. The Platform
lock selection predated the ID implementation and remained fixed throughout it.
Testkit's changed CBOR error variant has no Wire matches. Native qualification
uses prepared PocketIC **16.1.0**; production remains Testkit/Candid-free and
schema-only Core remains execution-free. No supplier patches or wrappers.

The prior Core **0.17.12** and Endpoint **0.8.0** tags select **6082752** and
**a967a9c**, from batches **batch-FcrckPoz** and **batch-vZAxlP49**. Both passed
full 20-stage Linux gates, final archives and registry readback; installed
Endpoint typed calls/admission/rollback passed on PocketIC 16.1.0. The
[receipt](../../releases/receipts/published-core01712-endpoint080/delivery.json)
owns exact identities/checksums. Git-only maintenance **72ab905** removed unused
identity hashing and clarified private-only release guidance; #49/#50 are closed.

Endpoint 0.8 requires aligned Runtime/Timers execution suppliers and one exact
0.20 guard. Wire already uses ordinary `cdk` macros. This supplier adoption
preserves frames, identities, budgets, exports and rollback; no regeneration.

## Research and protocol boundaries

Rejected bool outlining/chunk scans grow total/code Wasm at both profiles.
Outer-list visitor sharing saves only 0/18 standard-corpus Wasm bytes at opt-z/3;
1,593/1,594 IC rows are identical. Stress savings do not justify production
changes. See [bool/ID studies](../evidence/primitive-bulk.md) and
[bound-sharing](../evidence/bound-sharing.md). Do not repeat those candidates.
Their 7 November raw targets were found removed on 8 November; retained reports
are historical evidence, not claims that those artifacts remain available.

The [full comparison](../candid-comparison.md) remains the earlier 36-cohort,
Core 0.17.6/Testkit 0.89.4 study (2,105 cases, 8,420 primary comparisons).
The maintained roster now has 38 cohorts; this release does not claim a new full
matrix. Bool/nonzero/heap adoption contracts remain in the packaged README and
runtime docs. Account's [0.16 cut](../../releases/core/0.16.md) remains
`01 20 + 32 bytes` for present subaccounts, `00` for absent ones. Method IDs/name
defaults remain production behavior; fingerprint macros are research. Keep one
Serde/Postcard codec, no silent rewrites, and original retry/hash bytes because
collection/custom-Serde normalization remains consumer-owned. Metadata advice
is not universal coverage for codec-only/custom layouts.

## Evidence and recovery custody

Preserve current evidence, prepared server, journals and managed release state;
no cleanup is authorized. Earlier target paths may have been removed by external
cleanup; journal/receipt existence does not prove checkout availability. Prior
receipts remain under [releases/receipts](../../releases/receipts/README.md).
Failed **batch-1afAufRB** had no delivery effects and is superseded: do not resume.
Completed **batch-1i9z2IIG** was reconciled but its checkout was removed; deferred
lock **7fd813f** on `deferred/core0176-lock` was reapplied. The schema-1 0.17.5 batch
is historical and must never be resumed with schema 2. Repository history was
intentionally replaced: fresh-clone rather than merge obsolete refs; never
re-upload published versions to repair history. Future delivery uses explicit
groups, Core's full 20 stages and qualified Endpoint-only Linux's 11. Stage reuse
and final-candidate opt-ins remain disabled. Native macOS is not proved by Linux.
