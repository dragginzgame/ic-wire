# Current handoff

Core **0.22.1** (`24ade8f`, `core-v0.22.1`) and Endpoint **0.13.1**
(`285330f`, `endpoint-v0.13.1`) remain the published versions. Both full
20-stage Linux gates passed in `batch-hRrjJIGh`; all four registry archives
were downloaded and checksum-verified. No new crate publication is needed for
the current tooling/fixture maintenance.

## Current delivery

Toko Tooling **0.42.8** is adopted through the crates.io exporter in both
workspaces. The 96-file receipt selects checksum
`10dde4572301be0c5540b54a04ddeb71d3cee0fab68e6700e5c6866b6cb877e2`.
Only the pre-commit hook and hook installer payloads changed from 0.42.5.
Installation/tool checks, snapshot verification, pins, CI selection, disposable
Make/hook checks with CDPATH set, formatting and documentation links pass.
This Git-only delivery preserves the published Core/Endpoint versions.
The exact-pin exceptions are aligned. Deferred local lock edits select Types
0.20.2, Runtime/macros/guard 0.22.2, Host 0.77.2, Platform 0.32.2 and mio 1.2.5;
they are excluded from this delivery, preserved in the normal checkout after
delivery, and backed up under `target/deferred-locks-0428/`. They are not newly
adopted or fully qualified by this tooling task.
The 0.42.8 delivery archives differ from the prior 0.42.5 archives only in the
tooling dev-dependency pin/lock and Git provenance; product sources and runtime
contracts are unchanged. Compared archives are under `target/issue5960-after`
and `target/tooling0428-package`.

Issue [60](https://github.com/dragginzgame/ic-wire/issues/60) is closed with complete native
qualification at delivered source `fa56c2f`. Full run 43463094098 attempt 2 exposed the benchmark CLI
fixture's nonportable `/bin/true` assumption on both macOS architectures.
The delivered fix owns an executable stub with a distinct exit code; all 58 focused
CLI checks pass on Linux without measurements or simulator servers. Subsequent
[run 43465897242](https://github.com/dragginzgame/ic-wire/actions/runs/43465897242)
passed all nine Linux/macOS core, TypeScript and benchmark jobs. Native logs
confirm Rust 1.100.0 on aarch64-apple-darwin and x86_64-apple-darwin. This proof
does not cover the later working-tree supplier changes.

Issue [59](https://github.com/dragginzgame/ic-wire/issues/59) now has verified
hosted cache-miss/hit observations on both macOS architectures. Installation
fell 81→3 seconds on ARM64 and 248→8 on Intel, with restore and tools-check
costs reported separately. These are single hosted setup observations, not an
end-to-end deployment saving. Complete before/after deployment phase timing
remains unavailable. Keep candidate lint/stage reuse disabled. See the
[release cost review](../guides/group-validation.md#release-cost-review-12-november-2026).

## Compiler, suppliers and release boundary

Rust **1.100.0** is minimum and pinned: rustc `0ef1bc386`, Cargo `3acc79c0c`,
LLVM 23.1.5. Linux strict Clippy/Rustdoc, host/Wasm, standalone schema/codec and
Endpoint proofs passed in the delivered gates. The dated sanitizer-only nightly
remains separate. No new full performance-matrix or Wasm saving is claimed.

Both workspaces select Types **0.20.1**, Runtime/macros/guard **0.22.1**;
qualification selects Build **0.84.1**, Host **0.77.1**, Platform **0.32.1** and
Testkit **0.102.0**. The release editor uses Build 0.84.1. Candid 0.10.40 remains
a private dev oracle with the documented bigint 0.4.8 exception. Native dev
builds may compile Testkit/Build; production and Wasm graphs must exclude them.
Core/Endpoint consumers align Types 0.20 and Runtime 0.22 together. Bytes,
identities, limits and acceptance remain unchanged; Account absent/present
subaccounts remain `00` / `01 20 + 32 bytes`.

Preserve `target/release-frozen/batch-hRrjJIGh`, the matching
`.git/release-state/frozen/` records and `/tmp/wire0221-deploy.log`.
The cc 1.11.2 lock update was included in that delivery: do not reapply stash
`b8f16b92bc31602199410a1c9c2457c697b1d3f7`. Preserve completed batch-0wZOXntn
and its evidence. Shared tools live under `vendor/toko-tooling/`; the hook
selects its `.githooks`. Validation callers explicitly select the repository
root. Public release groups exclude all three qualification packages.

## Superseded deployment attempts

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

Batch `batch-akRasIRQ` captured `62c5f8b`. After a transient shared tool-store
lock cleared, its resumed gate passed release recovery and public tests, then
stopped at an obsolete qualification-tool assertion that rejected native dev
dependencies. The assertion now distinguishes library edges from native dev
edges and still rejects all native tools on Wasm. Preserve that batch and
`/tmp/wire022-final-resume.log`; corrected source needs a new capture.

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
