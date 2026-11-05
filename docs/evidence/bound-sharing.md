# Sharing bounded-list decoding, pending Core 0.17.1

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The selected change shares the allocation/growth loop for `List<T, N>` across
bounds for each element type. Tiny bounds (zero/one) and zero-sized elements
retain constant-bound specialization. Both call the same source algorithm;
there is one public codec, with no new options or dependencies. The return
visitor changes from `List<T, N>` to the equal-sized `Vec<T>`, wrapped afterward.
Wire bytes, limit checks, allocation growth, element-decoder order, budget
charges and typed failures are unchanged. Consumers need no migration.

Baseline is `0c8d68d` plus the already-qualified
[preceding cleanup](codec-cleanup.md), including the byte visitor.
The numbers below isolate the subsequent list change. **Do not add savings
from these different fixtures or project them onto an application.**

## Linked Wasm footprint

The exported benchmark selects every bound at runtime, so every instantiation
remains reachable after linking. Bounds are 8192+i, for i in 0..1/10/100, in four
list families: u32, small u64, unit and a record containing u64/bool/String.
Borrowed bytes/text use the same bounds as unchanged controls; owned bytes,
packed u64 and scalar u32 are controls too. Separate List<u32> bounds 0/1/8
exercise small-bound specialization. Repeating one bound in many fields is not
the same workload as these distinct bounds. Actual application savings are unknown.

Both profiles use the identical lock, Rust/Cargo **1.99.1**, LLVM **23.1.1**,
wasm32-unknown-unknown, LTO, one codegen unit, abort and stripped symbols.
`evaluation` uses opt-z; `benchmark-speed` uses opt-3. These are final Cargo-linked
modules, with no subsequent wasm-opt/deployment stage. Profiles are measurement
controls, not a change to downstream build settings. Each source has its own
target directory. All twelve guest artifact records are fresh; returned source
hashes are checked by the installed caller.

All figures are **uncompressed bytes**. Code size is the section-10 payload
length, including its function count and excluding the section ID/length prefix.

| Profile | Bounds/family | Before total / code | After total / code | Delta total / code |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 1 | 52,841 / 49,345 | 53,259 / 49,759 | +418 / +414 |
| opt-z | 10 | 137,730 / 134,092 | 131,547 / 127,914 | -6,183 / -6,178 |
| opt-z | 100 | 973,209 / 968,401 | 904,566 / 899,763 | -68,643 / -68,638 |
| opt-3 | 1 | 65,919 / 62,677 | 65,713 / 62,468 | -206 / -209 |
| opt-3 | 10 | 217,634 / 214,240 | 197,240 / 193,844 | -20,394 / -20,396 |
| opt-3 | 100 | 1,757,246 / 1,752,682 | 1,543,115 / 1,538,549 | -214,131 / -214,133 |

## IC instructions and regressions

The same prepared PocketIC **16.1.0** binary is admitted through Testkit **0.85.0**
startup/lifecycle, SHA-256
`b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.
The frozen lock includes Candid **0.10.39**, but this experiment does not execute
Candid or claim a refreshed Candid matrix. Published codec graphs are unchanged.

For each input/profile, eleven alternating unique-caller samples run four
operations per counter bracket. Setup, warmups and reply transport are outside
the counters. Values below divide counts by four, retaining loop overhead.
They are IC instructions, not physical cycles, native time or full endpoint cost.
All eleven counts per row match. The pair covers **384 rows / 4,224 calls /
768 operation comparisons**, with equal returned payload bytes.

Lengths are 0/1/32/4096; bound indices 0/9/99 for the six bounded families.
Small fixed-bound controls clamp length to 0/1/8. List values are u32::MAX, u64 7,
unit, or `{key:128, active:true, label:"entry"}`. Borrowed buffers contain ASCII x;
owned-byte control contains 0xa5; packed control is u64::MAX. Scalar controls
are 0/127/128/u32::MAX. Limits are bytes 2097152, allocation 268435456,
nodes 8388608, depth 64. Operations are encode, owned/borrowed decode,
truncated-input rejection and loop control.

| Decode case | opt-z before → after | opt-3 before → after |
| --- | ---: | ---: |
| Empty u32 list | 484.5 → 526.5 (+8.67%) | 250 → 278 (+11.20%) |
| One small u64 | 1160.5 → 1190.5 (+2.59%) | 628 → 724 (+15.29%) |
| 4,096 u32::MAX | 2,524,059.5 → 2,515,909.5 (-0.32%) | 1,227,994 → 1,187,072 (-3.33%) |
| 4,096 small u64 | 898,000.5 → 816,226.5 (-9.11%) | 732,340 → 699,678 (-4.46%) |
| 4,096 records | 7,207,177.5 → 6,797,679.5 (-5.68%) | 5,325,731 → 4,899,928 (-8.00%) |
| 4,096 units | 500,233.5 → 500,233.5 | 184 → 184 |

Encoding and all unit-list/borrowed-buffer/owned-byte/packed/scalar controls have
identical counters. The largest relative increase across operations is a tiny
truncated u64 input: **300 → 351 (+17%)** at opt-3. The one-u64 decode adds 96
instructions; it narrowly exceeds the optional `baseline × 1.05 + 64` screen
(723.4 versus 724), at each of the three bound indices. This is a disclosed
tradeoff, not a claim that the paired-cost regression gate passed. That gate
and its thresholds are unchanged. No additional Candid or native-time run.

## Rejected candidates

- Erasing every list bound from its visitor lost constant-bound unit-loop
  optimization: opt-3 4,096-unit decode rose **184 → 176,481** when outlined,
  or **184 → 155,964** with ordinary inlining. Neither is kept. The selected
  compile-time condition preserves all measured unit counts exactly.
- Sharing borrowed byte/text checks increased linked size in both profiles:
  **868,808 → 871,792 total / 864,026 → 867,006 code** at opt-z, and
  **1,484,295 → 1,490,973 total / 1,479,931 → 1,486,805 code** at opt-3 in
  the earlier exploratory fixture. It failed size screening and did not advance
  to IC timing. Borrowed visitors remain unchanged.
- An additional always-inline hint on the selected visitor wrapper produced
  byte-identical code sections in both profiles. It is not kept.

Final native tests include hidden lengths, zero-sized values, exact bounds and
budget refusal. A new side-effecting custom-decoder test confirms excess elements
are rejected without decoding them, including bound zero. Deferred borrowed-blob
ownership tests and strict Core library/test lint pass. The linked Wasm probes
also run the affected decoder under IC counters. No full CI, installed Endpoint
rerun, staging, commits, own-version changes or release were performed.
