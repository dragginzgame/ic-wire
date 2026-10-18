# Current handoff

## Published Build boundary — pending 0.4.2

This isolated contribution starts at released `3f5748f` (0.4.1), without the
normal checkout's unrelated pending runtime/compiler changes. Evaluation now uses
published, non-yanked IC Build 0.17.3 from crates.io rather than the diagnostics
branch at `4d8549a`. Its source requires the complete environment and explicit
child custody; both are supplied by the evaluation adapter. Cargo.lock removes
the old engine source and all three Host 0.25 packages, retaining the other
selected dependencies. The new direct Process edge is evaluation-only. Runtime
bytes, endpoint contracts, budgets and report schemas are unchanged.

Rust 1.91 passes all-feature example checking, eleven codec and eight macro
negative diagnostics with passing controls. Strict development example Clippy,
formatting, dependency pins/boundaries and changed-document links pass. Prepared
PocketIC 16.1.0 runs pass installed query/sync/async macro calls, typed failure and
sixteen rejections, plus the Chromium browser trial on the actual registry graph.
Browser dependencies reuse the exact unchanged package lock and prepared modules;
no installation occurs. Local listeners require an elevated execution context.
These Linux focused checks do not establish full CI, native macOS or live-IC
qualification. Own package versions remain 0.4.1; integration/release are separate.
Evidence: `/tmp/ic-build-consumer-finish-20261018/`.

## Baseline and incoming work

Main is released Git version 0.4.0 (ce56d5e). Runtime and derive packages were
first published at 0.3.0; evaluation and adapter-fixture packages remain private.
Pending notes select 0.4.1: the production API, wire contract and MSRV are unchanged.
The maintainer retired the contributor-only comparison command. No staging,
commits, own version changes, release or publication effects were performed.
Unrelated pending Shared Tooling and MIT work is preserved.

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
The new Wire API needs publication
before registry-only adoption; no sibling checkout or registry claim is changed.
The isolated Testkit patch reuses its existing prepared contribution, selects
published Build 0.17.2 and passes nineteen driver tests plus strict Clippy.
Reviewable source patches and before-images are under `/tmp/ic-wire-consolidation-*`.

Build 0.17.3 now supplies the retained evidence and diagnostic APIs required
by these evaluation consumers; the published-boundary entry above records the
registry adoption and current qualification. Build remains the canonical owner
under [#58](https://github.com/dragginzgame/ic-build/issues/58) and
[#66](https://github.com/dragginzgame/ic-build/issues/66).

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

Published registry Platform 0.1.9 supplies Principals, ic0 argument-size checks
and raw reply append/finalization/rejection calls; only its ic0 feature is enabled.
Reply order and UTF-8 reject bytes are unchanged, including the existing transport's
empty-append guard. Direct ic0 is also removed. Evaluation retains CDK 0.20.3 for
entrypoints, allocating argument/caller helpers and checked controller results,
plus stable read/write/grow in the rollback fixture;
[Platform #1 — Support the CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1)
tracks the remaining runtime surface. The staged decoder adoption was superseded
by retiring comparisons, rather than an outstanding IC Wire requirement.
The [0.1.7 consumer audit on Platform #1 — CDK boundary](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6248552044)
records the exact helper/raw-export gaps, minimal example and acceptance checks.
The macro fixture now routes trap bytes and stable-page counts through Platform's
existing trap/stable64_size exports. Source inspection proves CDK uses the same
upstream calls; the balance format, growth/read/write and trap behavior stay intact.
The new Agent/Candid/parser APIs have no current owned Wire consumer; only ic0
remains selected, with the existing Principal and ic0 package identities.
The previous 0.1.8 ic0 module was byte-identical to 0.1.7; its added management
snapshot/status DTOs stay disabled and do not replace the remaining CDK APIs.
The [0.1.8 recheck on Platform #1 — CDK boundary](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6248715562)
confirms the same remaining helper/entrypoint scope; no distinct new gap was found.
The [0.1.9 adoption on Platform #1](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6249331133)
records the completed reply/reject migration and remaining APIs. Its raw controller
result cannot replace CDK's checked helper without copying its unexpected-result
panic policy. The private macro generator retains CDK attributes; no published
Platform entrypoint frontend exists. Storage/allocation migration remains out of scope.

Candid remains transitive in CDK, the browser SDK and Testkit/PocketIC. This is the
explicitly selected test/transport exception, not a production codec mode.
The normal runtime/derive graphs remain independent of Candid, CDK and Platform.
Shared Tooling checks direct aliases and normal/build/dev/target declarations;
unused-root catalogs, enabled features, transitive and deployment exclusions still
require separate inspection. No local dependency facade or executor was added.

## Other pending 0.4.1 work

The custom license is replaced with standard MIT in all four workspace packages,
metadata and current READMEs. Existing IcyDB MIT attribution remains intact.
The earlier package metadata/file-list checks passed; nothing was republished.
The incoming Testkit 0.53 selection remains for installed Wire qualification.

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

## Validation

Published Platform 0.1.9 is selected from registry source `491c63d2`, archive
SHA-256 `bdfb66170e95bae421449fbd50e3e2a26196fbd6277ae13556026bf3001974c3`.
Its published README and feature/API surface were inspected; only its version and
checksum change in Cargo.lock. Strict evaluation Clippy, three native pipeline
tests, Rust 1.91 host/Wasm, direct boundary/pin checks and documentation checks pass.
Metadata/declaration scans include aliases, normal/build/dev and target-specific
edges; exactly ic0 is selected, no direct Candid/raw ABI dependency is restored,
and one upstream executor 2.0.0/ic0 1.2.0 identity remains. Production Wire's
Wasm/TypeScript graph stays Platform/CDK/Candid-free. Shared direct checks do not
enforce feature/transitive deployment exclusions; those graphs were inspected
separately, without claiming Candid absence from fixture Wasm.

Installed PocketIC 16.0.0 macro and Chromium browser trials pass. Both reports
match all prior non-provenance/Wasm-hash fields, including exact bytes, exports,
auth failures, typed replies and sixteen macro rejection/rollback cases. Testkit
retains simulator ownership. Reports and retained-build
checks finish successfully. No broad CI, native macOS or live-IC qualification ran.
Logs: `/tmp/ic-wire-platform019-*`; prior reports: `/tmp/ic-wire-platform019-reports-before/`.

The Platform 0.1.8 follow-up passes formatting/doc links, direct boundaries/pins,
strict evaluation Clippy, three native fixture tests and Rust 1.91.0 host/Wasm.
Only Platform's version/checksum changed in the lockfile. Metadata and graphs
confirm exactly ic0 enabled, no direct Candid/parser, unchanged executor/ic0
identities and production Wire independent of Platform/CDK/Candid. Installed
trials were not repeated for this version-only change; the 0.1.7 results below
remain separately qualified. No broad CI, native macOS or live IC ran.
Logs: `/tmp/ic-wire-platform018-*`.

The Platform 0.1.7 adoption passes direct boundaries/pins, formatting/doc links,
strict evaluation Clippy, three native fixture tests and Rust 1.91.0 host/Wasm.
Installed PocketIC 16.0.0 macro and Chromium browser trials pass: all prior macro
fields except provenance/Wasm hash match, including exact bytes, callable exports,
failures and 16 rejection/rollback cases. All browser non-provenance fields match.
Only Platform's version/checksum change in the lockfile; metadata confirms only
ic0 enabled, no direct Candid/parser and one upstream executor/ic0 identity.
Production Wire's Wasm/TypeScript graph remains Platform/CDK/Candid-free. No
broad CI, native macOS or installed-IC qualification ran for this follow-up.
Logs: `/tmp/ic-wire-platform017-*`; prior reports: `/tmp/ic-wire-platform017-reports-before/`.

The Platform 0.1.6 follow-up passes formatting, boundary/pin and documentation
checks, strict evaluation Clippy, three native fixture tests and Rust 1.91.0
host/Wasm checks. The lockfile changes only Platform's version and checksum;
the selected ic0/Principal/executor identities and enabled features are unchanged.
Metadata and graphs confirm no direct Candid and no Platform Candid feature.
Installed trials were not repeated for this dependency-only follow-up; the
0.1.5 installed evidence below remains separately identified.
Logs: `/tmp/ic-wire-platform016-*`.

The Wire-only retirement on Platform 0.1.5 passed the following checks:

Direct-boundary/pin checks, formatting/doc links, retained native tests and Clippy,
Rust 1.91 host/Wasm, both diagnostic suites on development/MSRV compilers, and
disposable Make/release fixtures pass. TypeScript conformance passes 84 vectors
and 16,384 differential mutations. Installed PocketIC 16.0.0 macro and Chromium
browser trials pass: callable exports, bytes, typed failures, controller rejection
and 16 macro rejection/rollback cases match the prior reports. Only provenance
and the macro Wasm hash are excluded from those result comparisons.

Cargo metadata confirms no direct Candid/parser declaration; Platform enables only
ic0. The production Wasm/TypeScript graph excludes Platform/CDK/Candid, while the
retained test graph documents the expected transitive SDK dependencies. The
pre-existing redundant license/license-file manifest warnings remain. No broad
CI, native macOS or installed-IC qualification ran. Logs use
`/tmp/ic-wire-no-candid-*`; prior installed reports are under
`/tmp/ic-wire-no-candid-reports-before/`.
