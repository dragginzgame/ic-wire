# Current handoff

## Delivered: Core 0.17.6 / Shared 0.31

Core **0.17.6** is pushed to origin/main and published to crates.io:
**ic-wire-derive** and **ic-wire**, tag **core-v0.17.6**, release commit
**163d6aa**. Shared **0.31.0 / 7431394** captured source **245ed7a** in schema-2
**batch-1i9z2IIG**. Endpoint/macros remain **0.7.2**. The normal main branch is
reconciled to delivery. The [receipt](../../releases/receipts/published-core0176/delivery.json)
records the source, archive hashes, graph and qualification boundaries.

The complete **20-stage Linux gate passed** in the captured checkout. Exact
0.17.6 archives passed all-feature verification; installed Endpoint admission and
transaction checks passed on PocketIC **16.1.0**. Fresh Cargo registry downloads
match the upload and all-feature archive hashes and identify **163d6aa**.
Completed-batch resume made no new publication attempt. Delivery retained Platform
**0.23.4**, Types **0.18.2**, Runtime/macros/guard **0.19.4**, Testkit **0.89.3**,
Build **0.75.0**, Host **0.74.4** and Rust **1.99.1**.

The first attempt, **batch-1afAufRB** from **2d9a983**, stopped before version
preparation because its checkout lacked the browser compiler. It had no external
delivery effects and is superseded; do not resume it. Corrected source installs
the locked browser dependencies during capture and checks them offline on recovery;
hosted TypeScript CI uses the same owner. Later host package updates changed the
native tool fingerprint after the full gate. Shared admitted the same pinned tool
versions for the new host inputs, then resumed the saved preparation phase.

Both attempts' recovery state is retained under `.git/release-state/frozen/`.
Raw delivery/readback logs remain under ignored `target/core0176-delivery/`;
the successful checkout is `target/release-frozen/batch-1i9z2IIG/`. Earlier workflow
qualification remains under `target/shared031-adoption/`. The completed schema-1
0.17.5 batch below is historical and cannot use the new runner.

A concurrent **Platform 0.23.5** lock edit was excluded from the capture. It is
preserved on local branch **deferred/core0176-lock** at **7fd813f**, then reapplied
uncommitted after reconciliation. This is the sole deferred working-tree change;
it is not qualified by the 0.17.6 delivery and was not published with it.

## Shipped in 0.17.6: public packed domain adapters

`serde_packed::nonzero_i64` and `serde_packed::heap_u64` now reuse the existing
Packed codec directly from `Vec<NonZeroI64>` / `BinaryHeap<u64>`. Metadata derives
accept those explicit paths and exact type aliases; browser typed arrays validate
nonzero values automatically. No second schema trait or codec is added.

Focused Rust byte/budget/malformed checks, generated Rust–TypeScript vectors,
standalone host/Wasm/no_std feature isolation and compile-fail type qualification
pass, as do strict affected Clippy and the 100-record frame controls. Fresh paired
qualification is complete: opt-3 large-case encode drops 67,399.25 → 33,480.25
and 67,252.25 → 33,900.25 IC instructions, with one fewer allocation. Complete
124-case fixture Wasm falls 198/579 uncompressed bytes at opt-z/3; all 1,090 Wire
operation pairs stay within 5% + 64 instructions. See the
[public comparison](../candid-comparison.md#borrowed-adapter-investigation).

Retaining domain fields is not a universal size win: versus already-packed
records, 1/10/100-type stress adds 647/4,045/8,217 total bytes at opt-z and
340/8,986/16,149 at opt-3. Select the adapters for domain hot paths; keep Packed
DTOs where footprint takes priority. Two non-inlining experiments were rejected
because they regressed the single-type fixture without consistent profile gains.
The 32 measured modules and raw results remain under ignored
`target/evidence/public-adapters-20261106/`. Real big-endian execution remains
unqualified; release checks are recorded above.
Both fresh sides preserve the concurrent Testkit **0.89.3** lock selection;
lock SHA-256 `04623e37662c068b19fba770c5be5170b3a906a8c7548fd21004b74f4f648192`.

Consumer adoption from ordinary nonzero `[-1]` changes `01 01` to
`08 ff ff ff ff ff ff ff ff`; heap `[1]` changes `01 01` to
`08 01 00 00 00 00 00 00 00`. Coordinate identities/consumers, regenerate clients
and requalify limits. Existing consumers' bytes, identities, limits and acceptance
remain unchanged. Packed DTO bytes match, but Serde's field wrapper can reduce
encoding allowance (64-bit `{tag:1, words:[-1]}`: 842 versus 1,098 allowance bytes;
decode: 842 for both). Heap reconstruction is not canonical: retain original
retry/hash bytes. Real big-endian execution is not qualified.

## Shipped in 0.17.6: expanded container qualification

The follow-up request to address the two large-container gaps is qualified as an
**explicit consumer Packed DTO pattern**, using the existing production codec.
The [paired study](../candid-comparison.md#packed-domain-conversion-follow-up)
includes encode copies, nonzero validation and heap reconstruction. At 4,096
values and opt-3, nonzero encode/decode drops **94.96% / 92.04%**; heap drops
**95.02% / 85.61%**. Both beat Candid, as do all 32 target comparisons including
empty/singleton controls. Matched complete fixture Wasm shrinks **660 / 801 bytes**
at opt-z/opt-3; this is not a projected application saving.

Keep the ordinary controls. Packed frames grow to **32,782 bytes** from
**28,688 / 24,525**, and encode copies add one allocation; empty-container
overheads also regress. That earlier DTO study added no public API; the adapter
promotion above is separate. Public examples now use the adapters; the private
`packed-containers` cohort retains the manual conversion and domain-admission
controls in the maintained qualification roster. A zero integer is valid in the DTO but must be rejected
before domain use; heap reconstruction accepts unordered input and duplicates.
Consumer adoption changes `[-1_i64]` from `01 01` to
`08 ff ff ff ff ff ff ff ff`, so coordinate contracts/clients and requalify
budgets. Existing consumer bytes, identities, limits and acceptance stay unchanged.

Paired raw evidence and frozen sources are under ignored
`target/evidence/packed-containers-20261106/`, with separate fresh targets, eight
verified guest modules, 5,188 stable IC operation rows and 16,656 matched native/
allocation/heap records. Thirteen corpus tests pass for both selections, affected
strict Clippy passes, and codec-only Rustdoc passes (16 tests, one pre-existing
ignored example). This does not replace the full delivery gate. Both studies
retain the supplier graph recorded below; the fresh baseline is separate from
the first 30-case expansion's measurements.

The requested type-coverage follow-up appends **30 cases** to `coverage-shapes`
(**124 total**), preserving all 94 historical case indices and input/frame hashes.
It covers nested byte options, packed/blob results, byte-array and Packed hash
batches, nonzero integer lists and binary heaps. This coverage is included in the
Core **0.17.6** release above.

The [expanded comparison](../candid-comparison.md#expanded-container-coverage)
records both profiles, two native timing runs, allocation/heap probes, four fresh
Wasm modules and three equal samples for each of 2,594 IC operation rows.
Wire wins **102/120 new encode/decode comparisons**. Packed hashes perform well;
4,096-element nonzero-i64 lists and u64 heaps retain **16.52× / 16.60×** Candid
encode gaps at opt-3 in their ordinary representation. The subsequent explicit
conversion study above addresses those fields without changing that behavior.
Candid's external vectors do not perform nonzero admission or heap reconstruction.
Do not claim transparent Packed replacement or automatic
metadata advice for unsupported custom types. Expanded fixture Wasm is
**216,791 / 272,389 bytes** at opt-z/opt-3 versus **710,048 / 1,054,195** for Candid.
These are complete fixture sizes, not projected application savings.

The isolated source and raw evidence are under ignored
`target/evidence/container-types-20261106/`. This run selects Runtime **0.19.4**
and the concurrent Testkit **0.89.2** lock update, preserving both edits. The
codec comparison does not qualify Endpoint's changed supplier graph or substitute
for the next delivery gate. The old full matrix below retains its own graph.

Production bytes, schemas, identities, budgets and acceptance remain unchanged.
For example, nested optional bytes still encode outer absent as `00`, inner
absent as `01 00` and present-empty as `01 01 00`; no consumer action is needed.

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

## Previous 0.17.5 graph

Delivery used Platform facade **0.23.4**, Types **0.18.2**, Runtime/macros/guard
**0.19.3**, Testkit **0.89.1**, Build **0.75.0**, Host **0.74.4** and Rust/Cargo
**1.99.1**. Its source lock hash and release lock hash are in the receipt; version
preparation changed only the five Core-group package versions in Cargo.lock.

A concurrent process advanced the normal checkout to Runtime/macros/guard
**0.19.4** during validation. The frozen source retained **0.19.3**. The 0.19.4
edit was preserved through a temporary Git stash, reapplied after fast-forward,
and checked byte-for-byte apart from the five expected Core version bumps.
That deferred Cargo.lock edit was the sole uncommitted input after delivery.
Neither supplier update was part of the 0.17.5 delivery. Core 0.17.6 qualified
Runtime 0.19.4 and Testkit 0.89.3 as recorded above. A dependency patch alone does
not require another package release.

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

Use the [documented captured Core command](../development.md) for future
authorized delivery. Core retains all 20 stages; qualified Endpoint-only Linux
changes use 11. Stage reuse/final-candidate execution remain disabled. A future
combined Core/Endpoint batch still needs complete supplier-first archive
qualification; its disposable hook fixture alone is not installed qualification.
Shared 0.31 uses automatic capture and schema-2 journals; Wire retains its
supplier-order and Cargo-target admission.
Rust **1.99.1** remains both floor and pin; the separate nightly is fuzz-only.

The maintainer replaced older repository history with a clean root. Fresh-clone
rather than merging discarded refs; historical registry archives and receipt
identities are unchanged. Never re-upload an existing version to repair history.
