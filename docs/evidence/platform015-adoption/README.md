# Platform 0.15 version alignment qualification

2026-10-31. Baseline is core Git 0.12.1 at
`ae9a3eda0449729008d849c6e078a8b7f849ab9e`, Platform 0.14 / Types 0.1.
Candidate selects published Platform/macros/guard/Types 0.15.0. No other lock
selections change; IC Build/Testkit stays 0.73. Own versions remain core 0.12.1 /
Endpoint 0.1.0. Pending notes select core 0.13 / Endpoint 0.2. No staging,
commits, releases, publication or sibling checkout edits were performed.

Types 0.1 → 0.15 changes Principal/Account Rust identity. Consumers must align
schema/typed clients and direct facade/Types suppliers on the supplying core minor;
endpoint users select the supplying Endpoint minor. Upstream now versions Types
with its workspace. Wire package isolation works while Types is held fixed; it
cannot preserve Rust identity when that supplier itself changes incompatibly.
No wrappers, compatibility lanes or alternate schema trait were added.

Protocol unchanged: frame header/identity, Serde order, primitive/malformed
acceptance, byte/allocation/node/depth budgets and retained retry/hash inputs.
Principal stays bounded bytes (max 29); Account stays owner then optional 32-byte
subaccount. `01 04 00` remains absent; `01 04 01` plus 32 zero bytes stays explicit
zero. Response trap and sync/async transaction rules are unchanged.

## Source and focused checks

[Registry source review](source-review.json) records package checksums. Platform's
112 source files, five macro files and runtime guard are byte-identical. Types'
ten source files differ only in its module documentation describing coordinated
versioning. No implementation change or optimization is claimed.

[Logs](checks/) retain `make check-platform-types` and `make check-endpoint`:
standalone native derives, current/MSRV host/Wasm, four schema-only/five combined
tests, exact Rust/TypeScript frames/refusals, renamed macro control, 11 expected
diagnostics, strict affected Clippy, optimized Wasm and core/schema/endpoint graph
isolation. Core has no runtime; schema-only has no CDK/executor/Candid; deployed
endpoint has one guard and no host serde_json. The prospective identity test now
resolves selected versions from metadata and constructs a simulated 0.16 facade
with Types held at 0.15. That does not predict upstream's actual next release.
Pins, boundaries, documentation links, syntax, ShellCheck and diff checks pass.

Installed `typed_calls_limits_and_transactions` passes on the actual candidate
Wasm, including limits/refusals and reply rollback. Native Testkit's separate
Platform 0.13 remains outside the deployed graph. No new downstream publication,
full CI/macOS, native timing or full Candid performance matrix is claimed.

## Repeated endpoint measurements

The matched seven-export, 12-case probe runs eight measured repetitions after an
excluded warmup per case/lane/profile: 384 logical calls / 608 physical executions.
Every paired instruction and cycle sample matches. Frame hashes, outcomes and
transaction oracles match. [Comparison](comparison.json) retains all absolute
instruction samples/ranges and cycle means; [raw traces](samples.json.gz) include
individual cycle debits and module/server identities. There are no measured
instruction/cycle or size regressions (0%).

| Profile | Total uncompressed Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| opt-z | 100,441 → 100,441 B | 86,544 → 86,544 B |
| opt-3 | 94,073 → 94,073 B | 81,127 → 81,127 B |

Small controls include empty and 32-byte blobs; the large control is a 65,522-byte
blob in a 65,536-byte request. Opt-z `echo/blob-0` is 253,817 instructions before
and after; opt-3 is 247,520. Rejections and async/composite calls remain visible
in the complete JSON. Binary hashes differ despite equal sizes and measured cost;
this is not a byte-identical artifact claim. Compressed raw traces are evidence
storage only; gzip artifact size is not an acceptance metric.

## Reproduction

Use the [recorded compiler](compiler.txt): Rust 1.99.1 / LLVM 23.1.1,
x86_64-unknown-linux-gnu host and wasm32-unknown-unknown guest; Rust 1.91 for the
minimum checks, Node 24.21 / TypeScript 7.0.2 for conformance. Profiles inherit
release LTO, one codegen unit, panic abort and stripping. Prepare locked registry
inputs and the [frozen instrumented server](server.sha256) explicitly.

```sh
make check-platform-types check-endpoint check-pins check-boundary check-doc-links
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic-server \
  bash docs/evidence/platform015-adoption/reproduce.sh /tmp/wire-platform015-repeat
```

The output directory must be fresh; local loopback is required. The runner uses
[baseline revision](baseline-revision.txt), an empty [baseline input patch](baseline-inputs.patch)
(no extra admitted inputs) and the [supplier-only patch](inputs.patch), isolated
sources/targets and the existing 12-case selection/analyzer. Original outputs:
`/tmp/wire-platform-alignment/comparison`. No historical evidence is relabelled.
