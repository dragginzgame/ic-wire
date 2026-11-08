# Current handoff

The [budget predicate screen](../evidence/cost-experiments.md#budget-predicate-screen-8-november-2026)
rejects combined comparisons and underflow-flag admission rewrites. Both grow
standard Wasm (154/196 bytes at opt-z) with mostly unchanged or slightly higher
instruction costs. All 860 instruction pairs pass the margin, but footprint
checks fail; that is not an optimization win. Keep the independent sticky-refusal,
overflow and no-partial-debit test. All lanes pass five library and 48 native
tests; retained test and Clippy pass. Production budget code is unchanged.
Native timing/memory and a full matrix were not run after rejection. Evidence:
target/evidence/budget-predicates-20261108/.

## Testkit 0.94 adoption

Private qualification now selects published ic-testkit 0.94.0. Wire has no
removed extension-trait/prelude imports, so no caller rewrite is needed. The
lock drops native Platform 0.26.1 and Host 0.75.1 duplicates, leaving Platform
0.27.0 and Host 0.76.0. Runtime/macros/guard 0.20.2 and Types 0.18.2 remain
unchanged. The prepared PocketIC 16.1.0 installed Endpoint catalogue, limits and
rollback test passes; codec/admission instruction drivers compile. Snapshot,
dependency-boundary and pin checks pass. No new performance matrix is claimed.
Codec bytes, identities, budgets and acceptance are unchanged. This private
supplier adoption alone requires no public release. Evidence: target/testkit094/.
No commit, push or publication is part of this adoption.

The [decoder tail screen](../evidence/runtime.md#decoder-tail-screen-8-november-2026)
rejects separated-tail and unrolled-u32 readers: both grow primary-profile Wasm
and slow the opt-z wide-u16 control by 12.9%. Unrolled u32 improves its own
wide decode 8.1%/6.0% at opt-z/3, but ten operation pairs regress; separated-tail
fails 18. Production parser remains unchanged. Keep the independent Postcard
oracle for u32/i32/u64/i64 ending bytes, overflow/nonminimal/truncated inputs,
trailing data and following-field consumption. All three lanes pass 48 native
tests; the retained test and strict test Clippy pass. No native timing/memory or
full matrix was run after rejection. Evidence: target/evidence/decoder-tail-20261108/.

The [wider writer screen](../evidence/encoder-followup.md#wider-writer-rejection-screen-8-november-2026)
rejects three follow-ups. Unrolled u64 reduces wide-list opt-z encode instructions
26.6% but adds 44.1% at opt-3; direct u16 saves 53.2% at opt-z but adds 32.4%
at opt-3. Reusing the u32 writer for u16 shrinks Wasm but slows u16 59.2% at
opt-3 and regresses retained u32 by 7.3%/20.3% at opt-z/3. All pass 47 native
tests; cost screens fail 12/four/12 operation pairs. Native timing/memory was
not run after rejection. No new production changes; the retained u32 writer
and Build 0.80 adoption remain intact. Evidence: target/evidence/wide-writers-20261108/.

## IC Build 0.80 adoption

The private diagnostic dependency and separately installed release-metadata
executable now select published ic-build 0.80.0. Its batch-report API cut has
no Wire callers. Required private dependencies add Host 0.76 and HMAC 0.13;
older Host identities remain for other private suppliers. Runtime/Types,
production codec bytes, identities, budgets and acceptance are unchanged.
Installation/offline executable admission, eight Core and thirteen Endpoint
diagnostic refusals with passing controls, both groups' metadata prepare/check
in a disposable fixture, oversize editor-input rejection, dependency boundaries
and tooling snapshot verification pass. No actual release effects occurred.
Evidence: target/build080/. This tooling-only adoption needs no additional
public release and does not requalify the preceding performance study's lock.

Adopt the [bounded unrolled u32 writer](../evidence/encoder-followup.md#bounded-unrolled-u32-writer-8-november-2026)
for pending Core 0.18.0. Wide-list encode IC instructions drop 45.1% at opt-z
and 14.0% at opt-3; all 860 hotspot instruction pairs pass. Signed32 modules
and all 1,070 counters remain identical. Allocations/heap match. Toko-shaped
Wasm shrinks 184/273 total bytes at opt-z/3; standard opt-z grows 218 bytes,
while standard opt-3 shrinks 1,590. The hotspot screen fails its opt-3 size
rule (+159 bytes), explicitly accepted for the instruction gain. Native
small/mixed opt-z regressions remain reported. Loop-only candidates are not
adopted. Native/runtime and signed32 tests plus Core Clippy pass; this is not a
full matrix or application measurement. Bytes, identities, budgets and acceptance
are unchanged: u32 128 stays 80 01, u32::MAX stays ff ff ff ff 0f. No migration,
client regeneration or retry/hash change; Endpoint needs no release for this.
Pending tooling edits are preserved. Evidence: target/evidence/u32-split-20261108/.

Release qualification now follows the workspace Endpoint minor series instead
of a fixed 0.8.0 pin. Current Core/Endpoint pairs are always tested; an old
published Endpoint tied to Core 0.17 is explicitly unsupported with Core 0.18.
Same-line identity splits still fail. Both current 0.17 and prospective 0.18
checks pass; four public package payloads match the approved source, excluding
Git provenance. Evidence: target/deploy018-prospective/ and
target/deploy018-equivalence/. No push or publication has occurred.

## Platform 0.27 qualification

The unpublished benchmark facade now selects published ic-platform 0.27.0.
Testkit 0.93.2 retains facade 0.26.1 in native qualification only. Runtime,
macros and the sole guard remain 0.20.2; canonical Types remain 0.18.2. Wire
does not call the removed management raw_rand API, so no source migration is
needed. Standard and coverage comparison controls pass (22 tests), as do
Clippy, the benchmark Wasm build, dependency boundaries, pins and tooling
snapshot/lock verification. No new performance or Wasm saving is claimed.
Production codec bytes, identities, limits and acceptance are unchanged. This
private supplier change alone does not require a public release. Evidence:
target/platform027/. Existing pending product releases remain separate.

## Toko Tooling 0.34 root-owned selection

Root Cargo.toml workspace metadata selects exact toko-tooling 0.34.0;
toko-tooling.lock owns registry identity/checksum independently of application
Cargo.lock. The installed snapshot matches checksum
5ca8ad09ad9359bc4945a4afdaafce301dd5a4b4955cd5c9f5058d7a54ffcf10.
The old workspace dependency and private dev-dependency are removed; Cargo
removed only the marker and its dependency edge. All seven real members remain,
including qualification’s tools/tests; there was no marker-only package to delete.
Use shared-tooling-update after changing the root version, or
shared-tooling-install to reinstall the existing tooling lock. Application Cargo
updates no longer select tooling. Custom CI, publication, recovery and local
release adapters delegate admission to shared-tooling-check before effects.
Product group gates and publication exclusions remain unchanged. Installation,
tool admission and 154-file snapshot verification pass; all 79 entrypoints remain.
Focused checks pass: seven-member/default-member/release-group consistency,
unchanged non-tooling lock selections, private compilation, dependency/pin
boundaries, formatting/docs, 16 CI classifications and 256 gate combinations.
Ten drifted-entrypoint refusals and the complete disposable Make routing/
publication/formatting-hook fixture pass, with real release effects substituted.
No active tooling dependency, marker-only package or cargo-machete exception
remains. Evidence: target/toko034/. No commit, push or release is part of this
migration.

Delivery preparation selects Core 0.18.0 then Endpoint 0.9.0, with frozen
source and full group gates. Endpoint’s inherited Core requirement advances
with the Core minor cut. Public crate pairs publish supplier-first; private
qualification packages remain excluded. Delivery is not yet complete.

Cleanup issues 51–56 are investigated; see the
[decisions](../development.md#cleanup-investigation-decisions-8-november-2026).
One CI qualification gate replaces three identical scaffold jobs; 192 local
state combinations and workflow lint pass. Issues 51, 52 and 54 remain declined.
Issue 53 now suppresses discarded schema rendering in the shared walker; the
[host study](../evaluation.md#host-schema-rendering-suppression-8-november-2026)
measures 40–56% faster opt-z validation and large allocation reductions. Small
generation adds 85 ns (4%); generated output and rejection results match. Native
tests, Clippy, TypeScript/endian conformance and dependency checks pass.
Issue 56 now tracks published-release adoption: upstream PR 219 already merged
the derive removal. Pinned-upstream codec/schema host/Wasm and error-byte checks
pass; isolating derive gives 2.53 → 1.85 s clean native build medians (26.9%).
The registry index still ends at old 1.1.3; production selection is unchanged.
See the [qualification](../evaluation.md#postcard-derive-dependency-qualification-8-november-2026).
No new codec or Wasm saving is claimed.
Hosted gate execution awaits delivery.

Shared 0.31.12 / eb3c8e0 updates cloc progress and terminal presentation;
codec and runtime behavior are unchanged.

The preceding Shared **0.31.11 / 6c529e6** adoption was exported from the pinned
upstream commit, excluding sibling working changes. CI restores authenticated
tool downloads before setup and saves them before product checks. One Gitleaks
scan remains outside the host/group matrix. Crate/module cloc selectors and the
updated ordinary-issue closure policy are adopted. This tooling-only update
does not select an additional public release or alter codec/runtime behavior.
Snapshot verification (146 files), 995 documentation references, actionlint,
crate/module cloc selection and disposable Make validation routing pass. Hosted
cache restore/save remains unexecuted locally. Evidence: `target/shared03111/`.

## Pending Core 0.18.0: rooted lint hard cut and i32 writer

Core 0.18 requires `lint_bulk_roots` and rooted acknowledgement paths. Serde
derive selection is explicit. Codec bytes, identities, runtime budgets and
acceptance remain unchanged; no stored-data migration.
Historical conformance timings, Endpoint extraction comparisons and the private
Boolean prototype are retired. Benchmark integrity fixes use captured source
and distinct query callers; Endpoint reports reject duplicate callers and unsafe
counters. Postcard still selects Serde derive transitively: direct feature
ownership cleanup does not yet save that compilation.

Focused checks pass: 97 Core tests (three timing probes ignored), 16 corpus tests,
three contract tests, 19 report corruption cases, Clippy, standalone host/Wasm
consumers, released Endpoint coherence and Rust/TS conformance (178 vectors,
39,448 mutations). Caller-storage orchestration
was checked with 23 substituted Cargo invocations; no new PocketIC cost study
or production Wasm saving is claimed. Evidence: `target/architectural-core/`,
`target/architectural-qualification/` and `target/architectural-compatible.log`.
Fingerprint cleanup removes 69 host-only lines; 22 mappings and 30 verifier cases pass.
The cohort roster now owns qualification subsets: dry dispatch preserves exactly
26 test/26 Clippy/20 Wasm calls and all 39 measurement rows. No matrix rerun.
Current lock selects Testkit 0.93.2; focused checks are not supplier qualification.
Removed `check-msrv`; dry dispatch retains host/Wasm checks and compiler admission.
Endpoint qualification now shares its scenario catalogue and result checker in
one private integration target. Installed checks also execute the unchanged cost
catalogue; explicit transaction-history proofs and independent trace validation
remain. This removes 71 net Rust lines from the two prior test roots, including
the new shared modules. Installed PocketIC 16.1.0, Wasm build, strict Clippy,
exact runner discovery and report-corruption checks pass;
no profiler cost rerun or production Wasm saving is claimed. Bytes, identities,
limits and acceptance are unchanged. Evidence: `target/endpoint-consolidation/`.
No full CI/delivery; package versions remain unchanged.

Adopt the [raw-tuple i32 writer](../evidence/encoder-followup.md#raw-tuple-i32-follow-up-8-november-2026).
The maintainer accepted opt-z as the primary decision profile and opt-3 as a
secondary regression check. At opt-z, 4,096-element small/wide-list/deque/set
encode instructions drop 46%/28%/37%/30%, at a cost of 150/90 total Wasm bytes
in focused/standard fixtures. Opt-3 saves 1,143/420 Wasm bytes; wide-list encode
adds 3.52%, while small-list/deque/set improve 54%/34%/30%. All 482 matched
operation pairs meet the instruction margin; the automated no-growth screen
still fails and is explicitly accepted, not reported as a pass. Allocation/heap
and Candid controls stay identical. These are fixture results, not application
savings or a fresh full matrix.

Bytes, schemas, identities, limits and acceptance remain unchanged: i32 -64
remains `7f`, 64 remains `80 01`. No consumer migration, client regeneration or
retry/hash change. This affects Core production; Endpoint needs no release.
Native tests, slice-output coverage, both standard/signed32 corpus tests and
Core/corpus Clippy pass. Fresh production signed32/standard Wasm matches all
four measured candidate modules byte-for-byte. The private roster
now has 39 cohorts, adding 49 [baseline cases](../candid-comparison.md#signed32-follow-up-8-november-core-01714).
Deque tests cover all 128-element storage splits and exact/one-short limits;
scalar prefix tests cover the four-/five-byte transition. Evidence: `target/evidence/signed32-20261108/`.

The subsequent [unsigned writer study](../evidence/encoder-followup.md#unsigned-writer-sharing-8-november-2026)
rejects a combined u32/i32 + u64/i64 rewrite. It saves 301 standard / 613
Toko-shaped opt-z Wasm bytes, but slows wide i32 by 5.56% at opt-z and wide u64
by 49.9% at opt-3. Twenty-four of 860 operation pairs fail the instruction screen;
frames, Candid controls and allocation/heap measurements match. No additional
production change. Evidence: `target/evidence/integer-writers-20261108/`.
An independent lock edit selects Build 0.79.1 / Host 0.75.1; it is preserved.
This study uses the previous frozen lock and does not qualify that supplier update.

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

Core **0.17.13** retains its [receipt](../../releases/receipts/published-core01713/delivery.json).
Its public ID field changes ordinary `01 + 16 bytes` to packed `10 + 16 bytes`; eight IDs change
`08 + 128 bytes` to `80 01 + 128 bytes`. New field adoption requires coordinated
identities/clients, original retained retry/hash bytes and reviewed budgets.
The [flat-Candid study](../evidence/primitive-bulk.md#public-16-byte-ids-and-flat-candid-8-november)
corrected the earlier nested-vector comparison; retain its ordinary controls and
borrowed-input copy costs. The latest copy optimization addresses its two large-ID
opt-z losses. Original evidence: `target/evidence/id16-flat-20261108/`, `target/id16-work/`.
Actual native big-endian execution remains unqualified.

Rejected experiments remain in [primitive bulk](../evidence/primitive-bulk.md)
and [bound sharing](../evidence/bound-sharing.md); do not repeat those candidates.
The earlier 7 November raw targets were found removed; summaries do not prove
those artifacts remain available. The [full comparison](../candid-comparison.md)
is still the 36-cohort Core 0.17.6/Testkit 0.89.4 study: 2,105 cases and 8,420
primary comparisons. The maintained roster has 39 cohorts; no fresh full run.

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
