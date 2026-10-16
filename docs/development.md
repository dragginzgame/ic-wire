# Local development

IC Wire provides an unpublished no_std + alloc Postcard runtime. See the
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
| `evaluate-canisters` | Build four artifacts and verify them in local PocketIC; requires explicit `POCKET_IC_BIN` |
| `check-typed-diagnostics` | Isolated offline compilation of valid and invalid prototype callers; requires fetched locked dependencies |
| `cloc`, `version` | Workspace source counts and canonical workspace version |
| `ci` | Complete scaffold/tooling gate; explicit local invocation or hosted CI |

Use focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
`make evaluate-codecs` prints the evaluation-only JSON report. Run
`cargo test --locked -p ic-wire --example codec-evaluation` for its focused tests;
`make test` also includes these tests. See [the payload screen](codec-screen.md)
for parameters and limitations. The library itself still has no codec API.
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
object or deploy anything. Every crate retains `publish = false`.

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
