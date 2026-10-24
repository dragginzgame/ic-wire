# Local development

IC Wire provides a no_std + alloc Postcard runtime. See the
[runtime guide](runtime.md) for the wire contract and [evaluation scope](evaluation.md)
for maintained qualification. Transport and endpoint integration belong to consumers.

## Setup

Prepare Bash, Git, GNU Make, curl with CA certificates, tar, gzip, xz, Perl,
a SHA-256 implementation and native Rust build prerequisites as described in the
reviewed [common setup guide](local-setup.md#bootstrap-prerequisites).
On macOS, install Xcode Command Line Tools and xz; on Linux, prepare a C compiler,
linker and the development packages needed by Cargo-installed tools.

```sh
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
rustup toolchain install 1.91.0 --profile minimal --target wasm32-unknown-unknown
make install-tools
make tools-check
make install-hooks
```

`make install-tools` also prepares IC Tooling's IC Build 0.68.1 release metadata
editor with its required `cli` feature. The diagnostic library selects IC Build
0.68.1 without default features; the CLI feature is only enabled for installation.
`make release-editor-check` verifies its selected executable offline. The release
adapter selects that path automatically; metadata inputs are limited to 8 MiB
per document. This tooling executable is separate from runtime dependencies.

Installation is explicit; checks install nothing. IC Wire selects only the
reviewed common host and Cargo toolsets. Platform executables are not part of its
setup/check aggregates. The vendored generic catalogs and installer remain owned
by the pinned Shared Tooling snapshot and are not product dependencies.
See [shared executable storage](local-setup.md#shared-executable-storage) before
selecting an alternative store. For direct shell commands:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

Hook installation selects repository-local `core.hooksPath`. The reviewed hook
sorts manifests and formats selected Rust inputs, refuses partial staging and
preserves unrelated edits. It never installs tools implicitly.

## Focused commands

Check for active builds sharing `target/` before compilation or source edits.
The development compiler is Rust 1.99.1; the library floor is Rust 1.91.0.

| Target | Scope |
| --- | --- |
| `build`, `check`, `test`, `lint` | Locked runtime builds, behavior and strict Clippy |
| `check-wasm`, `check-msrv` | Runtime Wasm and minimum-toolchain checks |
| `check-typed-diagnostics` | Offline compile-fail subjects plus a passing control |
| `check-qualification` | Allocation/borrowing fixtures and qualification lint/floor checks |
| `check-typescript` | Feature-enabled Rust tests/lints/MSRV, generated vectors and differential mutations |
| `evaluate-codec-matrix` | Full 21-cohort Wire/Candid matrix; requires explicit prepared PocketIC |
| `evaluate-allocation` | Explicit allocation/borrowing [native timing experiment](performance.md) |
| `fuzz-runtime` | Prepared nightly/C++ coverage-guided codec fuzzing |
| `fmt-check` | Grouped manifests, derive order and Rust formatting |
| `check-pins`, `check-boundary` | Dependency inheritance/pins, common external IC guard and private-control ownership |
| `shared-tooling-check` | Recorded snapshot bytes and modes |
| `tools-report` | Aggregate offline host/Cargo/local-tool diagnostics; runs every checker |
| `check-doc-links` | Maintained local documentation references |
| `check-tooling`, `check-release` | Disposable Make/hook and release-adapter fixtures |

Use the smallest meaningful checks under the
[validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` gate requires an explicit local request or the selected
maintainer release operation. Configured hosted CI remains independent.

### Standalone TypeScript qualification

Prepare Node 24.21.0 and npm 12.2.0, matching `examples/browser/.nvmrc` and
the private package metadata, then:

```sh
npm --prefix examples/browser ci
make check-typescript
```

`check-typescript` is included in `make ci` and release validation. Local release
operators must prepare the pinned Node/npm environment and locked package above
before starting the offline validation phase. Hosted CI prepares them on every
supported host. The target also runs feature-enabled Rust runtime/borrowed tests,
strict Clippy and the generator's Rust 1.91 library check. It also runs a disposable
consumer that forwards a local schema feature: native metadata and exact runtime
frames must work, while Wasm builds with and without forwarding exclude Wire's
optional `serde_json` metadata dependency from the normal graph. The consumer
uses `ic_wire::ByteBuf` without a direct serde_bytes dependency, and the check
requires serde_bytes with only its `alloc` feature in the Wasm runtime graph.
The forwarded Wasm build is checked on Rust 1.91 too. Run only this boundary check
with `bash scripts/ci/check-schema-targets.sh`; evidence remains under
`target/schema-targets/`. These graph checks do not measure installed Wasm size.

This package contains only the TypeScript compiler. The generated codec imports
no transport or browser SDK. Rust vectors, strict typechecking and Node mutation
checks qualify byte conformance; they do not qualify endpoint authentication,
certificate verification, installed rollback or browser transport.
See [consumer fixtures and packet inspection](consumer-fixtures.md).

The same target runs packed codecs with simulated big-endian numeric typed-array
storage, using DataView as an independent byte oracle. It checks all seven packed
types and bounded i16, signed/unsigned extrema, empty fields, both directions,
unaligned frames, owned output and malformed widths. This executes the TypeScript
swap branches; it does not qualify a native big-endian JavaScript engine.
`cargo test --locked -p ic-wire --lib endian_append_matches_wire_bytes` also
exercises Rust's big-endian append helpers on the local host. Full Rust big-endian
serialization and allocation/error paths still require a real target or emulator.

### Runtime fuzz seeds

`make fuzz-runtime` restores 112 deterministic packed frames before every run,
using `scripts/ci/seed-packed-fuzz.pl`. No archived corpus is needed on a clean
checkout. Seeds cover every supported word type, both directions, empty fields,
length-prefix boundaries and partial-word rejection. Existing discovered corpus
entries are preserved. Cell variant indices in the seed oracle must track the
qualification enum in `examples/fuzz_runtime/mod.rs`.

The runner requires the prepared `nightly-2026-07-29` toolchain and a C++ compiler;
it installs nothing. To inspect only the seed corpus, create an empty directory
and run `perl scripts/ci/seed-packed-fuzz.pl DIRECTORY`. The sanitizer default is
unchanged; on hosts where ptrace prevents LeakSanitizer shutdown, an explicit
`ASAN_OPTIONS=detect_leaks=0` run retains address checking but does not check leaks.

### Allocation timing modes

`make evaluate-allocation` retains decoding plus the application checksum as the
default. To isolate codec work, select a fresh output directory:

```sh
cargo run --locked --profile evaluation -p ic-wire-qualification --example allocation --features allocation-fixture -- target/allocation-codec evaluation --codec-only
```

Both modes include primitive validation and result destruction. Codec-only mode
omits the full payload scan, while retaining constant-time packed-length checks.
Reports use `format: 2`, label `decode_work`, and record `encode_repetitions` and
`decode_repetitions` separately; divide each sample column by its own count.
The old single `repetitions` field is absent. Each operation is calibrated to
at least 2 ms or the repetition cap. Historical format-1 results remain unchanged.

### Dependency preparation

Keep `Cargo.lock` selected and use locked builds. Prepare missing dependency
sources with `cargo fetch --locked` before explicitly offline gates. Runtime
normal/build dependencies remain separate from the private qualification package;
IC Build is restricted to that unpublished package. Alternative-codec dependencies
are retired.
No platform SDK or simulator is selected in any workspace dependency role.

## Snapshot maintenance and release boundary

The reviewed Shared Tooling revision is recorded in
[AGENTS.md](../AGENTS.md) and the [snapshot manifest](../.shared-tooling.snapshot).
The main snapshot owns the common setup, formatting, hooks, release runner and
receipt helper. Update through the canonical exporter and reviewed adoption;
do not patch snapshot-owned files. Product adapters stay local.

The maintainer-operated `release-patch`, `release-minor` and `release-major`
targets use one workflow: preflight, complete validation, version preparation,
receipt checks, commit, tag and direct Git delivery to origin/main. They do not
publish packages. Ordinary development leaves package versions, staging, commits,
refs and release execution to the maintainer.

The release gate runs the existing Clippy target after tool admission and before
expensive checks. Package checks and publication also run Clippy first, with
sequential recipes that stop on failure under parallel Make.

The local release fixture uses disposable repositories and a local remote. It
checks dirty/conflicting-source refusal, failed-validation retention, exact
version/tag/delivery and receipt rejection/resume without real release effects.
The Make fixture similarly confines hook/index effects to disposable repositories.

## First publication

The runtime and derive crates were first published at 0.3.0. Subsequent Git
delivery and registry publication are separate maintainer operations.

`make check-package` assembles and verifies both archives with all features,
permits dirty source for local review and may use the registry. Package allowlists
exclude tests and evaluation artifacts. Each archive includes its README and the
root MIT LICENSE as an identical package-local regular file; only the SPDX license
metadata is declared. This command uploads nothing.

After committing and releasing the reviewed source, the maintainer may rehearse
or execute the separately authorized publication with Cargo 1.99:

```sh
cargo publish --dry-run --locked -p ic-wire-derive -p ic-wire --all-features
make publish
```

Consumers retain Rust 1.91. Registry identity, credentials, publication status and
repository visibility remain maintainer concerns. The qualification package remains
`publish = false`.

## Supported hosts

The native scaffold supports Linux x86_64, macOS ARM64 and macOS x86_64 through
the configured CI matrix. Local Linux checks do not qualify either native macOS
host. Keep [host support](supported-hosts.md), compiler-floor checking and Wasm
compilation distinct from runtime performance or consumer deployment acceptance.

## Candid comparison benchmarks

Run `make check-benchmarks` for corpus correctness, lint and Rust 1.91 checks.
For the complete maintained comparison, set `POCKET_IC_BIN` to an explicitly
prepared server and run `make evaluate-codec-matrix`. The 21 cohorts cover the
standard/Toko corpora, 1/10/100 ordinary derived records, bounded packed-i16 and
ordinary controls, shared/varied bound records, and generic packed integer/hash
fields plus matching ordinary controls and 1/10/100-record scaling. These are
benchmark cohorts within the one codec, not selectable production strategies.

The runner freezes source/lock before measurement, requires a fresh output
folder, runs cohorts serially and retains both optimization profiles, two native
runs of eleven samples, allocation/peak-heap probes, raw Wasm/code sizes and IC
counters. Native and IC measurements remain separate. Prepare locked dependencies
first; the full runner is offline. Use `bash scripts/ci/evaluate-codec-matrix.sh
NEW_OUTPUT` for another run. It is an explicit experiment, outside ordinary CI.

Run `make evaluate-codecs` for native timing, allocation counts, peak live heap
and Wasm footprint under both optimization profiles. It requires the prepared
`wasm32-unknown-unknown` target, `gzip` and `shasum`; it does not install tools.
Set `POCKET_IC_BIN` to an explicitly prepared server to include IC instruction
measurements. The driver uses IC Testkit for server admission/lifecycle and never
downloads it implicitly. Output directories must not already exist. Use
`bash scripts/ci/evaluate-codecs.sh target/another-run` to retain another run.
See [methodology and results](candid-comparison.md) before interpreting ratios.

The optional `CORPUS_FEATURE` selects `hotspots` (ordinary integer/byte lists
with empty, singleton and byte-buffer controls), `toko`, `derive-scale-1`,
`derive-scale-10`, or `derive-scale` (100 distinct types). Omit it for the
standard corpus. Each selection uses the same runner and separate codec artifacts:

```sh
CORPUS_FEATURE=hotspots POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/hotspot-comparison
CORPUS_FEATURE=toko POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/toko-comparison
CORPUS_FEATURE=derive-scale POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/derive-100-comparison
```

`make check-consumer-benchmarks` qualifies these fixtures without timings;
`check-benchmarks` includes it. The Toko cohort contains copied small DTO shapes
and a labelled token-card projection, not a dependency on Toko or its endpoints.
The derive cohorts use Serde derives versus `CandidType`/Serde derives;
every selected type is exercised. Identical generated code may still be shared
by the compiler. `build-times.csv` records single build-command durations rounded
to whole seconds: dependencies may be cached, and these are neither isolated
macro-expansion timings nor statistically established compiler speedups.

### Uncompressed Wasm footprint

The benchmark profiles use `strip = "symbols"` for both codecs. This removes
function-name metadata and reduces total Wasm size, at the cost of diagnostic
backtrace names. It does not directly reduce the code section subject to the IC
code-size ceiling. Keep total Wasm and code-section measurements separate.

A consumer can select the same setting in its own workspace's deployment profile:

```toml
[profile.release]
strip = "symbols"
```

IC Wire cannot set a consuming workspace's profile. Preserve the consumer's other
profile choices; use a separate symbol-preserving build when diagnostics need it.
Existing stripped deployments may already obtain this saving. See the
[qualified comparison](candid-consumer-comparison.md#small-savings-retained-after-instruction-qualification).
