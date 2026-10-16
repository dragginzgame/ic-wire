# Local development

IC Wire remains an unpublished, dependency-free no_std scaffold. These commands
qualify contributor tooling and compilation; they do not establish a codec,
wire contract or performance result.

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
| `check-tooling` | Local Make routing, release refusal and real formatting-hook fixture |
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

The standard `release-patch`, `release-minor`, `release-major` and `release-resume`
names exist but fail before the release runner. No release delivery adapter or
destination is configured. Every crate retains `publish = false`; publication,
registry name reservation, remote creation, commits, pushes and releases remain
separately authorized maintainer actions. The initial pending changelog is
`0.1.0`, with no finalized release; this tooling batch changes no library contract
or package version.
