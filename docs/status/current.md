# Current handoff

## Baseline and incoming work

Main is released Git version 0.4.1 (3f5748f), reported live by the maintainer.
Runtime and derive packages were first published at 0.3.0; evaluation and
adapter-fixture packages remain private. Compatible pending notes select 0.4.2;
package metadata remains 0.4.1. No staging, commits, own version changes, release
or publication effects were performed in this issue batch. Independent Host
0.27.4 lock work and concurrent Testkit 0.54 manifest/lock work are retained and
excluded from this batch's changes. Early Box evidence uses Testkit 0.53; the final
Rust 1.99.1 installed run uses the current 0.54 graph.

## Bounded boxes — pending 0.4.2

[Issue #7](https://github.com/dragginzgame/ic-wire/issues/7#issuecomment-6249690535)
now has a local implementation: `Box<T: Wire>` reuses T's schema, bytes, nodes
and depth, adding `size_of::<T>()` plus nested heap to static admission. Overflow
refuses and zero-sized boxes add no allocation. Serde's ordinary Box allocation
can still abort on allocator exhaustion; no general OOM recovery is promised.
No new schema case, derive path, runtime dependency or TypeScript wrapper is added.

Sixteen finite tests pass on Rust 1.99.1 and 1.91.0, including exact boxed/unboxed
frames, nested boxes, early heap refusal, malformed/trailing input and saturation.
Both compilers pass twelve diagnostic subjects and a positive control; recursive
boxed schemas and Box<String> refuse. Strict runtime/evaluation Clippy, Rust 1.91
Wasm, local Make/formatting, pins/boundaries and documentation checks pass.
TypeScript passes ninety vectors and 17,524 differential mutations.

The existing browser fixture uses boxed query records/response fields and an
overflow enum payload. Its Chromium/PocketIC 16.0.0 report matches every prior
non-provenance field, including exact vectors and bundle size; retained-build
freshness checks pass. No full CI, native macOS or Auth acceptance ran. Auth's
source-breaking adoption awaits producer publication and its own qualification;
no sibling was edited and no cycle/instruction/Wasm savings is claimed.
Logs and task before-images are under `/tmp/ic-wire-box-*`.

[Issue #5](https://github.com/dragginzgame/ic-wire/issues/5) is closed: native
admission shipped in 0.3.4 and its exact-source Linux/macOS acceptance
[run](https://github.com/dragginzgame/ic-wire/actions/runs/39031765826) passed.
Receipt consolidation #9/PR #10 remains separate work; its three hosted jobs pass,
but it has not been merged here.

## Development compiler — pending 0.4.2

[Issue #8](https://github.com/dragginzgame/ic-wire/issues/8#issuecomment-6249741285)
selects Rust 1.99.1 (`58cf8f9c3`) in the development file, explicit CI input and
setup guide. Rust 1.91 remains the package/CI floor; historical measurements and
snapshot-owned illustrative versions are unchanged. Existing installed components
were reused. Strict runtime/evaluation Clippy, sixteen finite/three pipeline
tests, Wasm, twelve diagnostic subjects/control, ninety TypeScript vectors and
17,524 mutations, formatting, local Make, boundaries/pins and doc checks pass.
The installed Chromium/PocketIC report records 1.99.1 and matches all unboxed
non-provenance fields on the current Testkit 0.54 graph. Rust 1.91 host checking
also passes on that graph. No full local CI or native macOS execution ran; #8
stays open for delivery and the new hosted matrix. Logs: `/tmp/ic-wire-rust1991-*`.

## Shared TypeScript emission and build facts

The host-only `typescript::generate_registry` emits native registry aliases and
typed method objects with the same type renderer as finite `generate`. Reflection,
endpoint identities, resource allowances and scalar checks remain consumer-owned.
Registry serialization preserves own `__proto__` names. No runtime bytes,
production dependencies or finite admission rules change.

IcyDB's isolated caller removes its duplicate type/emission code, retaining all
reflection and scalar overlays. Actual metadata, both runtimes and 90 vectors
are byte-identical before/after; strict TypeScript and all twelve endpoints pass
3,111 truncations plus scalar/malformed/limit controls. This used the real generator
source in a temporary package: the captured baseline's normal build helper failed
against the selected Build constructor/environment API. Independent sibling work
subsequently changed that helper; its moving workspace is not qualified here.
Wire 0.4.1 now delivers the shared emitter; the isolated IcyDB caller still needs
owner integration and fresh registry-only qualification. No sibling was edited.
The isolated Testkit patch reuses its existing prepared contribution, selects
published Build 0.17.2 and passes nineteen driver tests plus strict Clippy.
Reviewable source patches and before-images are under `/tmp/ic-wire-consolidation-*`.

Published Build 0.17.2 lacks both retained evidence and diagnostics used by Wire.
Keep the existing Git pin and source/freshness/export checks until those APIs
and borrowed retained compiler facts ship. Existing owners are
[Build #58](https://github.com/dragginzgame/ic-build/issues/58) and
[Build #66](https://github.com/dragginzgame/ic-build/issues/66).
The maintainer-authorized [consumer feedback on #58](https://github.com/dragginzgame/ic-build/issues/58#issuecomment-6249237464)
records the published API gaps, retained-facts requirement and acceptance checks.

Focused checks pass: strict runtime Clippy, fourteen Rust 1.91 runtime tests,
Wasm with TypeScript enabled, 84 TypeScript vectors and 16,384 differential
mutations, direct boundaries/pins and document links. No broad CI, installed
PocketIC/browser rerun, native macOS or live-IC qualification is claimed.
The disposable local Make/formatting fixture passes with the shared emitter included.

## Wire-only codec and retained integration

The maintainer selected no owned Candid code or direct dependencies, while
explicitly retaining browser and installed Wire tests. Candid control DTOs,
staged comparison decoding, measurement endpoints, the raw_rand management probe,
comparison host and `evaluate-runtime` are removed. Recorded reports retain their
original identities and measurements; their reproduction commands are historical.
There is no universal ten-times-faster or current benchmark claim.

All owned codecs and browser endpoint payloads use IC Wire. Generated browser
clients send raw Wire bytes through HttpAgent. Zero-argument/manual-reply CDK
entries now select `hidden = true` to suppress unused Candid service metadata,
including in the private generator. This keeps callable Wasm exports and CDK
executor contexts. Direct Candid and its policy exception are removed.

Published registry Platform 0.1.12 and its coordinated macros 0.1.12 now own
fixture entrypoint generation and qualified CDK helpers. Browser attributes and
the private adapter generator use `ic_platform::cdk::{query, update}`; allocating
argument/caller and checked controller helpers use `cdk::api`. Their original
CDK implementations, 0/1/panic behavior, executor contexts and manual/hidden
exports are preserved. Owned codecs remain Wire-only. The selected `cdk` feature
also enables `candid`, `executor`, `ic0`; no parser, Agent or crypto is selected.

The rollback fixture's stable read/write/grow now use `ic_platform::cdk::stable`,
whose published exports are the original CDK 0.20.3 helpers. Root/member direct
CDK declarations and the feature edge are removed; the shared package policy now
rejects direct CDK-family dependencies everywhere. The existing eight-byte format,
growth policy and error/trap semantics are unchanged
([Platform #20](https://github.com/dragginzgame/ic-platform/issues/20)).
No storage implementation/migration, SDK copy or runtime facade was introduced.
The [0.1.10 adoption on #1](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6250235797)
records the completed generator/helper work and installed consumer evidence;
fleet-wide acceptance remains owned upstream.

Candid remains transitive in CDK, the browser SDK and Testkit/PocketIC. This is the
explicitly selected test/transport exception, not a production codec mode.
The normal runtime/derive graphs remain independent of Candid, CDK and Platform.
Shared Tooling checks direct aliases and normal/build/dev/target declarations;
unused-root catalogs, enabled features, transitive and deployment exclusions still
require separate inspection. No local dependency facade or executor was added.

## Delivered 0.4.1 ownership

The custom license is replaced with standard MIT in all four workspace packages,
metadata and current READMEs. Existing IcyDB MIT attribution remains intact.
The earlier package metadata/file-list checks passed; nothing was republished.
Release 0.4.1 selected Testkit 0.53; the independent pending 0.54 intake is
qualified separately in the final compiler/browser run above.

Shared Tooling 0.24.0 is adopted at cab12cd3dfa4fdb34846b25397f0d72881bb9ae0;
the receipt contribution remains separately pinned at
35775234560bd08f7e67f4feeaf339dc25fbd9e0. Role groups, paired formatting and
selected-source release metadata checks stay in place. Common Rust tools are
cargo-sort and cargo-sort-derives. Both snapshots and prior disposable setup,
formatting, hook and release checks passed; native macOS remains separate evidence.

Production diagnostics build/link only runtime and derive; the macro lane adds
the private adapter fixture. IC Build owns diagnostic execution and build
provenance; IC Host owns artifact inspection and bounded hashing; Testkit owns
PocketIC/browser sessions. IC Wire owns codec cases and integration expectations.
Earlier handoffs are [archived](through-0.4.0.md). Prior Platform migration logs
remain at `/tmp/ic-wire-platform*`; pre-retirement source is retained outside the
checkout at `/tmp/ic-wire-no-candid-before/`.

## Platform 0.1.12 intake

Both registry archives select 0.1.12 at `338b14baf97690abdfc0e386d6d5261b309a3ac8`.
Facade/macros SHA-256s are respectively
`e0ab384614a5d8fe251bc2c27c508263d4001d776edf31382b5b985a1a4ced2d` and
`9b5de9771169e34ca8da9bbb04c9c6196b1d2de8d2d0d21bc79d5921984ce6be`.
Published source/README confirm upstream stable-helper/error reexports; the
generator source and CDK/executor/ic0 requirements are unchanged. Only the three
stable-helper call paths change; no storage conversion or Candid checkpoint
helper is used. Unrelated pending work and package versions remain unchanged.

The incoming lockfile already selected both 0.1.12 registry packages; the final
lock change removes only the fixture's direct CDK edge. Manifests now require
0.1.12, with no direct Candid, CDK-family or ic0 declaration in any dependency
role, target or feature. The production Wire Wasm/TypeScript graph remains free
of Platform/CDK/Candid; the integration graph retains one CDK 0.20.3, executor
2.0.0, ic0 1.2.0 and Candid 0.10.38 through Platform and test tooling. Platform
features remain exactly `candid`, `cdk`, `executor`, `ic0`.

Focused evaluation tests/strict Clippy, development and Rust 1.91 host/Wasm,
eight macro diagnostic subjects/control, formatting, boundaries/pins and document
links pass. Installed PocketIC 16.0.0/Chromium trials pass with retained freshness
checks. Macro/browser report fields match 0.1.11 except provenance/Wasm identity.
The macro fixture now additionally proves first-page growth/write rollback when
a response codec trap occurs before any successful update; all sixteen existing
rejection cases and rollback of writes to an existing page still pass. Shared
checks enforce direct boundaries, not transitive deployed-Wasm exclusions;
the graph was separately inspected. No full CI, native macOS, live IC or unowned
callback/timer/cancellation qualification ran. Logs/before-images:
`/tmp/ic-wire-platform0112-*`.

## Earlier 0.1.11 intake

Registry search and both published archives select 0.1.11 at `ffc36b63`.
The facade adds upstream cycle and instruction/context-counter helpers, unused
by Wire's maintained fixtures. The macro generator source is identical to 0.1.10;
CDK/executor/ic0 requirements and feature selections are unchanged. No further
owned calls can migrate in this release; the existing stable-helper issue remains
the only direct CDK exception. No sibling checkout is a dependency or was edited.

The published facade/macros archive SHA-256s are respectively
`8d55106ea1d1ccd65bd67c7d2a681b7d8199f3c0206ae990e24589b3ce502bdf` and
`a2a7652251d6eee2c67653a8110cba93da8662b82e633325a125e710d2e0c25f`.
Only those two package identities change in the lockfile; unrelated pending
Host/Testkit and Box/compiler work is preserved. Owned Rust implementation and
wire contracts are unchanged.

`make check-canister-evaluation` passes native tests, strict all-feature Clippy,
development host/Wasm and Rust 1.91 host/Wasm checking. Eight macro diagnostic
subjects/control, formatting, direct boundaries/pins and document links pass.
Installed Testkit 0.54/PocketIC 16.0.0 macro and Chromium trials pass, with final
retained-build freshness verification. Macro behavior matches the retained
0.1.10 report except provenance/Wasm hash; browser results match the prior report
except provenance. Sixteen rejection/rollback cases remain covered. The first
sandboxed macro run could not bind localhost; the permitted rerun passes.

Locked metadata confirms one Platform/macros 0.1.11, CDK 0.20.3, executor 2.0.0,
ic0 1.2.0 and Candid 0.10.38. Platform features remain exactly `candid`, `cdk`,
`executor`, `ic0`; native and browser-only graphs have no direct fixture-CDK edge.
Wire's normal Wasm/TypeScript graph remains Platform/CDK/Candid-free. Shared
checks still do not enforce feature/target restrictions or transitive deployment
exclusions; those were inspected separately. No full CI, native macOS, live IC or
unowned callback/timer/cancellation qualification ran. Logs and before-images:
`/tmp/ic-wire-platform0111-*`.

## Earlier 0.1.10 validation

Published facade 0.1.10 and macros 0.1.10 identify registry source `4bf73410`.
Archive SHA-256s are respectively
`1d3eaa384ad518116caaece25b1aca069473b7010e46e17d2fa23f0a3e1d2898` and
`1f1157909c578de096910a18013d49f31772e039417032fe66b8e6639283cd9e`.
Actual published APIs/features/README were inspected. The lockfile changes only
Platform's version/dependency edges and the coordinated macro package; unrelated
pre-existing records and pending Box/compiler/Testkit work are preserved.

Strict evaluation Clippy, three native pipeline tests, Rust 1.91 host/Wasm,
eight macro diagnostic subjects/control on 1.99.1 and 1.91, formatting, local Make,
direct boundaries/pins and documentation checks pass. The first disposable Make
fixture exposed target-table ordering; canonical formatting fixed it and the
rerun passes. The missing stable API probe fails as expected with E0432.

Installed Testkit 0.54/PocketIC 16.0.0 macro and Chromium trials pass, including
sixteen rejection/rollback cases. Browser results match the immediately prior
non-provenance fields. Macro results match the retained 0.1.9-era baseline except
provenance/Wasm hash; the immediate target report had been cleaned. Exact bytes,
auth/refusal order, sync/async contexts and exports are preserved. Retained-build
freshness checks finish successfully. No full CI, native macOS, live IC or
unowned awaited-call/timer/cancellation qualification is claimed.

Metadata/declaration scans cover aliases, build/dev/target edges and features;
one CDK 0.20.3, executor 2.0.0 and ic0 1.2.0 identity remain. The normal Wire
Wasm/TypeScript graph stays Platform/CDK/Candid-free. Shared checks enforce the
package exception, not its optional/target restrictions or transitive deployment
exclusions; those were inspected separately. Candid remains transitive in fixtures.
Logs and before-images: `/tmp/ic-wire-platform0110-*`. Earlier Platform records
are [archived](platform-through-0.1.9.md).
