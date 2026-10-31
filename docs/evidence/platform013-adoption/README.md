# Platform 0.13 integration qualification

2026-10-31. Adopt published Platform 0.13.0 for both optional integrations,
from released Wire 0.11.0 at `568196014214a27b1a679b119ffde45284ed5eae`.
Own package versions remain 0.11.0; the pending release is Wire 0.12.0 because
Principal/Account and runtime supplier Rust identities change.
[#37](https://github.com/dragginzgame/ic-wire/issues/37) remains open until the
compatible Wire release is published and the registry consumer pair qualifies.

No Wire Rust implementation changes. Schema shapes, framing, codec bytes,
budgets, malformed-input acceptance and retry/hash inputs remain unchanged.
Principal remains bounded bytes (29 maximum); Account remains owner then optional
32-byte subaccount. Anonymous-owner payloads remain `01 04 00` for absent and
`01 04 01` followed by 32 zero bytes for explicit zero. Consumers align direct
Platform dependencies on 0.13 and typed clients on the supplying Wire minor.
The unpublished stable identity extraction and endpoint package split remain
separate work; neither is adopted here.

## Inputs and source review

[Baseline revision](baseline-revision.txt), [admitted baseline inputs](baseline-inputs.patch)
and the [supplier-only patch](inputs.patch) reproduce both dependency selections.
The baseline patch preserves an incoming `log` 0.4.35 lock update in both lanes.
Only Platform, its macros and runtime guard change from 0.12.0 to 0.13.0;
IC Build/Testkit stays 0.72.1 and all other dependencies are identical.
Testkit's native-only Platform 0.11.1 is outside the deployed runtime graph.

[Registry source review](source-review.json) records supplier checksums. The 118
retained Platform source files, all five macro source files and runtime guard
source are identical. The only Platform source changes remove its optional gzip
module/export/documentation, which Wire does not select. This source review is
supporting evidence, not a substitute for installed qualification.

## Focused qualification

[Retained logs](checks/) cover:

- `make check-platform-types`: standalone schema-only (four tests) and combined
  endpoint (five tests), Rust 1.91 host/Wasm, strict Clippy, exact Rust/TypeScript
  frames and malformed-input refusals, including absent/zero/nonzero subaccounts.
- `make check-endpoint`: three schema tests, method identity control, 11 expected
  compiler diagnostics and a passing control, strict Clippy, optimized Wasm and
  Rust 1.91 Wasm checks.
- Host/Wasm feature isolation: core has no Platform; schema-only has no CDK,
  executor or Candid; Wasm has no host serde_json; combined endpoint has one
  selected runtime guard.
- Three Candid corpus roundtrip/rejection tests, using
  `cargo test --locked --offline -p ic-wire-benchmarks --features wire,candid --test roundtrip`.
- `make check-boundary check-pins`, documentation links and whitespace checks.
- Installed candidate endpoint test `typed_calls_limits_and_transactions`, using
  the actual measured Wasm: admission/limits, malformed frames, mutation and
  response rollback, async and composite behavior.

## Measured endpoint comparison

The same seven-export, 12-case probe runs in both lanes and profiles, with one
excluded warmup followed by eight measured repetitions. There are 384 logical
measured calls / 608 physical executions. Request/reply hashes, outcomes and
transaction oracles match. Every paired instruction and cycle sample matches,
including the within-run variation of the async mutation case.

| Profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| `evaluation` (opt-z) | 99,947 → 99,951 B (+4 B, +0.004%) | 86,068 → 86,068 B (0%) |
| `benchmark-speed` (opt-3) | 93,518 → 93,518 B (0%) | 80,630 → 80,630 B (0%) |

The sole size regression is four non-code bytes in opt-z. No runtime improvement
or regression is measured. The following instruction means are identical before
and after; [comparison.json](comparison.json) also retains every instruction
sample, minimum/maximum, frame size, outcome and mean cycles.

| Method / input | opt-z IC instructions (both) | opt-3 IC instructions (both) | Change |
| --- | ---: | ---: | ---: |
| `echo` / `blob-0` | 253,815.00 | 247,492.00 | 0% |
| `echo` / `blob-32` | 256,230.00 | 249,729.00 | 0% |
| `borrowed_echo` / `blob-32` | 255,381.00 | 249,485.00 | 0% |
| `async_query` / `value-0` | 256,985.00 | 250,138.00 | 0% |
| `composite` / `self-call` | 1,488,232.00 | 1,473,022.00 | 0% |
| `async_mutate` / `reply-0` | 1,245,511.25 | 1,230,033.00 | 0% |
| `async_mutate` / `reply-117` | 1,727,584.00 | 1,711,693.00 | 0% |
| `default_blob` / `blob-32` | 252,355.00 | 247,824.00 | 0% |
| `default_mutate` / `reply-32` | 492,549.00 | 487,891.00 | 0% |
| `default_blob` / `blob-65522` | 3,884,431.00 | 3,879,734.00 | 0% |
| `async_query` / `truncated` | 249,778.00 | 246,849.00 | 0% |
| `default_blob` / `wrong-id-at-limit` | 243,786.00 | 243,392.00 | 0% |

Mean debited cycles also match: `default_mutate/reply-32` is 7,000,549 (opt-z)
and 6,995,891 (opt-3); `async_mutate/reply-0` is 18,038,386.25 and 18,022,908;
`async_mutate/reply-117` is 18,520,584 and 18,504,693. Other cases debit zero
in this harness. These are harness observations, not mainnet billing estimates.
[Raw traces](samples.json.gz) retain individual cycle/instruction samples, frame
hashes and module/server identities. Compression here only stores evidence;
gzip artifact size is not an optimization objective or acceptance metric.

## Reproduction and scope

Use the [recorded compiler](compiler.txt) (Rust 1.99.1, LLVM 23.1.1,
`x86_64-unknown-linux-gnu` host, `wasm32-unknown-unknown` guest), locked prepared
registry dependencies and [frozen instrumented server](server.sha256).
Both profiles inherit release LTO, one codegen unit, panic abort and stripping.
The runner reuses the existing 12-case selection and receipt analyzer under
[Platform 0.10/Tooling 0.71 evidence](../platform010-tooling071/README.md).

```sh
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic-server \
  bash docs/evidence/platform013-adoption/reproduce.sh /tmp/wire-platform013-repeat
```

The output directory must not exist. The runner copies the server, prepares
isolated sources/targets, verifies compiler identity, builds both suppliers,
installs disposable canisters and validates exact traces. It needs local loopback
permission. Original output is `/tmp/wire-platform013-adoption/comparison`.
Schema checks additionally use Rust 1.91.0, Node 24.21.0 and TypeScript 7.0.2.
No sibling working checkout is changed. No native timing, full Candid performance
matrix, full CI, macOS run or unpublished stable-types qualification is claimed.
The schema/endpoint checks qualify local Wire source with registry Platform;
publication and installed downstream adoption remain separate.
