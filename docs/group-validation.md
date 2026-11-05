# Wire release group validation

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Shared owns the roster/baseline observations, plan parser, execution, checkpoints
and run-evidence semantics. Wire owns product scope, source/configuration custody
and its existing release adapter's recovery authority. Full CI and release plans
use one [stage declaration](../ci/release-validation.json); the thin Perl caller
projects Cargo group names into Shared's roster helper. Make does not maintain
another list or silently dispatch after roster admission fails. Hosted execution
partitions remain separate from release groups; all nine native jobs stay intact.

| Release scope | Selected stages |
| --- | --- |
| Core | All 20; codec/derive identities affect schema, browser, benchmark and Endpoint consumers |
| Endpoint-only, qualified Linux x64/default compiler configuration | 11; common admission, Endpoint native/diagnostics/Wasm, Rust/TS schemas and borrowing/cost callers |
| Shared/Core/dependency/manifest/generator/uncertain changes, unavailable baseline or other native hosts | All 20 |

The product selector compares the **entire** source change against the selected
group's annotated `GROUP-vCURRENT_VERSION` tag. Shared observes ancestry and the
complete no-renames diff; availability is not qualification. Wire admits the group,
expected baseline, host and compiler configuration and maps actual affected paths.
Only explicit Endpoint and endpoint-macro source/docs, its installed/native fixture, borrowing probe
and status/Endpoint ledger paths qualify for narrow scope. Crossing that boundary
or changing any manifest, lock, compiler, Shared, CI or unknown input broadens to
full. So does introducing the selector itself. Linux x64 with Node 24.21/default
compiler controls is qualified; other native hosts and ambient Cargo configuration
retain full scope. A newly declared stage is required by default; adding it cannot
silently omit it from either full CI or release plans.

Nine unchanged obligations can be not applicable: CI/Make/release workflow
fixtures, Core check/tests/derive diagnostics, allocation qualification, codec
benchmarks and default Core Wasm. Each has an explicit reason in the selected
plan. Endpoint retains its native contracts, diagnostics, graph and Wasm checks,
including test-harness compilation of the borrowing consumer and ignored cost
caller. TypeScript retains schema, combined Endpoint and generated-browser checks.
No omission is reported as a test pass and no stage-proof reuse is enabled.

Installed execution remains required where runtime, contract or transaction
behavior changes. Prepare its server through Testkit, then run
`POCKET_IC_BIN=... make check-endpoint-ic` under the existing runtime policy.
Selection does not waive that obligation or relabel old installed measurements.
This CI-only migration changes no runtime bytes, budgets or production code.

## Verification and recovery

All tracked source, selected helpers, policy, configuration and actual recipes
remain in Shared's before/after checkpoints. Wire also checks working bytes and
modes against the selected Git tree, including index-hidden changes. A fresh
attempt under `target/release-validation/` retains failed and interrupted records.
After a successful run, **Shared's verifier** admits the complete plan/policy,
context, ordered events/timings, stage checkpoints and regular logs. Wire no longer
parses the runner's event/timing TSV protocol or maintains another baseline reader.

The copied receipt has an internal schema hard cut: old schema 1 admitted generic
run semantics locally; schema 2 binds the exact Shared-verified JSON component by
SHA-256. It copies the plan/context and only the raw records listed by Shared's
proof into Git-owned state, then verifies their bound bytes before publishing the
existing atomic validation receipt/pointer. Old internal and whole-gate receipts
require a fresh applicable gate; no conversion or compatibility lane is added.
This is release-evidence format, not the Wire message protocol.

Shared's verifier requires the original physical source/evidence location. Wire
therefore calls it **before** metadata preparation. Recovery uses its qualified
self-contained receipt, without rewriting absolute paths or re-running Shared's
verifier on changed metadata. It re-admits original source/product selection,
baseline, host/configuration and all copied bytes against the sealed component.
It works after the original runner directory is removed. Missing or changed
components/logs, a retagged baseline, wrong group/source/configuration or an old
receipt refuse. Existing source/index, metadata, unrelated-group, archive,
publication and protection checks remain with their prior owners. No version
projection, another executor or release authority is introduced.

## Focused qualification lifecycle

`make check-release-validation` exercises actual product selection, Shared
execution and sealed recovery in disposable Git repositories. It covers 20/11
command equivalence, required/excluded failures, new-stage growth/full fallback,
Cargo-group mismatch, source/input/baseline drift, missing/interrupted evidence,
copied-proof tampering, old-receipt refusal and recovery after metadata changes
with original runner output unavailable. Generic parser adversarial matrices
remain Shared-owned. `bash scripts/ci/test-local-make.sh --case validation`
checks actual full/partition recipes, failure propagation and single-roster growth
without running application gates, hooks or publication suites.

Full-versus-selected product runs establish or renew the mapping when affected
contracts, scope or qualification configuration change, or qualification is
explicitly requested. They are not permanently added after every CI group. The
original real pair retains its exact historical
source and times; Shared consolidation
records this migration's affected caller and Endpoint adapter proofs separately.
Linux-only narrowing and full fallback on other hosts are preserved. Ordinary
implementation does not execute a release from the real checkout.
