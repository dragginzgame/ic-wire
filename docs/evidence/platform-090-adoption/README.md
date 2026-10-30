# Shared Tooling 0.27.11 and IC Platform 0.9 adoption

The [0.9.1 packaging follow-up](platform-091/README.md) adopts the feature-scoped
runtime guard, with unchanged runtime/macro source and artifact sizes. The
instruction results below remain scoped to the original 0.9.0 comparison.

Measured 30 October 2026 against source release 0.7.25 at
`e026eb4c8a7a71d4e74b459d774a83490c351762`. Own package versions remain 0.7.25;
0.8.0 is the pending changelog target because endpoint users must coordinate
application runtime consumers on Platform 0.9. Core-only users need no migration.
Handler syntax, frame bytes, IDs, bounds, acceptance rules and retained retries
are unchanged. ID 7 / u8 42 still encodes as
`49 57 07 00 00 00 00 00 00 00 00 2a`.

## Adoption boundaries

Shared Tooling was exported from clean committed revision
`62edfbd43087a718bf0283e9040676a49211976d` (0.27.11), retaining all 116 selected
files. Only governance payloads changed. Installer catalogs and release/publication
mechanics are unchanged; no generic producer test matrix or installer was rerun.

The new module/lint rules are applied to active Rust source. Six benchmark target
roots now live together under `drivers/`, with ordinary shared directory modules.
Candid remains a dev dependency in the unpublished benchmark package. Corpus
values and selected algorithms are unchanged. Evidence source-path hashes follow
the new layout; historical reports retain their original identities.

Lint expectations are scoped to the drivers/configurations that leave corpus
helpers unused, and to Wasm exports that require unsafe attributes. Diagnostic
inputs retain one explained allowance: compile-fail phases and injected cfg inputs
cannot fulfill the same expectations in every case. No unfulfilled-expectation
suppression or replacement module-path mechanism is introduced.

Platform and its macros move from 0.8.3 to published 0.9.0. The Wasm endpoint
normal/build graph has one Platform runtime and no external `ic-cdk-executor`,
`ic0`, Candid or PocketIC. The core codec graph is still Platform-free. Native
Testkit 0.70.5 retains Platform 0.8.3 without selecting its executor; this is not
a second canister runtime. Build/Testkit manifest minima and the independently
installed release editor remain 0.70.3. Other lockfile selections are unchanged.

## Artifact footprint

Same 17-endpoint fixture, compiler, flags and inputs:

| Profile | Total uncompressed Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| opt-z | 103,610 → 99,939 (−3,671; −3.54%) | 89,070 → 86,068 (−3,002; −3.37%) |
| opt-3 | 98,631 → 93,510 (−5,121; −5.19%) | 85,191 → 80,630 (−4,561; −5.35%) |

These are complete fixture artifacts, not estimates from symbols or production
application deltas. Compression is used only for retained evidence storage.

## Endpoint behavior and instruction costs

All 17 exported endpoints / 178 cases pass in both lanes and profiles, with eight
measured repetitions after one excluded warmup: **5,696 logical calls / 6,048
physical executions**. Exact requests, replies, outcomes and transaction-state
oracles match, including async callbacks and rollback. The [comparison](comparison.json)
and [raw paired samples](samples.json.gz) retain every regression and improvement;
[receipt.json](receipt.json) records the input identities.

| Case | Profile | Complete IC instructions before → after | Change |
| --- | --- | ---: | ---: |
| Empty echo | opt-z | 254,606 → 253,815 | −0.311% |
| 32-byte echo | opt-z | 257,021 → 256,230 | −0.308% |
| Empty echo | opt-3 | 247,721 → 247,492 | −0.092% |
| 32-byte echo | opt-3 | 249,980 → 249,729 | −0.100% |
| Ready async update, empty reply | opt-3 | 488,095 → 488,224 | +0.0264% |
| Truncated async query | opt-z | 249,509 → 249,778 | +0.1078% |
| Truncated async query | opt-3 | 246,392 → 246,849 | +0.1855% |

All successful opt-z cases improve. The largest successful-call regression is
+129 instructions / +0.0264% at opt-3; its observed update cycle debit rises
6,990,095 → 6,990,224 (+0.00185%). The largest overall regression is the truncated
opt-3 query above. Four opt-z cases and 63 opt-3 cases increase in instructions;
this is a size win with small, explicitly retained instruction tradeoffs.
Counters include physical page charges. Query cycle debit is zero in this
simulator, not evidence of zero work; these are not native timings or end-to-end
network latency measurements.

## Reproduction and check scope

Use stable Rust 1.99.1 / wasm32, LTO, one codegen unit, panic abort, stripped
symbols and the repository's `evaluation` / `benchmark-speed` profiles. The Rust
floor remains 1.91.0. Candid stays 0.10.39, Postcard 1.1.3, Serde 1.0.229 and
Testkit/Host 0.70.5. Compare the retained [baseline lock](baseline-Cargo.lock)
and [candidate lock](Cargo.lock); no unrelated dependency update is included.

The prepared instrumented PocketIC 16.1.0 executable had SHA-256
`36e063995730df089c78f00c09fc209417386bfd6efff58bf66c863899569471`.
Both lanes record that identity. It differs from earlier studies' server bytes;
only this matched pair supports the before/after comparison.

With that server and locked sources already prepared, run
`POCKET_IC_BIN=/absolute/server bash docs/evidence/platform-090-adoption/reproduce.sh /absolute/new-output`.
The [runner](reproduce.sh) uses the maintained endpoint benchmark in private source
snapshots and separate Cargo targets. The [analyzer](analyze.mjs) checks exports,
module/server identities, exact request/reply hashes, outcomes and sample pairing.
Native codec time, allocation counts and gzip size are not measured or inferred.

The first archived-baseline invocation stopped before compilation because its
source archive lacked a Git directory. It is excluded. The completed baseline
recorded the known release revision explicitly; the retained reproduction uses a
private clone to satisfy the runner's Git identity read without editing a caller's
checkout. Final driver lint narrowing and a private unused-parameter rename occurred after
benchmark snapshots. They change no endpoint/cost-matrix source or report helper
used in those runs. The renamed parameter avoids a warning in the Wasm owned-output
control when its slice/scoped features are absent; no operation changes.

The [endpoint checks](endpoint-check.log) cover native/schema behavior, compile
failures, strict Clippy, the Wasm artifact, MSRV and dependency boundaries.
[Benchmark checks](benchmark-check-before-lint-narrowing.log) cover all maintained
corpora, roundtrips, damaged frames, driver linting and MSRV. That initial run
exposed unfulfilled corpus-wide expectations under Rust 1.91; the final
[driver-specific checks](fixture-lint-check.log) rerun every affected corpus on
both compilers without warnings. [Wasm driver Clippy](wasm-driver-clippy.log) and
[Wasm driver MSRV](wasm-driver-msrv.log) also pass. No full CI, macOS, consumer integration, release
or publication was run.
