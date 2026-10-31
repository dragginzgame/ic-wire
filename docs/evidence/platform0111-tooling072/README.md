# Platform 0.11.1 / IC Tooling 0.72 adoption

The workspace selects Platform 0.11.1 and Build/Testkit 0.72.0 with coordinated
macros, runtime guard and Host packages. The managed release editor is 0.72.0.
Shared Tooling's remote main is still the already-reviewed 0.27.11 snapshot at
`62edfbd43087a718bf0283e9040676a49211976d`; its 116 payloads verify unchanged.
The incoming dirty lock's Platform 0.11.1 selection is preserved. The benchmark
baseline is the released source, not that incoming lock.

Testkit now shares Platform 0.11.1 with Wire, removing its older 0.9.1 dependency.
Core-only Wire remains Platform-free; `platform-types` selects only Principal
and native schema support, while `endpoint` selects CDK. No Wire API, codec,
serialization, budget, malformed-input or retry-byte rule changes. Own Wire
versions stay at released 0.10.0; this compatible adoption has 0.10.1 patch notes.

## Review and checks

The [registry source review](source-review.json) compares every source file in
Platform/macros/runtime-guard, Build/Testkit and the three selected Host packages.
All source content matches the previous selections. Changes are dependency and
release metadata; the [input patch](inputs.patch) captures exact requirements,
registry checksums and the editor selection against the recorded
[baseline revision](baseline-revision.txt).

Passed: standalone schema consumers and combined endpoint graphs, Rust 1.91
native/Wasm, strict Clippy, exact Rust/TypeScript frames and malformed refusals;
endpoint native/schema checks, 11 expected diagnostics plus passing control,
Wasm/MSRV; installed limits and rollback; all three Candid comparison corpus
roundtrip/rejection tests; managed editor admission; snapshot, pins and dependency
boundaries; format and script syntax. The new editor passes the disposable
release adapter's refusal, failed-validation retention, delivery, resume and
receipt checks. Only disposable local repositories receive fixture commits,
tags and pushes, with expensive validation substituted. Nothing is released or
published from the real checkout. No full CI, macOS or sibling suite was run.

## Measurement scope

The baseline is Wire 0.10.0 with Platform 0.11.0 and Tooling 0.71.0. The candidate
applies only the recorded input patch. Both guests use the same candidate host
driver and instrumented server, unchanged input frames/budgets, compiler and
profiles (LTO, one codegen unit, panic abort, stripped symbols; opt-z or opt-3).
The [existing 12-case selection](../platform010-tooling071/cost-cases.patch)
covers seven exports, empty and 32-byte controls, a 65,522-byte blob, borrowed
and async paths, composite self-call, mutation/reply failure and malformed input.
Each case uses one warmup plus eight measured repetitions per lane/profile.
This is a focused endpoint supplier comparison, not the full Candid matrix or
an application/native-time/host-generation benchmark.

## Results

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| opt-z total uncompressed Wasm | 99947 B | 99951 B | 4 B (0.004%) |
| opt-z code section | 86068 B | 86068 B | 0 B (0.000%) |
| opt-z echo/blob-0 instructions | 253815 | 253815 | 0% |
| opt-z default_blob/blob-65522 instructions | 3884431 | 3884431 | 0% |
| opt-z default_mutate/reply-32 instructions | 492549 | 492549 | 0% |
| opt-z default_mutate/reply-32 observed cycle debit | 7000549 | 7000549 | 0% |
| opt-3 total uncompressed Wasm | 93518 B | 93518 B | 0 B (0.000%) |
| opt-3 code section | 80630 B | 80630 B | 0 B (0.000%) |
| opt-3 echo/blob-0 instructions | 247492 | 247492 | 0% |
| opt-3 default_blob/blob-65522 instructions | 3879734 | 3879734 | 0% |
| opt-3 default_mutate/reply-32 instructions | 487891 | 487891 | 0% |
| opt-3 default_mutate/reply-32 observed cycle debit | 6995891 | 6995891 | 0% |

All measured instruction and cycle samples match, not only means. Frames,
outcomes and transaction oracles also match. The study records 384 measured
logical calls and 608 physical executions. Zero query cycle debit is an observed
balance change in this harness, not a general billing claim. Native timing and
host-generation speed were not measured; no claim is made for them.

[Full results](comparison.json) retain absolute samples, ranges and percentages;
[raw samples](samples.json.gz) retain physical-execution evidence. The
[compiler](compiler.txt) and [server identity](server.sha256) bind the execution
environment. Compression is evidence storage only, not a Wasm metric.

## Reproduction

With cached registry dependencies, the recorded compiler, prepared Wasm target
and explicit instrumented PocketIC server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic-server \
  bash docs/evidence/platform0111-tooling072/reproduce.sh /absolute/fresh/output
```

The runner reconstructs both workspaces from Git and the input patch, builds
both profiles, checks installed behavior and measures the fixed subset. It
reuses the existing analyzer and retains exact compiler, server and artifact
identities. It performs no network fetch, installation or real release effects.
Run `make check-platform-types check-endpoint` for the consumer compile/schema
proofs, and the explicitly prepared editor through `make release-editor-check`
and `bash scripts/ci/test-local-release.sh` for the disposable adapter proof.
Original outputs are `/tmp/wire-platform0111-tooling072/`, focused logs are
`/tmp/wire-platform0111-tooling072-prep/`, and the standalone schema consumer is
`target/platform-types.pTChxI`.
