# Current handoff

## Git core 0.12.1; pending Platform alignment 0.13 / Endpoint 0.2

Baseline is `ae9a3eda0449729008d849c6e078a8b7f849ab9e` (Git core 0.12.1),
after completed merge `02c77d6`. Own versions stay core 0.12.1 / Endpoint 0.1.0.
Open notes select core 0.13.0 and Endpoint 0.2.0 because Types 0.1 → 0.15 changes
Principal/Account Rust identity. The maintainer has delegated delivery of this batch. Use the standard explicit
group workflows, core before Endpoint; no sibling checkout changes are authorized.

Current selections are published Platform, macros, guard and Types 0.15.0;
Build/Testkit and managed `cli` editor remain 0.73.0. Shared Tooling now selects 0.27.19 at
`54fc49e12f439dbe18d232f622a6d78e61773353`. The initial `99d04d1` export exposed
a committed VERSION mismatch ([Shared #331](https://github.com/dragginzgame/shared-tooling/issues/331));
upstream corrected VERSION only, and a canonical follow-up refresh selected the
corrected revision. Selected payloads are unchanged, so qualification is reused. No other lock selections change. Endpoint 0.1.0 is already published; publish the supplying
core minor before the new Endpoint minor through the two existing groups.

The supplier implementation is unchanged: 112 Platform source files, five macro
files and the guard match; Types' ten source files differ only in its module
versioning documentation. Published 0.15 aligned Types with its workspace. Platform #118 now reports a
qualified local restoration of independent runtime/Types groups, retaining Types
0.15.0 without a reset; maintainer delivery remains pending. This supersedes the
intervening policy decision, not the published 0.15 identity.
Wire's package separation isolates runtime changes while the selected Types
identity stays fixed; it cannot preserve Rust identity across upstream Types minor
bumps. No compatibility lane, schema mirror or range workaround was introduced.

[Current qualification and reproduction](../evidence/platform015-adoption/README.md)
retain standalone Rust/TypeScript exact bytes/refusals, Rust 1.91 host/Wasm,
feature isolation, endpoint diagnostics/Clippy and installed limits/rollback.
The prospective identity fixture now derives the current and simulated next
versions from metadata, rather than hard-coded 0.14/0.15 values. Its simulated
0.16 facade holds Types 0.15 fixed; that is not a claim about upstream release policy.
All 384 logical measured calls / 608 physical executions retain identical
instruction and cycle samples, frame hashes, outcomes and transaction oracles.
Total Wasm stays 100,441 B (opt-z) / 94,073 B (opt-3); code sections stay
86,544 / 81,127 B. No measured regression or optimization is claimed. Installed
limits/rollback pass. Raw outputs: `/tmp/wire-platform-alignment/comparison`.
No full CI/macOS, native-time or new Candid matrix was run. Measurements and
limits remain specific to their recorded supplier selections.

## Shared release-layout adoption

The reviewed canonical refresh uses the existing 119-file selection, with no
snapshot-owned edits. Current package paths, numeric versions and every resolved
Cargo metadata field are unchanged (after source-path normalization); Cargo.lock
is byte-identical to the incoming Platform 0.15 work. Root Cargo has no workspace
version. Every member has its current literal version, and a comment map lists
all groups/members/authorities/ledgers beside `[workspace]`.

The root changelog is an index; canonical ledgers are
[core](../../releases/core/CHANGELOG.md) and
[endpoint](../../releases/endpoint/CHANGELOG.md). Coordinated history through
0.11.0 is retained once in [history](../../releases/history.md). Finalized entry
prose, headings/dates/scope and versions match the prior inputs, except rebased
relative links. The former endpoint ledger is a navigation pointer.

Make/hook routing, missing/empty/unknown group refusals and both publication
rosters pass in disposable fixtures. Real group preparation/resume checks pass
against both groups, including exact file custody and unchanged unselected
manifests/ledgers. Snapshot integrity, dependency pins/boundaries and links pass.
Logs and input preservation proof: `/tmp/wire-shared-02718/`; accepted runs are
`release-final.log` (both groups) and `routing-final.log`. An earlier fixture
was invalidated by editing its script mid-run; its retained failure is not used
as acceptance. Final scripts were frozen before the successful rerun.
The corrected supplier's [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/41209009495)
was still running at the bounded inspection; no upstream/native success is claimed.
No runtime or
generator code, feature graph or lock resolution changes; no new performance
benchmark applies to this metadata/tooling batch. No full CI/macOS run is claimed.

## Implemented boundary and migration

[#36](https://github.com/dragginzgame/ic-wire/issues/36) has a local implementation
of the [approved boundary](../stable-identities.md). `ic-wire` retains codec,
schema trait, Limits/Error and canonical Principal/Account schema impls. Its
`platform-types` selects Types 0.15 directly, with no runtime facade/CDK dependency.
Platform 0.15 reexports those exact types. New `ic-wire-endpoint` owns attributes,
Contract, guards and trapping replies, with the same maintained derive generator.
Remove `ic-wire/endpoint`, move `ic_wire::endpoint` imports to `ic_wire_endpoint`,
and select Endpoint `typescript` for native Contract schemas. One-time typed
Testkit and Metrics alignment is still required for the supplying core minor.

Core exposes the fixed request-header size and validator; Endpoint delegates
pre-copy admission to that same validator, then performs complete decoding.
No frame layout, schema model, type wrapper, codec or compatibility lane is added.
Bytes, budgets, primitive/malformed acceptance, field order, handler transactions
and original retry/hash inputs are unchanged. Principal stays bounded bytes
(max 29); Account stays owner then optional 32-byte subaccount. Absent `01 04 00`
and explicit zero `01 04 01` plus 32 zero bytes stay distinct.

Explicit `core`/`endpoint` release groups use shared plans for exact file custody,
metadata preparation, receipts, tags and resume. Build 0.73 supplies `group-manifest`.
The 0.27.19 follow-up changes governance only; it delegates scoped shipping
without per-command confirmation. Current explicit deployment targets are Git
origin/main and crates.io, core 0.13 before Endpoint 0.2.
No group defaults: release, resume and publication require explicit
`RELEASE_GROUP=core` or `endpoint`. Publication selects only that group's public
packages (derive before core) and refuses cross-group PACKAGE subsets.
Endpoint 0.1.0 was the initial extraction. The current alignment has separate
core 0.13 / Endpoint 0.2 notes; no release or publication was executed this batch.

## Retained extraction qualification

[Evidence and reproducible runners](../evidence/stable-identities-0120/README.md)
retain standalone derives, Rust 1.91 host/Wasm, core/schema/endpoint isolation,
exact Rust/TypeScript frames and malformed refusals, 11 endpoint diagnostics and
renamed-import control, strict affected Clippy, core tests and installed endpoint
limits/rollback. A simulated next Endpoint 0.2 / Platform 0.15 fixture changes
those package identities while retaining identical core/derive/Types package IDs;
it passes real schema/Contract tests and one-guard Wasm graphs. This is not a
claim that future runtime versions are published or behaviorally qualified.

Source-aligned copies of released Metrics 0.22 and Testkit 0.73 compile their
real schema and typed-call implementations with the pending core and shared
Limits/Error identities, including Rust 1.91. Only disposable manifest paths
change; no Cargo patches or sibling source edits. Registry adoption remains
pending publication of the new core. Testkit's native-only Platform 0.13 stays
outside the deployed endpoint runtime graph.

The matched seven-export, 12-case study compares the previous monolithic
Platform 0.13 integration with the split/Platform 0.14, both on Tooling 0.73.
There are 384 measured logical calls / 608 executions, eight repetitions after
warmup per case/lane/profile. Frames/outcomes/transaction oracles match.
Total uncompressed Wasm: 99,951 → 100,665 B (+714, +0.714%) opt-z and
93,518 → 94,073 B (+555, +0.593%) opt-3. Code sections: 86,068 → 86,766 B
and 80,630 → 81,127 B. Largest instruction regression is +0.0255%; some async
opt-z cases improve slightly. No size-saving claim or gzip criterion is used.
Original study: `/tmp/wire-identity-split/comparison-final`.

Both groups' disposable release qualification covers actual metadata preparation,
validation failure, file custody, receipt tampering, exact Git delivery and resume;
publication selection/order uses sentinel uploads. No real Git/index effects.
Tooling diagnostics and Make/hook routing pass. No full CI, macOS matrix, native
timing or new full Candid performance matrix is claimed.

## Delivery and remaining issues

Only [#36](https://github.com/dragginzgame/ic-wire/issues/36) remains open; there
are no open PRs. It now records delivered core 0.12.1's complete successful
[12-job CI matrix](https://github.com/dragginzgame/ic-wire/actions/runs/41201306818)
on Linux/macOS ARM64/Intel separately from the dirty 0.15 supplier adoption.
The latest Platform #118 report prepares independent groups at existing Types
0.15; it awaits maintainer delivery and is not treated as a published new supplier.

[#37](https://github.com/dragginzgame/ic-wire/issues/37) is closed as superseded
(not completed historical Platform 0.13 support). Its remaining minimal/actual
Blob schema pairing, bytes/budget/refusal and isolation obligations are recorded
on #36. No legacy lane or source workaround was added. Deliver/qualify the current
Wire supplier change and perform ordered core/Endpoint publication, then check
the documented registry pairing. Consumer adoption remains with each owner;
it is not an extra fleet-wide producer closure gate.

[Tooling #274](https://github.com/dragginzgame/ic-tooling/issues/274) is already
closed for its delivered Wire 0.11 adoption in Tooling 0.73; do not reopen it for
later Wire minors. New public Limits/Error and Metrics metadata alignments need
their current supplying releases. This inspection changed issue tracking and
status documentation only; no new runtime benchmark or broad suite applies.

#31 (CLI), #32 (old dependency cycle) and #34 (published 0.12 pairing) remain
closed. [Published-pair evidence](../evidence/published-pair-0110/README.md)
includes Blob 0.60's real schema example with Wire 0.11 / Platform 0.12 /
Metrics 0.22; that historical result is not new Blob adoption. The earlier
[Platform 0.13 study](../evidence/platform013-adoption/README.md) and
[0.12 qualification](../evidence/platform012-adoption/README.md) retain their
original supplier inputs and measurements.

Shared 0.27.13/.14 canonical refresh, snapshot/pin integrity and unchanged local
Make/release fixture evidence remain in `/tmp/wire-shared-02713` and
`/tmp/wire-shared-02714`. No vendored scripts were patched for this implementation.
[Tripwire research](../evidence/tripwire-followup-0101/README.md) remains unadopted;
[Candid matrix](../evidence/candid-comparison-0725/README.md) retains ordinary-vector
controls and explicit Packed lanes. Candid stays development-only. Evidence Cargo
manifests remain snapshots, not active workspace files.
