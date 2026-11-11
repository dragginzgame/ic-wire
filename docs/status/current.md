# Current handoff

Core 0.21.1 (db58463) was delivered to origin/main and crates.io on 2026-11-11.
Its complete 20-stage gate passed; batch-CaPBWJVP is complete and reconciled.
Both Core packages are published. Endpoint remains at published 0.12.0 (009f3eb),
delivered in the earlier completed/reconciled batch-qfrz5NEG. Private qualification
packages remain unpublished. No cleanup or batch retirement is authorized.

## Pending Core 0.21.2 — host generator allocations
Borrow ordinary escaped-path inputs and active recursion names. Matched synthetic
plain/shared-reference fixtures remove 39%/65% of allocations; shared-reference
latency improves, with escaped-name and noisy nested regressions recorded in
[the paired study](../evidence/native-registry.md#borrowed-lint-names--11-november-2026).
87 active native tests and strict Core/private-harness Clippy pass. Output,
warning paths, codec bytes, identities, budgets and acceptance are unchanged.
[Direct registry append](../evidence/native-registry.md#direct-registry-append--11-november-2026)
also removes a full-registry temporary: 18%/20% less allocated traffic in plain/shared
fixtures, with opt-3 timing regressions recorded. Rust/TypeScript conformance passes.
[Direct declarations/conditional escaping](../evidence/native-registry.md#direct-declarations-and-conditional-key-escaping--11-november-2026) remove another 20.7%/17.2% allocated traffic; timing regressions are recorded. Exact outputs and conformance pass; no Wasm/IC claim.
[Direct composite rendering](../evidence/native-registry.md#direct-composite-rendering--11-november-2026) removes 31.7% of large-record allocations, improving measured time in both profiles. Types 0.19.3 is now selected in both locks for the authorized Core 0.21.2 / Endpoint 0.12.1 frozen release; full qualification is pending.
Rejected follow-ups: [empty acknowledgements](../evidence/native-registry.md#empty-acknowledgement-shortcut--rejected-11-november-2026) and
[bulk-write inlining](../evidence/cost-experiments.md#bulk-write-inlining--rejected-11-november-2026). [Shared framed decode admission](../evidence/cost-experiments.md#shared-framed-decode-admission--not-promoted-11-november-2026) saves 338 opt-z bytes but grows opt-3; no runtime change is retained.
[Fused nonzero decode](../evidence/cost-experiments.md#fused-nonzero-packed-decoding--rejected-11-november-2026) and [boolean block validation](../evidence/cost-experiments.md#block-validation-for-packed-booleans--rejected-11-november-2026) regress opt-3/opt-z respectively; both rejected.

## Released Core 0.21.1 — browser and host costs

Browser alias detection and reusable host lint paths are released; see the
[browser study](../evidence/typescript-writer.md#alias-chain-scaling--11-november-2026)
and [host study](../evidence/native-registry.md#reusable-lint-paths--11-november-2026).
Regenerate clients for browser changes; valid bytes/identities/budgets stay the
same, malformed-cycle callback counts can differ. Small long-chain regressions
are recorded. Set-based aliases, Unicode regex and
[budget outlining](../evidence/cost-experiments.md#shared-budget-outlining--rejected-11-november-2026)
were rejected. Release gates used Types 0.19.1; later root updates remain deferred.

## Toko feedback — released Core 0.21.0

Stable host names replace positional WireType indices. Generated definitions use
readable Rust type stems and deterministic full-type suffixes; explicit roots
alias them independently of registration order. Regenerate schemas/clients and
migrate positional imports once. Schema artifact hashes change; runtime bytes,
method identities, budgets and acceptance do not. See the [host study](../evidence/native-registry.md#registration-independent-names--11-november-2026)
for the generation-time and TypeScript-text growth tradeoff. Root/generic/namespace/
collision/recursive tests and regenerated Rust/TypeScript conformance pass.

Nested allocation accounting now has exact empty/single/batch boundary tests and
clearer documentation. Local Canic selects Wire 0.18.0 in both locks: no 0.16
backport is prepared. Its runtime-configuration limits reserve 16,384 bytes and
allow 262,144 allocation units, 16,384 nodes, depth 32. Toko's actual value is
unavailable, so its exact reported thresholds are not reproduced. Do not weaken
admission or infer physical heap use from this weighted traversal allowance.

Canic's host generator passes an empty acknowledgement map. Wire already supports
exact-path reasons and bounded report Display; downstream must pass reviewed
acknowledgements consistently through producer and CLI. No sibling files changed.
The [numeric Trailing experiment](../evidence/codec-cleanup.md#unread-byte-error-payload--rejected-11-november-2026)
was rejected for Wasm growth/regressions; compact Error and mismatch tests remain.
Tooling 0.39.9 and Runtime/macros/guard 0.21.1 passed the release boundary checks;
transitive ic0 belongs to Runtime, not Core/schema-only hosts.
Installed endpoint calls, limits and transaction qualification passed with the
prepared PocketIC 16 server; both product/integration graph checks pass.
Both complete release gates passed with these suppliers. During deployment,
another process changed the normal root lock to Types 0.19.2. That edit is
preserved for the next candidate, outside this release; qualification still
selects 0.19.1. Do not treat 0.19.2 as qualified by these release gates.

## Upstream decoder hard cut — released in Core 0.20.0

Core 0.20.0 uses upstream Postcard in place of the private parser.
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
Both full release gates passed. The released-source 39-cohort matrix is now
complete; see the current comparison below.
Prior parser measurements below are historical for this decoder change.

## Optimization and matrix refresh — 10 November

Earlier qualified finite-f32, unit/newtype-tag and consolidated depth-admission
changes remain documented in [primitive bulk](../evidence/primitive-bulk.md)
and [encoder follow-up](../evidence/encoder-followup.md), including their accepted
opt-3 regressions. They preserve bytes, identities and admission. Do not repeat
rejected tag-inlining, tuple-body or f64 probes. Opt-z remains primary; fixture
Wasm changes are not application savings.

The [Core 0.20 matrix](../benchmarks/candid-comparison.md#core-020-full-matrix--10-november-2026)
is historical evidence from frozen source 3c5176c. The current refresh follows.

## Complete full comparison — 11 November

The [Core 0.21 checkout matrix](../benchmarks/candid-comparison.md#core-021-checkout-full-matrix--11-november-2026)
verifies all 39 cohorts: 2,314 case entries, 48,610 IC rows and 156,640 native/frame
checks. All 156 guest builds are fresh; all 78 total/code Wasm pairs favour Wire.
Primary instruction wins: 4,157/4,628 opt-z and 4,260/4,628 opt-3. Ordinary-list
controls still exceed 10× Candid; mixed-width tuple encode remains 2.63×/6.68×.
Full exports and frozen sources: `target/evidence/codec-matrix-core021`; preserve
`target/codec-matrix-core021-build`. Qualification uses Types 0.19.1, not the
pending root-only 0.19.3 (externally advanced from 0.19.2 during the run).
This is not a matched regression or application study.
Native timings share the host with other builds. The current deployment is a separate qualification.

## Documentation and tooling
Both workspaces now select Toko Tooling 0.39.11, its reviewed upstream references and
installed executable receipt; no vendored shared-doc tree remains. Installation,
snapshot/tools, pin/boundary, documentation and disposable Make routing checks
pass. LOC narrow/wide rendering preserves numeric columns across 133 rows; stage reuse stays disabled.

## Compiler, suppliers and release boundary

Rust 1.99.1 is the minimum/pinned compiler; only the opt-in sanitizer fuzzer has
a separately approved nightly. Both workspaces now select Toko Tooling 0.39.11.
Root owns four public packages; `qualification/` owns three private packages
and its independent lock. Canonical Types 0.19, Runtime/guard 0.21 and Testkit
0.96 composition is qualified. Manifests/locks own exact supplier selections.
Candid 0.10.40 is a private dev-only oracle, with exact-one-result framing and
quotas; the removed Platform depth pre-scan does not constrain this comparator.
Its bigint 0.4.8 exception remains approved; production Types own their dependency.
The historical Core 0.20 matrix used Types 0.19.0; this refresh uses 0.19.1.
Both locks now select Types 0.19.3 for the new release gates; earlier evidence remains tied to its recorded supplier.

Core 0.21.1 and Endpoint 0.12.0 are published. Future delivery uses explicit
release groups and qualification exclusions; do not re-upload those versions.
Historical delivery narratives remain in Git at 1e701b8 and existing
[delivery receipts](../../releases/receipts/README.md).

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
