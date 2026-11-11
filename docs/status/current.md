# Current handoff

Core 0.20.0 (efdc034) and Endpoint 0.11.0 (b556f09) were delivered to
origin/main and crates.io on 2026-11-10. Both complete 20-stage release gates
passed; supplier-first batch-aZjXt3P2 is complete and reconciled. All four public
packages are published; the three private qualification packages stay unpublished.
The earlier failed batches remain retained. Release qualification corrected two
private Candid DTO declarations, unchanged qualification-manifest custody, the
frozen redundant-varint assertion and identity-fixture lock ownership.

## Toko feedback — pending Core 0.21.0

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
is rejected: +1,103/+1,022 codec Wasm bytes at opt-z/opt-3, +994 endpoint bytes,
small instruction regressions and a failed Candid-control hash gate. Compact Error
is restored; Result-wrapper mismatch coverage and diagnostic guidance remain.
Deployment is authorized: Core 0.21.0 then Endpoint 0.12.0, including crates.io.
Generated public names require a minor Core release and aligned Endpoint adoption.
Tooling 0.39.9 installation, snapshot/tools, boundary and roster checks pass.
The maintainer approved including Runtime/macros/guard 0.21.1 and both locks
now select that supplier with Types 0.19.1. Its transitive ic0 binding belongs
to Runtime; Core/schema-only exclusion and direct-dependency admission remain.
Installed endpoint calls, limits and transaction qualification passed with the
prepared PocketIC 16 server; both product/integration graph checks pass.
The supplier-first frozen batch will run both complete release gates.

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

The paired studies retain provenance, absolute counters and accepted regressions.
Rejected tag-inlining and tuple-body probes remain recorded in encoder-followup;
keep both admission passes. The expanded sweep adds 20 application cases without
changing old indices. Native times are shared-host observations, and fixture
Wasm gains are not projected application savings. Opt-z remains primary.

The [current full matrix](../benchmarks/candid-comparison.md#core-020-full-matrix--10-november-2026)
uses frozen released source 3c5176c under target/evidence/codec-matrix-core020.
All 39 cohorts verify: 2,314 overlapping case entries, 48,610 IC rows and
156,640 native/frame checks. All 156 guests were freshly built; Wire is smaller
in total/code Wasm in all 78 profile/cohort pairs. The 20 new application cases
win 34/40 opt-z and 32/40 opt-3 primary comparisons; all new decodes win.
Mixed-width tuple encode remains 2.63×/6.68× Candid at opt-z/opt-3, with 25%
larger frames. Investigate aggregate encoding/admission without removing limits.
Ordinary integer/byte-vector controls still expose large gaps; preserve their
explicit Packed/byte alternatives and lint guidance. This is a codec comparison,
not a matched regression study or full application benchmark. The prior matrix
and paired optimization studies remain historical evidence. No runtime changes
or public release are needed for this report; no cleanup is authorized.

## Documentation and tooling

The docs index separates guides, benchmarks, findings and historical reports.
Both workspaces retain Toko Tooling 0.39.9, its reviewed upstream references and
installed executable receipt; no vendored shared-doc tree remains. Prior snapshot,
pin and caller checks pass. Stage reuse stays disabled.

## Compiler, suppliers and release boundary

Rust 1.99.1 is the minimum/pinned compiler; only the opt-in sanitizer fuzzer has
a separately approved nightly. Both workspaces now select Toko Tooling 0.39.9.
Root owns four public packages; `qualification/` owns three private packages
and its independent lock. Canonical Types 0.19, Runtime/guard 0.21 and Testkit
0.96 composition is qualified. Manifests/locks own exact supplier selections.
Candid 0.10.40 is a private dev-only oracle, with exact-one-result framing and
quotas; the removed Platform depth pre-scan does not constrain this comparator.
Its bigint 0.4.8 exception remains approved and documented; production Types
use their own dependency. During the current matrix, an external edit changed
the normal root lock to Types 0.19.1. Deployment now aligns both locks to it;
the frozen performance matrix remains Types 0.19.0 evidence, not a new-supplier
performance claim.

Core 0.20.0 and Endpoint 0.11.0 are published. Future delivery uses explicit
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
