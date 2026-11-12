# Current handoff

Core **0.21.5** (`e249cdf`, tag `core-v0.21.5`) is delivered to origin/main and
crates.io. Both Core packages were independently downloaded from the registry.
All 20 release stages passed in recovered batch-W7zKUDXE; normal main is
reconciled. Endpoint remains **0.12.2** (`2a51973`); its source and dependency
requirements were unchanged. Pending issue 59 changes add hosted compiled-tool
caching; no product release is required solely for this CI change. See the
[release cost review](../guides/group-validation.md#release-cost-review-12-november-2026)
for the no-go on candidate lint reuse and outstanding hosted timing qualification.

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

Both workspaces select Toko Tooling 0.39.17. Its authenticated crates.io snapshot
retains 96 payload files, including the newly exported Cargo-tool cache key. Tracked Rust sources
already satisfy the no-sibling-foo.rs/foo-directory rule. Installation, tool and
snapshot admission, pins and CI callers passed. Exact-pin exceptions match.
Rust 1.99.1 remains minimum and pinned; only the opt-in sanitizer fuzzer has its
separately approved nightly. Stage reuse and final-candidate opt-ins stay disabled.

The release qualifies Types 0.19.5, Runtime/macros/guard 0.21.3, native Platform
0.31.3 and Host 0.76.5 lock selections. Qualification selects Build 0.83.0 and
Testkit 0.101.0; the release editor remains independently selected at Build 0.81.0.
Testkit owns PocketIC setup/lifecycle and exposes native client 16.1.0 handles.
Installed Endpoint calls, limits and rollback passed with prepared server 16.0.0
before capture. The resumed full gate requalified the frozen supplier graph.
Candid 0.10.40 is a private comparison oracle; its bigint 0.4.8 exception remains.
No published package selects Candid. Public dependency requirements are unchanged.

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
