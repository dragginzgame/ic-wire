# Current handoff

Core 0.19.0 (7132e08) and Endpoint 0.10.0 (01c9890) were delivered to
origin/main and crates.io on 2026-11-09. Their full release gates passed;
batch-Hq3YAiQU is complete and reconciled. Current optimization work is
committed/unpublished. Delivery is now requested for Core 0.20.0 followed by
Endpoint 0.11.0, using the complete gates and the supplier-first frozen batch.
All four public packages are selected; private qualification stays unpublished.

## Upstream decoder hard cut — focused qualification complete

The pending Core release is 0.20.0: upstream Postcard replaces the private parser.
`80 00` now accepts integer zero; `02 61 62` accepts Rust char `'a'` instead of
Malformed. Browser varints/Result tags follow the same rule; collection lengths
use the Wasm32 five-group width (64-bit Rust hosts admit ten). Rust char has no browser schema.
Encoder bytes, identities and budget rules remain unchanged. Consumers regenerate
clients, review rejection assumptions and retain original retry/hash bytes.
No storage rewrite or fallback reader is introduced. The [paired study](../benchmarks/candid-comparison.md#upstream-postcard-hard-cut--10-november-2026)
saves 287/247 uncompressed opt-z/opt-3 Wasm bytes, with explicit decode/rejection
regressions: worst +11.1% on a tiny u16 and +9.2% on a large byte-array batch at
opt-z. Nineteen opt-z rows exceed the instruction screen; none do at opt-3.
The automated checker stops on provenance identity differences; independent
verification checks all frame/input, allocation, source and counter evidence.
Core/Clippy/browser/Platform/corpus checks and 10,000 sanitizer fuzz inputs pass.
Full release CI and the whole 39-cohort matrix have not run for this cut.
Prior parser measurements below are historical for this decoder change.

## Optimization and matrix refresh — 10 November

Four qualified runtime changes are retained. No new codec, API, bytes, identity,
limits or acceptance change; no consumer migration is required:

- [Finite f32 scanning](../evidence/primitive-bulk.md#paired-f32-validation--10-november-2026):
  large opt-z encode/decode saves 32.7%/11.6% instructions for five extra Wasm
  bytes. The strict no-growth screen fails; this tradeoff is recorded. f64's
  follow-up was rejected and restored. Singleton f32 remains `04 00 00 80 3f`.
- [Unit enum tags](../evidence/encoder-followup.md#unit-enum-tags--10-november-2026):
  saves 9.0% large opt-z enum encode and 13/11 opt-z/opt-3 Wasm bytes.
  Tag 128 remains `80 01`; opt-3 counters are unchanged.
- [Consolidated depth admission](../evidence/encoder-followup.md#consolidated-encoding-depth-admission--10-november-2026):
  saves 796/16 opt-z/opt-3 Wasm bytes; small instruction regressions stay below
  0.6%. Unit-tag and depth studies pass the strict paired cost screen.
- [Newtype enum tags](../evidence/encoder-followup.md#newtype-enum-tags--10-november-2026):
  saves 326 opt-z Wasm bytes and 16.6%/9.2% large result-map/mixed-enum encode
  instructions. Explicit tradeoff: page encode adds 1.9%/5.4% at opt-z/opt-3;
  opt-3 adds 232 Wasm bytes and a tiny result adds 131 instructions (20.8%).
  The strict screen fails four opt-3 instruction rows and both size checks.
  Tag 128 with u8 body 7 remains `80 01 07`; decode/rejection are unchanged.

The latest matched 154-case candidate is
`target/evidence/newtype-tags-20261110/candidate/`, against the prior depth
candidate. Fresh guests/source identities, 3,202 IC rows, 10,280 native/frame
checks and 5,140 unchanged allocation/heap observations verify. Candid artifacts
and counters are identical. All 82 active native tests and Clippy pass; newtype
tag boundary/admission tests pass on both original and candidate implementations.
Prior Core doctests and follow-up configurations remain qualified. Native times
are noisy shared-host evidence; Wasm measurements are uncompressed fixture
modules, not projected application savings. Opt-z is primary; retain opt-3 losses.

[Preflight attribution](../evidence/encoder-followup.md#aggregate-preflight-attribution--10-november-2026)
measured separate calls, not an additive phase decomposition. The first tuple
probe was invalid and marked superseded; only the corrected study is valid.
All temporary harness changes are restored. Keep both admission passes.
Forcing the private tag helper inline produced identical Wasm in both profiles;
that annotation was removed. See the [rejected probe](../evidence/encoder-followup.md#newtype-tag-inlining-probe--10-november-2026).
The [ordinary tuple body probe](../evidence/encoder-followup.md#newtype-body-tuple-simplification--10-november-2026)
adds 52/16 opt-z/opt-3 Wasm bytes and is also rejected. Runtime effects were not
measured for that size-screen failure. The qualified encoder is restored exactly;
its all-feature tests/doctests pass. Stop this experiment line for the patch.

The [expanded hotspot sweep](../benchmarks/candid-comparison.md#expanded-hotspot-sweep--10-november-2026)
adds 20 application cases, preserving old indices. Snapshot runners now copy
qualification correctly; 58 CLI and 44 cost-checker tests pass. Cargo advice
covers signed integers, hashes and finite adapters. Fixture/oracle drift is
rejected, and maintained borrowed/Nat lane labels are explicit.

The separate 39-cohort baseline uses frozen pre-optimization/pre-expansion source
at `target/evidence/codec-matrix-20261110/source`, with direct Candid 0.10.40.
All 39 cohorts now verify: 2,234 overlapping cases, 46,978 IC rows and 151,392
native/frame checks. Wire is smaller in total/code Wasm in all 78 pairs. See the
[completed report](../benchmarks/candid-comparison.md#completed-full-baseline--10-november-2026).
It is not the new runtime's full matrix. No cleanup is authorized.

## Documentation and Toko Tooling 0.39.7

The [docs index](../README.md) separates guides, benchmarks, topic findings and
historical reports. Forty product files moved with links, selectors and fixtures.
Current guides now use native sizing APIs and the independent qualification
workspace, without old supplier matrices, issue-status tables or retired probe
instructions. Link checks pass; no runtime or protocol change is implied.
Tooling 0.39.5 now keeps reference documentation upstream: the exporter retired
66 managed Markdown files and unneeded derived helpers. One root baseline and
92 executable/support files remain. Product links use reviewed commit 12459b6;
no docs/toko-tooling directory or shared-doc migration remains outstanding.
The directly called validation-evidence.pl is an explicit retained entrypoint.

Both exact pins, locks and pin exceptions select 0.39.7. Installation/tool,
snapshot, pin and documentation checks pass. CI selection checks the retained
baseline rather than a removed guide; release validation retains its helper.
Current adoption evidence: /tmp/wire0397-*.log; doc-retirement and caller
qualification: /tmp/wire0395-*.log. Earlier routing and private-metadata proofs
remain in /tmp/wire0392-*.log. Stage reuse stays disabled. Codec work and the
incoming crc32fast update are preserved. The moved development guide has an
intent-to-add entry only for pin-check evidence; no contents are staged.

## Compiler, suppliers and release boundary

Rust 1.99.1 is the minimum/pinned compiler; only the opt-in sanitizer fuzzer has
a separately approved nightly. Both workspaces now select Toko Tooling 0.39.7.
Root owns four public packages; `qualification/` owns three private packages
and its independent lock. Canonical Types 0.19, Runtime/guard 0.21 and Testkit
0.96 composition is qualified. Manifests/locks own exact supplier selections.
Candid 0.10.40 is a private dev-only oracle, with exact-one-result framing and
quotas; the removed Platform depth pre-scan does not constrain this comparator.
Its bigint 0.4.8 exception remains approved and documented; production Types
use their own dependency. The incoming root crc32fast update is user-owned.

Tooling 0.38.2 delivery is repository maintenance only: fresh archive comparisons
show no product source/production-requirement change from published versions.
No public bump was needed for that adoption. Current runtime changes do affect
Core; 0.20.0 is the prepared hard-cut scope, with its pending changelog and explicit
regression tradeoffs. Endpoint runtime and its compatible ^0.19.0 requirement are unchanged. Current
package metadata changes affect both public groups; assess both at delivery. Manifest versions remain at the
published values for the tooling-owned release transaction. Full release gates,
commit/tag/push and publication have not run. Release groups and qualification
exclusions remain mandatory; this handoff is not a delivery. Historical completed
supplier/qualification narratives are retained in Git at 1e701b8 and existing
[delivery receipts](../../releases/receipts/README.md), not duplicated here.

## Remaining boundaries and custody

Postcard's derive-feature removal remains an unreleased supplier follow-up
(issue 56); do not repeat the unchanged experiment. Native big-endian execution
remains unqualified. Preserve original retry/hash bytes and consumer-owned
normalization. Account present/absent bytes remain `01 20 + 32 bytes` / `00`;
method IDs remain production behavior and fingerprint macros remain research.

No cleanup is authorized. Preserve all evidence, active targets, journals and
release state. Missing externally removed artifacts cannot be proved by receipts.
Failed batches batch-mzOGyRZA, batch-Oup1NG9b and batch-1afAufRB are superseded;
do not resume them. Completed batch-1i9z2IIG was reconciled, its checkout removed
and deferred lock 7fd813f reapplied. Never resume schema-1 0.17.5 with schema 2.
History was intentionally replaced: do not re-upload published versions to repair
Git. Future delivery uses explicit groups and the current roster; stage reuse
and final-candidate opt-ins remain disabled.
