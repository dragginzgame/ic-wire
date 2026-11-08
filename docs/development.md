# Local development

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

IC Wire provides a no_std + alloc Postcard runtime. See the
[runtime guide](runtime.md) for the wire contract and [evaluation scope](evaluation.md)
for maintained qualification. The optional [endpoint integration](endpoints.md)
delegates IC exports/execution to Platform; application policy remains consumer-owned.

The benchmark package's `benches/*.rs` files are Cargo target roots, colocated
with their shared modules; report utilities live under `tools/`. Test bodies
and test-only helpers use `tests/`, `tests.rs` or `*_tests.rs`. Unpublished
packages declare their LOC roles in Cargo metadata; `make cloc` reports those
roles and flags ambiguous source for review. Ordinary module discovery keeps
benchmark-only dependencies out of public libraries.

## Setup

Prepare Bash, Git, GNU Make, curl with CA certificates, tar, gzip, xz, Perl,
a SHA-256 implementation and native Rust build prerequisites as described in the
reviewed [common setup guide](local-setup.md#bootstrap-prerequisites).
On macOS, install Xcode Command Line Tools and xz; on Linux, prepare a C compiler,
linker and the development packages needed by Cargo-installed tools.

```sh
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
make install-tools
make tools-check
make install-hooks
```

`make install-tools` also prepares IC Tooling's IC Build 0.79.0 release metadata
editor with its required `cli` feature. The diagnostic library selects IC Build
0.79.0 without default features; the CLI feature is only enabled for installation.
`make release-editor-check` verifies its selected executable offline. The release
adapter selects that path automatically; metadata inputs are limited to 8 MiB
per document. This tooling executable is separate from runtime dependencies.
Core defaults compile the codec without `ic-wire-derive`; schema-generating
consumers explicitly select `typescript` (or its `platform-types` forwarding).
`check-schema-targets.sh` checks standalone default/schema host/Wasm builds and
proves the metadata crate is absent from codec-only artifacts.
Production Endpoint selects Runtime; optional schemas select canonical Types.
Native endpoint/codec controls import the async PocketIC client and DTOs from
`ic_testkit::pocket_ic`, with Testkit also owning server admission/lifecycle.
Drivers/tests own one Tokio runtime, await client operations and explicitly
await `instance.close()` before dropping an owned server. Tokio stays in native
qualification dev dependencies; no per-operation blocking wrappers are added.
Testkit 0.90 constructors receive an explicit 300-second request budget, preserving
the former default allowance. It covers one complete exchange, including response
reads and accepted-operation polling. The independent 60/90-second startup
allowances remain unchanged; expiration is propagated without replaying calls.
The Platform facade supplies external Candid/crypto controls. See the
[current handoff](status/current.md#compiler-suppliers-and-release-boundary) for the
qualified supplier cohort; root manifests and Cargo.lock own selections. The
release metadata executable is pinned separately.

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
Rust 1.99.1 is both the minimum supported and pinned development/release compiler.
`make check-rust-policy` reads the workspace floor, verifies the pin and records
actual Cargo/rustc identities. `make check` and `make check-wasm` qualify the
host and Wasm targets under that same compiler. There is no separate MSRV lane.

| Target | Scope |
| --- | --- |
| `build`, `check`, `test` | Locked runtime builds and behavior |
| `lint` | Public-package strict Clippy with schema features; `RELEASE_GROUP=core` or `endpoint` narrows packages, full release plans explicitly lint both groups |
| `check-wasm` | Runtime compilation for Wasm under the pinned compiler |
| `check-typed-diagnostics` | Offline compile-fail subjects plus a passing control |
| `check-qualification` | Allocation/borrowing, frozen bytes/admission and contract-diff tests |
| `check-compatible-consumers` | Published/current Core, derive and Endpoint combinations; standalone Rust 1.99.1 host/Wasm checks; included in `check-endpoint` |
| `check-endpoint` | Standalone typed endpoint/native metadata, compiler diagnostics, lint, Wasm/MSRV and dependency isolation |
| `check-endpoint-ic` | Installed endpoint/rollback cases; explicitly prepared `POCKET_IC_BIN` required |
| `evaluate-endpoints` | All exported endpoint costs and rejection paths; fresh absolute `EVIDENCE_DIR` and instrumented `POCKET_IC_BIN` required |
| `check-typescript` | Feature-enabled Rust tests, generated vectors and differential mutations; public lint belongs to `lint` |
| `check-platform-types` | Standalone Principal/Account derives, exact Rust/TS bytes and malformed inputs, Rust 1.99.1 host/Wasm, schema-only/combined endpoint graphs; included in `check-typescript` |
| `check-stable-identities` | Current and simulated next endpoint/runtime consumers retain identical codec/derive/Principal/Account package IDs; included in `check-platform-types` |
| `evaluate-codec-matrix` | Full Wire/Candid cohort roster; requires explicit prepared PocketIC |
| `evaluate-allocation` | Explicit allocation/borrowing [native timing experiment](performance.md) |
| `fuzz-runtime` | Prepared nightly/C++ coverage-guided codec fuzzing |
| `fmt-check` | Grouped manifests, derive order and Rust formatting |
| `check-pins`, `check-boundary` | Dependency inheritance/pins, external IC/CBOR guard, Rust source and browser import boundaries |
| `check-candid-boundary` | Read-only Candid/CBOR source checks over crates, examples and the generator probe; no Cargo or macro expansion |
| `check-browser-boundary` | Shared SDK/CBOR checks for the browser example, crate runtime/fixture JS/TS and inspection helper |
| `shared-tooling-check` | Recorded snapshot bytes and modes |
| `tools-report` | Aggregate offline host/Cargo/local-tool diagnostics; runs every checker |
| `check-doc-links` | Maintained local references and every snapshot-declared Shared Markdown file |
| `check-release-validation` | Source-bound release selector/runner/receipt failure and recovery fixtures; disposable Git only |
| `check-ci-selection` | Workflow classification/gate fixtures in disposable Git histories; prepared host yq and Node required, no Rust build |
| `check-tooling`, `check-release` | Disposable Make/hook and release-adapter fixtures |
| `ci-core`, `ci-typescript`, `ci-benchmarks` | Aggregate hosted CI groups; each admits prepared tools first |

The [Candid/CBOR source check](candid-boundary.md) uses ci/candid-boundary.json to
admit only named unpublished Candid comparison DTOs/helpers, with no CBOR
exceptions. Shared owns the codec inventory; Wire does not duplicate it.
The root Cargo check covers all seven workspace members. Materialized schema,
bulk-advice, platform-type, compatibility, future-identity and generator CLI
workspaces invoke the same dependency check before compilation. Generated
Rust fixture bodies come from the checked source roots; copied Platform source
remains the external producer. TypeScript conformance, platform-type and identity
fixtures additionally check their actual generated browser roots. These checks
do not cover arbitrary old target/ scratch or prove dynamic imports, macro
expansion, cross-module alias provenance or byte compatibility. Snapshot-only
adoption is not a claim that every independent workspace has been validated.
New raw benchmark output defaults to `target/evidence/`; keep only compact topic
results under the [research retention policy](evidence/README.md).

Private corpus correctness checks now use the guest's ordinary test modules:
`cargo test --locked -p ic-wire-benchmarks --features wire,candid --example codec-wasm`.
Add a single cohort feature to select another corpus. The former standalone
`roundtrip` integration target is retired; no alias or second fixture library remains.

Use the smallest meaningful checks under the
[validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` gate requires an explicit local request or the selected
maintainer release operation. Configured hosted CI remains independent.

### Selecting change-specific qualification

The ordinary gates prove correctness and compilation; `check-benchmarks` does
not run a performance comparison. Select these additional proofs by the changed
boundary, keeping baseline/candidate compiler, lock, inputs and profiles matched:

| Changed boundary | Additional proof |
| --- | --- |
| Codec traversal, buffering, parsing or bulk representation | `evaluate-codec-matrix` on isolated sources, then `check-codec-costs` on completed runs; include small and malformed controls, IC instructions and total uncompressed Wasm/code sections |
| Endpoint expansion, admission, replies, traps or execution supplier | `check-endpoint-ic` with a prepared server; `evaluate-endpoints` for cost-affecting changes |
| Host schemas or advice | Rust/TypeScript conformance, exact acknowledgement tests and paired host-generation timings; do not claim codec/Wasm savings |
| Public codec/schema/macro interfaces | `check-compatible-consumers` (also in `check-endpoint`) plus consumer contract snapshots and frozen vectors |

The installed fixture's `admission-contract` feature runs the same frozen byte
and allocation-threshold assertions on actual Wasm32 as on the native host.
It is absent from default cost fixtures, keeping their measured endpoint roster
unchanged. Visitor charges depend on the target's value/reference sizes; host
thresholds are not a substitute for the Wasm execution proof.

### Checking a consumer contract snapshot

The private qualification CLI compares two JSON snapshots containing `types`
(the registry object), `methods` and/or `payloads` (generator declaration arrays),
and optional `vectors` (named arrays of exact retained bytes). Omitted sections
default to empty. Both snapshots pass Wire's existing schema validation first.

```sh
cargo run --locked -p ic-wire-qualification --example check-contract \
  --features schema-fixture -- BEFORE.json AFTER.json
```

Equal declarations/vectors succeed; changes or invalid inputs fail. Declaration
ordering is ignored by name, but field/variant ordering, limits, identities and
literal bytes are compared. The CLI displays at most eight changed paths and
never updates the baseline. Keep snapshots in the consumer's contract tests,
review changes and qualify adoption before replacing them. This conservative
tool compares supplied vectors; consumer tests must generate and verify those
vectors against the actual Rust/TypeScript implementations. The
diff can flag harmless metadata changes; it cannot prove arbitrary custom Serde
behavior, deployment compatibility or retry safety. It adds no runtime fingerprint,
wire field, fallback reader or published API. Retain original retry/signature
bytes independently of any normalized decode/re-encode result.

For code changes, hosted CI runs those three groups independently on Linux x86_64,
macOS ARM64 and macOS Intel. Each matrix job owns its checkout, `target/`, tool
preparation and failure artifacts. Only the TypeScript group prepares Node/npm.
The existing `scaffold (OS)` status names remain final gates for the selected
qualification, rejecting failed, cancelled or incomplete work.

The exact [prose allowlist](../ci/docs-only-paths.txt) covers design, evaluation,
performance and the current handoff. Reviewed readers use these files as human
documentation and link-check inputs, without embedding, doctests, generation,
packaging or Shared companion declarations. Packaged READMEs, fixture inputs,
governance, workflows, tooling and all unlisted files retain the full native
matrix. Additions require reviewing their readers and the consumer fixture.

The [shared classifier contract](ci-change-selection.md) compares exact tested
commits using complete checkout history and identical committed policy bytes.
Eligible prose changes run documentation links and whitespace checks, while
snapshot integrity and the pinned redacted history secret scan run on both
paths. Missing history or uncertain classification selects full CI; classifier
job failure fails the final gates. The gates use GitHub's
[dependency/always semantics](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-jobs#defining-prerequisite-jobs)
so a skipped branch cannot hide failed required work. A docs-only pass does not
claim native execution. No measured wall-clock saving is claimed.

The groups use the same reviewed validation runner and partition `make ci`'s
complete target roster, including the focused CI-selection fixture. The explicit
`make ci` command always runs the complete roster with sequential dispatch. Use separate checkouts for concurrent group execution;
`make -j ci` does not parallelize the shared runner's sequential dispatch.
Ordinary Rust tests already run concurrently within each test binary. Explicit
timing/allocation studies remain outside these CI groups and run in isolation.

### Release group qualification

Release verification uses [Wire's group plan](group-validation.md), independently
of the three hosted execution partitions above. Core retains all 20 stages;
qualified Endpoint-only Linux changes require 11. Shared, dependency, compiler,
generator and uncertain inputs expand to the full gate. Ordinary repository CI
and all native hosts remain covered; this does not enable validation-proof reuse.
The JSON policy is the single full-roster declaration; Make and release selection
consume Shared's admitted projection. For roster/receipt caller changes, use
`make check-release-validation` and
`bash scripts/ci/test-local-make.sh --case validation`. The latter selects only
real Make dispatch/failure routing with substituted gates, leaving its unrelated
formatting/publication fixture cases unrun.

For Shared release-entrypoint changes, use
`bash scripts/ci/test-local-make.sh --case release-routing`. This exercises both
groups' real Make includes, explicit-group refusal and runner arguments/status
with release effects substituted in a disposable repository. It does not run
formatting, publication or application gates. It also checks automatic single-group capture, explicit roster order,
group consistency and Cargo target isolation. Only the outer launcher drops
Wire's default target export; candidate Make invocations select their own target,
and caller overrides are refused.

`bash scripts/ci/test-local-release.sh core --batch` selects the two-group frozen
hook fixture. It uses real metadata, source admission, journals, tags, local Git
pushes and publication receipts in disposable repositories. Application gates
and registry uploads are substituted. It edits the normal fixture lock during
Core validation, interrupts Endpoint publication, checks uncertain-outcome
refusal, then checks explicit recovery and completed-resume idempotence.
This is hook qualification, not installed/archive or full product qualification.
Without `--batch`, the single-group fixtures exercise the standard hooks inside
a simulated captured checkout. They do not select live-checkout delivery.

Shared 0.31 captures single-group releases automatically. For an authorized Core
patch deployment use
`make release-patch RELEASE_GROUP=core RELEASE_PUBLISH_TARGET=publish`.
This includes registry publication. A multi-group delivery requires explicit
`RELEASE_GROUPS=core,endpoint` in supplier order, with `RELEASE_GROUP=core`.
The old `RELEASE_FROZEN_*` variables and runner have been removed upstream;
there are no compatibility aliases. Wire supplies `release-candidate-setup`:
Shared invokes it before version lookup, prepares the selected tools/cache for a
new group, including the locked browser compiler needed by both release plans,
and checks them offline during recovery. Preflight does not repeat setup
inside a batch. Publication remains explicit rather than a default side effect.

Use Shared's `release-batch-status`, `release-batch-resume`,
`release-batch-reconcile` and `release-batch-retire` with `BATCH=...` for the saved
roster. Reconciliation requires a clean normal checkout; retirement explicitly
deletes that completed batch's retained state. The old completed schema-1 Core
0.17.5 receipt remains historical and must not be relabeled as schema 2 or resumed
with the new runner. Its delivery was already reconciled into main.

Core 0.17.7 / Endpoint 0.7.3 qualified both full gates, real all-feature archives,
registry readback and installed Endpoint checks inside one managed checkout.
The 0.31 hook fixture proves both groups, concurrent
normal-lock isolation, uncertain publication refusal, saved commit identity,
recovery, reconciliation and retirement with substituted gates/uploads. Future
two-group deliveries retain complete supplier-first archive qualification.
See the supplied [batch contract](release-batches.md) for custody and recovery.
Core remains full-gate-only; Endpoint retains its qualified implementation paths.
The default-feature Core `check` and schema-enabled host `lint` prove different
feature graphs, and `check-wasm` proves a different target; none is removed as a
duplicate compilation. No elapsed-time saving is claimed from this adoption.

### Standalone TypeScript qualification

Prepare Node 24.21.0 and npm 12.2.0, matching `examples/browser/.nvmrc` and
the private package metadata, then:

```sh
bash scripts/ci/prepare-browser-tools.sh prepare
make check-typescript
```

`check-typescript` is included in `make ci` and release validation. Generation
retains stderr in `target/typescript-conformance/generation.log` and
replays normal bounded diagnostics. Historical handwritten-versus-derived timing
experiments are no longer part of conformance.
Local release operators select the pinned Node/npm environment; captured release
setup installs the locked package inside its own checkout. Recovery uses
`bash scripts/ci/prepare-browser-tools.sh check` without installation or network
access. Hosted TypeScript CI uses the same preparation script. The target also
runs feature-enabled Rust runtime/borrowed/native tests;
the single public `lint` stage owns strict schema Clippy. It also runs a disposable
consumer that forwards a local schema feature: native metadata and exact runtime
frames must work, while Wasm builds with and without forwarding exclude Wire's
optional `serde_json` metadata dependency from the normal graph. The consumer
uses `ic_wire::ByteBuf` without a direct serde_bytes dependency. A separate
primitive-only host/Wasm consumer excludes Wire metadata derive and checks that
Wire directly selects only Serde alloc. Postcard currently selects Serde derive
transitively, so this does not remove that compiler dependency. Consumers needing
Serde derives explicitly select them. The check
requires serde_bytes with only its `alloc` feature in the Wasm runtime graph.
The forwarded Wasm build is checked on Rust 1.99.1 too. Run only this boundary check
with `bash scripts/ci/check-schema-targets.sh`; evidence remains under
`target/schema-targets/`. These graph checks do not measure installed Wasm size.

The target also runs `bash scripts/ci/check-bulk-advice.sh`: a disposable Cargo
build-script consumer checks default warning output, independent per-field
suppression, complete acknowledgement and stale acknowledgement visibility.
Its logs remain under `target/bulk-advice/`; no installed consumer is modified.

This package contains only the TypeScript compiler. The generated codec imports
no transport or browser SDK. Rust vectors, strict typechecking and Node mutation
checks qualify byte conformance; they do not qualify endpoint authentication,
certificate verification, installed rollback or browser transport.
See [consumer fixtures and packet inspection](consumer-fixtures.md).

The same target runs packed codecs with simulated big-endian numeric typed-array
storage, using DataView as an independent byte oracle. It checks all supported packed
types and bounded i16, signed/unsigned extrema, empty fields, both directions,
unaligned frames, owned output and malformed widths. This executes the TypeScript
swap branches; it does not qualify a native big-endian JavaScript engine.
`cargo test --locked -p ic-wire --lib endian_append_matches_wire_bytes` also
exercises Rust's big-endian append helpers on the local host. Full Rust big-endian
serialization and allocation/error paths still require a real target or emulator.

### Runtime fuzz seeds

The fuzz engine belongs to the unpublished qualification package and is selected
only through its native `fuzz-runtime` feature. Ordinary codec and qualification
tests do not select it. `make fuzz-runtime` retains the prepared nightly/C++
runner and restores deterministic packed frames and payload-only seeds before every run,
using `scripts/ci/seed-packed-fuzz.pl`. No archived corpus is needed on a clean
checkout. Seeds cover every supported word type, both directions, empty fields,
length-prefix boundaries and partial-word rejection. Existing discovered corpus
entries are preserved. Cell variant indices in the seed oracle must track the
qualification enum in
`crates/ic-wire-qualification/tools/fuzz_runtime/mod.rs`.
Payload-only checks include maps/sets with reordered or duplicate entries,
recursive values, zero-sized collections and borrowed text/bytes. They exercise
fixed and input-dependent limits and compare output-buffer APIs. Maps/sets use
semantic round trips: collection normalization is consumer-owned, so the fuzzer
does not require byte-identical re-encoding. Inputs are capped at 4,096 bytes.

The runner selects a separately pinned nightly in
[`fuzz-runtime.sh`](../scripts/ci/fuzz-runtime.sh), solely for sanitizer/coverage
instrumentation. This is an approved exception to the normal Rust 1.99.1 lane;
the package floor and development/release pin stay unchanged. Prepare that
toolchain and a C++ compiler explicitly; the runner installs nothing.
To inspect only the seed corpus, create an empty directory
and run `perl scripts/ci/seed-packed-fuzz.pl DIRECTORY`. The sanitizer default is
unchanged; on hosts where ptrace prevents LeakSanitizer shutdown, an explicit
`ASAN_OPTIONS=detect_leaks=0` run retains address checking but does not check leaks.

### Allocation timing modes

The private benchmark's native `allocations` and `heap` features select
`stats_alloc` and `dhat` respectively. Ordinary timing/correctness builds select
neither; enable only one profiler per build. Both remain outside Wasm dependencies.
`check-qualification` enforces these graphs through
`scripts/ci/check-qualification-tools.sh` and selects allocation-fixture explicitly,
so routine qualification does not enable fuzzing via `--all-features`.

For the focused caller-slice versus owned/reused output study, see
[caller-storage qualification](evidence/caller-storage.md) and run
`bash scripts/ci/evaluate-caller-storage.sh NEW_DIRECTORY BASELINE_COMMIT` with
an explicitly prepared `POCKET_IC_BIN`. It retains IC instructions, separate
native timing/allocation/heap samples and uncompressed Wasm/code sizes. The
ignored benchmark tests are explicit measurements, not ordinary CI gates.

`make evaluate-allocation` retains decoding plus the application checksum as the
default. To isolate codec work, select a fresh output directory:

```sh
cargo run --locked --profile evaluation -p ic-wire-qualification --example allocation --features allocation-fixture -- target/evidence/allocation-codec evaluation --codec-only
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
The default codec graph has no Platform or simulator dependencies. The independent
`ic-wire-endpoint` package selects Runtime's Candid-free `cdk` surface. The optional
`platform-types` integration independently selects `principal` and host TypeScript
support; its standalone consumer check also qualifies the combined endpoint graph.
The unpublished benchmark and endpoint fixture packages use Testkit's async
PocketIC client and lifecycle for qualification; Platform supplies external
Candid/protocol controls. The historical inline/extracted Endpoint comparison
has been retired; its [results](evidence/endpoints.md) remain historical.

For the complete exported-endpoint matrix, run
`POCKET_IC_BIN=/path/to/instrumented/pocket-ic-server make evaluate-endpoints EVIDENCE_DIR=/absolute/new/path`.
It freezes the working tree, builds both existing Wasm profiles, checks coverage
against actual exports and records repeated full physical execution costs,
observed cycle debits, frame sizes and whole-artifact Wasm sizes. Query cycle
debits may be zero; instruction costs are measured independently. Async calls
include self-call, reply callback and cleanup executions. See the
[matrix evidence and limitations](evidence/endpoint-matrix.md).
This explicit benchmark is separate from ordinary compile checks and installed
standard-server correctness tests. It requires the profiler-enabled server in
the evidence receipt; a standard PocketIC server lacks the trace API.

## Snapshot maintenance and release boundary

The reviewed Shared Tooling revision is recorded in
[AGENTS.md](../AGENTS.md) and the [snapshot manifest](../.shared-tooling.snapshot).
The main snapshot owns the common setup, formatting, hooks, release runner and
receipt helper. Update through the canonical exporter and reviewed adoption;
do not patch snapshot-owned files. Product adapters stay local.

The maintainer-operated `release-patch`, `release-minor` and `release-major`
targets require explicit `RELEASE_GROUP=core` or `endpoint` and use one workflow: preflight, complete validation, version preparation,
receipt checks, commit, tag and direct Git delivery to origin/main. They do not
publish packages. Ordinary development leaves package versions, staging, commits,
refs and release execution to the maintainer.

### Decide whether a public release is needed

Group membership in `ci/cargo-release-groups.json` assigns qualification and
coordinated version ownership; it is not a publication trigger. In particular,
Core owns the private benchmark/qualification packages without requiring a Core
release for every change to them. `release-patch` is an explicit request to
release the selected group, not a command for landing ordinary maintenance.

Before selecting groups for delivery, compare the complete pending batch with
the last delivered source and trace its effect on the public packages:

| Changed input | Release decision |
|---|---|
| Private benchmark code, fixtures or dev-only supplier selection | Qualify the affected caller. If public package contents and dependency contracts are unchanged, land as repository maintenance; do not bump or publish Core/Endpoint or request downstream adoption. |
| Public source, features, dependency requirements, consumer compiler floor or package contents | Assess the owning group's consumer impact and select its release increment. |
| Build script or generator, including one in a private package | Trace its output and build-time consumers. `publish = false` does not exempt an input that changes the published product. Qualify the generated result and release its public owner when affected. |
| Root manifest or lockfile | Inspect inherited metadata, public normal/build/optional dependencies and feature selections separately from dev-only suppliers. A root-file change is neither an automatic release trigger nor proof of private-only impact; uncertain changes need investigation. |

Use Cargo's actual package contents instead of guessing from directory names.
For a private-only candidate, retain before/after archives from
`cargo package --allow-dirty --locked --offline --no-verify --all-features`
with explicit `-p` selections for the affected public packages and separate
`--target-dir` directories. Compare extracted files, including normalized
`Cargo.toml`, `Cargo.toml.orig`, packaged `Cargo.lock`, source, README and LICENSE.
Inspect `.cargo_vcs_info.json` separately: source/dirty provenance may change
without changing the product. Record the source revision, patch, compiler and
commands under `target/evidence/`; never treat these uncompiled comparisons as
replacement release qualification. Archive equality alone also cannot excuse an
unreviewed external build/generator input. Use the existing package and graph
checks for an actual release; do not add a second release planner.

Keep meaningful private maintenance notes in the owning ledger's numbered,
undated pending entry under the adopted Shared changelog rules, explicitly marked
as deferred maintenance, and summarize the decision in the current handoff.
That candidate heading does not change package versions or require publication;
carry the notes into the next independently justified public release. Ordinary
authorized source delivery can land the work without release tags or uploads.
Do not rewrite previously published versions to undo an unnecessary release.

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

`make check-package RELEASE_GROUP=core` (or `endpoint`) assembles and verifies
the selected group's two public archives with all features,
permits dirty source for local review and may use the registry. Package allowlists
exclude tests and evaluation artifacts. Each archive includes its README and the
root MIT LICENSE as an identical package-local regular file; only the SPDX license
metadata is declared. This command uploads nothing.

After committing and releasing the reviewed source, the maintainer may rehearse
or execute the separately authorized publication with Cargo 1.99:

```sh
make publish-check RELEASE_GROUP=core
make publish RELEASE_GROUP=core
# Independent endpoint publication, after its supplying core:
make publish-check RELEASE_GROUP=endpoint
make publish RELEASE_GROUP=endpoint
```

Publication is not atomic across packages. If a previous attempt uploaded the
derive crate but stopped before the runtime, retain the released versions and
resume only the remaining package after the derive version is available in the
registry index:

```sh
make publish RELEASE_GROUP=core PACKAGE=ic-wire
```

The runtime-only retry applies to the same released version. If another release
has advanced the core group version, its matching derive version must be published
too: use `make publish RELEASE_GROUP=core PACKAGE="ic-wire-derive ic-wire"`. A tooling or Git
release does not publish either crate, and a new release is not required merely
to finish an earlier partial publication.

The shared [publication adapter](cargo-publication.md) selects derive/core for
explicit `RELEASE_GROUP=core`, or Endpoint for `RELEASE_GROUP=endpoint`. Missing,
empty and unknown groups refuse before setup; PACKAGE subsets cannot cross groups. It observes locked offline workspace metadata, admits the public roster,
resolves exact `name@version` selectors, runs lint, then makes one Cargo attempt
against `crates-io`. `publish-check` uses the same selection and gates with
`--dry-run`; both require clean source. Keep `check-package` for dirty-source,
all-feature archive review. Selection/attempt logs and exit statuses stay under
`target/publication`; interrupted attempts retain a lock for inspection.

Explicit selection avoids retrying an immutable uploaded version; it does not
automatically infer registry state or ignore upload failures. Empty, duplicate
and private-package selections are rejected before validation or upload. The old
`PUBLISH_PACKAGES` option is refused; use command-line `PACKAGE` instead.
For a direct Cargo retry, use
`cargo publish --locked -p ic-wire`. Check that the existing derive upload is the
intended release; published versions cannot be replaced.

Consumers retain Rust 1.99.1. Registry identity, credentials, publication status and
repository visibility remain maintainer concerns. The qualification package remains
`publish = false`.

## Supported hosts

The native scaffold supports Linux x86_64, macOS ARM64 and macOS x86_64 through
the configured CI matrix. Local Linux checks do not qualify either native macOS
host. Keep [host support](supported-hosts.md), compiler-floor checking and Wasm
compilation distinct from runtime performance or consumer deployment acceptance.

## Candid comparison benchmarks

For CLI-only changes, run `make check-benchmark-cli` and
`cargo test --locked -p ic-wire-benchmarks --example codec-report`.
The smoke check uses disposable files, exercises actual Cargo/CLI callers and
builds the copy-only generator probe; it starts no measurements or servers.
The initial locked workspace build supplies dependencies for the subsequent
offline probe. Argument errors exit 2, help exits 0, and suite/report outputs
remain create-only. Paths use native OS values; parsing happens before
measurement and server setup.

Run the aggregate `make check-benchmarks` for CLI/corpus correctness, lint and
Rust 1.99.1 checks.
This is not a performance regression gate. A full measured comparison is a
separate run; the [current report](candid-comparison.md)
records the latest full matrix. The [0.7.10 report](full-comparison-0710.md)
retains its matched previous-release qualification; the current refresh is not
a matched regression comparison against that older dependency/server graph.
Platform 0.7.5's bounded Candid decoder admits at most 16 constructed wire levels.
The shared tree comparison therefore uses depth 7 (eight record/vector pairs).
Depth 8 stays covered as Wire-success / typed Candid nesting refusal, rather than
a successful-decode speed comparison. Historical `tree-depth-8` measurements and
the new `tree-depth-7` row have different inputs and must not be paired. The
upstream guard also adds Candid admission work; old codec reports keep their
recorded Platform version and do not quantify this new comparison cost.
For the complete maintained comparison, set `POCKET_IC_BIN` to an explicitly
prepared server and run `make evaluate-codec-matrix`. The maintained 38-cohort
roster (including `hotspots`) covers standard/Toko corpora, 1/10/100 ordinary
derived records, bounded packed-i16 and
ordinary controls, shared/varied bound records, and generic packed integer/hash
fields plus matching ordinary controls, byte-key maps, focused cost controls and
the contract-shape cohort described in [evaluation scope](evaluation.md). These are
benchmark cohorts within the one codec, not selectable production strategies.
`coverage-shapes` adds external byte-adapted Candid controls, owned Principal
lengths, nested packed containers, deterministic hash collections, bounded Nat,
standard-library types and float/bool size sweeps. `research-bulk` pairs that
same corpus with private float/bool byte-buffer fields; it adds no public codec,
Packed word, schema or generated client. The latest full report includes both
cohorts while separating research from production results; their original
qualification remains in the [follow-up study](evidence/primitive-bulk.md).
Domain-container cohorts additionally compare ordinary Serde, copying Packed
adapters and the public borrowed encode adapters. Their 1/10/100-record controls
compare those public adapters against preconstructed Packed DTOs, retaining
monomorphization cost separately from conversion latency.

The runner freezes source/lock before measurement, requires a fresh output
folder, runs cohorts serially and retains both optimization profiles, two native
runs of eleven samples, allocation/peak-heap probes, raw Wasm/code sizes and IC
counters. Native and IC measurements remain separate. Prepare locked dependencies
first; the full runner is offline. Use `bash scripts/ci/evaluate-codec-matrix.sh
NEW_OUTPUT` for another run. It is an explicit experiment, outside ordinary CI.
Separate baseline/candidate source copies must use separate `CARGO_TARGET_DIR`
directories. Cross-check local compiler-artifact freshness and the driver's baked
source hashes against each snapshot; sharing a cache across copied sources can
reuse artifacts whose original timestamps precede the other source's changes.
The matrix's cohorts may share a target because they use one frozen source.

Run `make evaluate-codecs` for native timing, allocation counts, peak live heap
and Wasm footprint under both optimization profiles. It requires the prepared
`wasm32-unknown-unknown` target and `shasum`; it does not install tools.
Set `POCKET_IC_BIN` to an explicitly prepared server to include IC instruction
measurements. The driver uses IC Testkit for server admission/lifecycle and never
downloads it implicitly. Output directories must not already exist. Use
`bash scripts/ci/evaluate-codecs.sh target/another-run` to retain another run.
See [methodology and results](candid-comparison.md) before interpreting ratios.

The optional `CORPUS_FEATURE` selects `hotspots` (ordinary integer/byte lists
with empty, singleton and byte-buffer controls), `toko`, `derive-scale-1`,
`derive-scale-10`, `derive-scale` (100 distinct types), or `contract-shapes`.
The [cohort roster](../scripts/ci/codec-cohorts.psv) owns the full selection list:
`name|features|primary-codec|qualification`. The last column selects `host`
(test/Clippy) and/or `wasm` compile qualification; an empty column selects neither.
All rows still participate in the explicitly requested measurement matrix.
Omit it for the standard corpus. Each selection uses the same runner and separate
codec artifacts:

```sh
CORPUS_FEATURE=hotspots POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/hotspot-comparison
CORPUS_FEATURE=toko POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/toko-comparison
CORPUS_FEATURE=derive-scale POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/derive-100-comparison
CORPUS_FEATURE=id16 POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh /absolute/new/id16-output
CORPUS_FEATURE=id16-control POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh /absolute/new/id16-control-output
CORPUS_FEATURE=cost-smoke POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/cost-smoke-comparison
CORPUS_FEATURE=contract-shapes POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/contract-shapes-comparison
CORPUS_FEATURE=signed32 POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/evidence/signed32-comparison
```

`make check-consumer-benchmarks` qualifies these fixtures without timings;
`check-benchmarks` includes it. The Toko cohort contains copied small DTO shapes
and a labelled token-card projection, not a dependency on Toko or its endpoints.
The derive cohorts use Serde derives versus `CandidType`/Serde derives;
every selected type is exercised. Identical generated code may still be shared
by the compiler. `build-times.csv` records single build-command durations rounded
to whole seconds: dependencies may be cached, and these are neither isolated
macro-expansion timings nor statistically established compiler speedups.

### Paired cost regression check

After preparing two matched `cost-smoke` runs with the same fixture/driver,
dependencies, compiler, environment and explicitly supplied PocketIC binary:

```sh
make check-codec-costs \
  CODEC_BASELINE_DIR=/absolute/completed/baseline \
  CODEC_CANDIDATE_DIR=/absolute/completed/candidate
```

This only reads completed evidence. It requires both profiles, paired Wire/Candid controls,
exact input/frame hashes, complete operation coverage, three identical IC counts
with distinct query callers, and the four physical uncompressed Wasm artifacts.
It checks total bytes and section-10 payload bytes independently against their
reported hashes and sizes. Candidate build JSON must show the Wire library and
guest actually rebuilt; copied source with older mtimes can otherwise reuse
stale Cargo artifacts. Use one fixed worktree and refresh changed input mtimes
before building both sides; do not share its target directory with another job.
The compiler/profile and guest features must match. Keep global Cargo config and
any other build inputs fixed too: recorded identities are not a hermetic build.
Old reports lacking the current producer metadata require a fresh matched pair.
This includes the shared fixture-inventory and operation-layout source hashes;
producer consolidation does not relax the independent verifier.

The automated screen permits no growth in total or code-section bytes; each Wire operation
must cost at most baseline × 1.05 + 64 IC instructions. Candid artifacts/counters
must remain unchanged. Native timing, allocation and heap findings still need
separate review; the gate does not infer cycles or application code-limit headroom.
Opt-z is the primary decision profile for our deployments; opt-3 remains a
secondary regression check. Adoption does not require winning both profiles.
Small measured uncompressed Wasm growth may be accepted for substantial
instruction savings with an explicitly recorded tradeoff. Keep the screen's
failure visible rather than relaxing thresholds or reporting it as a pass;
evidence-integrity requirements still apply. The approved i32 writer's
[study](evidence/encoder-followup.md#raw-tuple-i32-follow-up-8-november-2026)
records one such decision.
Unknown codec/operation labels and Candid cases without Wire counterparts are
rejected; optional Wire buffer-reuse rows need no Candid reuse measurement.
It accepts no compressed artifact as a raw Wasm measurement. It never rebuilds,
starts a server, downloads tools or rewrites the baseline. The cheap synthetic
corruption tests run with `make check-codec-costs-tests` and as part of
`check-benchmark-cli`; ordinary CI does not run the performance experiment.

The [matched 0.13.4 study](evidence/cost-experiments.md) retains
exact source, artifacts, samples and reproduction commands, including a discarded
cache-contaminated attempt and the corrected fresh-build checks.

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


Endpoint's `query`/`update` generator is the `ic-wire-endpoint-macros` package;
its unit/lint inputs belong to the Endpoint release group. Publish that supplier
before `ic-wire-endpoint`, or select both with `RELEASE_GROUP=endpoint`. Core's
metadata derive has no endpoint feature. The maintained
`evaluate-endpoint-generation.sh BEFORE_SOURCE NEW_OUTPUT` probe compares exact
expansion tokens and eleven alternating-lane host samples against a saved
expander source, building the current Endpoint macro owner.

## Cleanup investigation decisions (8 November 2026)

The architectural follow-ups distinguish source simplification from potential
host/runtime optimizations. No codec-byte, schema, identity, budget or acceptance
change is adopted by these investigations.

| Issue | Decision and evidence |
| --- | --- |
| [51](https://github.com/dragginzgame/ic-wire/issues/51) | Retain admission APIs. Contract guards know the identity and call the shared full-header preflight; manual composition can admit size before it obtains an argument buffer. Both paths share the length check, and standalone decode must validate its own inputs. Unifying the signatures would broaden the manual API and add header-copy work rather than remove a duplicate implementation. |
| [52](https://github.com/dragginzgame/ic-wire/issues/52) | Retain bounded packed-i16 metadata. Rust already shares the word writer and bounds-erased decoder; TypeScript already shares packed read/write branches. A unified grammar would introduce optional element limits, word-width/overflow rules and client regeneration to replace a small explicit bounded form. No demonstrated net reduction justifies that cut. |
| [53](https://github.com/dragginzgame/ic-wire/issues/53) | Implemented: one shared walker suppresses rendering in validation paths. Matched opt-z validation improves 40–56%, with 95–99.5% fewer allocations in large/nested controls. Small generation adds 85 ns (4%); output/rejections match. See the host study in docs/evaluation.md. |
| [54](https://github.com/dragginzgame/ic-wire/issues/54) | Retain benchmark constructor boundaries. Generic lane/case helpers already share codec setup. Bulk paired constructors deliberately use different Wire and Candid DTOs; packed-i16 preserves ordinary-vector controls; scale macros instantiate nominally distinct types to measure growth; Toko DTOs preserve their field/layout semantics. Collapsing these into one constructor would require more feature/generic configuration and could change measured monomorphization. No fixture rewrite or new matrix saving is claimed. |
| [55](https://github.com/dragginzgame/ic-wire/issues/55) | Replace three identical scaffold jobs with one qualification gate. GitHub reported no main branch protection and no rulesets at investigation time. All nine native OS/suite jobs and docs-only routing remain; 192 combinations of classifier, scope and aggregate job states pass the local truth-table check. Workflow lint passes. Final hosted execution awaits delivery. |
| [56](https://github.com/dragginzgame/ic-wire/issues/56) | Upstream PR 219 already merged the fix. Qualified pinned upstream with codec/schema host/Wasm consumers and all error variant bytes. Isolating the derive feature gives 2.53 → 1.85 s clean native build medians (26.9%); macro dependencies remain via thiserror. The registry still supplies old 1.1.3, so track published-release adoption rather than submit a duplicate fix. See docs/evaluation.md. |

Focused evidence lives in `target/cleanup-issues/`; it is regenerable and is not
a publication or full CI receipt. Issues 51, 52 and 54 remain declined; 55 is locally implemented. Issue 53 is now locally implemented and qualified; issue 56 now tracks adoption of a published supplying Postcard release.

## Updating the tooling bundle

The private qualification member selects crates.io `toko-tooling = "0.33"` as
a dev-dependency. Cargo.lock pins the release/checksum; the installed snapshot
records the authenticated archive and its exported entrypoints. Run
`make shared-tooling-update` for a targeted Cargo update plus installation.
After a separate Cargo update changes this selection, run
`make shared-tooling-install`. Use `make shared-tooling-status` and the offline
`make shared-tooling-check` to verify selection and installation. Cargo builds
never install scripts implicitly. Registry operations need no sibling checkout;
the old checkout was used only for initial Git ownership proof.

Tooling adoption does not alter Wire’s product release-group selection, frozen
gates, publication exclusions or delivery authority.
