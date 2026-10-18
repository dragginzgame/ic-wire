# Local development

IC Wire provides a no_std + alloc Postcard runtime. See the
[runtime guide](runtime.md) for its contract and [adoption evidence](runtime-adoption.md)
for installed correctness checks. Compilation alone establishes no performance win.

## Setup

Prepare Bash, Git, GNU Make, curl with CA certificates, tar, gzip, xz, Perl,
a SHA-256 implementation and the native Rust build prerequisites described in
the reviewed [common setup guide](local-setup.md#bootstrap-prerequisites).
Use the host package manager for these bootstrap dependencies. On macOS, install
Xcode Command Line Tools and xz; on Linux, install a C compiler/linker and the
development packages required by Cargo-installed tools.

```sh
rustup toolchain install 1.99.0 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
rustup toolchain install 1.91.0 --profile minimal --target wasm32-unknown-unknown
make install-tools
make tools-check
make install-hooks
```

Installation is explicit and may download verified executables or build pinned
Cargo tools. `tools-check` is offline and installs nothing. Common pins come only
from the reviewed `ci/tool-versions.env` and `ci/ic-tools.tsv`; there are no local
pin exceptions. IC Wire explicitly includes IC executables in its ordered local
setup/check targets alongside the common host and Cargo toolsets. The reviewed
IC catalog now selects ICP CLI 1.7.0. This adds no runtime crate dependencies.
Shared storage retains prior installations and failed evidence.
See [storage configuration](local-setup.md#shared-executable-storage) before
selecting an alternative `SHARED_TOOLING_STORE`; use the same selection for
setup, checking and evidence collection.

Make selects this checkout's tools automatically. For direct shell commands:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/common/bin:$PATH"
```

Hook activation sets repository-local `core.hooksPath`; it requires an actual Git
checkout and refuses to replace unrelated hooks. The reviewed pre-commit hook
sorts manifests and formats selected Rust inputs, rejects partial staging and
preserves unrelated edits. Formatting never installs tools. No remote is created
by setup.

## Commands

Run `make` for help. Check for active builds sharing `target/` before compilation
or source edits. Make uses this checkout's `target/` and keeps its artifacts.
The pinned development compiler is Rust 1.99.0; Rust 1.91.0 is the library MSRV.

| Target | Scope |
| --- | --- |
| `build`, `check`, `test`, `lint` | Locked builds/checks for `ic-wire`; Clippy denies warnings |
| `check-wasm` | Library compilation for `wasm32-unknown-unknown` |
| `check-msrv` | Native and Wasm library compilation on Rust 1.91.0 |
| `fmt`, `fmt-check` | Pinned cargo-sort over root/member manifests, then rustfmt |
| `shared-tooling-check` | Offline snapshot hashes and executable modes |
| `check-doc-links` | Local references in IC Wire's maintained documentation |
| `check-pins` | Shared declaration checker with Cargo inheritance checks; requires Git and prepared host tools |
| `check-release` | Disposable Git release, validation-failure and recovery fixture |
| `check-tooling` | Local Make routing and real formatting-hook fixture |
| `check-canister-evaluation` | Test Wire browser/macro fixtures, lint and compile host/Wasm; no emulator |
| `check-macro-diagnostics`, `evaluate-macros` | Evaluation-only macro compilation and installed query/update proof; the latter requires PocketIC |
| `check-typed-diagnostics` | Isolated offline production derive, caller and sync/async adapter diagnostics; requires fetched locked dependencies |
| `cloc`, `version` | Workspace source counts and canonical workspace version |
| `ci` | Complete scaffold/tooling gate; explicit local invocation or hosted CI |

Use focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The retired [payload screen](codec-screen.md) retains historical evidence only.
`make test` and `make check-typescript` exercise the selected production codec.
`make evaluate-browser` and `make evaluate-macros` retain installed Wire integration
tests and require prepared PocketIC. The Candid comparison runner is retired.
The conformance generator writes `target/typescript-conformance/mutations.jsonl`
with codec, direction, bytes and Rust acceptance per record. The JavaScript runner
streams those records and reports the failing line for replay; independent value
oracles (including bigint and signed zero) remain executable checks.
The local tooling fixture checks
its actual Make callers and hook; upstream owns the common helper regression
matrix. Dependency preparation uses locked fetching when needed; to explicitly
request offline Rust checks, use `CARGO_NET_OFFLINE=true make check` (or another
Rust target). Tool admission and formatting are always offline.

CI calls these same Make targets on Ubuntu 24.04, macOS 15 arm64 and macOS 15
Intel. Those are the declared native development hosts. Configuration alone is
not native execution evidence; qualification results and limits are recorded in
[the current handoff](status/current.md). Each workflow/ref keeps the newest run
and retains failed tool/build logs through the reviewed evidence action.

## Snapshot maintenance and release boundary

[.shared-tooling.snapshot](../.shared-tooling.snapshot) records Shared Tooling
0.24.0 at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`.
The unmerged [receipt contribution](https://github.com/dragginzgame/shared-tooling/pull/279)
is exported separately in [.shared-tooling-receipts.snapshot](../.shared-tooling-receipts.snapshot)
at `35775234560bd08f7e67f4feeaf339dc25fbd9e0`; only its receipt helper
is selected. Its required integrity helpers are byte-identical to the main
snapshot, and `check-tooling` verifies the contribution receipt too.
Refresh both manifests through the canonical exporter until that contribution
is integrated; never edit their ownership records or payloads in place. Then
consolidate the contribution with `--takeover-snapshot` as described in
[snapshot adoption](consuming-snapshots.md). Review each preview before applying it.
Do not copy uncommitted sibling edits or edit snapshot-owned payloads.

### IC Platform boundary

Registry IC Platform 0.1.8 is the selected evaluation boundary (crates.io checksum
`4a3adb6d9785133587b854b66b37dfdd8f7945555bdc59b6d0be26995187484d`).
Cargo's refreshed registry index selected this published release; the sibling
checkout is not the dependency. Only its `ic0` feature is enabled. Examples use
the published `ic_platform::Principal`, which retains ic_principal 0.1.5 identity.
Agent, Candid, ledger/management types, certification, executor and native parsing features
are not enabled; this repository has no direct consumer of those surfaces to migrate.
The additional Agent/Candid/parser exports have no current owned Wire consumer;
Principal and ic0 retain their existing upstream identities.
The published 0.1.8 ic0 module is byte-identical to 0.1.7. Its additional
management snapshot/status DTOs have no current owned Wire consumer and stay disabled.

All owned codecs, including browser requests/replies, use IC Wire. The retired
Candid benchmark, its control DTOs/decoder and external management probe are
removed; [recorded measurements](runtime-comparison.md) retain their original
source identities. All early argument-size checks use
`ic_platform::ic0::msg_arg_data_size`, the same upstream ic0 1.2.0 identity;
the direct ic0 dependency and its boundary exception are removed.
The macro fixture also routes trap bytes and stable-page counts through
`ic_platform::ic0::{trap, stable64_size}`. CDK 0.20.3 delegates to these same
IC 1.2.0 calls; trap UTF-8 bytes, page units and the eight-byte balance layout
stay unchanged. Stable allocation/read/write policy remains with the test fixture.

CDK 0.20.3 remains a direct evaluation dependency for entrypoint
macros, caller/controller checks, argument copying, replies/rejections and stable read/write/grow
operations: [Platform #1 — Support the CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1).
The [0.1.7 consumer audit on #1 — CDK boundary](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6248552044)
lists the exact remaining helpers and raw exports with a minimal fixture and
installed acceptance checks. Build, hashing and simulator tooling retain their
separate owners; those are intentional dependencies outside Platform.
The [0.1.8 recheck on #1 — CDK boundary](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6248715562)
confirms those requirements remain unchanged; no additional missing API was found.
The private adapter generator emits zero-argument, manual-reply CDK entries with
`hidden = true`. This suppresses unused Candid metadata and removes generated
absolute Candid references; it does not hide callable Wasm exports. Browser
entries use the same options. No direct Candid dependency is required. Exact
exports, request/reply bytes, refusal and rollback behavior remain tested.
No compatibility alias, replacement executor, parser or storage migration is added.

`make check-boundary` uses the recorded Shared Tooling checker and
`ci/cargo-dependency-boundaries.json`. CI rejects direct Candid and ic0 declarations;
only the unpublished evaluation package may declare CDK. It checks canonical names across
normal/build/dev/target declarations, including aliases. All current manifests
and the root catalog were inspected; no independent Cargo workspace was found.
The checker does not enforce unused root-catalog entries or transitive reachability.

The inspected evaluation Wasm graph still contains Candid 0.10.38, CDK/macros
0.20.3, ic0 1.2.0 and one executor 2.0.0 identity. CDK reaches Candid and ic0;
Platform selects only ic0/Principal. Host tests reach Candid through Testkit/PocketIC.
The browser SDK also contains Candid internally; the client sends raw generated
Wire bytes through HttpAgent without a Candid Actor or IDL encoding.
These test/transport dependencies are explicitly retained; no candid_parser is selected.
The production Wire Wasm and host TypeScript graphs remain Candid/CDK/Platform-free.
There is no executor-only integration to migrate in this workspace. These graph
observations are not a claim of codec absence from evaluation Wasm or size savings.

### Dependency roles and deployment boundary

The root catalog separates `Production dependencies` from `Tests and tooling only`.
Syn, Quote and Proc Macro 2 are production build-time dependencies of the derive;
Serde JSON is production through the optional TypeScript feature. The `ic-wire`
catalog entry is used by evaluation consumers; the production package itself is
selected as the graph root, not treated as a forbidden dependency on itself.
Member comments separately identify production and test/tooling packages.

Both formatting commands use grouped sorting. Release preparation checks and
committed-source checks use the same grouped policy. Disposable adapter fixtures
verify idempotence, rejection of disorder within a group, and admission of the
selected commit despite disorder in the surrounding working tree.

The 0.24.0 adoption inspected locked normal/build graphs and passed library checks
for `ic-wire` on `wasm32-unknown-unknown` with default features, `ic-wire` on
`x86_64-unknown-linux-gnu` with `typescript`, and `ic-wire-derive` on that host.
No test/tooling-only catalog entry was reachable, excluding the selected root.
These checks qualify those selections, not every target/feature combination.

**Enforcement gap:** CI checks pins/inheritance, direct package boundaries and
builds, but does not enforce transitive role exclusions. This library repository has no production canister
deployment pipeline. Downstream deployment owners must enforce their selected
graph and bind source, lockfile, target/features and artifact digest to deployment.
Snapshot integrity, sorting and registry publication are separate claims; none
constitutes deployment admission. Graph logs: `/tmp/ic-wire-tooling024-*-tree.txt`.

The maintainer-operated `release-patch`, `release-minor`, `release-major` and
`release-resume VERSION=X.Y.Z` commands use direct Git delivery to `origin/main`.
They validate, bump the coordinated Cargo workspace version and local lockfile
entries, finalize the changelog, commit, create an annotated tag and atomically
push the branch and that tag. They do not publish packages, create a GitHub Release
object or deploy anything. The evaluation crate retains `publish = false`.

Commit intended source changes before invoking a release. Preflight requires a
clean index/worktree, checks the pending changelog against the computed candidate,
prepares the locked Cargo cache and selected common tools, then checks the graph.
`RELEASE_CACHE_PREPARE=0` uses only cached dependencies and checks installed tools;
explicit Cargo offline settings remain authoritative. Rust toolchains/targets
remain prerequisites from the setup section above. Only direct delivery is
configured; PR delivery rejects before metadata mutation.

Every release kind runs `make ci` through the shared validation logger with Cargo
offline. This is the complete scaffold gate, including the local release fixture;
the separately invoked browser/PocketIC experiments are not release gates.
Validation logs and source-bound receipts remain under `.git/release-state/`.
Preparation uses the pinned Shared Tooling Cargo metadata adapter with rollback;
only `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md` are release metadata. The private
browser harness version is independent and is not bumped by a Rust release.

An interrupted release resumes with the shared saved-plan machinery. Before
commit/tag/push, local checks require the selected validation identity and exact
prepared metadata bytes. Later checks read the selected committed source.
`make check-release` exercises these adapters and the real runner against a local
bare remote, substituting expensive setup/validation targets. Prepare its cache
with `cargo fetch --locked` first (CI does this explicitly). It covers dirty
source, candidate mismatch, failed-validation logs, interrupted delivery, receipt
rejection, exact branch/tag push and idempotent completed resume. It does not
establish live GitHub permissions or hosted macOS qualification.

Release execution, package publication and registry name reservation remain
separately authorized maintainer actions. Preparing this tooling changes no
workspace version and performs no release in the working repository.

## Coverage-guided runtime fuzzing

`make fuzz-runtime` runs 100,000 libFuzzer inputs with seed 42, address/leak
sanitizers, a 2,048-byte maximum input, a five-second per-input timeout and a
2,048 MiB RSS ceiling. `FUZZ_RUNS=1000000 make fuzz-runtime` extends the run.
This is an explicit experiment, separate from ordinary CI. Prepare prerequisites
explicitly: `rustup toolchain install nightly-2026-07-29 --profile minimal`, a
native C++ compiler (Xcode command-line tools on macOS), and `cargo fetch --locked`.
The target uses offline Cargo and disables automatic toolchain installation.
It needs no installed cargo-fuzz command: the example uses the upstream
[libfuzzer-sys harness](https://github.com/rust-fuzz/libfuzzer) directly.

The harness checks arbitrary frames and inserts arbitrary payloads behind valid
request/reply headers. This reaches bounded nested containers, explicit variants,
UTF-8, 128-bit integers, finite floats and canonical serialization without first
requiring discovery of the method identity. Accepted frames must reproduce exact
bytes, match the length helpers and reject the opposite direction. This supplements
the Rust/TypeScript differential corpus; it does not fuzz JavaScript or prove
allocation budgets, complete branch coverage or absence of defects.

Instrumented builds, reusable corpus and failure artifacts live under
`target/fuzz-runtime/`, separate from ordinary builds. The script supplies sanitizer
flags explicitly and selects the host target to isolate build scripts/proc macros.
The input sequence depends on the retained corpus; a seed alone does not identify
a replay. Pass an artifact filename directly to the built `fuzz-runtime` executable
under `target/fuzz-runtime/<host>/release/examples/` to replay that input.
Retain any reproducible failure as a focused regression before repairing it.
Run outside ptrace/restrictive process sandboxes when LeakSanitizer cannot inspect
threads; a sanitizer shutdown failure is not evidence of a codec defect.

Qualified here on Linux x86_64 with Rust 1.99.0-nightly (26ae60a9e), LLVM 22.1.8,
libfuzzer-sys 0.4.13 and release profile: 100,000 inputs, no codec assertion or
sanitizer failures. The smoke run ended at 780 coverage counters and 2,009 features;
these are engine observations, not a source coverage percentage. Native macOS
execution and sustained campaigns remain unqualified. The normal runtime dependency
graph and its Rust 1.91 Wasm floor are unchanged; libfuzzer-sys is dev-only.

## Explicit diagnostics and managed PocketIC

`make check-typed-diagnostics` and `make check-macro-diagnostics` use one offline
runner with explicit Rust cases under each owning crate’s `tests/ui/`. A valid
control compiles first; every negative must exit with compiler code 1 and include
all expected literal diagnostics. The local Rust example supplies cases and
compiler flags to [IC Build PR #67](https://github.com/dragginzgame/ic-build/pull/67),
pinned at `4d8549a14c8cb41b09a9c69ea15e5c0fda88a9f4`. IC Build selects exact
host library artifacts from bounded Cargo JSON and owns bounded process capture.
Logs remain under `target/ui-diagnostics/`: `cases.txt` maps zero-based
negative-case log names; `control.stderr` records the passing control. No source
is copied or rewritten. Prepared Rust and fetched locked host dependencies are
required; artifact selection no longer uses jq. The same targets run under
`RUSTUP_TOOLCHAIN=1.91.0` for the floor check. Normal Cargo tests compile the valid
cases; the focused Make targets own negative-case execution. Both diagnostic
lanes are included in the CI target.

Evaluation hosts use the workspace-selected Testkit 0.53 for bounded startup, owned server
lifetime and create/fund/install operations. Each experiment still starts a fresh
instance; topology, 10-trillion-cycle top-ups, raw calls and measurement policies
remain local. Startup has a 60-second deadline. Explicit `POCKET_IC_BIN` selection
and recorded server versions/hashes remain required; no automatic tool setup is
introduced. The browser host retains its owner through live-server shutdown.
Testkit and the explicit IC Host Wasm inspector are host development dependencies,
not dependencies of the published codec. The lockfile records their exact versions.

Macro and browser evaluations use the pinned IC Build retained artifact API, with exact
build/input digests and matching Wasm hashes in its report. Keep source files
unchanged throughout an installed evaluation; persistent changes refuse report
finalization. Before/after guards cannot detect a transient edit that is reverted.
The old `evaluate-canisters` prototype is retired; its historical reports identify
the source checkout needed for reproduction.

Durable command/replay qualification has moved to Relay’s opt-in receipt probe
([Relay #1 — Own durable receipt-policy qualification](https://github.com/dragginzgame/ic-relay/issues/1)).
`evaluate-replay` is retired here; `evaluate-browser` keeps single-canister transport
checks without receipt caching, expiry/audience policy or peer forwarding and the runtime tests keep exact request/reply-byte coverage.

## First publication

0.3.0 is the first published crates.io release of `ic-wire` and `ic-wire-derive`.
After committing subsequent reviewed changes, the maintainer runs the selected
Git release command; Shared Tooling updates the
workspace version, local dependency requirements, lockfile and changelog together.
Git delivery remains separate from registry publication.

`make check-package` assembles and builds both archives, including all features,
without uploading. It permits dirty inputs for local review and may access the
registry; it is separate from offline CI. Cargo stages the local derive package
while verifying the runtime. Package allowlists exclude tests, fuzz harnesses and
evaluation artifacts; LICENSE and package-local README files are included.

After Git release delivery, check the clean selected release source and credentials, then
run the first command below to rehearse. The second command uploads both packages
and is a separate maintainer action; Cargo orders the derive before the runtime.

```sh
cargo publish --dry-run --locked -p ic-wire-derive -p ic-wire --all-features
make publish
```

Publication must use Cargo 1.99 (the prepared development toolchain). Library
consumers retain the Rust 1.91 floor. The repository remains private; packaged
README/Rustdoc and LICENSE are self-contained, while supplemental GitHub links
require repository access. Preparation does not change visibility or reserve
names. Authentication, name ownership and upload acceptance are checked by the
registry at publication time.

## Evaluation report provenance

Fresh macro and browser reports share a `provenance` object containing
`rustc`, `rustflags`, `encoded_rustflags`, `pocket_ic_version`,
`pocket_ic_sha256`, `sources` and `build_records`. The build driver captures
compiler/server identity once, owns source inventories, and uses IC Host FS bounded
streaming hashes (16 MiB per source, 512 MiB for the server). Wasm hashes come from
IC Build's retained export receipts under the existing 64 MiB export cap. The
selected `RUSTC` supplies the compiler probe when configured.

All trials write a pending report; the driver adds provenance and promotes it to
`report.json` after measurements and retained-build freshness checks succeed.
Browser source inventory includes native vectors and the selected session helper.
This replaces the prior per-runner compiler/server/hash fields in fresh reports;
there is no compatibility reader. Frozen `docs/evidence/` reports are unchanged.
Runtime reports retain codec parameters, separate instruction phases and measured
bytes; this consolidation makes no performance improvement claim.
