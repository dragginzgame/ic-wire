# Small-u64 Serde specialization (pending 0.6.5)

The private Postcard admission adapter forwards u64 values below 128 through
Postcard's u8 serializer. Both representations emit the same single byte; this
avoids constructing/copying the temporary varint buffer. Larger values retain
the original serializer. This applies to encoding and canonical reserialization,
not just vectors. Admission still charges the original u64 size, nodes and depth.
There is one codec, no public specialization mode and no consumer changes.

For example, u64 values 0, 127 and 128 still encode as `00`, `7f` and `80 01`.
Framing, canonical rejection, retry bytes, hashing and TypeScript layouts remain
unchanged. The new boundary test covers all 0–255 values, both sides of every
seven-bit transition, u64::MAX, vector payloads and exact/insufficient budgets.

## IC instruction results

Ordinary owned Serde representation, opt-3; result destruction and loop overhead
are included. Candid is measured with the same inputs in the same experiment.

| Case / operation | Wire before | Wire after | Change | Candid |
| --- | ---: | ---: | ---: | ---: |
| One small u64 in a list / encode | 1,243.25 | 1,122.25 | -9.73% | 14,651.25 |
| 1,024 small u64s / encode | 306,115.25 | 182,211.25 | -40.48% | 31,333.25 |
| 16,384 small u64s / encode | 4,883,417.25 | 2,900,953.25 | -40.60% | 277,316 |
| 16,384 small u64s / decode | 4,247,476.5 | 3,575,732.5 | -15.82% | 1,965,433 |
| 16,384 wide u64s / encode | 7,865,489 | 7,750,801 | -1.46% | 277,260 |

The large small-integer frame stays 16,398 bytes versus Candid's 131,084.
Its encode ratio improves from 17.61x to 10.46x Candid: useful progress, not
parity. Wide-vector encoding remains a major Candid win. Empty-list costs,
1 MiB generic byte-vector costs and 1 MiB owned-blob costs are unchanged at opt-3.
Owned-blob decoding remains 2,524,535 instructions (about 2.37x Candid).

Tradeoffs remain. At opt-z, the wide 16,384-element list's fresh encode increases
from 19,269,560 to 19,597,240 instructions (+1.70%). The largest standard-suite
percentage regression is scalar u64 128/16,383 reuse: 1,141.5 to 1,161.5 (+1.75%).
The 100-type fixture's maximum regression is +0.89% on opt-z encoding. All 100
opt-3 record encodes and decodes improve; opt-z encode/decode rows regress.
These are matched fixture costs, not application-wide speed guarantees.

## Uncompressed Wasm

Whole Wire fixture bytes, before to after; no symbol-attributed savings claim.

| Fixture/profile | Total Wasm | Code section |
| --- | ---: | ---: |
| Standard opt-z | 228,821 → 228,906 (+85) | 193,545 → 193,629 (+84) |
| Standard opt-3 | 321,261 → 321,407 (+146) | 271,236 → 271,382 (+146) |
| 100 types opt-z | 323,997 → 324,087 (+90) | 305,180 → 305,264 (+84) |
| 100 types opt-3 | 426,480 → 426,626 (+146) | 391,954 → 392,100 (+146) |

The gain justifies this small, bounded code increase; it does not justify the
larger/regressive [output-writer candidates](serde-short-write-study.md).
Candid artifacts are unchanged between sides; their sizes/hashes are in the
artifact table. The 100 nominal records repeat sixteen structural widths and
are not an installed application's size prediction.

## Qualification and reproduction

Baseline is 0.6.4 (`0300e81`) with the incoming dirty lockfile (Tooling 0.64.2,
transport 0.49.3). The selected variant changes only the private u64 forwarding;
final source adds an explanatory comment and a regression test. Dependencies are
identical and their incoming changes are preserved. Rust 1.99.1 / LLVM 23.1.1,
wasm32-unknown-unknown, LTO, one codegen unit, panic abort and stripped symbols;
evaluation/benchmark-speed select opt-z/opt-3 respectively. PocketIC 16.1.0 uses
performance counter 0 with three identical samples, four operations through
64 KiB and one above. Setup/transport is excluded; no overhead is subtracted.

The 114-case standard suite and 100-type suite produce 4,958 rows per side.
All 9,916 rows preserve paired input/frame hashes, frame lengths and repetition
counts. Candid binaries/counters are identical. Exact frames and damaged-frame
operations are checked by the driver. Runtime, borrowing, canonical-comparison,
boundary/budget tests, TypeScript conformance, strict lint and native/Wasm MSRV
checks pass. No new native-time, heap, installed-consumer or full-CI claim.

[Operations/controls](evidence/small-u64-065/instructions.csv),
[artifact sizes/hashes](evidence/small-u64-065/artifacts.csv),
[aggregate changes](evidence/small-u64-065/summary.csv) and
[compiler/lock identity](evidence/small-u64-065/identity.json) retain the results.
The [raw archive](evidence/small-u64-065/raw.json.gz) contains every sample, the
complete baseline source/lock, selected encoder source, build scripts and focused
validation logs. Local binaries/logs remain under `/tmp/wire-opt065/`.

Reproduce by extracting the archive's source path/text mapping to a fresh
workspace and selecting the baseline/selected encoder from its variants map.
Build and run the same codec-wasm/codec-instructions commands documented in the
[short-write study](serde-short-write-study.md#evidence-and-reproduction), using
separate target/result directories per variant. Add derive-scale to both build
and driver feature lists for the 100-type cohort. Retain the archived lockfile.
