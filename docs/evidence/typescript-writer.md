# Incremental TypeScript writer qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Issue [#22](https://github.com/dragginzgame/ic-wire/issues/22). Baseline is the
unchanged TypeScript runtime at release 0.7.6,
`accf8158d0c932e1fed614a59d06e2f4cc751f78`; candidate is the pending 0.7.7 writer
in the source archive. Incoming Rust parser/metadata changes are outside this delta.

The private buffer starts at at most 64 bytes, doubles when needed or grows directly
for a larger write, and never exceeds the directional byte ceiling. Full buffers
are returned directly; partial buffers are trimmed. Outputs remain independently
owned. No pool, second sizing pass or additional custom-validator visits were added.
Wire bytes, framing, identities, admission and callback order are unchanged.
Consumers regenerate clients; no schema or retry-byte migration is required.

## Method and reproduction

Retired raw output (see retention notice) contains before/after TypeScript and compiled JavaScript,
`bench.cjs`, `replay.sh`, inputs with frame SHA-256 identities, allocation requests,
raw samples and validation logs. Extract into a disposable directory and set
`WIRE_TSC` to the prepared TypeScript compiler before running `bash replay.sh`.
The runner verifies identical frames/errors before measurement. It has 23 cases,
including small-limit controls, empty values, invalid input, UTF-8, bytes, packed
integers, ordinary integers, floats, exact byte limits and asymmetric directions.

Measured on Linux x86_64, Ryzen Threadripper 7970X, Node 24.21.0, TypeScript 7.0.2,
ES2022 CommonJS with strict/noUncheckedIndexedAccess/exactOptionalPropertyTypes.
Timing pins CPU 0 with boost enabled. Each variant gets 1,000 warm-up calls per
case/run, calibrated batches of at least 5 ms (up to 65,536 calls), and 11 samples
in each of three runs. Variant order alternates; GC runs outside timed batches.
Results are medians of the 33 batch means, not individual-operation percentiles.
No compilation ran alongside the final timing. Host noise remains possible.

Allocation instrumentation is separate from timing. It records requested
Uint8Array backing-buffer bytes, including result copies and text temporaries;
it excludes views, DataViews, object overhead and GC. It is neither peak memory nor
an allocation-failure guarantee. Total requested bytes can exceed the frame limit
because growth copies coexist temporarily. Timing includes allocations/GC within
batches. These are Node encode measurements, not IC instruction measurements.

The generated unminified CommonJS runtime grows from 19,948 to 20,725 bytes
(+777 bytes, +3.90%). This is JavaScript, not Wasm. No Rust runtime change or Wasm
size saving is claimed; the TypeScript generator is a native-host feature.

## Correctness

`make check-typescript` passes 159 ordinary vectors, 33,560 mutations, 24 native
metadata vectors, 13 admission shapes, 54 simulated-endian cases and the new
writer tests. Those test exact ceilings, both directions, empty/invalid values,
UTF-8, packed integers, float32/64 growth, output ownership, callback count and
Number.MAX_SAFE_INTEGER allowances for tiny frames. The allocation regression
fails against the old writer. Real big-endian and fresh macOS execution were not
performed. Full CI and publication remain maintainer steps.

## Results

Retired raw output (see retention notice). Time is microseconds per encode. Buffer bytes
are cumulative requests per call, separate from the unchanged wire-frame size.

| Case | Before µs | After µs | Time change | Buffer bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| unit-128 | 1.187 | 1.112 | -6.30% | 139 → 75 |
| unit-2MiB | 107.652 | 0.863 | -99.20% | 2097163 → 75 |
| unit-response-3MiB | 214.341 | 0.870 | -99.59% | 3145739 → 75 |
| invalid-unit-2MiB | 68.317 | 8.160 | -88.06% | 2097152 → 64 |
| scalar-128 | 1.226 | 1.320 | 7.64% | 141 → 77 |
| bytes-0 | 63.943 | 1.373 | -97.85% | 2097164 → 76 |
| bytes-32 | 66.886 | 1.254 | -98.12% | 2097196 → 108 |
| bytes-4096 | 152.669 | 2.418 | -98.42% | 2101261 → 4173 |
| bytes-1048576 | 150.358 | 87.902 | -41.54% | 3145742 → 1048654 |
| bytes-tight-4096 | 2.702 | 2.530 | -6.36% | 8218 → 4173 |
| ascii-0 | 66.124 | 2.029 | -96.93% | 2097164 → 76 |
| ascii-32 | 69.250 | 2.244 | -96.76% | 2097292 → 204 |
| ascii-4096 | 72.157 | 5.116 | -92.91% | 2113549 → 16461 |
| unicode-4096 | 73.929 | 10.944 | -85.20% | 2107917 → 10829 |
| packed-i16-0 | 62.110 | 1.398 | -97.75% | 2097164 → 76 |
| packed-i16-32 | 180.894 | 2.444 | -98.65% | 2097228 → 268 |
| packed-i16-4096 | 175.908 | 2.936 | -98.33% | 2105357 → 8269 |
| packed-i16-524288 | 125.705 | 51.089 | -59.36% | 3145742 → 1048654 |
| u32-list-32 | 91.637 | 9.382 | -89.76% | 2097324 → 620 |
| u32-list-4096 | 574.461 | 560.707 | -2.39% | 2117645 → 85965 |
| u32-list-65536 | 9607.965 | 9499.617 | -1.13% | 2424846 → 1376206 |
| u32-list-tight-4096 | 617.878 | 638.489 | 3.34% | 40986 → 53197 |
| floats-1000 | 392.303 | 81.584 | -79.20% | 2105165 → 24333 |

Tiny 2 MiB-budget unit requests reduce requested buffer bytes by 99.9964%;
1 MiB blobs reduce them by 66.6649%. The tight integer-list control increases
cumulative allocation by 29.79% (40,986 → 53,197 bytes) and takes 3.34% longer
(+20.61 µs). The small scalar control regresses 7.64% (+93.70 ns). Those costs
are retained tradeoffs, not hidden by the much larger roomy-budget improvements.

An initial candidate routed every byte through a capacity helper and regressed
the tight integer control by 33.9% in its screening run. Moving growth out of
the ordinary byte-write path reduced that regression; the archived first-candidate
and focused-probe results are screening evidence, not the retained final timings.

## Alias-chain scaling — 11 November 2026

Retain constant-memory cycle detection instead of enumerating every registry key
once per operation. Checkpoints spaced 1, 2, 4, ... hops apart eventually span
any cycle, which repeats a checkpoint name. Names, not object identity, detect
cycles even when accessors return fresh reference objects. There is no chain set
or cross-call cache: additions and changed definitions remain visible.
Valid reference/concrete checks retain their order and count. Malformed cycles
can stop after a different number of callbacks than the former registry-sized
hop allowance. Payload bytes, identities, value budgets and valid-schema
acceptance are unchanged. Regenerate clients; no contract/storage migration.

Matched current-runtime probe: Node 24.21.0 transforms TypeScript to ES2022 in
memory using node:module.stripTypeScriptTypes. Two runs reverse lane order,
11 batches/run, 2,000 calls/batch for scalars and 50 for batches/large strings,
200 warm-up calls. Both lanes check identical payload bytes and decoded values.
This is shared-host Node timing, not browser fleet, IC instruction or Wasm
qualification. Other native qualification was active: small timing differences
are noisy. Inputs include u8 7 (07), lists of 128/4,096 sevens, strings, registries
of 1/298/1,657 definitions, and chains of 1/2/8 aliases. Registries always contain
at least as many definitions as their chain length.

Per-run medians, microseconds/op (first / second run):

| Registry / chain / operation | Before | After |
| --- | ---: | ---: |
| 1 / 1 / encode scalar | 0.576 / 0.366 | 0.560 / 0.295 |
| 1 / 1 / decode scalar | 0.132 / 0.106 | 0.173 / 0.114 |
| 1,657 / 1 / encode scalar | 29.753 / 29.525 | 0.237 / 0.273 |
| 1,657 / 2 / decode scalar | 28.579 / 29.517 | 0.123 / 0.124 |
| 1,657 / 2 / decode 4,096 values | 316.273 / 313.350 | 288.766 / 285.965 |
| 8 / 8 / encode 4,096 values | 554.447 / 545.137 | 742.860 / 661.210 |
| 8 / 8 / decode 4,096 values | 919.933 / 904.781 | 1,074.730 / 1,066.220 |

The small-registry long-chain regression is retained explicitly: repeated chain
checks cost more than a once-per-operation scan of eight keys. This synthetic
control adds about 0.12–0.19 ms encode and 0.15–0.16 ms decode. Accept that host
tradeoff to remove service-wide scaling without caches or per-value allocations;
this is not a universal throughput win. No canister runtime source changes,
IC instruction gains or Wasm savings are claimed.

The initial set-based candidate is rejected: 4,096 values through two aliases
in a small registry decoded in 504/557 µs versus 326/347 µs before. An
object-identity checkpoint probe avoided that allocation but is superseded by
name checkpoints so fresh-object accessors cannot conceal cycles. Tests cover
512-link chains, 99 tail/cycle combinations, fresh-object accessors, mutable
registries, valid callbacks and zero enumeration. Existing exact frames, budgets,
recursive types and malformed-input conformance remain mandatory.

Raw source pairs, source hashes, all samples and executable measure.mjs:
`target/evidence/browser-followup-20261111/`. Reproduce with
`node target/evidence/browser-followup-20261111/measure.mjs before constant`.
The final resolver differs from timed constant.ts only in explanatory comments.
No gzip or production bundle-size saving is claimed.

### Rejected Unicode validation probe

Replace the post-encoding UTF-8 roundtrip check with a Unicode-mode lone-surrogate
regular expression, retaining the prior length-error precedence. Compare `aliases`
and `text` sources with the same runner (large strings use 50 calls/batch).
Short Unicode encoding improves from 0.972/0.926 to 0.738/0.708 microseconds,
but 16,384 astral characters regress from 97.305/95.732 to
195.323/186.134 microseconds. ASCII-large timings are noisy and mixed.
Reject the change; the existing TextEncoder/TextDecoder path remains authoritative.
Retain exhaustive single-UTF-16-code-unit roundtrip/refusal coverage and boundary
surrogate pairs. No new string API, encoding, browser requirement or fallback.

## Numeric collection-length decoding — 12 November 2026

Retain only Reader.length's use of the existing numeric u32 varint reader.
Previously it decoded through BigInt, converted the maximum to BigInt, compared,
then converted back to Number. All u32 results are exactly representable as
Number. Existing bytes and acceptance stay unchanged: 80 00 still decodes as
zero; fifth-byte overflow and unterminated prefixes remain malformed, and an
otherwise valid length above the field allowance remains a limit error before
payload reads. Regenerate clients; no schema, identity, storage or retry-byte
migration. Rust/Wasm runtime and allocation budgets are unchanged.

Matched before/combined/decode-only runtime sources, Node 24.21.0, transformed
in memory using node:module.stripTypeScriptTypes. Three runs alternate lane order;
30 shapes cover scalar controls, bytes/text/lists/packed u16 at lengths
0/1/127/128/16383/16384/65536, plus 128 nested three-element lists. Every lane
checks exact payload bytes and decoded values. Each operation warms 2,000 calls,
then records 11 calibrated batches (5–10,000 calls); raw samples, iteration counts
and frame/source hashes are retained. The repeat is pinned to CPU 0 on the
shared Linux host; concurrent Rust qualification and GC remain noise sources.
This is Node latency, not browser-fleet, IC-instruction or Wasm qualification.

Decode medians in microseconds/op, three independent runs:

| Case | Before | Retained decoder |
| --- | --- | --- |
| Scalar u32 control | 0.088 / 0.092 / 0.089 | 0.089 / 0.101 / 0.104 |
| Empty bytes | 0.146 / 0.193 / 0.227 | 0.107 / 0.162 / 0.173 |
| 128-byte text | 0.295 / 0.593 / 0.600 | 0.212 / 0.513 / 0.522 |
| List of 128 u8 | 2.712 / 3.257 / 3.157 | 2.926 / 3.266 / 3.405 |
| Packed 128 u16 | 0.483 / 1.221 / 1.032 | 0.456 / 0.789 / 0.794 |
| 128 nested three-byte lists | 23.377 / 19.098 / 17.505 | 14.887 / 13.773 / 14.869 |
| 64 KiB bytes | 15.368 / 10.519 / 11.919 | 13.201 / 11.762 / 12.057 |

Nested-list decoding improves in all runs (15–36%). Flat-list control regresses
up to 8%; the unchanged scalar control varies by 1–15 ns. Large-buffer results
are mixed; do not claim universal throughput improvement. Retain the smaller
implementation for its repeated nested-length gain and simpler exact-domain
arithmetic, with these regressions visible. Transformed unminified JS decreases
22,449 → 22,439 bytes before explanatory comments; no production bundle claim.

The combined numeric-encoding candidate needs a wide-host-buffer fallback to
preserve old encoding behavior and increases transformed JS to 22,525 bytes.
Its encode timings are inconsistent in allocation-heavy cases; do not retain the
extra writer method or its call-site changes. Original encoding stays unchanged.

Final-source Rust/TypeScript conformance passes, including exact frames,
malformed mutations, 1 MiB buffers, 16 admission shapes, custom scalars and 78
simulated-endian cases. Added browser cases check length-prefix boundaries,
u32::MAX against zero field limits, 2^32 overflow, truncated/unterminated prefixes
and redundant zero for bytes, text and lists. These use an independent BigInt
prefix oracle in tests only.

Evidence and reproduction: `target/evidence/lengths-f64-20261112/browser/`.
Run `taskset -c 0 node target/evidence/lengths-f64-20261112/browser/repeat.mjs`;
before.ts, after.ts (combined), decode.ts, repeat.jsonl and the initial screening
results are retained. Final production source differs from timed decode.ts only
by explanatory comments. Final generated conformance artifacts are under
`target/typescript-lengths-final-20261112/`. No full comparison matrix was rerun.

## Packed-boolean result construction — 12 November 2026

Retain a single exactly sized boolean array followed by indexed writes, replacing
Array.from's iterator and mapping callback. The complete byte span is still
validated before result allocation. No packed representation, admission, callback
or ownership change: 02 00 01 still decodes to [false, true], invalid byte values
remain malformed, and results never alias the input. Regenerate browser clients;
Rust runtime, Wasm, method identities and retry/hash bytes are unchanged.

Matched current runtime (including numeric length decoding) versus this one
change, Node 24.21.0 transformed in memory with stripTypeScriptTypes, Linux CPU 0.
Three runs alternate lane order. Each of 12 shapes checks exact encode bytes
and decoded values before timing. Small shapes warm 1,000 operations; large
shapes warm 20. Eleven calibrated batches per operation/run use 2–10,000 calls.
Shapes cover packed bool lengths 0/1/8/128/16384/1048576, matching ordinary-list
controls through 16384 and a u32 scalar. Shared-host timing includes GC; no
browser-fleet or IC performance claim.

Decode medians, microseconds/op, three runs:

| Case | Before | After |
| --- | --- | --- |
| Packed empty | 0.338 / 0.313 / 0.257 | 0.157 / 0.137 / 0.102 |
| Packed singleton | 0.478 / 0.400 / 0.425 | 0.135 / 0.101 / 0.105 |
| Packed 128 | 2.749 / 4.527 / 2.839 | 0.621 / 0.609 / 0.614 |
| Packed 16,384 | 372.891 / 359.777 / 363.182 | 126.631 / 129.988 / 128.167 |
| Packed 1,048,576 | 37,065.364 / 35,503.263 / 36,627.836 | 7,583.724 / 7,671.527 / 7,890.847 |
| Ordinary 128 control | 2.529 / 2.817 / 2.760 | 2.391 / 2.757 / 2.603 |
| Ordinary 16,384 control | 458.265 / 447.852 / 420.456 | 480.705 / 444.675 / 454.357 |
| Scalar control | 0.117 / 0.085 / 0.090 | 0.095 / 0.093 / 0.087 |

Large packed decode improves 64–80% across runs. Ordinary 16,384 decode varies
from 0.7% faster to 8.1% slower. Unchanged encode controls vary too: packed-128
1.412/1.588/1.247 → 1.585/1.521/1.420 us, and packed-1MiB
3846.925/3692.953/3613.679 → 3552.237/3641.971/3968.994 us.
Do not attribute an encode saving or universal improvement to this decode-only
change. The large decode effect is substantially larger than those controls.
Transformed unminified JS grows 22,439 → 22,553 bytes (+114), excluding added
explanatory comments; no Wasm or production bundle saving is claimed.

Final generated-runtime conformance passes, including 1 MiB byte buffers,
16 admission shapes, exact frames/mutations and 78 simulated-endian cases.
New tests cover empty/singleton/block-boundary bool arrays in both directions,
owned results, and all 254 invalid byte values at five positions spanning a
129-element input. Instrumented Array construction proves those invalid inputs
fail before allocating the boolean result. Existing partial/trailing-input
checks remain active. No new full Candid matrix was run.

Evidence: `target/evidence/browser-bools-20261112/` retains before.ts, after.ts,
measure.mjs, results.jsonl and source/frame hashes. Reproduce with
`taskset -c 0 node target/evidence/browser-bools-20261112/measure.mjs`.
Final source differs from after.ts only by explanatory comments. Generated test
artifacts are under `target/typescript-bools-20261112/`.
