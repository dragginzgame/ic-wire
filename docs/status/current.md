# Current handoff

## Released 0.7.21; pending 0.7.22 IC dependency adoption

HEAD is `15c8b7ac881c008b2af0bd2c3d79502605a36d0d` / release 0.7.21.
Both published Wire and derive archives identify that exact source. The
[matching CI run](https://github.com/dragginzgame/ic-wire/actions/runs/40726709588)
passes all nine core/TypeScript/benchmark qualification jobs across Linux,
macOS ARM64 and macOS Intel, plus all three final scaffold gates. This supersedes
0.7.19's historical macOS release-editor installation failure for current acceptance.
No workflow was dispatched or rerun during this reconciliation.

The published CI result above qualifies 0.7.21, not the pending dependency/fixture
changes below. Own package versions remain 0.7.21; the 0.7.22 changelog is a draft.
Leave staging, commits, versions, release and publication to the maintainer.

Canonical SDK producer acceptance and remaining consumer boundaries are in
[#29 — Payload-only client codecs and bounded caller storage for SDK integration](https://github.com/dragginzgame/ic-wire/issues/29#issuecomment-6475585612).
No additional codec implementation was needed for that reconciliation. The new
adoption also leaves Wire protocol bytes, identities and admission unchanged.

## Registry-backed consumer qualification

Fresh isolated consumer: `/tmp/ic-wire-issue29-0721.xUugkv/`, with its own Cargo
workspace, lock and target. Metadata confirms registry Wire and derive 0.7.21,
without local path substitutions. Logs/generated output/metadata remain under
`target/issue29-0721/`; `probe.path` records the scratch source location.

The probe passes unit/tuple payload bytes, scoped scratch and owned fallback,
admission and short-storage refusal, framed identity/direction checks and
standalone schema validation. The published runtime and generated tuple client
compile strictly with TypeScript 7.0.2 and execute on Node 24.21.0: exact `07`
bytes, empty unit and malformed/trailing refusal pass. The delivered hosted suite
covers recursive payloads, scalar facades, callback lifetime refusal and failures.
This is library/client qualification, not installed cdk-rsx adoption or endpoint
execution/rollback qualification. The [SDK recipe](../sdk-integration.md) retains
transport, copying, authentication, persistence, retries and consumer ownership.

## Parallel hosted CI

`ci-core`, `ci-typescript` and `ci-benchmarks` run independently on each supported
OS, with separate checkouts/targets/tools and unique failure artifacts. Node/npm
preparation belongs to TypeScript only. Existing `scaffold (OS)` status names
remain final gates requiring success across every group and host.

One fixed Make roster owns all 17 gates. Core filters out the specialist targets;
both specialist groups admit tools first. Local/release `make ci` keeps its
original sequential roster/order. No shared snapshot runner changed. TypeScript
now executes feature-enabled native tests alongside runtime/borrowed tests.
Actionlint and disposable routing/failure/formatting/release fixtures pass;
logs: `target/parallel-ci-0721/`. Concurrent correctness execution passed 75 tests
(53 native, 16 runtime, six borrowed); logs: `target/parallel-suite-audit/`.
Hosted qualification is linked above; no CI wall-time saving was measured.

## Warning output and SDK helpers

The synthetic TypeScript benchmark now consumes computed advice without printing
141,804 repeated warnings; all 36 ordinary warnings remain visible. CI retains
stderr in `target/typescript-conformance/generation.log` and refuses synthetic
Page warnings before replay. The [warning study](../evidence/generator-warnings-0721/README.md)
retains counts, identical generated files and shell failure/replay evidence.
Its host timing reductions remove diagnostic IO, not production codec work.

`native::with_encoded` selects scratch/owned fallback after preflight, retains
two-pass bounded Postcard encoding, drops owned input before its synchronous
callback and never calls it on failure. Borrow/future escape is rejected.
The [output study](../evidence/with-encoded-0721/README.md) records unchanged
owned-control Wasm and scoped u64 instruction changes: opt-z 842 → 425 (-49.52%),
opt-3 586 → 289 (-50.68%). Scoped fixture total/code costs +722/+704 bytes at
opt-z and +2,490/+2,495 at opt-3. Opt-3 unit adds 16 instructions (+6.27%);
native unit and large-vector regressions remain recorded. Scratch-fitting output
allocates zero; large fallback allocates once. Query returns may reuse execution;
native comparisons are paths within one executable.

Host-only `typescript::validate_payload_schema` checks the root and complete
registry using existing shape/reference/alias-cycle and shared work/depth checks,
without emitting a module. The [validator study](../evidence/schema-validator-0721/README.md)
records one/100-type medians of 1,414/83,863 ns versus 5,178/158,205 ns for
module generation (-72.69%/-46.99%). All 229 generated files match; transient
existing-generation timing regressions and repeats are retained. This is not a
production generator speedup or IC/Wasm cost claim.

## Dependency and measurement boundaries

Root minima select Build/Testkit 0.69.12 and Platform 0.8.1; the managed release
editor remains Build 0.69.12 with `cli`. The latest incoming lock already selected
Build/Testkit and host helpers 0.69.14; those unrelated updates are retained.
Platform 0.8.1 is added for the benchmark caller; Testkit still requires the 0.7
line and retains Platform 0.7.6 internally. Both facades reuse the same external
Candid/PocketIC type identities; benchmark host callers compile together.
The incoming graph is retained in `target/platform-adoption-0722/incoming.Cargo.lock`;
only the added Platform selection and its qualified dependency references differ.

Platform 0.8.1's published source is
`10c2610511d6a5525816185c999b25fd159c1d89`. The 0.8 cut separates Principal
conversion/self-authentication features and owns selected ledger DTOs; Wire
selects neither the affected owned Principal API nor ledger DTOs. The selected
Candid implementation matches 0.7.6. Native profiler source changes are outside
Wire's feature selection. No caller migration, Wire protocol change, production
SDK dependency or new codec is introduced. The pre-1.0 patch draft stays valid
because this dependency is qualification-only. Shared Tooling remains 0.27.3 at
`7ee5c83744aa822ecd070703dbf0a7d95076d5f6`, with its snapshot intact.

Platform 0.7.5 introduced a 16-constructed-level bounded Candid nesting cap.
The pending paired corpus therefore uses `tree-depth-7` (16 record/vector levels),
with depth 8 explicitly covered as Wire-success / typed Candid nesting refusal.
No raw-decoder bypass or unsupported-pair reporting lane was added. Historical
depth-8 and new depth-7 rows must not be paired; old reports also exclude the
new Candid admission cost.

Latest focused logs are in `target/platform-adoption-0722/`: three standard
corpus/boundary/Nat tests, strict benchmark all-target Clippy with both codecs and
IC host callers, Rust 1.91 native and both-codec Wasm checks, dependency pin/
boundary checks, documentation links and diff checks pass. Source and lock hashes
are in `receipt.json`. No fresh PocketIC execution, full matrix, hosted CI or
native macOS execution was run. No timing or Wasm saving is claimed: the
production codec is unchanged and the selected Candid source is identical.

Earlier `target/ic-adoption-06912/` logs qualify Build 0.69.12's managed editor
and disposable release adapter, diagnostics, and that batch's older lock graph;
they do not qualify the independently incoming Build/Testkit 0.69.14 update.
Earlier adoption logs also remain under `target/ic-adoption-06911/` and
`target/ic-adoption-0698/`. SDK studies freeze Build/Testkit 0.69.6 /
Platform 0.7.3; historical
[caller-storage](../evidence/caller-storage-0719/README.md),
[runtime](../evidence/runtime-0716/README.md) and
[encoder](../evidence/encoder-followup-0716/README.md) studies retain their inputs.
Installed SDK qualification remains consumer-owned.
