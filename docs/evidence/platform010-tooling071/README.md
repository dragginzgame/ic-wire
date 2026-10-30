# Platform 0.10 / IC Tooling 0.71 adoption

30 October 2026. Baseline: Wire source release 0.8.0 at the
[recorded commit](baseline-revision.txt), Platform 0.9.1, resolved Build/Testkit/Host
0.70.5 and independently selected release-editor CLI 0.70.3. Candidate: Platform
0.10.0 and Build/Testkit/Host plus the managed CLI at 0.71.0. See the retained
[before lock](baseline-Cargo.lock), [after lock](Cargo.lock) and [manifest](Cargo.toml).
Wire package versions stay 0.8.0; pending notes select 0.9.0 because endpoint users
must coordinate runtime-consuming dependencies on Platform 0.10. Core-only Wire
consumers need no migration. Shared Tooling stays at reviewed 0.27.11.

## Scope and dependency identity

No Wire runtime, macro or benchmark algorithm changed. The registry
[source review](source-review.json) finds 118 existing Platform source files and
all five macro files unchanged. Platform adds its prepared-call module and its
root declaration; existing transport, executor and IC0 source remains identical.
Wire continues using the existing endpoint integration; the newly added call API
is not substituted into consumer code.

The canister normal/build graph has one Platform 0.10 runtime and no Candid,
PocketIC, external executor or external IC0. Core-only Wire excludes Platform.
Testkit 0.71 retains native-only Platform 0.9.1 without its executor. That host
identity is not a second canister runtime. The old registry IC0 package disappears
from the lock; unrelated dependency versions are retained.

Wire framing, handler declarations, identities, limits, acceptance and retained
retry bytes are unchanged. ID 7 / u8 42 remains
`49 57 07 00 00 00 00 00 00 00 00 2a`. No compatibility lane or automatic
migration is added. Endpoint applications must coordinate runtime dependency
identities; Wire versions, commits, publication and releases remain maintainer-owned.

## Matched measurements

Both lanes use Rust 1.99.1, wasm32, LTO, one codegen unit, panic abort, stripped
symbols, and the existing opt-z / opt-3 profiles. Sources use separate Cargo
targets. The same candidate host driver and server observe both guest lanes;
this measures guest costs, not Testkit or Build host execution time.

| Profile | Total uncompressed Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| opt-z | 99,939 → 99,947 (+8; +0.0080%) | 86,068 → 86,068 (0%) |
| opt-3 | 93,510 → 93,518 (+8; +0.0086%) | 80,630 → 80,630 (0%) |

The artifacts are not byte-identical. [Hashes and sections](artifacts.json)
record differences without treating equal size as proof of identical code.
Compression is used only to retain evidence, not as an optimization metric.

The [focused matrix](comparison.json) covers **12 cases across seven exports**:
empty/32-byte owned input, borrowed input, large blobs, ready async queries,
composite self-calls, async update/reply rollback, malformed input and early
wrong-ID rejection. Each lane/profile uses eight measured repetitions after one
excluded warmup: **384 logical calls / 608 physical executions**. This is not the
full 178-case suite or a new Candid comparison. The common original fixture still
checks its defined coverage against all compiled exports before the isolated
[case filter](cost-cases.patch) selects the measured subset.

Complete IC instructions, before → after:

| Case | opt-z | opt-3 |
| --- | ---: | ---: |
| Empty owned echo | 253,815 → 253,815 | 247,492 → 247,492 |
| 32-byte owned echo | 256,230 → 256,230 | 249,729 → 249,729 |
| 65,522-byte blob | 3,884,431 → 3,884,431 | 3,879,734 → 3,879,734 |
| Async update/reply trap | 1,727,584 → 1,727,584 | 1,711,693 → 1,711,693 |

All measured instruction samples, observed cycle debits, frame hashes, outcomes
and transaction-state oracles match across lanes. Percentage instruction changes
are 0% throughout this subset. Counters include physical page charges; simulator
query cycle debit of zero does not mean zero work. No native-time, allocation,
whole-application performance or general regression-free claim is made.
[Raw receipts](samples.json.gz) retain warmups, repetitions and physical traces.
The prepared instrumented PocketIC 16.1.0 server identity is in
[server.sha256](server.sha256); it is different from older studies, so comparison
claims are restricted to this matched pair.

## Checks and reproduction

Passed: locked offline metadata; dependency boundaries and core/endpoint graphs;
[endpoint/schema/diagnostic, strict Clippy and MSRV checks](endpoint-check.log);
[benchmark driver Clippy](benchmark-clippy.log); [corpus roundtrips](roundtrip.log);
and [installed endpoint/rollback behavior](installed.log).

The existing managed installer [prepared the 0.71 editor](install-release-editor.log)
and its [offline check passed](release-editor-check.log). The consumer's
[disposable release fixture](release-fixture.log) uses that actual editor to check
metadata preparation, dirty/conflicting-source refusal, failed validation,
committed-source reads and resume/recovery. Its validation gate is substituted
and its Git delivery stays inside temporary repositories; no real checkout
release, publication or push occurred. Expected fixture Make overrides are visible
in its retained log. Full CI, macOS and sibling adoption were not run.

With locked sources and the identified server prepared, run:

```sh
POCKET_IC_BIN=/absolute/instrumented/server \
  bash docs/evidence/platform010-tooling071/reproduce.sh /absolute/new-output
```

The [runner](reproduce.sh) reconstructs both sources from the baseline plus the
retained candidate manifest/lock, builds in separate targets, then applies only
the private host case filter. It keeps the compiler identity fixed and uses the
same host graph for both lanes. The [analyzer](analyze.mjs) validates actual module
and server hashes, complete trace counters, request/reply equality and sample
pairing before writing measurements. Raw original run: `/tmp/wire-platform010-tooling071/`.
