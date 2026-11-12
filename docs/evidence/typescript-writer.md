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

## Record keys and direct UTF-8 experiments — 12 November 2026

Retain passing field declarations directly to the existing object validator.
Previously every record built `fields.map(f => f.name)`; now the validator reads
string keys or field names directly. No cache, compiled schema or extra codec is
introduced. Each record avoids one temporary array (128 records avoid 128 such
arrays); this is a source-level allocation count, not measured JS heap bytes.
Own-key enumeration remains necessary, including symbols and non-enumerable keys.
All keys are checked before reading field values. `{x: 7}` still encodes `07`
for a single u8 field. Bytes, identities, limits, callbacks and rejection rules
are unchanged; regenerate browser clients. Rust/Wasm/IC costs are unaffected.

Node 24.21.0, CPU affinity 0, TypeScript stripped with Node's transform mode.
The retained `target/evidence/records-text-20261112/` sources/scripts record exact
source/frame SHA-256 identities, all samples and transformed JS sizes. Baseline
is the released 0.21.4 runtime. Both independent candidates compare against it,
not against each other. The primary study has 23 shapes, three alternating-order
runs, 11 calibrated batches per operation, 2–10,000 calls/batch, and 20 large or
1,000 small warmups. Exact frames and decoded results match all three lanes.
A focused record-only repeat uses five runs and eight shapes. This is shared-host
native JavaScript timing, not an IC, browser-fleet or application benchmark.

Focused repeat encode medians, microseconds per call (five runs):

| Shape | Before | Record candidate |
| --- | --- | --- |
| Empty record | .431/.489/.407/.402/.342 | .425/.479/.398/.349/.311 |
| One field | .645/.482/.498/.490/.545 | .527/.467/.465/.465/.497 |
| Eight fields | .920/.895/.918/.928/1.047 | .861/.855/.937/.878/.924 |
| 32 fields | 2.588/2.566/2.597/2.585/2.562 | 2.441/2.485/2.445/2.482/2.467 |
| 128 fields | 11.234/12.618/11.103/12.145/11.296 | 11.136/10.959/10.902/11.780/10.858 |
| 128 eight-field records | 73.650/102.418/75.699/74.399/73.380 | 70.604/69.093/75.497/74.271/69.868 |
| 128 32-field records | 280.647/292.599/292.255/293.031/296.137 | 279.951/355.407/274.033/271.936/270.791 |
| Unchanged scalar u32 | .303/.300/.295/.316/.235 | .316/.254/.232/.440/.227 |

32-field records improve 3–6% consistently, but do not claim a uniform speedup:
the second 32-field batch regresses 21.5%, an eight-field record run regresses
2.1%, and unchanged scalar controls fluctuate substantially. The three-run
primary study has batch improvements around 2–7%, with tiny-record regressions.
Retain for the simple allocation removal and consistent medium-record result,
with these losses explicit. Transformed unminified JS grows 22,553 → 22,569 bytes
(+16, excluding the subsequent explanatory source comment). No bundle-size claim.

**Reject direct UTF-8 writing.** The independent text candidate scans UTF-16 to
compute the exact UTF-8 length and detect lone surrogates, then calls encodeInto
on the writer's own storage. This removes the temporary encoded buffer, its copy
and a roundtrip decode check, but the JS scan is much slower on large strings.
It grows transformed JS by 691 bytes. Encode medians, microseconds (three runs):

| Text shape | Before | Direct candidate |
| --- | --- | --- |
| Eight ASCII characters | .76/.81/1.16 | .49/.42/1.05 |
| 128 ASCII characters | 1.35/1.22/1.27 | .93/1.46/1.75 |
| 262,144 ASCII characters | 185.88/238.71/203.01 | 1355.06/1314.45/1285.35 |
| 262,144 BMP characters | 993.14/1048.32/978.99 | 2171.57/2231.79/1749.38 |
| 262,144 emoji | 1853.13/1980.00/1879.65 | 3949.06/3923.08/3628.98 |

The independent differential script passes 394,233 frame/error comparisons,
including all 65,536 UTF-16 code units at six limits and surrogate boundary pairs.
That supports the prototype's tested acceptance, not its adoption. Production
retains native TextEncoder/TextDecoder validation. No text optimization is kept.

Reproduce from the repository root with `node target/evidence/records-text-20261112/prepare.mjs`
only on the recorded baseline, then `taskset -c 0 node target/evidence/records-text-20261112/measure.mjs`
and `taskset -c 0 node target/evidence/records-text-20261112/repeat.mjs`.
Run `node target/evidence/records-text-20261112/check.mjs` for differential tests.
The evidence sources stay frozen; do not overwrite them from a newer runtime.

Rust mixed-record investigation reused the existing
[aggregate attribution study](encoder-followup.md#aggregate-preflight-attribution--10-november-2026)
instead of rerunning the same experiment. Its historical mixed-tuple preflight
is 35.6% of full opt-z encoding (20.2% opt-3), not an additive removable saving.
Current encode paths still size/admit before allocating and then perform bounded
writing. Removing this pass changes resource/error/custom-Serde behavior. The
actual tuple fixture also uses i16::MIN, u32::MAX, i64::MIN and u64::MAX: its six
fields occupy 30 Postcard payload bytes versus 24 fixed-width value bytes, before
collection/schema overhead. That explains inherent wide-varint work, not all of
the Candid timing gap. No new Rust codec variant, unsafe allocation hint or
unqualified admission shortcut is introduced. Existing failed wide-writer and
one-pass experiments remain rejected; no fresh Rust timing/Wasm saving is claimed.

## Numeric first-byte decoding — 12 November 2026

Retain an immediate return for terminal first bytes in the numeric u16/u32
reader. Remaining groups start at index one with the first seven bits already
accumulated. All callers use 16/32-bit widths; u8/i8 remain raw bytes. Lengths,
Result/enum tags and signed ZigZag values share this existing reader. `7f` stays
127, `80 00` still accepts zero, and truncation/overflow/trailing-byte rules remain
unchanged. Bytes, identities, limits and callbacks are unchanged. Regenerate
browser clients; there is no Rust runtime or Wasm change.

Baseline includes pending record-field reuse; the rejected direct-key-loop
candidate below is excluded. Node 24.21.0, CPU affinity 0, five alternating-order
runs, 11 calibrated batches of 2–10,000 calls, 20 large/1,000 small warmups. The
36 shapes cover u16/u32 scalar/list widths, empty through 64 KiB byte buffers and
an unchanged boolean control. Exact encodings and decoded values match. Source,
frame SHA-256 identities, samples and scripts are retained in
`target/evidence/browser-varint-20261112/`. Candidate lane `record` is merely the
reused harness label: the only candidate change is Reader.numberVarint.

Decode medians, microseconds per call (five runs):

| Shape | Before | First-byte candidate |
| --- | --- | --- |
| u16 7 | .087/.087/.089/.082/.082 | .071/.082/.073/.067/.067 |
| u32 7 | .084/.087/.083/.087/.082 | .067/.077/.066/.067/.066 |
| u32 maximum | .133/.136/.134/.135/.136 | .126/.136/.129/.132/.131 |
| 128 u16 values of 7 | 4.618/8.727/5.957/5.936/6.283 | 2.563/3.843/3.013/3.007/3.193 |
| 128 u32 values of 127 | 4.371/6.038/5.993/6.084/6.031 | 2.438/3.092/3.062/3.241/3.378 |
| 128 u32 values of 128 | 5.954/7.372/7.214/7.411/7.333 | 5.675/6.118/5.947/6.163/6.009 |
| 128 u32 values of 16,384 | 7.750/8.970/9.035/9.353/9.085 | 7.358/7.800/7.661/8.109/7.814 |
| 128 maximum u32 values | 11.989/12.599/12.505/12.664/13.026 | 11.077/11.267/11.042/11.272/11.104 |
| Empty byte buffer | .139/.110/.114/.108/.107 | .147/.111/.108/.105/.104 |
| 64 KiB byte buffer | 11.570/9.652/10.445/11.045/8.995 | 9.219/10.222/10.006/9.407/10.291 |
| Unchanged bool | .102/.067/.063/.063/.064 | .071/.068/.063/.063/.063 |

Small-list gains are roughly 44–56%; wide lists also improve. Empty bytes regress
in two runs and the 64 KiB copy-dominated buffer is mixed (up to 14.4% slower).
This is shared-host Node evidence, not browser-fleet, allocation or application
savings. No encode timing gain is claimed. Transformed unminified JS grows
22,569 → 22,653 bytes (+84, excluding explanatory comments), with no Wasm impact.

The independent differential runner passes 403,974 decode/error comparisons:
every one/two-byte combination, continued prefixes through excessive group counts,
signed/unsigned widths and zero-bound byte/list lengths. Maintained conformance
adds a BigInt reference parser for signed/unsigned widths, redundant groups,
truncation, overflow and trailing input. Strict TypeScript, full Rust/browser
vectors/mutations, writer allocation/ownership and simulated endian checks pass.
Run `taskset -c 0 node target/evidence/browser-varint-20261112/measure.mjs` and
`node target/evidence/browser-varint-20261112/check.mjs` against retained sources.

### Rejected direct key loop

`target/evidence/record-loop-20261112/` replaces the object validator's `.some`
callback with a `for...of` loop, keeping the prior field-array removal. Five runs,
16 record/Option/Result/enum/scalar shapes and the same sampling method show no
consistent record win and repeated Result regressions. 32-field records are
2.435/2.784/2.793/2.746/2.734 µs before versus
2.477/2.751/2.830/2.747/2.730 after. Batches of 128 Ok results are
27.730/25.195/25.036/24.378/24.524 before versus
26.294/26.399/26.018/25.960/25.922 after. JS grows 57 bytes.
Reject the candidate; do not infer a saving merely from removing a callback.
This also avoids changing iteration behavior for unsupported sparse metadata.

## BigInt first-byte shortcuts — not retained, 12 November 2026

Investigated extending the numeric first-byte shortcut to the 64/128-bit BigInt
reader. Baseline is the pending numeric-reader/record-key work above. No BigInt
change is retained: both prototypes trade substantial small-value improvements
for repeated maximum-width regressions and generated JavaScript growth.
This is not a claim that every tradeoff is unacceptable; the current batch keeps
the better-qualified numeric-only improvement without broadening it automatically.

The `record.ts` research lane consumes the first byte, returns BigInt(first) for
terminal bytes and starts the continuation loop at index one. The `peek.ts` lane
only peeks at the first byte and leaves the original continuation loop untouched.
Both are isolated under `target/evidence/browser-bigint-20261112/`, with full
source/frame hashes, raw samples, JS sizes, differential checks and reproduction
scripts. Neither candidate was copied into production.

Node 24.21.0, CPU affinity 0, 31 scalar/list/signed/control shapes, five alternating
lane-order runs, 11 calibrated batches per row, 2–10,000 calls per batch and
20 large/1,000 small warmups. Each lane matches encoder frames and decoded values.
A separate four-shape, five-run repeat examines small u64, maximum u64/u128 and
an unchanged u32 control. Timing is shared-host native JavaScript, not IC
instructions, browser-fleet results or application savings. No Wasm is generated
or changed; no heap or encode-time saving is claimed.

Focused repeat decode medians, microseconds per operation, five runs. Each list
has 128 values. Each candidate has its own matched baseline run:

| Candidate / case | Before | Candidate |
| --- | --- | --- |
| Consume first / u64 7 | 4.899/6.086/5.648/6.080/5.844 | 2.448/2.277/2.219/2.328/2.366 |
| Consume first / u64 maximum | 42.487/41.773/41.414/42.746/48.424 | 39.353/42.803/42.309/41.913/42.361 |
| Consume first / u128 maximum | 81.230/79.704/78.753/79.516/77.255 | 77.715/84.139/81.645/84.083/85.143 |
| Peek / u64 7 | 4.849/5.699/5.551/5.787/5.829 | 2.480/2.338/2.262/2.519/2.711 |
| Peek / u64 maximum | 41.692/43.254/47.484/41.786/42.965 | 41.138/42.004/40.878/41.713/42.091 |
| Peek / u128 maximum | 78.271/77.669/78.856/77.901/80.637 | 80.499/78.385/78.379/80.263/86.813 |
| Peek / unchanged u32 scalar | .097/.073/.071/.075/.108 | .132/.073/.070/.071/.071 |

The full first study also contains a 48.7% maximum-u128 slowdown in one run;
its focused repeat still regresses in four of five runs, by up to 10.2%. The
peek repeat improves small-u64 lists 49–59%, but maximum-u128 regresses in four
of five runs, by 0.9–7.7%. Controls and some full-study rows have large noise;
these repeats prevent presenting only the strongest small-value result.
Transformed unminified JavaScript grows 22,653 → 22,752 bytes (+99) for consuming
the first byte, or → 22,821 (+168) for peeking. The extra peek branch apparently
changes generated-engine behavior even though the continuation source is intact;
that cause is a hypothesis, not established attribution.

Each candidate passes 283,652 baseline/candidate decode/error comparisons:
every one/two-byte combination plus continued prefixes through 20 groups, signed
and unsigned 64/128-bit schemas. This is prototype differential qualification,
not a full product suite or a reason to accept its performance losses. Runtime
bytes, identities, limits and acceptance in the repository remain unchanged.

Reproduce the full and focused runs with `taskset -c 0 node` followed by
`target/evidence/browser-bigint-20261112/measure.mjs`, `repeat.mjs`,
`measure-peek.mjs` or `repeat-peek.mjs` respectively. Differential scripts are
`check.mjs` and `check-peek.mjs` in the same directory. Preserve the recorded
sources and raw runs; do not overwrite them with a newer baseline.

## Array length encoding — not retained, 12 November 2026

Tested a narrower alternative to the earlier Writer.length helper: switch only
ordinary-list and packed-bool Array lengths from BigInt varint emission to the
existing numeric writer. Bytes/text/typed arrays retain their original path.
Normal JavaScript Array lengths fit u32, so the two-call-site change initially
appeared to preserve bytes while shrinking transformed unminified JS by four
bytes (22,653 → 22,649). No Rust/Wasm change is involved.

Matched sources and samples are under `target/evidence/array-length-write-20261112/`.
Node 24.21.0, CPU 0, five alternating-order runs of 37 scalar/bytes/text/list/
packed-u16/packed-bool/nested shapes; both encode and decode, 100 warmups and
11 calibrated batches of 5–10,000 calls. Every lane matches normal-input bytes
and decoded values. The full run is noisy, including unchanged controls. A
six-shape focused encode repeat uses seven runs and 2,000 warmups, otherwise the
same method. Focused medians in microseconds per call:

| Case | Before | Numeric array lengths |
| --- | --- | --- |
| Empty list | .267/.267/.264/.279/.280/.310/.279 | .397/.592/.250/.260/.265/.281/.267 |
| 128 u8 list | 5.673/5.766/5.698/5.731/5.783/5.767/5.893 | 5.516/5.475/5.469/5.574/5.710/5.540/5.769 |
| 128 packed bools | 1.499/1.371/1.527/1.609/1.694/1.557/1.675 | 1.501/1.359/1.516/1.508/1.396/1.449/1.538 |
| 128 nested three-u8 lists | 23.293/22.641/24.298/22.555/23.482/23.373/22.619 | 19.618/19.203/20.316/19.264/19.870/19.455/20.524 |
| Unchanged u32 scalar | .268/.266/.257/.268/.255/.266/.264 | .264/.270/.294/.274/.276/.302/.286 |
| Unchanged 128 bytes | .607/.575/.590/.613/.602/.633/.631 | .603/.583/.572/.605/.645/.693/.628 |

Nested encoding improves 9–17% in all seven focused runs. Empty-list regressions
and unchanged-control variation remain visible; no universal speedup is claimed.
390 additional frame/admission comparisons pass, including maximum sparse Array
lengths refused by short output buffers without iterating their contents.

**Do not retain the unconditional substitution.** `Array.isArray` also accepts
Proxy-backed Arrays, whose `length` getter can return a value outside u32. The
`proxy.mjs` reproducer supplies an empty packed-bool iterator, reports length
2^32 and uses a byte limit above that length. The baseline emits
`80 80 80 80 10`; the candidate silently wraps that to `00`. Neither lane traverses
or allocates billions of elements: this is a small, adversarial metadata/value
interaction. Baseline does not guarantee roundtrips for self-inconsistent custom
iterators, but that is not permission for an optimization to silently truncate
its observed length. No supported normal-Array defect is established here.

A safe future implementation must either retain checked wide-length fallback or
make an explicit acceptance-rule decision about exotic values. The earlier
helper/fallback study did not justify promotion; do not introduce it based only
on these unguarded measurements. Production remains unchanged, including encoder
bytes, identities, budgets and acceptance. No new full conformance/build suite
is claimed for this rejected prototype.

Reproduce using `taskset -c 0 node` with `measure.mjs` or `repeat.mjs` from the
retained evidence directory. Run its `check.mjs` and `proxy.mjs` directly with
Node for admission comparisons and the counterexample. Scripts retain frozen
before/after sources, source/frame SHA-256 identities and raw samples.

## Checked Array length writing — retained, 12 November 2026

Follow the rejected unguarded experiment with a checked helper used only by
ordinary-list and packed-bool Array lengths. Number.isInteger plus the inclusive
0..u32::MAX range admits the existing numeric writer; all other values retain
BigInt conversion and the original writer. The guard does not coerce objects or
invoke getters. This differs from the earlier broader length-helper experiment:
bytes, strings and typed arrays retain their old paths, and the baseline includes
the pending numeric decoder/record-key changes.

Normal 128 stays `80 01`; Proxy-reported 2^32 stays `80 80 80 80 10`, rather
than wrapping to `00`. The helper changes neither observed length reads nor
conversion errors. Bytes, identities, limits, callbacks and acceptance remain
unchanged. Regenerate clients; no Rust runtime/Wasm change. A self-inconsistent
custom iterator still has no roundtrip guarantee.

Evidence: `target/evidence/checked-length-write-20261112/`. Node 24.21.0,
CPU affinity 0, matched frozen runtime sources with source/frame SHA-256 identities.
The full study has 37 shapes, five alternating-order runs, encode/decode,
100 warmups and 11 calibrated batches of 5–10,000 calls. Exact normal frames and
decoded values match. Six focused encode shapes repeat for seven runs with
2,000 warmups. This is shared-host native JavaScript timing, not browser-fleet,
IC or application qualification. Raw samples and iteration counts are retained.

Focused encode medians, microseconds per operation (seven runs):

| Shape | Before | Checked helper |
| --- | --- | --- |
| Empty list | .258/.298/.553/.291/.293/.269/.275 | .318/.267/.254/.412/.263/.255/.254 |
| 128 u8 list | 5.705/5.996/5.448/5.480/5.735/5.427/5.675 | 5.213/5.602/5.272/5.490/5.618/5.452/5.434 |
| 128 packed bools | 1.711/1.671/1.395/1.469/1.540/1.497/1.559 | 1.584/1.491/2.369/1.520/1.594/1.332/1.427 |
| 128 nested three-u8 lists | 21.105/22.490/27.576/21.466/21.995/21.290/22.782 | 19.195/20.320/23.485/18.953/19.149/19.732/18.855 |
| Unchanged u32 scalar | .270/.269/.353/.414/.266/.273/.265 | .273/.262/.591/.382/.274/.272/.259 |
| Unchanged 128 bytes | .769/.643/.628/.605/.581/.632/.621 | .652/.609/.654/.619/.639/.594/.611 |

Nested-list gain is 7.3–17.2% across all seven focused runs; all five full-study
nested encode rows also improve (5.7–14.3%). Retain this modest shared helper
for that repeated gain. Small controls are mixed: packed bools regress in three
focused runs, including a 69.8% outlier accompanied by a 67.4% unchanged-scalar
regression; empty lists regress twice. Do not attribute every outlier to the
candidate or claim every case improves. Full unmodified decode/control rows also
vary. No decode or heap-allocation saving is claimed. Transformed unminified JS
grows 22,653 → 22,804 bytes (+151, excluding subsequent explanatory comments).
This is an explicit JS-size tradeoff, not an uncompressed-Wasm growth or saving.

390 ordinary frame/admission comparisons pass, including maximum sparse Arrays
refused before traversal by a short output allowance. Another 65 Proxy cases
match bytes/error classes and getter counts for wide, negative, fractional,
nonfinite, string, BigInt, null/undefined, Symbol and coercible-object lengths.
The original truncation reproducer now matches the baseline. Maintained tests
retain the wide Proxy prefix and packed-bool boundaries. Strict TypeScript,
full Rust/browser conformance and simulated-endian checks pass.

Run `taskset -c 0 node target/evidence/checked-length-write-20261112/measure.mjs`
or its `repeat.mjs`; differential scripts are `check.mjs`, `exotic.mjs` and
`proxy.mjs`. These use retained before/after sources; do not replace them with
current source or overwrite earlier samples.
