# Current handoff

## Delegated patch delivery

The maintainer requested deployment: Git releases to origin/main and crates.io
publication, using the established explicit groups. Select compatible Core
**0.13.1** and Endpoint **0.2.1**; deliver/publish Core before Endpoint. Current
package metadata is still Core 0.13.0 / Endpoint 0.2.0 until standard preparation.
The scoped source commit, full local release gates, metadata, tags and publication
are authorized. No sibling edits or post-release cleanup are authorized.

Initial checks found remote main at 01bc240, no candidate tags or registry uploads,
and no active build sharing Wire target. Retain attempt logs under
`target/deploy-0131/` and standard .git/release-state journals. Historical plans
for older unrelated 0.2.1 attempts are retained and do not select this delivery.

## Current issue cleanup

[#39](https://github.com/dragginzgame/ic-wire/issues/39) is closed with the fix in
the local working tree:
move the byte-identical fuzz target into private qualification with explicit native
fuzz-runtime selection; make native benchmark profilers opt-in through allocations
and heap. Ordinary codec/benchmark/qualification graphs exclude these tools.
The maintained graph check also verifies explicit inclusion and Wasm exclusions.
No new workspace/package or codec API/bytes/budgets/acceptance change is introduced.
Core pending notes remain 0.13.1; own versions and registry selections are unchanged.

[Qualification evidence](../evidence/qualification-tools-0131/README.md) retains
49 codec tests, four qualification tests, three default Candid corpus controls,
a 1,000-run address-sanitized fuzz smoke, both nonzero profiler smoke tests,
Clippy/Rust 1.91 checks and both expected mutual-exclusion diagnostics. Syntax,
formatting, pins/boundaries and links pass. Leak shutdown was explicitly disabled
for the local fuzz run. No full local CI/macOS matrix, new instruction benchmark,
build-time or Wasm-size saving is claimed. Changes remain uncommitted.

#36 is closed with delivered release/hosted evidence and the matching published
Testkit/Metrics consumer qualification. #39 is also closed after the completed local
fix and focused acceptance; this does not claim a new release or hosted matrix.

## Current uncommitted adoption

Shared Tooling now selects latest clean committed revision
**`143a9abb9ae81feae9400ff3844b03b1ddfdd75a`** through the canonical exporter.
Upstream's committed VERSION is **0.27.21**; the commit/changelog describe a
0.27.22 follow-up without changing VERSION. The snapshot records that exact
source identity and annotation; no owned payload was edited locally.

Root Cargo now catalogs all six workspace members in contiguous Core/Endpoint
blocks, each with a changelog line. The duplicate header map is removed. Private
qualification members also have path/version declarations and Core policy aliases,
so release preparation updates unused catalog entries. External dependencies retain
production/native-schema/proc-macro/test role headings. For that catalog-only step, full Cargo metadata
and Cargo.lock matched its incoming work. The subsequent #39 cleanup above changes
qualification dependency edges, with registry/package versions preserved. Host/Cargo catalogs and prepared tools are unchanged.
Normal Cargo cleanup retains its whole-workspace scope; none was run.

Current registry suppliers: Platform/macros/guard **0.15.2**, canonical Types
**0.15.1**, Build/Host **0.73.1**, Testkit **0.75.0**. Testkit retains arithmetic-only
Metrics **0.23.2**. The earlier supplier lock edits are preserved; the obsolete
native Platform 0.13 entry is gone. Build's managed CLI editor is prepared at
0.73.1 and passes offline admission. No staging, commits, releases, publication
or sibling checkout edits were performed for this adoption.

Own versions stay Core **0.13.0** / Endpoint **0.2.0**. Pending notes select
compatible **Core 0.13.1 / Endpoint 0.2.1**: public Types remains on 0.15, with no
codec/schema byte, budget, handler API or admission/rollback contract change.

## Qualification and limits

[Latest snapshot adoption](../evidence/shared143a9-catalog/README.md) records
impact/refresh, complete-catalog and lock/metadata preservation, local formatting,
pins/boundaries/tool admission and disposable Make/both-group release checks.
This is build/release metadata work with unchanged runtime/generator inputs, so
no new runtime performance benchmark applies. Earlier focused runtime evidence
below remains tied to its original suppliers.

[Current tooling evidence](../evidence/shared02720-testkit075/README.md) retains
snapshot integrity, pins/boundaries/formatting, offline tools, disposable Make and
both real release-group adapters, Testkit 0.75 installed limits/rollback and
Rust 1.91 benchmark examples. Release fixtures substitute expensive validation
and uploads; their Git effects stay in disposable repositories. Essential logs
and frozen consumer inputs live under docs, outside removable target outputs.

The registry-only consumer composes Testkit 0.75 typed query/update APIs,
Endpoint 0.2, Wire/Derive 0.13, Platform 0.15.2 / Types 0.15.1 and all four native
Metrics 0.24 schemas on Rust 1.91. One registry Core/Derive/Types identity resolves;
no Cargo patch, wrapper, local supplier or source-aligned manifest is needed.
Its DTO derives Serialize/Deserialize/Wire with Metrics, Principal and Account
fields. Testkit also selects arithmetic-only Metrics 0.23.2; these are two Metrics
versions sharing the selected Core identity. Root production gains no dependency.
This supersedes the old source-only supplier qualification, not application adoption.

The production endpoint Wasm normal/build graph is equal to the incoming graph.
Testkit's 28 src files are unchanged from 0.74. Prior relevant runtime evidence is
reused; no new runtime/generator cost or performance saving is claimed.
[Platform supplier qualification](../evidence/platform0152-testkit074/README.md)
retains standalone derives, Rust 1.91 host/Wasm, exact Rust/TypeScript frames and
malformed refusals, future identity isolation, 11 endpoint diagnostics/strict
Clippy, installed transaction oracles and default Wire/Candid corpus controls.
Core/schema-only exclude runtime machinery; deployed Endpoint has one guard and
excludes host serde_json, Candid, PocketIC and external executor dependencies.

That matched artifact comparison keeps opt-z **100,665 B total / 86,766 B code**
and opt-3 **94,073 B total / 81,127 B code** (0 B / 0%). The explicitly prepared
PocketIC server passes installed limits/rollback but lacks the instruction-trace
API. Fresh instructions/cycles, repeated cost samples and a new full Candid matrix
remain unmeasured. No full local CI or new hosted/native matrix ran for these
uncommitted adoptions. Historical measurements retain their original inputs.

## Delivered releases and issue status

The previous delegated Git/crates.io delivery completed Core/Derive **0.13.0**
(`core-v0.13.0`, `65d1b91600d64a1e4255774f6bfc8ad9752a63cb`) and Endpoint **0.2.0**
(`endpoint-v0.2.0`, `dfd4978bf6792d12d3fdab6ee9a7b2affeabde1c`). Each standard
release passed full local CI and atomic main/tag delivery; dry runs and actual
publication succeeded, Core before Endpoint. All three non-yanked public registry
checksums match the upload archives. [Delivery proof](../evidence/published-pair-0130/README.md)
retains exact identities and registry-only Rust/MSRV/Wasm/TypeScript acceptance.
Do not replay those effects.

The delivered source's final [12-job hosted matrix](https://github.com/dragginzgame/ic-wire/actions/runs/41212986937)
passed on Linux and ARM/Intel macOS. #36 is closed; that historical result does
not qualify the new dirty adoption as hosted-green. Published Testkit 0.75 and Metrics 0.24 now supply the
matching typed/schema producers. Application/Blob adoption remains consumer-owned.
#37 remains superseded, without a legacy integration lane. Do not reopen the
completed Wire 0.11 scope of Tooling #274.

## Boundary, protocol and release ownership

The [approved boundary](../stable-identities.md) keeps codec, schema trait,
Limits/Error and canonical Types schema impls in Core. Endpoint owns Contract,
attributes, admission and trapping replies; Platform owns exports/execution.
Core platform-types selects canonical Types without the runtime facade/CDK.
Move ic_wire::endpoint imports to ic_wire_endpoint and select Endpoint typescript
for native Contract schemas. Future runtime-only updates retain Core identity
while the Types supplier is compatible. There is one production Serde/Postcard
codec and one schema trait; no compatibility or alternate encoding lane.

Protocol unchanged: frame bytes, field order, budgets, primitive/malformed
acceptance, handler transactions and original retry/hash inputs. Principal stays
bounded bytes (max 29); Account stays owner then optional 32-byte subaccount.
Absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes remain distinct.
Collection/custom-Serde normalization is consumer-owned; retain exact retry bytes.

Root Cargo is virtual with literal member versions. Explicit core/endpoint
policies use Build group manifests and shared plans/receipts; missing, empty or
unknown release/resume/publication groups refuse before effects. Root CHANGELOG
indexes the [core](../../releases/core/CHANGELOG.md),
[endpoint](../../releases/endpoint/CHANGELOG.md) and
[shared historical](../../releases/history.md) ledgers. Derive publishes before
Core. New delivery requires authorization; the completed release delegated no
unrelated effects.

[Extraction evidence](../evidence/stable-identities-0120/README.md) retains
ownership, endpoint rollback, diagnostics and group recovery proofs. The future
fixture holds Core/Types fixed while simulating another runtime/Endpoint identity;
it makes no promise about unpublished upstream releases. Historical
[Blob pairing](../evidence/published-pair-0110/README.md) and
[Candid matrix](../evidence/candid-comparison-0725/README.md) remain bound to their
recorded versions. Candid stays development-only; evidence Cargo files are snapshots.
