# Current handoff

Core 0.18.0 and Endpoint 0.9.0 were delivered at df071a4. Pending local work is
explicit finite field adapters plus the supplier/tooling adoptions below. Delivery is now authorized for both groups. Core 0.19.0 and Endpoint 0.10.0
notes are pending because canonical Types Rust identities change. Package version
preparation and registry delivery have not occurred.

## Toko Tooling 0.38 adoption

Adopted the authenticated published 0.38.0 snapshot and both workspace pins.
The three private packages now live directly under `qualification/`, without
a redundant `crates/` layer. Updated manifests, callers, source fingerprints
and documentation together; package identities, bytes, limits and acceptance
remain unchanged. Both moved workspaces pass locked metadata and fetch; benchmark and qualification
all-target builds, formatting, documentation, snapshot and dependency boundaries
pass. Release routing and delivery qualification are in progress.

## Independent qualification workspace

The four public packages now resolve through the root manifest/lock without
Build, Testkit, the Platform facade or Tokio. The three existing private packages
moved to `qualification/` and share `qualification/Cargo.toml` and its
own lock. They consume current public sources by path; private versions remain
unchanged. Native corpus/allocation work leaves `ic` and `diagnostics` disabled,
so simulator and Build dependencies are absent from those compilation graphs.
Optional canonical Types schemas and Endpoint's Runtime owner are retained.

Ordinary tool setup excludes the release editor. Release setup selects it
explicitly; the metadata adapter uses Shared's transaction and Build's editor
to update public identities in both locks atomically, including rollback when
private resolution refuses. No private supplier refresh is implied by release.
The full/Endpoint release rosters remain 20/11; future facade simulation and
integration graph checks belong to the explicit benchmark qualification stage.

Product tests, native corpus/allocation/contracts, both diagnostic suites,
private native Clippy, benchmark/Endpoint Wasm checks, current/future identity
consumers, CLI routing, source boundaries, formatting and docs are qualified.
The isolation fixture proves an incompatible Testkit selection fails only the
private workspace. Both groups' disposable metadata fixtures qualify lock
updates, unchanged private versions and rollback. Pins are checked in the
tracked disposable fixture: the normal checkout's new lock remains untracked,
so its tracking guard correctly refuses until delivery captures the new files.
Evidence: `target/wire-isolation-*.log` and product/integration-boundary reports.
No full CI, simulator/cost matrix, commit, push or publication was performed.
This refactor changes no codec bytes, identities, limits or acceptance and alone
requires no public release. No wall-time or artifact-size saving is claimed.

## Canonical Types 0.19 schema adoption (#58)

Core's optional platform-types selects Types 0.19.0; Endpoint and benchmark IC0
select Runtime/macros/guard 0.21.0 directly. Principal/Account schema code and
byte vectors are unchanged: Principal is at most 29 bytes, absent subaccount is
00 and present is 01 20 plus 32 bytes. Align consumers' Rust Types generation;
no byte, method identity, admission-budget or persisted-data migration is needed.

Standalone Rust/TypeScript roundtrips and malformed inputs, host/Wasm isolation,
combined Endpoint composition and simulated future facade identities pass. The
installed Endpoint catalogue/limits/rollback test passes on the existing PocketIC
server. Evidence: target/issues58-56/. The consumer uses local unpublished Wire
with registry Types 0.19; registry-only Wire delivery remains outstanding.

Testkit 0.96 now selects native Platform 0.28 and canonical Types 0.19. Native
callers use that identity directly; the temporary RawPrincipalId conversion is
removed. Private comparison selects upstream Candid 0.10.40 directly; the old
Platform 0.27 facade and Types 0.18 have left the graph. Bigint 0.4.8 remains only
the approved upstream-Candid dev-type exception; Platform independently uses
0.5.1. Core and Endpoint retain one Runtime/guard generation.

The oracle now uses upstream configured decoding with the same byte ceiling,
quotas and exact-one-result framing, without Platform's former depth-16 pre-scan.
Depth-8 trees now pass. Production Wire acceptance is unchanged. Evidence records
`upstream-exact-one-no-platform-nesting-prescan`; previous matrix comparisons
are historical until a fresh matched run. No speed or Wasm savings are claimed.
Evidence: target/platform028-testkit096/. The upstream tooling exception is
released as 0.37.5 (Git 1ea86ca and crates.io) with focused refusal tests;
its authenticated registry snapshot is installed and admitted (156 files).
Wire itself is not being delivered.

All 26 maintained host corpus selections pass correctness tests and Clippy; all
20 Wasm selections compile. Default oracle tests cover exact arity, trailing
bytes, byte ceilings and the changed nesting acceptance. Native drivers compile;
installed Endpoint catalogue/limits/rollback, stable identities, 22 graphs/eight
checker tests, source/Cargo boundaries, pins, formatting and docs pass. An initial
network-restricted Cargo attempt was resumed offline after preparing the new
tooling package; both logs are retained. This is not a new performance matrix.

Issue 56 was rechecked on 9 November: crates.io still supplies only Postcard
1.1.3, so its already-qualified upstream derive removal remains unreleased.

## Finite float adapters

The [public finite-adapter qualification](../evidence/primitive-bulk.md#public-finite-adapters--9-november-2026)
implements explicit finite_f32 / finite_f64 Serde fields with typed-array clients,
actual Vec type checks, finite-before-copy validation and lint advice. No new
codec or Packed word type. Ordinary fields and runtime admission are unchanged.
Adoption changes singleton f32 prefix 01 to 04, f64 01 to 08; coordinate
identities/clients, preserve bounds and retained bytes, and requalify budgets.

Matched public opt-z 1 MiB f32 encode/decode costs 7,210,930/10,159,614 versus
ordinary 72,877,809/19,930,633, saving 473 total/543 code Wasm bytes. f32 encode
still costs 3.40 times raw Candid. Opt-3 public Wasm saves 645 total/612 code
bytes, but 20 small-control instruction comparisons fail the margin. Keep the
large-field choice explicit; this is not a passing all-profile cost screen.

Native tests, malformed/allocation refusal, nine compiler refusals, standalone
host/Wasm consumers, generated Rust/TS bytes and simulated browser endian tests
pass. Native big-endian execution is not qualified. Final adapter hashes and
fresh modules verify; results are fixture measurements, not application savings.
Raw evidence: target/evidence/finite-float-cost-20261109/{public,public-qualified}/.
The maintained follow-up corpus adds ten finite-only cases (134 total); both
coverage/research correctness suites pass. No full enlarged matrix was run.

The [full Candid matrix](../candid-comparison.md) completes 39 cohorts, 2,194
overlapping cases and 8,776 primary comparisons on the pending supplier lock.
Wire is smaller in total/code Wasm in all 78 cohort/profile pairs. All 156 guest
builds, 46,178 IC rows and 148,832 native/frame comparisons verify. Native time
is shared-host evidence. Raw source/results: target/evidence/codec-matrix-20261109/.
The mixed cohort's raw float prototype does not enforce finite-only semantics.

The rejected 9 November [reader studies](../evidence/runtime.md) and
[u32 threshold experiment](../evidence/encoder-followup.md#u32-length-thresholds-9-november-2026)
remain recorded with their size/instruction losses; do not repeat them unchanged.

## Previous Toko Tooling 0.37.5 qualification

Before workspace isolation, published 0.37.5 was selected at the root Cargo dev
pin inherited by all seven packages. This selection is superseded by 0.38.0 above. The authenticated exporter preserves 156 snapshot files and
all explicit entrypoints. Registry checksum:
9fc1a752f4f8203d999c4cd593afb9ea18717b28d407492fc98aab5722526be3.

The narrow upstream boundary exception admits only unpublished
ic-wire-benchmarks' unconditional Candid dev dependency. Normal/build,
target-specific, public, other-owner and other-SDK edges remain refused.
Tooling was explicitly authorized for delivery: Git 1ea86ca, tag v0.37.5 and
crates.io publication are verified. Wire installation/tool admission, pin and
source/Cargo boundary checks pass. Its 22 identity graphs and eight checker tests
now require one Types generation; no temporary older-Types exception remains.
Evidence: target/platform028-testkit096/. Wire is uncommitted and unpublished.

Use toko-tooling-check/update/install/status, TOKO_TOOLING_* variables,
TOKO-TOOLING.md and .toko-tooling.snapshot. No old-command aliases are retained.
Historical delivery receipts preserve their original stage names.

## SHA-2 update and bigint probe

Private qualification selects SHA-2 0.11.1 with defaults disabled. Explicit hex
formatting replaces the removed digest LowerHex implementation; existing corpus
hash assertions pass. No production codec or protocol change, no measured Wasm
or instruction saving. Unrelated supplier selections are preserved.

num-bigint 0.5.1 fails the real Candid Nat conversion with E0308: Candid 0.10.40
exposes the distinct 0.4 BigUint type. The working 0.4.8 selection and unchanged
measurement path are retained with maintainer approval (9 November) of the dev-only
[exception](../evaluation.md#candid-bigint-type-identity). Evidence and focused
host/Wasm checks: target/deps0374/. No commit, push or publication.

Default/coverage/signed corpora pass (6/16/9 tests), including retained digest
goldens and Nat conversion bounds. Benchmark and qualification all-target host
checks/Clippy, Endpoint test compilation and coverage guest Wasm compilation pass.
Tool setup/check, 156-file snapshot admission, pins, dependency boundaries,
20 Testkit graphs, CI selection, 20/11-stage release routing and docs pass.
No full matrix or simulator rerun was required for these private evidence changes.

## Build 0.81 / Testkit 0.95 / Host 0.76.1 adoption

Root declarations and the release-metadata executable now select Build 0.81.0
and Testkit 0.95.0. Host artifacts/fs/process remain at the already selected
0.76.1; Testkit adds native Host HTTP 0.76.1. Wire has no callers of the removed
Build batch-error wrapper or generic PocketIcInstance, so no source migration
is needed. The targeted lock update changes only Build/Testkit and adds Host
HTTP. Pre-existing Platform 0.27.1, Runtime/macros/guard 0.20.3, Types 0.18.3,
Syn 3.0.10 and itoa 1.0.19 edits are preserved.

Eight Core and thirteen Endpoint diagnostic refusals plus passing controls,
codec/admission instruction-driver compilation, the installed PocketIC 16.1.0
Endpoint catalogue/limits/rollback test, both groups' disposable metadata edits
and oversize editor-input refusal pass. Tool installation/admission, snapshot,
dependency boundaries and pins pass. Evidence: target/adoption-20261109/.
Codec bytes, identities, budgets and acceptance are unchanged; no performance
matrix or new Wasm saving is claimed. These private supplier updates alone need
no public release. Changes are uncommitted; no push or publication was requested.

## Native Testkit qualification boundary (#57)

The existing check-boundary stage now runs check-testkit-boundary: eight checker
tests and twenty locked host/Wasm dependency graphs pass. Private native harnesses
use raw Testkit transport with its optional Wire adapter disabled; published
graphs remain Testkit-free. Local Wire, Types and Runtime identities are checked,
including one execution guard. Evidence: target/testkit-boundary.hPbjkJ/.

Core and Endpoint retain their existing product proofs and 20/11-stage release
plans. No reciprocal wait for Testkit's optional adapter is introduced. Ownership
and callers are documented in [evaluation](../evaluation.md#native-qualification-ownership).
Bytes, identities, budgets and acceptance remain unchanged. This change is local,
uncommitted and unpushed; no simulator rerun or performance saving is claimed.

## Remaining boundaries and custody

Postcard's derive-feature removal remains a published-supplier follow-up; Wire
still selects 1.1.3. See the [qualification](../evaluation.md#postcard-derive-dependency-qualification-8-november-2026)
and issue 56 rather than treating the previous upstream probe as an adoption.
Native big-endian execution remains unqualified. Keep one codec, no silent field
rewrites, original retry/hash bytes and consumer-owned normalization. Account's
[0.16 cut](../../releases/core/0.16.md) remains `01 20 + 32 bytes` for present
subaccounts and `00` for absent ones. Method IDs remain production behavior;
fingerprint macros remain research.

Preserve evidence, target/id16-work/pocket-ic, journals and release state; no
cleanup is authorized. Older paths may have been removed externally; receipts
do not prove checkout availability. [Delivery receipts](../../releases/receipts/README.md)
retain prior identities. Failed batches batch-mzOGyRZA, batch-Oup1NG9b and
batch-1afAufRB are superseded: do not resume them. Completed batch-1i9z2IIG was
reconciled but its checkout was removed; deferred lock 7fd813f was reapplied.
Never resume schema-1 0.17.5 with schema 2. History was intentionally replaced:
fresh-clone obsolete refs and never re-upload published versions to repair Git.
Future delivery uses explicit groups, Core's 20 stages and qualified Endpoint-only
Linux's 11; stage reuse/final-candidate opt-ins remain disabled.
