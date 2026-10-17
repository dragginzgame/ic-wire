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
pin exceptions. The common host, IC and Cargo toolsets are installed even when
the scaffold does not use every executable. This does not add runtime crate
dependencies. Shared storage retains prior installations and failed evidence.
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
| `check-canister-evaluation` | Lint the evaluation package and compile both Wasm feature extremes; no emulator |
| `check-macro-diagnostics`, `evaluate-macros` | Evaluation-only macro compilation and installed query/update proof; the latter requires PocketIC |
| `check-typed-diagnostics` | Isolated offline production derive, caller and sync/async adapter diagnostics; requires fetched locked dependencies |
| `cloc`, `version` | Workspace source counts and canonical workspace version |
| `ci` | Complete scaffold/tooling gate; explicit local invocation or hosted CI |

Use focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The retired [payload screen](codec-screen.md) retains historical evidence only.
`make test`, `make check-typescript` and `make evaluate-runtime` exercise the
selected production codec; the last target requires prepared PocketIC.
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
0.15.4 at `879eb2b021522252a67e908dbc217a0f4efb2a6f`. Runtime commands use only
this snapshot. Refresh from a clean, reviewed source checkout with its canonical
exporter, following [snapshot adoption](consuming-snapshots.md). Preserve the
recorded entrypoints on refresh and review the preview before applying it.
Do not copy uncommitted sibling edits or edit snapshot-owned payloads.

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
control compiles first; each negative must fail with its expected diagnostic.
The runner selects exact Cargo artifact paths from JSON, retains per-case logs
under `target/ui-diagnostics/`, and never edits or copies production source.
Prepared Rust and the shared jq tool are required. The same targets run under
`RUSTUP_TOOLCHAIN=1.91.0` for the floor check. Normal Cargo tests compile the valid
cases; the focused Make targets own negative-case execution.

Evaluation hosts use published Testkit 0.50.0 for bounded startup, owned server
lifetime and create/fund/install operations. Each experiment still starts a fresh
instance; topology, 10-trillion-cycle top-ups, raw calls and measurement policies
remain local. Startup has a 60-second deadline. Explicit `POCKET_IC_BIN` selection
and recorded server versions/hashes remain required; no automatic tool setup is
introduced. The browser host retains its owner through live-server shutdown.
Testkit 0.50 uses Host 0.23 internally alongside the explicit Wasm inspector’s
Host 0.24; both are host dev dependencies, not runtime/canister dependencies.

Runtime, macro and browser evaluations use IC Build 0.12 retained artifact records, with exact
build/input digests and matching Wasm hashes in its report. Keep source files
unchanged throughout an installed evaluation; persistent changes refuse report
finalization. Before/after guards cannot detect a transient edit that is reverted.
The old `evaluate-canisters` prototype is retired; its historical reports identify
the source checkout needed for reproduction.

Durable command/replay qualification has moved to Relay’s opt-in receipt probe
([Relay #1 — Own durable receipt-policy qualification](https://github.com/dragginzgame/ic-relay/issues/1)).
`evaluate-replay` is retired here; `evaluate-browser` keeps volatile transport
checks and the runtime tests keep exact request/reply-byte coverage.

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
