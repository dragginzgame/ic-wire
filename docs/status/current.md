# Current handoff

## Released baseline and pending 0.7.3

0.7.2 is released at `a4da015251beb8fa3346fb3780c0b3dc85c0ef32`; package versions
remain 0.7.2. Pending 0.7.3 combines the existing macOS schema-target correction
with published IC Tooling 0.68.1 and IC Platform 0.6.15 adoption. The incoming
lockfile already selected those releases and is preserved byte-for-byte. Incoming
changelog, handoff and schema-script work were retained. Staging, versions,
commits, publication and pushes remain maintainer-owned; siblings are read-only.

## Dependency adoption

Tooling supplier release: `c9c5ef7c44e3d0c3b23c8531a5d43c1800656c0a`.
Platform supplier release: `6dcf54a229201f85775d218b7a9f41809297878f`.
Root requirements now select Build/Testkit 0.68.1 and Platform 0.6.15; the
release metadata editor is also prepared/admitted at 0.68.1. The three shared
host packages resolve to 0.68.1. No new features, dependency owners or runtime
packages were introduced. Shared Tooling's snapshot remains 0.26.10 at
`4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.

Registry verification confirms the selected releases. Cargo's update dry run
found zero further compatible updates under Rust 1.91 and the existing ranges;
TypeScript's npm outdated check is empty. Wire's normal Wasm dependency tree
is unchanged. This is host/dev dependency adoption, not a codec optimization;
wire bytes, limits, normalization, hashes and retry contracts are unchanged.
Task-start copies and qualification logs are under `/tmp/wire-deps-073/`.

Adoption validation passed: boundary/doc links, release-editor admission,
disposable release/Make fixtures, tooling checks, schema forwarding, nine typed
diagnostics plus their passing control, and the benchmark gate (including cohort
tests, Clippy and Rust 1.91 native/Wasm checks). The incoming lockfile and normal
Wasm dependency tree compare byte-for-byte with their task-start copies.

The updated host driver also ran against the retained bulk-advice Wasms using
the explicitly prepared, supported PocketIC 16.0.0 server. All 2,180 rows across
109 cases exactly match the archived samples, including payloads and instruction
counts. This qualifies host integration with existing artifacts, not fresh codec
performance or artifact-size savings; no runtime/generator code changed. The
server identity, artifacts and samples are recorded under
`/tmp/wire-deps-073/ic-check/instructions/`.

## Benchmark report completeness

The pending benchmark-report fix selects `wire-packed-i16` as the required lane
for the packed-i16 cohort, matching the matrix manifest. Previously the runner
required only its native control, allowing missing packed rows to go unnoticed.
Both lanes remain visible in reports. Reporter regression tests cover rejection
of native-only results and successful output containing both lanes. This changes
qualification completeness only; runtime, generator and protocol are unchanged,
so codec performance measurements are not applicable to this fix.
The three reporter tests, focused Clippy, runner shell syntax, formatting and
documentation links pass.

Report input validation now rejects missing/invalid identifiers, missing input
hashes, nonpositive timings and nonpositive/noninteger encoded sizes before
pairing. Mismatched input hashes return an error rather than panicking, and
directory iteration errors propagate. Regression coverage includes eleven
malformed-row cases and an identity mismatch, alongside valid paired output.
Reporter tests and focused Clippy pass. This is evidence validation only; codec
cost and wire acceptance are unchanged, so no performance benchmark is required.

## Schema forwarding and issue status

Issue #18 is closed: delivered Shared adoption and applicable setup/tooling checks
pass on Linux and both macOS architectures in Actions run 40098944755.
Issue #17 remains open pending delivery and hosted macOS qualification of the
schema-script correction. Both macOS jobs failed at empty options[@] expansion;
Rust/TypeScript frame vectors had already passed, and Linux CI passed in full.

The pending script keeps Cargo argument arrays nonempty under Bash 3.2 nounset.
Runtime-only and schema-forwarded selections retain their original arguments
and checks. Local native tests, both Wasm feature/dependency graphs and Rust 1.91
forwarding passed when the fix was prepared; Linux does not replace macOS evidence.

## Retained codec direction and evidence

### Wasm budget-sharing experiment (0.7.3)

Two isolated variants change only the three Wasm budget-method inlining hints:
remove forced inlining (automatic), or force shared calls (outlined). Neither is
retained: modest code-size savings cost substantial IC instructions. Production
source and protocol remain unchanged; these are research, not codec modes.

| Types / variant | Total Wasm bytes | Code-section bytes | Encode instructions | Decode instructions |
| --- | ---: | ---: | ---: | ---: |
| 1 / baseline | 53,841 | 42,841 | 3,856.5 | 2,316.5 |
| 1 / automatic | 53,859 | 42,857 | 3,959.5 | 3,056.5 |
| 1 / outlined | 53,687 | 42,684 | 4,774.5 | 3,314.5 |
| 100 / baseline | 310,664 | 291,897 | 4,523.5–7,515.5 | 3,269.5–5,796.5 |
| 100 / automatic | 307,441 | 288,671 | 4,865.5–8,607.5 | 3,848.5–7,467.5 |
| 100 / outlined | 308,174 | 289,403 | 5,277.5–9,469.5 | 4,104.5–8,173.5 |

At 100 types, automatic saves 1.04% total / 1.11% code bytes but raises encode
instructions 7.56–14.53% and decode 17.71–28.83%. Outlined saves 0.80% total /
0.85% code bytes but raises encode 16.67–26.00% and decode 25.54–41.01%.
These fixture results do not establish installed-consumer savings.

Baseline: task-start working tree based on `a4da015`, with the current identical
dependency lock for all variants. Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown,
opt-z, LTO, one codegen unit, panic abort, symbols stripped. PocketIC 16.0.0;
three samples of four operations, median counter 0 instructions per operation,
no overhead subtraction. All 1,515 rows preserve input/frame hashes and lengths.
No native-time or opt-3 claim. [Sizes](../evidence/wasm-budget-073/sizes.json),
[per-case instructions](../evidence/wasm-budget-073/instructions.csv) and
[source, runner, compiler, artifacts and raw samples](../evidence/wasm-budget-073/raw.tar.gz)
retain reproduction inputs; extract the archive and run `bash replay.sh` with
prepared Rust/dependencies and an explicit `POCKET_IC_BIN`.

### Further Wasm experiments (0.7.3)

Isolated scalar-reader sharing did not improve size: outlining u16/u32/u64/u128
adds 247 total / 245 code bytes at one type, and 148 total / 146 code bytes at
100 types. Outlining only u64 yields byte-identical baseline Wasms. These variants
were rejected at the size screen; instruction measurements were not needed.

Removing the two encoding-wrapper forced-inline hints initially looked viable;
the broader qualification below rejects it. Budget hints remain unchanged. At 100 types it saves
2,768 total/code bytes (0.89% total, 0.95% code), while encode instructions vary
from -0.29% to +0.23% and decode instructions are identical. At one type it adds
354 total / 353 code bytes (0.66% total), and encode rises 3,856.5 to 3,986.5
instructions (+3.37%); decode stays 2,316.5. These initial results covered opt-z only.

Changing only the build optimization level also exposes a consumer tradeoff:

| Types / optimization | Total Wasm bytes | Code-section bytes |
| --- | ---: | ---: |
| 1 / z | 53,841 | 42,841 |
| 1 / s | 49,845 | 39,449 |
| 1 / 3 | 74,844 | 48,102 |
| 100 / z | 310,664 | 291,897 |
| 100 / s | 317,655 | 299,570 |
| 100 / 3 | 398,712 | 364,218 |

At one type, opt-s saves 7.42% total / 7.92% code bytes, with encode instructions
3,856.5 to 2,317.5 (-39.91%) and decode 2,316.5 to 1,761.5 (-23.96%). At 100
types opt-s grows total Wasm 2.25% / code 2.63%, while encode instructions fall
48.79–55.62% and decode 33.89–39.21%. Opt-3 was rejected on size without an IC run.
No profile or production source was changed. Consumer builds must measure their
own contract mix rather than infer savings from the one-type control.

Settings and baseline match the budget experiment above, except the explicitly
varied optimization levels. IC samples use three queries of four repetitions;
all 2,020 rows retain matched input/frame hashes and lengths. No native timing
claim. [Sizes](../evidence/wasm-scalar-073/sizes.json),
[absolute per-case instruction results](../evidence/wasm-scalar-073/instructions.csv)
and [source, scripts, raw samples and artifacts](../evidence/wasm-scalar-073/raw.tar.gz)
retain evidence. Extract and run `bash replay.sh` with prepared dependencies and
an explicit `POCKET_IC_BIN`. Protocol and runtime behavior remain unchanged.

### Encoding-wrapper follow-up: rejected

The full focused comparison removes only the two Wasm forced-inline hints in
`native/encode`, leaving budget checks intact. Baseline and candidate use identical
dependencies, fixtures and compiler settings per profile. No production source or
profile changes were retained: the size wins do not justify the encode regressions.

| Cohort | Total Wasm delta, opt-z | Total Wasm delta, opt-3 | Worst fresh-encode IC change, opt-3 |
| --- | ---: | ---: | ---: |
| Standard (298 cases) | +1,588 bytes | -270 bytes | +100.97% |
| Toko (10 cases) | -1,131 bytes | -744 bytes | +13.17% |
| Bulk (109 cases) | +144 bytes | +2,031 bytes | +29.81% |
| 1 type | +354 bytes | +445 bytes | +16.53% |
| 100 types | -2,768 bytes | -2,900 bytes | +46.55% |

The worst standard encode is the 16,384-element small-u32 list: 1,606,561.25 to
3,228,721.25 instructions at opt-3; at opt-z, 3,573,602.5 to 4,147,000.5 (+16.05%).
Reused encoding also regresses up to 100.99%. Decode instruction counts are
identical across every measured pair. All 6,246 paired rows match input/frame
hashes and byte lengths, including the retained malformed-header/truncation paths.

Same task-start runtime/lock, Rust 1.99.1 / LLVM 23.1.1, LTO, one codegen unit,
panic abort and symbols stripped; opt-z and opt-3 are test conditions, not proposed
consumer settings. PocketIC 16.0.0, three samples, four repetitions per short
operation and one above 64 KiB; counter overhead is not subtracted. No native-time
claim. [Absolute total/code sizes](../evidence/encoder-inlining-073/sizes.json),
[per-case instruction changes](../evidence/encoder-inlining-073/instructions.csv)
and [source, compiler, artifacts, samples and replay script](../evidence/encoder-inlining-073/raw.tar.gz)
retain reproduction evidence. Extract and run `bash replay.sh` with prepared
dependencies and explicit `POCKET_IC_BIN`; replay uses a fresh temporary directory.
Protocol is unchanged. Formatting and documentation-link checks pass.

### Production direction

One production Serde/Postcard codec remains. Decode validates primitives in one
pass; collection ordering/uniqueness and custom Serde normalization are consumer
responsibilities. Retain original frames for exact retries and raw-byte signatures/
hashes. Successful decoding does not guarantee identical re-encoding. No legacy
reader or migration shim exists.

Use ByteBuf/Bytes for binary data and Packed<T> for supported bulk integer/hash
fields, including i8. The optional host schema lint reports ordinary primitive
lists, supports exact-path acknowledgements with reasons and stale detection,
and can fail consumer CI. It does not rewrite fields. Preserve bounds and custom
validation on adoption; tiny fields or small varints can favor ordinary vectors.

The [0.7.1 bulk qualification](../evidence/bulk-advice-071/README.md) retains 109
cases and 2,180 matched IC rows. Every selected packed encode/decode case beats
Candid in both measured profiles, with documented small-input regressions versus
ordinary fields. Those archived dependencies/measurements remain historical.
The [one-pass study](../evidence/one-pass-070/README.md) and
[0.6.13 full matrix](../full-comparison-0613.md) retain their original scope.
See the [bulk guide](../packed-fields.md) and [runtime contract](../runtime.md).
Real big-endian execution remains unqualified; consumer integration and storage
contracts remain consumer-owned.
