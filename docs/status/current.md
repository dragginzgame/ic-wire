# Current handoff

Core **0.21.5** (`e249cdf`, tag `core-v0.21.5`) is delivered to origin/main and
crates.io. Both Core packages were independently downloaded from the registry.
All 20 release stages passed in recovered batch-W7zKUDXE; normal main is
reconciled. Endpoint remains **0.12.2** (`2a51973`); its source and dependency
requirements were unchanged. Issue 59's hosted compiled-tool caching was pushed
in `29d9778`; no product release is required solely for this CI change. See the
[release cost review](../guides/group-validation.md#release-cost-review-12-november-2026)
for the no-go on candidate lint reuse and outstanding hosted timing qualification.
Pending issue 60 work adopts Tooling 0.41.0 and Rust 1.100.0; Core 0.22.0 and
Endpoint 0.13.0 notes record the compiler-floor change. Nothing is published.

Deployment batch `batch-5W4C7_rt` captured source `8e3dd70` and stopped at
`check-benchmarks` integration admission: Testkit 0.101.0 selects Types 0.19.5
alongside product Types 0.20.0. Product-boundary and installed smoke checks passed,
but they do not satisfy the stricter complete native graph contract. No version
preparation, tag, push or upload occurred. This historical refusal came from a
whole-native-graph Types count. Tooling 0.41.0 now admits isolated native supplier
generations; the working-tree check instead verifies the supplier-pruned typed
graph and the full graph's Wire identity/Runtime guard. Preserve the failed batch
and `/tmp/wire022-resume.log`; its captured source still has the old policy.
The initial setup-only npm mismatch was resolved using pinned npm 12.2.0 under
`target/release-tools/npm-12.2.0`; the shell default is 12.2.1. Changed supplier
inputs will need a fresh gate because stage reuse is not qualified.

Batch `batch-ioqO0BTf` captured `4e1c4cd` and stopped before release effects at
`check-tooling`: the local Make fixture still expected the retired Clippy warning
flags. The fixture now verifies Cargo warning denial for both groups and
publication callers; the disposable routing/formatting suite passed. Preserve
`/tmp/wire022-fresh-deploy.log` and that failed batch. A fresh source capture is
required; neither failed batch may be resumed to deliver the corrected source.

## Delivered behavior

Core 0.21.5 includes browser record-field reuse, terminal first-byte numeric
varint decoding and checked Array-length writing. Regenerate browser clients.
Exact codec bytes, identities, limits, callback rules and acceptance are unchanged:
`7f` stays 127; `80 00` still accepts zero; a Proxy-reported 2^32 Array length
retains `80 80 80 80 10` instead of silently truncating to `00`.
The Rust codec and Wasm runtime are unchanged by these JavaScript optimizations.

The [browser studies](../evidence/typescript-writer.md) record matched measurements,
small-control regressions and transformed-JavaScript size tradeoffs. Strict
TypeScript, Rust/browser vectors, mutation/admission checks, ownership and
simulated-endian tests passed again in the fresh release gate. Simulated endian
coverage is not native big-endian execution.
Rejected direct text writing, key-loop, BigInt shortcuts and unguarded length
writing remain research; do not promote their best-case results in isolation.
The checked length helper is the qualified successor to the unguarded experiment.
Other rejected Rust experiments remain at [cost experiments](../evidence/cost-experiments.md)
and [encoder follow-up](../evidence/encoder-followup.md).

## Tooling and suppliers

Both workspaces select Toko Tooling 0.41.0; its authenticated snapshot retains
96 files. Rust 1.100.0 is the minimum and pin (rustc `0ef1bc386`, Cargo `3acc79c0c`,
LLVM 23.1.5). Both roots inherit the common lint policy; the benchmark's local
unsafe-export exception retains equivalent checks. The dated sanitizer-only
nightly is unchanged. Stage reuse and final-candidate opt-ins stay disabled.
The earlier 0.40.1 adoption changed deployment guidance only. Before delivery, inspect
nested gates and retain the existing focused wiring proofs. On failure, identify
invalidated inputs and disclose unsupported recovery/full-rerun scope explicitly;
use the retained timings in the release cost review, not an assumed cache saving.
Installer/tool admission, snapshot verification, pins and documentation links pass
for that adoption. An unrelated qualification-lock change to rustls-webpki 0.103.17 was
observed and preserved during adoption; the Platform adoption below subsequently
compiled that selection and ran the installed Endpoint smoke check. This is not
a separate TLS behavior qualification.
Linux public tests, strict Clippy/Rustdoc, host/Wasm and isolated consumers pass.
The 26 native and 20 Wasm benchmark cohorts pass strict linting, with allocation
and heap modes checked separately; no performance measurements were run.
Policy fixtures reject unsafe code and ignored must-use results. Direct-rustc
diagnostics now read Cargo's artifact directories instead of assuming `deps/`;
Core and Endpoint compile-fail controls pass with Rust 1.100's output layout.
Native macOS proof remains pending; historical 1.99 performance is not remeasured.
Codec bytes, identities, budgets and acceptance remain unchanged. No performance
or Wasm-size saving is claimed from this compiler/lint migration.

The delivered release qualified Types 0.19.5, Runtime/macros/guard 0.21.3, native Platform
0.31.3 and Host 0.76.5 lock selections. Pending qualification now selects Build 0.84.0 and
Testkit 0.102.0, with the release editor aligned at Build 0.84.0. Their required
Host 0.77.0 and native Platform 0.32.0 replace the prior private suppliers;
the qualification lock now has only canonical Types 0.20.0.
Testkit owns PocketIC setup/lifecycle and exposes native client 16.1.0 handles.
Installed Endpoint calls, limits and rollback passed with prepared server 16.0.0
before capture. The resumed full gate requalified the frozen supplier graph.
Candid 0.10.40 is a private comparison oracle; its bigint 0.4.8 exception remains.
No published package selects Candid.

Pending adoption selects published Types 0.20.0 and Runtime/macros/guard 0.22.0
in both workspaces. Align consumer Types/Runtime requirements together; these
versions require Rust 1.100.0. Core 0.22.0 and Endpoint 0.13.0 notes cover this
Rust identity cut. Schema and codec bytes, limits and admission remain unchanged:
Account absent/present subaccounts stay `00` / `01 20 + 32 bytes`. Testkit
0.102.0 now shares Types 0.20.0 with product/schema/Endpoint graphs. The revised
native admission still permits independently owned older supplier identities
while rejecting a direct/local identity split or duplicate Runtime guard. No supplier patches or
new compatibility wrappers are introduced. Standalone Rust/TypeScript byte and
malformed-input checks, host/Wasm feature isolation, compatible current consumers,
Endpoint diagnostics/lints and opt-z Wasm build passed. Installed calls, limits
and rollback passed with prepared PocketIC 16.0.0; boundary and pin checks passed.
Published Endpoint 0.12.2 remains on Runtime 0.21 and is not a supported mixed
pair with the new Types generation. No performance/size comparison was run.

Tooling 0.41 adds Cargo dependency/inheritance lints and strict Rustdoc table
checks. `make deps-list` reports all seven local packages and release membership
offline. The display-only root Endpoint dependency alias and unused qualification
repository metadata are removed; all four public release members remain.
Testkit/Build and allocation profilers are native-only dev-dependencies of their
actual private tool/test consumers. Native dev builds may compile them without
execution features; ordinary library and Wasm graphs remain isolated. The benchmark Endpoint probe
is also a dev-dependency. Fresh opt-z standard-fixture builds before/after that
placement measured total/code bytes 1,746,489/1,661,306 → 1,746,379/1,661,196;
this single fixture check is not an application-size or instruction-saving claim.
No opt-3 or runtime performance comparison was run. Raw build inputs remain in
`target/evidence/tooling041-placement/` with separate before/after Cargo targets.
Public host/Wasm Clippy and strict Rustdoc, private tool/Endpoint Clippy, allocation/
heap/endpoint-input mode Clippy, installed Endpoint rollback and typed diagnostics
pass. No lint suppression or dummy imports were added.

Build 0.84 diagnostic controls and Testkit 0.102 installed Endpoint calls, limits
and rollback passed. The prepared Build 0.84 release editor rendered both actual
group plans and both locks into disposable candidates; parsed changes matched
selected version/dependency edits and preserved registry packages. No release
versions were changed.

Rust 1.100 follow-up is implemented locally: the public Clippy adapter uses
`CARGO_BUILD_WARNINGS=deny` without adding compiler warning flags. The existing
policy fixture rejects a real warning on fresh and cached artifacts and accepts
a clean control. Plain-JSON collectors and other strict callers are unchanged;
no release-time saving is claimed. The derive adapter parser decodes each literal
once via `trim_prefix`; all six supported adapter forms and repeated-prefix
refusals pass regression coverage. Matched actual-expansion studies in both
profiles save two allocations / 31 bytes per adapter; timing overlaps and an
opt-3 control regression prevent a general speed claim. See the
[measured prefix cleanup](../evidence/cost-experiments.md#derive-adapter-prefix-cleanup--13-november-2026).
Generated tokens, codec bytes, identities, limits and acceptance remain unchanged.
Custom allocators, Vec constructor rewrites, never-type spelling and DropGuard
remain rejected as unnecessary for this change.

Root owns four public packages; qualification owns three unpublished packages
and its own lock. `make deps-update` updates the two locks sequentially.
Hosted CI fetches both locks before offline checks; the compiler-free snapshot
check runs before Rust setup, with full admission in the native CI routes.
Optional supplier refreshes do not automatically require public releases.

## Evidence and cleanup recovery

External cleanup interrupted the first 0.21.5 gate by deleting its active frozen
checkout and validation output. No push/tag/publication had occurred. Durable
Git state retained the clean captured source `ca2ede9`. After the maintainer
confirmed cleanup stopped, the same checkout was restored from that index,
tools were prepared again and all 20 stages reran successfully. No failed-stage
reuse or validation bypass was used.

Preserve `target/release-frozen/batch-W7zKUDXE` and
`.git/release-state/frozen/batch-W7zKUDXE`. The latter retains release journals,
Git objects and the completed intent. The resumed log is
`/tmp/wire-0215-recovery-release.log`; the interrupted log is
`/tmp/wire-deploy-0215.log`. No cleanup by this agent is authorized.

Cleanup also removed historical raw artifacts: the checked-length and numeric
varint evidence directories, and `target/evidence/codec-matrix-core021`, were
confirmed absent. Committed topic reports preserve the measured results, but
missing sources/samples cannot be reconstructed or verified from summaries alone.
Do not describe those target paths as currently available. New performance work
must prepare fresh matched inputs; the fresh release gate is correctness evidence,
not a rerun of those historical performance studies.

The [latest full-matrix report](../benchmarks/candid-comparison.md#core-021-checkout-full-matrix--11-november-2026)
remains historical: 39 cohorts, 2,314 case entries, 48,610 IC rows; all 78 linked
Wasm size pairs favour Wire. It used Types 0.19.1, not this release's suppliers.
Ordinary-list controls retain large Candid gaps; no current full-matrix claim.

## Remaining contracts and recovery boundaries

Postcard issue 56 remains supplier-blocked: the live sparse index checked on
12 November still ends at 1.1.3 with normal Serde `derive` enabled.
Core 0.20 uses upstream Postcard, with consumer-owned collection normalization.
Retain original retry/hash bytes. Account present/absent bytes stay
`01 20 + 32 bytes` / `00`. Method IDs remain production; fingerprint macros
remain research. Schema naming is registration-independent since Core 0.21.0.
IcyDB storage/key encodings and downstream orchestration remain out of scope.

Historical [delivery receipts](../../releases/receipts/README.md) and Git history
own completed narratives. Failed batches batch-mzOGyRZA, batch-Oup1NG9b and
batch-1afAufRB are superseded; do not resume them. Completed batch-1i9z2IIG was
reconciled and its deferred lock reapplied. Never resume schema-1 0.17.5 with
schema-2 tooling. History was intentionally replaced; never re-upload published
versions to repair Git. Do not infer retained artifact availability from receipts.
