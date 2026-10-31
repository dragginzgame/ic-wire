# Current handoff

## Released 0.11.0; pending stable identity migration 0.12.0

Published core is 0.11.0, release `568196014214a27b1a679b119ffde45284ed5eae`.
Existing package versions remain 0.11.0; new Endpoint's initial package is 0.1.0.
The pending changelog is 0.12.0. Leave staging, commits, existing version changes,
release effects and publication to the maintainer. No sibling checkout changed.

Current selections: Platform 0.14.0, canonical Platform Types 0.1.0, Build/Testkit
and managed `cli` release editor 0.73.0; all published registry suppliers.
Shared Tooling remains reviewed 0.27.14 at
`b8d2c6e27a755a7bae46ef25ccd2b5b4f53e8e57`. Incoming log 0.4.35 is preserved.

## Implemented boundary and migration

[#36](https://github.com/dragginzgame/ic-wire/issues/36) has a local implementation
of the [approved boundary](../stable-identities.md). `ic-wire` retains codec,
schema trait, Limits/Error and canonical Principal/Account schema impls. Its
`platform-types` selects Types 0.1 directly, with no runtime facade/CDK dependency.
Platform 0.14 reexports those exact types. New `ic-wire-endpoint` owns attributes,
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
The default group is core. Publication independently selects derive/core before
Endpoint, with exact package versions; see the migration guide for commands.
Initial Endpoint 0.1.0 is delivered with the core migration before independent
incrementing releases. No real release or publication was executed.

## Qualification

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

#36 remains open pending delivery/required hosted checks and published consumer
pairing. [#37](https://github.com/dragginzgame/ic-wire/issues/37)'s pending 0.13-only
pairing is superseded by the stable 0.14 identity migration; it also remains open
until the supplying Wire release and downstream registry check. Published Wire
0.11 still pairs with Platform 0.12 / Metrics 0.22. Tooling
[#274](https://github.com/dragginzgame/ic-tooling/issues/274) tracks matching typed
client publication. Do not claim registry Metrics/Testkit already use Wire 0.12.

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
