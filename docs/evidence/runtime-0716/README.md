# Runtime optimization qualification for 0.7.16

Baseline: released 0.7.15, `47f011702edd0fce99d4272fd80b02991ee5f94f`.
The retained [decoder patch](decoder.patch) replaces the u16 tail loop with
explicit second/third byte-group checks. Signed i16 uses that parser after
zigzag conversion. One-byte values retain their early return. Encoding, shortest
representations, overflow rejection, typed errors and admission stay unchanged:
`ff ff 03` still means u16 65535; `80 00` and `ff ff 04` stay malformed.
No consumer migration is required. There is one production Serde/Postcard codec.

All runs use the frozen baseline lock: Postcard 1.1.3, Candid 0.10.39, Platform
0.6.16 and Build/Testkit/host 0.68.5. A concurrent checkout update to Platform
0.6.17/tooling 0.68.6 is preserved and excluded. [Compiler](compiler.txt) and
[host](host.txt) record Rust 1.99.1 / x86_64 Linux. Wasm targets
wasm32-unknown-unknown; both evaluation (opt-z) and benchmark-speed (opt-3) use LTO,
one codegen unit, aborting panics and stripped symbols. No extra RUSTFLAGS.

## IC instruction costs

The 298-case standard corpus runs all operations for both variants. Each has
7,006 operation rows with three equal repeated counter samples. The maintained
runner uses four operations per sample for small frames and one above 65,536
bytes. Values below are instructions per operation, including result destruction
and loop overhead; setup/transport are excluded and loop control is not subtracted.
All paired input/frame hashes match, and Candid control counters are unchanged.

| Owned decode | Profile | Before | After | Change |
|---|---|---:|---:|---:|
| Wide u16 list, 16,384 | opt-z | 7,215,104.5 | 6,494,208.5 | −9.99% |
| Wide i16 list, 16,384 | opt-z | 6,428,617.5 | 5,707,721.5 | −11.21% |
| Wide u16 list, 16,384 | opt-3 | 1,839,374.5 | 1,708,302.5 | −7.13% |
| Wide i16 list, 16,384 | opt-3 | 2,035,982.5 | 1,904,910.5 | −6.44% |
| Small u16 list, 16,384 | opt-3 | 815,349.5 | 815,349.5 | 0% |
| Singleton u16 list | opt-3 | 693.5 | 693.5 | 0% |
| Scalar u16 maximum | opt-3 | 313.5 | 308.5 | −1.59% |

There are 54 improved Wire operation rows and five regressions (excluding loop
controls). Both empty-u16-list decode controls grow 298.5 → 300.5 (+0.67%) at
opt-3; truncated u16-max/i16-min/i16-max scalars grow 287.75 → 291.75 (+1.39%).
These are retained costs. Wide-list truncation also improves: opt-3 u16 falls
1,839,327.75 → 1,708,263.75 (−7.13%). Existing ordinary-list versus Candid gaps
remain; this does not replace byte-oriented/Packed bulk-field adoption.

[All counters/identities](decoder-instructions.json.gz) and
[owned comparisons](decoder-instruction-changes.csv) retain every result.
The 100-type comparison is artifact equivalence, not fresh IC execution.
This is focused qualification, not a fresh full Candid matrix or installed endpoint.

## Uncompressed Wasm

| Fixture/profile | Total before → after | Code section before → after |
|---|---:|---:|
| Standard/opt-z | 377,811 → 377,818 (+7) | 337,782 → 337,789 (+7) |
| Standard/opt-3 | 573,444 → 573,226 (−218) | 518,734 → 518,516 (−218) |
| 100 types/opt-z | 310,738 → 310,738 | 291,971 → 291,971 |
| 100 types/opt-3 | 398,739 → 398,739 | 364,245 → 364,245 |

The opt-3 reductions are 0.038% total / 0.042% code. The seven-byte opt-z increase
is an accepted regression for its measured instruction reductions. Both 100-type
artifacts are byte-identical. [Artifact identities](decoder-sizes.json) retain
complete hashes. No general application-footprint saving is inferred. Gzip is
used only to store raw evidence, never as an optimization objective.

## Native timing

The maintained 87-case hotspot suite runs twice per variant at opt-3: seven
samples per operation, at least 1 ms calibrated batches, alternating pair order.
Both binaries finish compiling before serial timing; no task-owned compilation
runs alongside it. Sample pooling yields 14 observations per operation. Medians
below use the midpoint for even sample counts. Result destruction is included.

| Owned decode | Before ns | After ns | Change |
|---|---:|---:|---:|
| Wide u16 list, 16,384 | 25,323.06 | 23,062.88 | −8.93% |
| Wide i16 list, 16,384 | 30,037.96 | 27,830.59 | −7.35% |
| Empty u16 list | 13.56 | 13.98 | +3.06% |
| Singleton u16 list | 22.77 | 24.26 | +6.57% |
| Small i16 list, 16,384 | 12,890.86 | 13,614.88 | +5.62% |
| Byte-buffer control, 1 MiB | 17,500.91 | 17,441.06 | −0.34% |

Small-input regressions and run variation are material. For example, singleton
candidate run medians are 26.21/22.94 ns versus baseline 23.46/22.72; small-i16-list
candidate medians are 14.85/12.17 µs versus 12.90/12.69. Do not claim universal
native speedups from these local warm-cache observations. Native time and IC
counters are different measurements, not conversions of one another.

[Raw native samples](native.json.gz) include both run identities and pooled
summaries; [all native comparisons](native-changes.csv) include improvements and
regressions. The [analyzer](native-analysis.mjs) checks input/frame hashes, matched
settings and counts. Native benchmark Git observations describe the invocation
checkout; measured source is the frozen revision plus decoder patch and is bound
by each executable hash. No new allocation-count/heap savings are claimed.

## Rejected candidates

Each patch was measured separately against the same baseline. Table cells are
**total Wasm byte deltas**; corresponding code-section sizes/hashes are retained.

| Candidate | Standard z / 3 | 100 types z / 3 | Decision |
|---|---:|---:|---|
| [Combined budget predicates](budget.patch) | +157 / +786 | −57 / +457 | Predominantly grows Wasm; size-only rejection. |
| [Outlined admission](outlined.patch) | −2,701 / +4,720 | −2,369 / +2,830 | Grows opt-3; size-only rejection. |
| [Shared 16-bit writer](encoder.patch) | −395 / −921 | 0 / 0 | Wide-u16 encoding regresses 51.3% IC instructions. |
| [Inline Output helpers](output.patch) | −66 / +2,372 | −451 / −2,796 | Mixed footprint; no instruction qualification or promotion. |

The writer's opt-3 wide-u16 fresh encode rises 1,852,251.25 → 2,802,520.25;
small-u16-list encode rises 1,246,043.25 → 1,557,336.25 (+24.98%). Scalar u16 zero
rises 647.25 → 673.25 (+4.02%). [Writer counters](encoder-instructions.json.gz)
and [comparisons](encoder-instruction-changes.csv) retain those regressions and
controls. Native binaries were built for it but not timed after IC rejection.
Only the decoder is promoted. Rejected patches are research controls, not features.

## Correctness and reproduction

Existing tests exhaust every u16 and i16 value. The added independent control
checks 131,072 three-byte patterns against Postcard decode plus canonical
re-encoding, including trailing input and malformed forms. Focused checks pass:
2 unit, 6 borrowed, 40 feature-enabled native, 15 runtime tests; strict Clippy;
Rust 1.91 native tests; 165 Rust/browser vectors and 35,228 differential mutations;
feature/target isolation, formatting and documentation links. Full CI, installed
consumers and fresh macOS execution were not run.

Prepare cached locked dependencies, pinned Rust/Wasm toolchains, Node and the
PocketIC 16.0.0 server identified in raw instruction identities. Run serially:

```sh
bash docs/evidence/runtime-0716/measure.sh decoder /absolute/new-experiment
bash docs/evidence/runtime-0716/measure-native.sh /absolute/new-experiment
```

The [Wasm runner](measure.sh) reconstructs the frozen source and applies/reverses
one patch in that disposable directory. The [native runner](measure-native.sh)
builds both binaries before timing and archives analysis under that directory.
Use budget/outlined/encoder/output instead of decoder to reproduce their size
experiments. Original runners were equivalent isolated builds; retained scripts
have syntax/patch-admission checks and preserve their exact inputs/settings.

For IC counters, follow the [preceding instruction-runner commands](../followup-0712/README.md#reproduce)
against the new source and standard baseline/candidate directories. The driver
features remain wire,candid,ic; Candid artifacts are built once and reused unchanged.
Run the preceding analyzer to verify paired hashes and counter repeats. The
current lock update is deliberately not substituted into this reconstruction.
