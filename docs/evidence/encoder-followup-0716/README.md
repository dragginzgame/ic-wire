# Encoder follow-up for 0.7.16: no additional production change

Five isolated candidates were evaluated. None is promoted. The retained u16
decoder from the [preceding study](../runtime-0716/README.md) stays intact; this
batch changes no production codec, protocol bytes, acceptance rules or API.

## Source and measurement identity

Baseline is released 0.7.15 (`47f011702edd0fce99d4272fd80b02991ee5f94f`) plus
the frozen [baseline patch](baseline.patch): the retained decoder/oracle and the
concurrent lock update to Build/Testkit/host 0.68.6 and Platform 0.6.17. This is
uncommitted source, not a released 0.7.16. Candid is 0.10.39, Postcard 1.1.3.
Unlike the preceding study, these experiments include the newer lock graph.

All variants use identical source paths, dependencies, inputs and settings:
Rust 1.99.1 (`58cf8f9c3`), wasm32-unknown-unknown, evaluation/opt-z and
benchmark-speed/opt-3, LTO, one codegen unit, stripped symbols, aborting panics
and no extra RUSTFLAGS. The maintained standard corpus has 298 cases and 7,006
operation rows per variant; the derive-scale corpus has 100 types and 2,000 rows.
Each counter has three equal repeated samples, four repetitions for small frames
and one above 65,536 bytes. Result destruction and loop overhead are included;
transport/setup are excluded. Native wall-clock time was not measured.

[Raw IC identities and samples](instructions.json.gz) include compiler, limits,
server and artifact hashes. PocketIC 16.0.0 server SHA-256 is
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
Every paired input/frame hash matches; Candid control counters are unchanged.
The baseline is reused only where artifact hashes match exactly. These are
codec qualification fixtures, not installed endpoint or fleet measurements.

## Uncompressed Wasm

Baseline total/code bytes are 377,817/337,789 (standard z), 573,226/518,516
(standard 3), 310,738/291,971 (100 types z), 398,739/364,245 (100 types 3).
Cells below give **total / code byte changes** from those absolute baselines.
Full absolute sizes and artifact hashes are in [sizes.json](sizes.json).

| Candidate | Standard z | Standard 3 | 100 types z | 100 types 3 |
|---|---:|---:|---:|---:|
| [Inline only try_push](push.patch) | −36 / −36 | +3,494 / +3,467 | 0 / 0 | −2,509 / −2,505 |
| [Inline only try_extend](extend.patch) | −30 / −28 | −423 / −440 | −451 / −449 | −299 / −295 |
| [One pass, generic reserve](one-pass.patch) | −12,684 / −12,626 | +5,123 / +5,118 | −8,655 / −8,614 | −9,393 / −9,388 |
| [One pass, outlined reserve](one-pass-shared.patch) | −12,684 / −12,626 | −7,457 / −7,443 | −8,655 / −8,614 | −16,657 / −16,646 |
| [One pass, cold growth](one-pass-fast.patch) | −12,626 / −12,568 | +569 / +583 | −8,597 / −8,556 | −13,645 / −13,640 |

Outlined one-pass reserve gives the largest 100-type reduction: total
398,739 → 382,082 (−4.18%), code 364,245 → 347,599 (−4.57%). It fails instruction
qualification below. Cold growth reduces that fixture to 385,094/350,605
(−3.42%/−3.74%), but grows standard opt-3 and regresses several inputs.
Inline-push is rejected on mixed size alone; its instructions were not measured.
Gzip is used only to store evidence and is not a selection criterion.

## IC encoding results

These are opt-3 fresh-encode instructions per operation. Wide lists have 16,384
elements; the last record row is a small-input control from the 100-type fixture.

| Candidate / case | Before | After | Change |
|---|---:|---:|---:|
| Inline extend / wide u16 | 1,852,251.25 | 3,638,108.25 | +96.42% |
| Inline extend / unit | 385.25 | 385.25 | 0% |
| Generic reserve / wide u16 | 1,852,251.25 | 1,601,219.25 | −13.55% |
| Generic reserve / wide i16 | 2,638,675.25 | 2,108,953.25 | −20.08% |
| Generic reserve / unit | 385.25 | 431.25 | +11.94% |
| Generic reserve / nested blobs, 128 | 142,680 | 218,201 | +52.93% |
| Outlined reserve / wide i16 | 2,638,675.25 | 4,484,805.25 | +69.96% |
| Outlined reserve / ordinary byte vector, 1 MiB | 78,644,279 | 127,407,894 | +62.01% |
| Outlined reserve / unit | 385.25 | 549.25 | +42.57% |
| Cold growth / wide u16 | 1,852,251.25 | 1,552,523.25 | −16.18% |
| Cold growth / wide i16 | 2,638,675.25 | 2,076,798.25 | −21.29% |
| Cold growth / unit | 385.25 | 542.25 | +40.75% |
| Cold growth / nested blobs, 128 | 142,680 | 209,379 | +46.75% |
| Cold growth / byte buffer, 1 MiB | 1,049,704 | 1,050,397 | +0.07% |
| Cold growth / record_000 | 1,695.25 | 1,794.25 | +5.84% |

Inline-extend's smaller Wasm is insufficient to accept its near-doubling of wide
u16 instructions. Sharing every reserve call adds hot-path overhead. Moving only
growth out of line recovers list performance, but does not resolve small-input,
nested-blob, footprint and memory tradeoffs. Opt-z and all remaining operations,
including reuse, decode and rejection, remain in the archive. The
[analyzer](analyze.mjs) validates pairing/repeats and emits every Wire row, including
regressions. None of these results eliminates ordinary-list versus Candid gaps.

## One-pass contract and allocation tradeoffs

Production still performs admission/sizing followed by writing. Research replaces
that with one bounded serialization and fallible geometric output growth, starting
at 64 bytes and capped by the frame byte limit. Successful deterministic corpus
frames are identical, but allocation may precede a later admission refusal and
custom Serialize runs once rather than twice. This is a documented contract
change, not a safe inlining-only substitution. No compatibility lane was added.

The generic-reserve prototype passes 2 unit, 6 borrowed, 39 native and 15 runtime
tests when the existing two-pass-specific test is excluded
([logs](prototype-tests.json.gz)). The unmodified full run fails
`reusable_output_keeps_capacity_and_clears_all_failure_paths`: its
FailSecond serializer returns an error on the second call; research calls it
once and returns success. This expected failure is retained as a qualification
limit, not described as a passing production suite. Allocation failure injection
and complete custom-serializer/failure-path qualification were not performed.

The cold-growth prototype was also measured with the 87-case native allocation
suite (opt-3). This is one allocation-count observation per operation, not native
timing or peak-live-heap measurement. [Raw allocation evidence](allocations.json.gz)
and [paired output results](allocation-changes.csv) retain hashes and controls.

| Fresh encoding | Frame bytes | Output capacity before → after | Reallocations before → after |
|---|---:|---:|---:|
| Empty i16 list | 12 | 12 → 64 | 0 → 0 |
| Wide u16 list, 16,384 | 49,166 | 49,166 → 65,536 | 0 → 10 |
| Ordinary byte vector, 1 MiB | 1,048,590 | 1,048,590 → 2,097,152 | 0 → 15 |
| Byte buffer, 1 MiB | 1,048,590 | 1,048,590 → 1,048,590 | 0 → 1 |

All four fresh paths still make one initial allocation. Reuse with sufficient
capacity remains allocation-free in these controls. Geometric growth increases
reserved output memory and may recopy data; it is not a free removal of sizing.

## Reproduction and validation

Use the pinned compiler/Wasm target, cached locked dependencies, Node and prepared
PocketIC server. The [runner](measure.sh) reconstructs the baseline in a new
absolute directory and applies each candidate independently, restoring baseline
source between builds. Original measurements used equivalent isolated builds.

```sh
bash docs/evidence/encoder-followup-0716/measure.sh one-pass-fast /absolute/new-run
node docs/evidence/encoder-followup-0716/analyze.mjs docs/evidence/encoder-followup-0716 > /tmp/encoder-changes.csv
```

For counters use the [preceding IC commands](../followup-0712/README.md#reproduce)
with that new run directory. Build one driver per cohort: wire,candid,ic for
standard and wire,candid,ic,derive-scale for hundred. Build matching Candid Wasm
with candid or candid,derive-scale respectively, then run both variants. This
research does not change driver, corpus, limits or operation definitions.

For allocation reproduction, apply one-pass-fast only to the candidate native
build and use `cargo bench --no-run --locked --offline --profile benchmark-speed
-p ic-wire-benchmarks --bench suite --features wire,candid,hotspots,allocations
--message-format=json`. Copy each reported executable before rebuilding; invoke
each from the real repository as `BINARY NEW-OUTPUT benchmark-speed 3 1`.
The latter two arguments do not repeat allocation observations. Invocation Git
observations describe the real checkout; the baseline/candidate patches, lock,
fixture hashes and executable hashes identify measured source.

Retained evidence passes paired-frame/control/repeat analysis, patch admission,
shell/JavaScript syntax and documentation links. Production Rust was not changed
by this follow-up; the preceding decoder checks remain its owning validation.
Full CI, installed consumers, macOS and a fresh full native timing suite were not run.
