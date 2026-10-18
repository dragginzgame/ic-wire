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
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
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
The pinned development compiler is Rust 1.99.1; Rust 1.91.0 is the library MSRV.

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
0.24.2 at `48f5640e744512929bfba301fe734d8ea6c6e049`, including the receipt helper.
The canonical exporter's explicit `--takeover-snapshot` workflow transferred the
former contribution closure into this main snapshot; the supplementary manifest
and its separate integrity dispatch are retired. Refresh the main manifest through
the canonical exporter as described in [snapshot adoption](consuming-snapshots.md).
Review each preview before applying it. Grouped formatting comes from the shared
defaults, while the local release adapter retains grouped metadata checks, file
selection, receipt schemas and recovery policy. The supplier's new receipt archive
helper is available but not called by this adapter; its existing behavior is unchanged.
Do not copy uncommitted sibling edits or edit snapshot-owned payloads.

### IC Platform boundary

Registry IC Platform 0.1.12 is the selected evaluation boundary (archive checksum
`e0ab384614a5d8fe251bc2c27c508263d4001d776edf31382b5b985a1a4ced2d`).
It selects the coordinated published `ic-platform-macros` 0.1.12 generator. The
registry sources and published README were inspected; no sibling checkout is a
consumer dependency. Examples retain `ic_platform::Principal` and its upstream
identity. The evaluation selects `cdk`, which enables Candid, executor and ic0;
Agent, parser, crypto, certification and ledger/management features stay disabled.

Owned codecs and browser payloads remain IC Wire. The private generator and
browser entries use `ic_platform::cdk::{query, update}` with zero Rust arguments,
`manual_reply = true` and `hidden = true`. Their callable exports, exact Wire
bytes and original executor contexts are retained, without Candid service metadata
or a direct Candid declaration. `cdk::api::{msg_arg_data, msg_caller, is_controller}`
are the original CDK implementations, including allocation and controller
0/1/unexpected-result panic behavior. Consumer byte/auth admission stays in place.
Raw argument-size, append/finalize/reject, trap and stable-page calls remain on
`ic_platform::ic0`; response-codec traps retain local rollback semantics.

The rollback fixture uses `ic_platform::cdk::stable::{stable_read, stable_write,
stable_grow}`, the original CDK helpers now explicitly exported by Platform
([Platform #20](https://github.com/dragginzgame/ic-platform/issues/20)). The final
direct CDK declaration and its feature edge are removed. The eight-byte balance
format, growth/read/write policy and error identities stay unchanged; no storage
migration or local SDK implementation is added. Platform's Candid checkpoint
helpers are unused; the fixture retains its existing raw stable bytes.
[Platform #1](https://github.com/dragginzgame/ic-platform/issues/1)
owns the broader runtime/generator qualification. Build, hashing, simulator and
JavaScript SDK tooling retain their separate owners.

`make check-boundary` uses the recorded Shared Tooling checker and
`ci/cargo-dependency-boundaries.json`. CI rejects direct Candid, CDK-family and ic0
declarations in every package. Canonical names, aliases and normal/build/dev/target
declarations are checked. There are no direct dependency exceptions. Root
catalogs, enabled features,
transitive reachability and final deployment exclusions need separate inspection;
there is no independent Cargo workspace in this repository.

The inspected fixture graph retains upstream CDK/macros 0.20.3, Candid 0.10.38,
one executor 2.0.0 and ic0 1.2.0, now behind Platform's qualified frontend.
Testkit/PocketIC and the browser SDK also reach Candid transitively. The browser
still sends generated Wire bytes through HttpAgent without a Candid Actor or IDL
encoding. These test/transport dependencies are intentionally retained. Production
Wire's normal Wasm/TypeScript graph remains independent of Platform, CDK and Candid.
No host parser or Agent is selected. These graph observations do not prove codec
absence from fixture Wasm or measured code-size savings.

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
