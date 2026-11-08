# Current handoff

## Delivered: Core 0.17.14

Core/derive **0.17.14** is on origin/main and crates.io. Tag **core-v0.17.14**
selects **c0bfb8b**, from source **5c9bc81** in frozen batch **batch-InHQ5_ZW**.
All **20 Linux gate stages** passed, including benchmark correctness, standalone
consumers, Endpoint compatibility and Rust/TypeScript conformance. Final
all-feature packages passed; independent registry downloads match uploaded and
all-feature checksums and release provenance. The normal checkout is reconciled.
The [delivery receipt](../../releases/receipts/published-core01714/delivery.json)
owns exact identities. Evidence: `target/core01714-delivery/` and the frozen
checkout. Hosted CI is asynchronous; native macOS is not claimed.

The existing Packed visitor now uses a checked slice copy for signed bytes and
16-/32-byte hashes on little-endian targets. Wider words and big-endian readers
stay intact. Matched opt-z IC decode drops **112,047 → 67,100** for 4,096 IDs and
**394,599.5 → 17,879.5** for 16,384 signed bytes. Whole-fixture total/code Wasm
shrinks **30/19 bytes** (ID) and **70/28 bytes** (bulk) at opt-z/3. Singleton
opt-z decode can add up to **104 instructions**. All **1,032 operation pairs**
pass the cost policy; allocation counts and heap stay unchanged. Focused Packed
byte/admission/endian tests and Clippy pass. The
[matched study](../evidence/primitive-bulk.md#byte-aligned-packed-copy-8-november)
records native time, regressions and reproduction under
`target/evidence/packed-copy-20261108/`. These are fixture measurements, not
application savings or a fresh full matrix. Bytes, schemas, identities, limits
and acceptance remain unchanged: one packed ID is still `10 + 16 bytes`, and
eight remain `80 01 + 128 bytes`. No consumer migration or regeneration is needed.

## Suppliers and Endpoint

Rust/Cargo **1.99.1** remains floor and pin. This release selects Shared
**0.31.9 / d95f271**, Build **0.79.0**, Host **0.75.0**, Platform **0.26.0**,
Types **0.18.2**, Runtime/macros/guard **0.20.1** and Testkit **0.93.1**.
Testkit retains native-only Platform **0.25.1**; execution selects one guard.
The previously deferred Testkit selection at **c134348** on
`deferred/endpoint081-testkit0931` is now included and qualified. No supplier
patches, wrappers or sibling checkout edits. Production stays Candid/Testkit-free;
schema-only Core stays Runtime/CDK-free. PocketIC **16.1.0** was used for the
matched instruction study. No installed application performance proof is claimed.

Endpoint/macros remain at **0.8.1**. Tag **endpoint-v0.8.1** selects **5bb495a**,
from source **e0f42ca** in batch **batch-sLAg4biN**. Its full 20-stage Linux gate,
all-feature archives and independent registry readback passed; see its
[receipt](../../releases/receipts/published-endpoint081/delivery.json).
That release used Testkit **0.93.0**. Its preceding installed typed-call, limit
and transaction checks used PocketIC 16.1.0, not newly installed registry packages.
Endpoint 0.8 aligns Runtime/Timers with one exact 0.20 guard and ordinary `cdk`
macros; frames, identities, budgets, exports and rollback remain unchanged.

Shared 0.31.9 scopes formatting to workspace members, adds the hashing policy
and replaces the LOC selector with `CLOC_DEPTH` / `cloc.sh -d`. It does not change
the downloader involved in the earlier hosted HTTP 504 bootstrap failures.
Adoption evidence remains under `target/adoption-20261108/`.

## Prior qualification and contract boundaries

Core **0.17.13**, tag **core-v0.17.13** at **c719ece**, retains its
[receipt](../../releases/receipts/published-core01713/delivery.json) and
`target/core01713-delivery/` evidence. Its public ID field deliberately changes
ordinary `01 + 16 bytes` to packed `10 + 16 bytes`; eight IDs change
`08 + 128 bytes` to `80 01 + 128 bytes`. New field adoption requires coordinated
identities/clients, original retained retry/hash bytes and reviewed budgets.
The [flat-Candid study](../evidence/primitive-bulk.md#public-16-byte-ids-and-flat-candid-8-november)
corrected the earlier nested-vector comparison; retain its ordinary controls and
borrowed-input copy costs. The latest copy optimization addresses its two large-ID
opt-z losses. Original evidence: `target/evidence/id16-flat-20261108/` and
`target/id16-work/`. Actual native big-endian execution remains unqualified.

Rejected bool outlining/chunk scans grew Wasm. Outer-list visitor sharing saved
only 0/18 standard-corpus bytes at opt-z/3 with effectively unchanged instructions.
See [primitive bulk](../evidence/primitive-bulk.md) and
[bound sharing](../evidence/bound-sharing.md); do not repeat those candidates.
The earlier 7 November raw targets were found removed; summaries do not prove
those artifacts remain available. The [full comparison](../candid-comparison.md)
is still the 36-cohort Core 0.17.6/Testkit 0.89.4 study: 2,105 cases and 8,420
primary comparisons. The maintained roster has 38 cohorts; no fresh full run.

Keep one Serde/Postcard codec and no silent field rewrites. Collection/custom
Serde normalization remains consumer-owned, so preserve original retry/hash bytes.
Metadata advice does not cover every codec-only/custom layout. Account's
[0.16 cut](../../releases/core/0.16.md) remains `01 20 + 32 bytes` for present
subaccounts and `00` for absent ones. Method IDs/name defaults remain production
behavior; fingerprint macros remain research.

## Evidence and recovery custody

Preserve current evidence, prepared server, journals and managed release state;
no cleanup is authorized. Older paths may have been removed externally; receipts
alone do not prove checkout availability. Prior records live under
[releases/receipts](../../releases/receipts/README.md), including Core 0.17.12 /
Endpoint 0.8.0. Failed Core batches **batch-mzOGyRZA** (source **89e1240**) and
**batch-Oup1NG9b** (source **e83ad0b**) had no delivery effects and are superseded;
retain their state without resuming. Core 0.17.13's receipt records their fixes.
Failed **batch-1afAufRB** is also superseded. Completed **batch-1i9z2IIG** was
reconciled but its checkout was removed; deferred lock **7fd813f** on
`deferred/core0176-lock` was reapplied. Never resume the schema-1 0.17.5 batch
with schema 2. History was intentionally replaced: fresh-clone rather than merge
obsolete refs, and never re-upload published versions to repair Git history.
Future delivery uses explicit groups, Core's 20 stages and qualified Endpoint-only
Linux's 11. Stage reuse/final-candidate opt-ins remain disabled.
