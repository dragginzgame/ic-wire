# Platform 0.15.2 / Testkit 0.74 adoption

2026-10-31. This batch updates dependency requirements and the managed Build
release editor, leaving own versions Core 0.13.0 / Endpoint 0.2.0. Pending notes
select compatible Core 0.13.1 / Endpoint 0.2.1. No staging, commit, release,
publication or sibling checkout modification was performed.

| Supplier | Current selection |
| --- | --- |
| Platform / macros / runtime guard | 0.15.2 |
| canonical Platform Types | 0.15.1 |
| IC Build / Host artifacts, fs, process | 0.73.1 |
| IC Testkit | 0.74.0 |
| IC Metrics (arithmetic-only, through Testkit) | 0.23.2 |

Latest non-yanked selections were checked against the public crates.io sparse
index. [Resolved supplier checksums](selections.json) identify the exact sources.
[Platform notes](https://github.com/dragginzgame/ic-platform/blob/3587cd2633b5c94fd7ebf9dea3eefdb0433ea59d/releases/platform/CHANGELOG.md)
cover independent runtime/Types groups;
[Testkit notes](https://github.com/dragginzgame/ic-tooling/blob/1a78892fe04edd43e07cb3d1709b36088a1a7e49/releases/integration/CHANGELOG.md)
identify its Platform identity migration and arithmetic-only Metrics selection.

The incoming working tree already updated Build/Host to 0.73.1,
Platform/macros/guard to 0.15.2 and Types to 0.15.1 in Cargo.lock. These changes
were preserved. This adoption additionally updates requirements/editor selection,
Testkit 0.73 to 0.74 and its Metrics 0.22 to 0.23.2 selection. The obsolete native
Platform 0.13 lock entry disappears; there is now one Platform package.

## Source and compatibility

[Source review](source-review.json) compares the committed suppliers with the
current registry sources. Platform's 112 src files, five macro files, runtime
guard, Testkit's 28 src files and Build's 71 src files are identical. Types changes
only module documentation; Metrics changes only application documentation.
Host fs changes one test file; Host artifacts/process src files are identical.
Package/feature metadata changes are qualified through the real consumer graphs.

Types retains the compatible 0.15 identity. Core schema coherence, limits,
framing, codec bytes, admission/primitive acceptance, retry/hash inputs and endpoint
rollback semantics are unchanged. No compatibility shim or new production mode
is introduced. Account absent `01 04 00` and explicit zero `01 04 01` followed by
32 zero bytes remain distinct.

Testkit's optional `wire` feature still selects registry Core 0.12.1; this
workspace does not enable that feature. These endpoint tests use the existing raw
PocketIC transport with Core-owned frame encoding/decoding. They do not claim
that Testkit's optional public Limits/Error types already align with Core 0.13.
Metrics native metadata likewise stays disabled. Those producer adoptions remain
separate from using the current Testkit server lifecycle and measurement API.

## Focused qualification

[Checks](checks/) retain:

- `make check-platform-types check-endpoint`: standalone derives; Rust 1.91
  host/Wasm; four schema-only and five combined tests; exact generated
  Rust/TypeScript Principal/Account frames and malformed refusals; strict affected
  Clippy; 11 expected endpoint diagnostics and passing renamed-import control;
  no runtime in core/schema-only, one guard in deployed Endpoint.
- The existing future-runtime fixture changes Platform 0.15.2 to simulated 0.16
  and Endpoint 0.2 to simulated 0.3 while retaining exact Core/Derive/Types IDs.
  [Identities](identities.json) describe that simulation, not published releases.
- `make check-endpoint-ic`, explicitly selecting the already prepared server
  below: installed `typed_calls_limits_and_transactions` passes, including limits,
  malformed input, reply traps and synchronous/asynchronous rollback behavior.
- The three default Wire/Candid corpus roundtrip/refusal tests pass. All default
  benchmark examples with `wire,candid,ic` check on Rust 1.91. This is correctness
  and compilation qualification, not a rerun of the full performance matrix.
- Dependency pins/boundaries and links pass. Build 0.73.1's CLI metadata editor
  was explicitly prepared and its offline selection check passes.

Installed server:
`/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic`,
SHA-256 `69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
This server lacks the instruction-trace API needed by the maintained endpoint
matrix. Fresh instruction/cycle samples, repetitions and small/large controls
are therefore **not measured**; no runtime speed or instruction-saving claim is
made. A fresh full Candid matrix and hosted macOS matrix were not run.

## Artifact comparison

[Reproduction runner](reproduce.sh), [baseline revision](baseline-revision.txt)
and [supplier-only inputs](inputs.patch) compare the committed release source
with this selection in isolated copies. Both retain Core 0.13 / Endpoint 0.2,
identical endpoint inputs and source, and all unrelated dependencies. The
candidate patch includes the incoming supplier lock updates. Sources and targets
are isolated; no root or sibling checkout is modified by the runner.

[Compiler](compiler.txt): Rust 1.99.1 / LLVM 23.1.1,
x86_64-unknown-linux-gnu host, wasm32-unknown-unknown guest. Both profiles use
release LTO, one codegen unit, aborting panics and stripped symbols. These are
artifact-size observations, not native timing measurements. Each deterministic
artifact is built once; no repeated performance samples are implied.

| Profile | Total uncompressed Wasm before → after | Code section before → after | Change |
| --- | ---: | ---: | ---: |
| opt-z (evaluation) | 100,665 → 100,665 B | 86,766 → 86,766 B | 0 B / 0% |
| opt-3 (benchmark-speed) | 94,073 → 94,073 B | 81,127 → 81,127 B | 0 B / 0% |

[Artifact hashes and deltas](artifacts.json) retain absolute results. Hashes
differ; equal sizes do not establish identical modules or instruction costs.
No optimization or gzip-size claim is made. Original isolated outputs remain at
`target/adoptions/platform0152-testkit074/artifact-comparison/`.

With the locked registry inputs and matching compiler prepared:

```sh
bash docs/evidence/platform0152-testkit074/reproduce.sh /tmp/wire-platform0152-repeat
make check-platform-types check-endpoint
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make check-endpoint-ic
cargo test --locked -p ic-wire-benchmarks --features wire,candid --test roundtrip
cargo +1.91.0 check --locked -p ic-wire-benchmarks --examples --features wire,candid,ic
make release-editor-check check-pins check-boundary check-doc-links
```
