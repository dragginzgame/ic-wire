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
