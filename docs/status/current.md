# Current handoff

## Core 0.17.13 deployment in progress

The maintainer authorized Git and crates.io delivery of Core 0.17.13 from main.
Endpoint stays 0.8.0. Source is ready for the frozen Core workflow; the full
20-stage gate, version preparation, tag/push and publication remain pending.
No delivery is claimed by this handoff. Root manifests/lock own package versions.

Public `Packed<[u8; 16]>` reuses the existing bulk codec, with `hash16` metadata,
flat `Uint8Array` clients, partial-ID rejection and default/suppressible list
advice. No arbitrary-width generic family, second codec or dependency was added.
Adopting one ID changes `01 + 16 bytes` to `10 + 16 bytes`; eight IDs change
`08 + 128 bytes` to `80 01 + 128 bytes`. Coordinate identities and clients,
retain original retry/hash bytes, and requalify buffer budgets and field bounds.
Existing field bytes, identities, limits and acceptance remain unchanged.

The [fresh flat-Candid comparison](../evidence/primitive-bulk.md#public-16-byte-ids-and-flat-candid-8-november)
corrects the earlier nested-vector interpretation. For 4,096 owned IDs, opt-3
Wire encode/decode is **66,637 / 66,620**, versus flat Candid **149,171 / 85,862**
instructions. Wire wins 78/80 primary comparisons; both losses are large-list
opt-z decode, **112,047 vs 96,673** for the owned case (15.9% more). All nonempty
cases improve over ordinary Wire arrays; empty regressions are 23–186 instructions.
Borrowed-input encoding includes a copy: 132,725 opt-3 instructions and 64 KiB
extra heap. This is not a universal 129× advantage.

Whole ID-fixture uncompressed Wasm saves **1,484 / 448 total bytes at opt-z/3**.
The existing 109-case bulk fixture is byte-identical before/after when the new
type is unused. These are fixture measurements, not application savings. All
800 IC rows have three identical samples. Candid frames/counters/sizes match
across cohorts; feature-build module hashes differ. Focused Rust/admission,
178 browser vectors, 39,448 mutations, 66 simulated endian cases, standalone
no_std host/Wasm, Clippy, advice and dependency-boundary checks pass. Evidence:
`target/evidence/id16-flat-20261108/`, `target/id16-work/`. Actual native big-endian
and installed application qualification are not claimed.

## Suppliers and prior delivery

Rust/Cargo **1.99.1** remains floor and pin. Shared **0.31.5 / b2026a1** owns the
snapshot. Current selections: Build **0.78.0**, Host **0.75.0**, Platform **0.25.1**,
Types **0.18.2**, Runtime/macros/guard **0.20.0**, Testkit **0.93.0**. The Platform
lock selection predated the ID implementation and remained fixed throughout it.
Testkit's changed CBOR error variant has no Wire matches. Native qualification
uses prepared PocketIC **16.1.0**; production remains Testkit/Candid-free and
schema-only Core remains execution-free. No supplier patches or wrappers.

Core **0.17.12** and Endpoint **0.8.0** are published. Tags select **6082752** and
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
