# Current handoff

## Delivered Core 0.13.2 / Endpoint 0.2.1

The maintainer's scoped deploy request is complete:

- **ic-wire / ic-wire-derive 0.13.2**: core-v0.13.2,
  `eff8c18d053506be5e0d3a8676f1e48ae9ce99d0`.
- **ic-wire-endpoint 0.2.1** remains unchanged at endpoint-v0.2.1,
  `a6cfed5b367fff84997985a357ceeb6731ec5400`. No new Endpoint release/upload.

Source preparation 1029a8f contains the reviewed 28 files: eight tooling/docs
updates and 20 evidence files. Core's standard patch release passed the full real
local `make ci` gate in **532 s**, finalized metadata, committed/tagged and pushed
main/tag atomically. Publication dry run and derive/Core uploads passed. Both
non-yanked registry checksums match downloaded/upload archives; clean VCS
provenance equals the exact tag. [Delivery evidence and reproduction](../evidence/published-core-0132/README.md)
retain these observations. Effects are complete; do not replay them. The checkout
has no pending release entry; later work creates its own numbered draft.

## Current suppliers and qualification

Build/Host **0.73.2**, Testkit **0.75.1**, arithmetic-only Metrics **0.24.1** and
the managed Build CLI are selected. The actual lock keeps Platform/macros/guard
**0.15.3**, compatible minimum 0.15.2, and canonical Types **0.15.1**.
[Adoption evidence](../evidence/tooling0732-metrics0241/README.md) retains source
review, diagnostic controls, qualification/MSRV, isolated real group editor
checks and installed endpoint limits/malformed refusals/transaction/reply rollback.
Production endpoint Wasm dependency graphs equal the prior delivered source.
No unused Host Tools/native-measurement package was introduced.

Published registry-only Rust 1.91 qualification passes four schema-only and five
combined endpoint tests, standalone derives, both Wasm graphs and exact generated
Rust/TypeScript Principal/Account bytes/malformed refusals. Schema-only has no
execution/macros; combined has one guard. Production Wasm excludes Candid,
PocketIC, external executor, host serde_json and fuzz/profiler tools. A separate
executed typed/schema test composes four Metrics schemas, Principal/Account,
Endpoint Contract and Testkit Limits/Error/query/update signatures, with one each
registry Core/derive/Types/Testkit/Metrics identity and no patch/local Wire supplier.
This typed API test does not execute remote calls. Both native metadata and
Testkit now resolve Metrics 0.24.1 rather than separate Metrics versions.

[Release hosted CI](https://github.com/dragginzgame/ic-wire/actions/runs/41231267294)
was in progress when inspected; no new hosted/macOS matrix pass is claimed.
The [previous delivery's final hosted run](https://github.com/dragginzgame/ic-wire/actions/runs/41227979803)
completed successfully. Local release CI and hosted observations are separate.

## Maintained boundaries and closed issues

The recorded Shared snapshot is **0.27.21** at
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` (the supplying commit calls its follow-up
0.27.22 without changing VERSION). Canonical export owns all 119 payloads.
Root Cargo catalogs all six members once, grouped by release/changelog, including
private/unused aliases. Explicit core/endpoint release groups and ordinary whole-
workspace Cargo cleanup remain intact. [Earlier joint delivery](../evidence/published-pair-0131/README.md)
retains Core 0.13.1 and Endpoint 0.2.1 qualification.

#36 is closed: the [approved stable identity boundary](../stable-identities.md)
is published. Core owns one Serde/Postcard codec, Limits/Error, the sole schema
trait and canonical Types implementations. Endpoint owns Contract/attributes/
admission/trapping replies; Platform owns execution. No compatibility lane,
wrapper, schema mirror or alternate runtime is added. #39 is closed and shipped
in Core 0.13.1: native fuzz/profilers are explicit private qualification options;
default/native/Wasm graph checks enforce isolation. [Qualification evidence](../evidence/qualification-tools-0131/README.md)
retains codec tests, 1,000 fuzz runs and profiler/conflict controls. #37 remains
superseded.

Protocol bytes, field order, budgets, primitive/malformed acceptance, rollback and
exact retry/hash inputs are unchanged. Principal stays bounded bytes max 29;
Account absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes stay
distinct. Collection/custom-Serde normalization stays consumer-owned. No runtime
or generator implementation changed, so no fresh performance benchmark applies
and no instruction/Wasm-size/build-time saving is claimed. Existing
[Candid measurements](../evidence/candid-comparison-0725/README.md) retain their
original inputs; the prepared server lacks the instruction-trace API.

Root Cargo remains virtual with literal member versions. Root CHANGELOG indexes
[Core](../../releases/core/CHANGELOG.md), [Endpoint](../../releases/endpoint/CHANGELOG.md)
and [coordinated history](../../releases/history.md). No sibling checkout or
artifact cleanup was touched. Evidence manifests remain inactive snapshots.
Future delivery requires its own scoped authorization.
