# Packed qualification and rejected optimization screens

Baseline: release 0.6.11 (`445aa56`), with the incoming working-tree Cargo.lock
preserved. This batch strengthens qualification; **production code, wire bytes,
acceptance rules, metadata and retry behavior are unchanged**. Neither timing
candidate is retained. These are native/JavaScript measurements, not fresh IC
instruction measurements or a new full Candid comparison.

## Malformed-input qualification

The native suite now covers every supported packed element at eight input offsets
and 74 byte lengths, including 127/128, 255/256 and 16383/16384 prefix boundaries.
It checks 4,144 offset/type/length combinations against an independent byte-buffer
encoding, malformed widths, truncation, trailing bytes, noncanonical and overflowing
lengths, exact allocation thresholds, and byte/node limits. Seven packed variants
also join the runtime fuzz request/response model, which checks canonical roundtrip
and direction rejection.

Both default and TypeScript-feature native suites pass 17 tests. The seeded
AddressSanitizer fuzz run passes 100,000 inputs (seed 42, max length 2048).
The first run completed its inputs but LeakSanitizer failed at process shutdown
because this environment uses ptrace. The successful rerun used
`ASAN_OPTIONS=detect_leaks=0`; address checks remained enabled. No leak-check claim
is made. The runner's default sanitizer configuration is unchanged.

## Native u16 investigation

Matched Rust 1.99.1, evaluation/opt-z, same lockfile and 68-case bulk corpus;
two runs per variant, 11 timing samples per operation, 2 ms minimum sample.
All binaries were built before timing. Raw samples include small inputs,
encode/decode and Candid controls, with input/frame identities and host settings.
Forcing concrete word readers from `inline` to `inline(always)` did not resolve
the regression: wide u16 decode was 7,530/7,448 ns before and 7,424/7,942 ns after
(-1.4%/+6.6%). The candidate was rejected.

Both opt-z Wasms are byte-identical: **69,603 total bytes / 59,159 code bytes**,
SHA-256 `8916031c14476479c6d6050b62b85f3295982d1762a869c16041e475fafd4890`.
These sizes use current dependencies; do not compare them to differently pinned
historical artifacts as a savings claim. No IC counters were rerun.

The linked native i16/u16 visitor loops contain the same instructions, with
no per-word function call. Their addresses differ: the u16 loop crosses a
64-byte boundary. A diagnostic rebuild with
`-C llvm-args=-align-all-functions=6` changes wide-u16 decode to 4,823/4,652 ns
(-36.0%/-37.5%). This strongly supports code-layout sensitivity, rather than a
missing Serde fast path; it does not isolate a specific processor mechanism.
Native ELF `.text` grows from 626,136 to 683,720 bytes (+57,584; +9.2%).
This global compiler flag is not a portable library fix and is **not adopted**.
No production Rust change or Wasm saving is claimed.

## TypeScript single-word experiment

Node 24.21.0, Linux x64; same schemas, values, frame limits and runtime except
for the candidate branch. Three runs per type in separate processes, alternating
variant order, 5,000 warmup calls and 11 samples of at least 5 ms per operation.
Lengths 0/1/2/8/128/16384 cover all seven packed types and bounded i16.
The candidate constructs one owned typed array directly for a single 16/32-bit
word. Bulk inputs keep the existing byte-buffer copy. Bounds and bytes do not change.

Median of the three run medians, native JavaScript decode ns/op:

| Input | Before | Candidate | Change |
| --- | ---: | ---: | ---: |
| i16 × 1 | 511.1 | 173.2 | -66.1% |
| u16 × 1 | 520.6 | 170.7 | -67.2% |
| i32 × 1 | 643.0 | 172.7 | -73.1% |
| u32 × 1 | 523.9 | 180.0 | -65.6% |
| i16 × 16384 | 4895.1 | 4046.0 | -17.3% |
| u16 × 16384 | 4240.8 | 3967.0 | -6.5% |
| i32 × 16384 | 7316.0 | 7584.9 | +3.7% |
| u32 × 16384 | 8024.7 | 6640.7 | -17.2% |
| i64 × 2 (control) | 372.5 | 634.7 | +70.4% |
| i64 × 16384 (control) | 13409.0 | 15789.6 | +17.8% |

Single-word gains repeat, but controls remain noisy and regressions cannot be
excluded. Large i64 decode regresses in all three runs (+9.9%/+20.1%/+15.8%).
The candidate passes strict TypeScript compilation, but is **not adopted**;
it has not passed the full conformance suite. The unchanged production runtime
retains its existing qualification. No generated-JavaScript change is shipped,
so there is no new production Wasm or IC cost to measure.

## Reproduction and evidence

[Raw evidence](evidence/packed-0612.tar.gz) contains the exact baseline source and
lockfile, rejected candidate, native identities/inputs/samples, Wasm screen,
linked disassembly, TypeScript source/runner/results and fuzz corpus/logs.
Compression is only for evidence storage, not an optimization metric.

Extract under `/tmp/wire-packed-0612` (or adjust the runner paths). The native
runner uses the checkout's Git identity; its code-section reader can be built
from the archived `codec-report` example. It builds both variants before timing.
Recreate `candidate` from `before` and replace its packed module with
`candidate-packed.rs`. For the alignment diagnostic, rebuild the baseline bench
using `cargo rustc --locked --profile evaluation -p ic-wire-benchmarks --bench
suite --features wire,candid,bulk -- -C llvm-args=-align-all-functions=6`, then
invoke the resulting suite with `OUTPUT evaluation 11 2`, twice.

Compile both TypeScript runtime files with the repository's TypeScript compiler
using `--ignoreConfig --strict --noUncheckedIndexedAccess
--exactOptionalPropertyTypes --target ES2022 --module ES2022 --skipLibCheck`.
Run `node --expose-gc typescript/focused.mjs TYPE` for each type; the runner expects
its archived absolute directory. For fuzzing, restore the archived qualification
source and corpus to `target/fuzz-runtime/corpus`, then run
`ASAN_OPTIONS=detect_leaks=0 FUZZ_RUNS=100000 make fuzz-runtime` with the pinned
nightly installed. Neither broad CI nor big-endian or installed-consumer execution
was performed. Sibling repositories were not changed.
