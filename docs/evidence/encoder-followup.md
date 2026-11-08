# Encoder experiments

## Signed i32 writer screen (8 November 2026)

Retain none of three rewrites against Core **0.17.14**. The new `signed32`
qualification cohort contains 49 scalar, distribution, container and Packed
controls; the standard fixture keeps all 391 cases. Existing encodings and
admission rules were preserved in each candidate, but no candidate consistently
reduces total and code-section Wasm at both profiles. Production stays unchanged.

- **Unsigned shortcut:** zigzag then call Wire's existing u32 writer, including
  its one-byte shortcut, following the existing i64 pattern.
- **Small-only shortcut:** emit a raw byte for zigzag values below 128 and
  otherwise retain Postcard's signed writer.
- **Direct unsigned:** zigzag then call Postcard's unsigned writer directly.

Whole uncompressed module **total / code-section bytes**. Each cell gives
candidate absolute sizes followed by total/code deltas against its row baseline.
These are linked fixtures, not application savings or symbol attribution.

| Fixture/profile | Baseline | Unsigned shortcut | Small-only shortcut | Direct unsigned |
| --- | ---: | ---: | ---: | ---: |
| signed32 / opt-z | 75,350 / 63,270 | 75,651 / 63,567 (+301/+297) | 75,651 / 63,567 (+301/+297) | 75,350 / 63,270 (0/0) |
| signed32 / opt-3 | 92,373 / 81,148 | 92,112 / 80,887 (-261/-261) | 92,077 / 80,852 (-296/-296) | 92,373 / 81,148 (0/0) |
| standard / opt-z | 594,543 / 547,747 | 594,530 / 547,734 (-13/-13) | 594,594 / 547,798 (+51/+51) | 594,544 / 547,747 (+1/0) |
| standard / opt-3 | 891,435 / 830,841 | 891,628 / 831,034 (+193/+193) | 891,593 / 830,999 (+158/+158) | 891,376 / 830,782 (-59/-59) |

The direct-unsigned signed32 modules are byte-identical to baseline at both
profiles. Its tiny standard-module tradeoff does not justify another override.
All three stop at this footprint screen: no candidate native-time or IC-speed
improvement is claimed. The separate current-codec/Candid signed32 run measures
the new workloads; it is not before/after evidence for a retained optimization.

Raw inputs and reproduction scripts are under
`target/evidence/signed32-20261108/`: `screen.sh`, `screen-direct.sh`, `sizes.csv`,
`measure.sh`, four source snapshots and separate Cargo targets. Baseline source
is **0349559** plus the private corpus/evidence changes; lock SHA-256
`d06ec6f62d553f54d94c6d771b182a0c2f6311897fe9b376a20a561f4aef3cee`.
Rust/Cargo **1.99.1**, LLVM **23.1.1**, `wasm32-unknown-unknown`, fat LTO, one
codegen unit, panic abort and stripped symbols; `evaluation` opt-z and
`benchmark-speed` opt-3, no post-link optimizer. All 16 guest modules rebuilt;
each lane/profile compiled its runtime afresh before reusing that exact runtime
for the standard fixture. Code size is final section-10 payload bytes.

## Raw-tuple i32 follow-up (8 November 2026)

A fourth candidate follows the existing i16 direct-byte approach rather than
delegating to another Postcard integer writer. Remove `serialize_i32` from
scalar forwarding; zigzag into u32, calculate the exact width with
`((32 - value.leading_zeros()).max(1) as usize).div_ceil(7)`, start an inner tuple
of that width, emit low seven-bit groups with the continuation bit while shifting
right by seven, then emit the last byte and end the tuple. Admission still visits
the original i32 once; raw implementation bytes never enter the bounded adapter.
Only Postcard output/sizing serializers receive this private tuple.

Matched whole-module uncompressed **total / code-section bytes**:

| Fixture/profile | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| signed32 / opt-z | 75,350 / 63,270 | 75,500 / 63,418 | +150 / +148 |
| signed32 / opt-3 | 92,373 / 81,148 | 91,230 / 80,005 | -1,143 / -1,143 |
| standard / opt-z | 594,543 / 547,747 | 594,633 / 547,837 | +90 / +90 |
| standard / opt-3 | 891,435 / 830,841 | 891,015 / 830,421 | -420 / -420 |

This uses the preceding baseline, compiler, lock and profiles, with a separate
`rawtuple/source` and `rawtuple/build` under the same study directory. Scripts
`screen-rawtuple.sh` and `measure-rawtuple.sh` retain reproduction; the candidate
diff is `rawtuple.patch`. The existing scalar byte/sizing/roundtrip and exact
allocation/node/depth test passes, including all signed width boundaries.

The maintained deque test now keeps logical input fixed across every physical
storage split at 128 elements, plus empty/singleton cases. It compares Vec and
VecDeque bytes in both codecs and checks exact encoder admission and one-short
allocation, node and depth rejection. Serde's `collect_seq` visits `&i32` iterator
items, so native accounting uses reference size for children, not i32 size. These
tests exercise correctness/admission; wrapped-deque performance is not measured.

**Adopted for pending Core 0.18.0** after the maintainer explicitly accepted
the tradeoff and made opt-z primary, with opt-3 retained as a regression check.
The matched policy admits all **482 Wire operation
pairs** within its `baseline × 1.05 + 64` instruction threshold, but rejects
opt-z total/code growth. This failure remains recorded; adoption does not claim
that the no-growth check passes. All **1,070 IC rows** have three identical samples from
distinct callers. Candid modules/counters, inputs and frames match exactly;
allocation/heap measurements are unchanged in both profiles. Fresh runtime/guest
builds and baked encoder/lock identities pass. Initial sandboxed server startup
could not bind localhost; the preserved `instructions-failed-localport` directory
is not evidence. `instructions-retry2.log` records the completed local-server run.

Selected **encode IC instructions**, 4,096 elements:

| Shape | opt-z before → after | opt-3 before → after |
| --- | ---: | ---: |
| Small list | 2,147,946.5 → 1,152,618.5 | 947,070.25 → 430,973.25 |
| Wide list | 3,245,674.5 → 2,348,650.5 | 1,278,846.25 → 1,323,901.25 |
| Mixed deque | 2,575,922.5 → 1,618,166.5 | 989,259.25 → 652,099.25 |
| Unique set | 3,263,246.5 → 2,291,726.5 | 1,383,715.25 → 963,107.25 |

Small controls matter: scalar zero changes **1,463.5 → 1,220.5 / 761.25 →
644.25** at opt-z/3, while i32::MAX changes **1,731.5 → 1,512.5 / 834.25 →
882.25**. The latter is a **5.75% (+48 instructions)** opt-3 regression; the wide
list adds **3.52%**. Passing the policy margin does not mean no regression.

Two serial native runs/profile, eleven samples/run, **encode median ns**:

| Shape/profile | Baseline runs 1 / 2 | Candidate runs 1 / 2 |
| --- | ---: | ---: |
| Small list / opt-z | 60,492 / 61,280 | 29,648 / 30,166 |
| Wide list / opt-z | 71,636 / 69,901 | 58,688 / 57,109 |
| Deque / opt-z | 66,391 / 63,282 | 38,788 / 39,906 |
| Set / opt-z | 75,559 / 75,298 | 45,569 / 46,651 |
| Small list / opt-3 | 18,963 / 19,022 | 7,825 / 7,659 |
| Wide list / opt-3 | 31,403 / 31,564 | 21,878 / 21,863 |
| Deque / opt-3 | 22,916 / 22,789 | 11,360 / 11,965 |
| Set / opt-3 | 30,194 / 29,391 | 14,772 / 14,575 |

Native wide-list encode improves while opt-3 IC instructions regress: do not
substitute host timing for IC evidence. These are focused fixture results; the
standard corpus was screened for Wasm only, and no installed application or
native macOS result is claimed. The production writer now uses this algorithm;
the measured source's experimental comment is replaced with the byte/admission
contract. No second codec, per-consumer codec variant or new public API is added.
Fresh `adopted/source` builds with that production comment yield byte-identical
signed32 and standard Wasm at both profiles versus the measured candidate;
`adopted/artifact-comparison.json` records hashes and fresh guest/runtime checks.
Current native tests (47 passed, one explicit timing test ignored), the focused
slice case, signed32/standard corpus tests (9/6), Core/corpus Clippy, formatting
and documentation links pass. No full release gate was run.
Protocol bytes, schemas, identities, limits and acceptance remain unchanged;
i32 -64 remains `7f`, and 64 remains `80 01`. Consumers need no migration or
client regeneration and retain their existing retry/hash bytes.

## Unsigned writer sharing (8 November 2026)

The next baseline includes the adopted i32 writer above. `share32` moves that
loop into `serialize_u32` and makes i32 delegate after zigzag, preserving u32's
one-byte shortcut. `direct64` applies the same raw-tuple algorithm to u64 while
keeping its shortcut; i64 already delegates after zigzag. Exact tuple width is
`(bits - value.leading_zeros()).div_ceil(7)` after the one-byte return. These are
private scalar writers in the existing codec, not per-consumer codec variants.

Initial independent screens, whole linked **total / code-section Wasm bytes**:

| Fixture/profile | Baseline | Shared u32/i32 | Direct u64/i64 |
| --- | ---: | ---: | ---: |
| hotspots / opt-z | 83,508 / 72,547 | 83,204 / 72,259 | 83,233 / 72,295 |
| hotspots / opt-3 | 103,969 / 93,675 | 103,660 / 93,365 | 103,545 / 93,251 |
| standard / opt-z | 594,633 / 547,837 | 594,601 / 547,805 | 594,364 / 547,585 |
| standard / opt-3 | 891,015 / 830,421 | 888,459 / 827,866 | 875,264 / 814,672 |

The combined writer is measured directly; gains are not inferred by adding the
two screens. Baseline/combined **total / code-section Wasm bytes**:

| Fixture/profile | Baseline | Combined | Change |
| --- | ---: | ---: | ---: |
| hotspots / opt-z | 83,508 / 72,547 | 82,929 / 72,007 | -579 / -540 |
| hotspots / opt-3 | 103,969 / 93,675 | 103,236 / 92,941 | -733 / -734 |
| standard / opt-z | 594,633 / 547,837 | 594,332 / 547,553 | -301 / -284 |
| standard / opt-3 | 891,015 / 830,421 | 872,689 / 812,098 | -18,326 / -18,323 |
| Toko-shaped / opt-z | 80,916 / 69,754 | 80,303 / 69,183 | -613 / -571 |
| Toko-shaped / opt-3 | 103,366 / 92,866 | 101,221 / 90,722 | -2,145 / -2,144 |

Toko-shaped means the private ten-case DTO fixture, not an installed Toko build.
The standard fixture contains many shapes/types; its larger savings are not
application savings or symbol-attribution claims.

Reproduction is under `target/evidence/integer-writers-20261108/`:
`prepare.mjs`, `prepare-combined.mjs`, `screen.sh`, `screen-combined.sh`,
`screen-toko.sh`, `measure.sh` and `follow-up.sh`. Compiler/profiles match the
preceding study: Rust/Cargo 1.99.1, LLVM 23.1.1, wasm32-unknown-unknown, fat LTO,
one codegen unit, panic abort, stripped symbols, no post-link optimizer. Each
lane has its own source and Cargo target. The combined target seeds completed
dependency caches from baseline, with fresh product inputs and artifact checks.

The matched sources retain lock SHA-256
`d06ec6f62d553f54d94c6d771b182a0c2f6311897fe9b376a20a561f4aef3cee`.
A separately updated checkout lock selects Build 0.79.1 / Host 0.75.1 and is
preserved; these measurements do not qualify that supplier update. Native
opt-z runs overlapping the first footprint screen are retained as `*-initial`
and excluded; `follow-up.sh` replaces both with serial runs after all screens.

**Do not adopt the combined candidate.** Across the 87-case hotspot corpus,
the matched study retains **1,904 IC rows**, each with three identical samples
from distinct callers. Input/frame hashes and Candid modules/counters match;
allocation and heap measurements are identical in both profiles. Fresh changed
runtime/guest checks and baked encoder/lock identities pass. The automated screen
still fails **24 of 860 Wire operation pairs**: four opt-z and twenty opt-3.
It passes both Wasm metrics, but the instruction regressions are material.

Selected **encode IC instructions**, 16,384 elements except the mixed record:

| Shape | opt-z before → after | opt-3 before → after |
| --- | ---: | ---: |
| Wide u32 | 12,207,902 → 8,177,369 (-33.0%) | 4,326,408 → 5,047,297 (+16.7%) |
| Wide i32 | 7,948,064 → 8,390,361 (+5.56%) | 4,752,384 → 5,276,673 (+11.0%) |
| Wide i64 | 18,024,221 → 15,271,641 (-15.3%) | 6,996,982 → 10,372,097 (+48.2%) |
| Wide u64 | 17,811,229 → 15,058,649 (-15.5%) | 6,767,606 → 10,142,721 (+49.9%) |
| Mixed record, empty blob | 5,197.5 → 4,577.5 (-11.9%) | 2,136.25 → 2,081.25 (-2.6%) |

The i32 delegation introduces the unsigned shortcut branch into its path; the
combined candidate regresses the newly adopted writer even at the primary opt-z
profile. The branch is a plausible follow-up target, but branch/inlining/layout
effects have not been isolated. This regression is not justified by the footprint
gain. The opt-3 long-word losses also exceed a
minor secondary-profile tradeoff. A singleton wide u64 adds 17.3% opt-3 encode
instructions; empty and byte-field controls remain visible in the raw study.
The 1 MiB mixed-blob record saves only 688/55 encode instructions at opt-z/3;
do not project the empty-record percentage onto blob-dominated work.

Two serial native runs/profile, eleven samples/run, **encode medians in ns**:

| Shape/profile | Baseline runs 1 / 2 | Combined runs 1 / 2 |
| --- | ---: | ---: |
| Wide u32 / opt-z | 235,770 / 232,941 | 193,405 / 185,328 |
| Wide u64 / opt-z | 281,186 / 288,942 | 310,701 / 304,993 |
| Wide u32 / opt-3 | 104,565 / 105,405 | 84,213 / 80,837 |
| Wide u64 / opt-3 | 139,160 / 141,485 | 289,252 / 166,213 |

Host timing varies substantially on some controls (see all absolute runs in
`analysis.json`); it is not substituted for deterministic IC counters. Native
u32 improves at opt-3 while IC instructions increase. Only the combined candidate
has instruction/timing/memory qualification; do not attribute those numbers to
either standalone footprint screen. Native macOS, installed application costs,
the full standard-corpus performance matrix and a new full 39-cohort run are
unqualified. Toko-shaped qualification here is Wasm-only.

Production retains the adopted i32 writer and existing unsigned/i64 paths.
The candidate's native suite passes 47 tests with one explicit timing test
ignored, including Postcard byte parity and exact resource boundaries. The
failure is a measured cost tradeoff, not a reproduced encoding defect.
There is no protocol or API change, migration, schema regeneration or new codec:
u32/u64 128 remains `80 01`, and i64 -64 remains `7f`. Existing identities,
limits, admission and retry/hash bytes remain intact. The next narrower candidate
could share a private u32 raw-byte helper while keeping the shortcut in the u32
wrapper, outside i32's path. That alternative is not implemented or measured here.

## Historical follow-up for 0.7.16

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Five isolated candidates were evaluated. None is promoted. The retained u16
decoder from the [preceding study](runtime.md) stays intact; this
batch changes no production codec, protocol bytes, acceptance rules or API.

## Source and measurement identity

Baseline is released 0.7.15 (`47f011702edd0fce99d4272fd80b02991ee5f94f`) plus
the frozen Retired raw output (see retention notice): the retained decoder/oracle and the
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

Retired raw output (see retention notice) include compiler, limits,
server and artifact hashes. PocketIC 16.0.0 server SHA-256 is
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
Every paired input/frame hash matches; Candid control counters are unchanged.
The baseline is reused only where artifact hashes match exactly. These are
codec qualification fixtures, not installed endpoint or fleet measurements.

## Uncompressed Wasm

Baseline total/code bytes are 377,817/337,789 (standard z), 573,226/518,516
(standard 3), 310,738/291,971 (100 types z), 398,739/364,245 (100 types 3).
Cells below give **total / code byte changes** from those absolute baselines.
Full absolute sizes and artifact hashes are in Retired raw output (see retention notice).

| Candidate | Standard z | Standard 3 | 100 types z | 100 types 3 |
|---|---:|---:|---:|---:|
| Retired raw output (see retention notice) | −36 / −36 | +3,494 / +3,467 | 0 / 0 | −2,509 / −2,505 |
| Retired raw output (see retention notice) | −30 / −28 | −423 / −440 | −451 / −449 | −299 / −295 |
| Retired raw output (see retention notice) | −12,684 / −12,626 | +5,123 / +5,118 | −8,655 / −8,614 | −9,393 / −9,388 |
| Retired raw output (see retention notice) | −12,684 / −12,626 | −7,457 / −7,443 | −8,655 / −8,614 | −16,657 / −16,646 |
| Retired raw output (see retention notice) | −12,626 / −12,568 | +569 / +583 | −8,597 / −8,556 | −13,645 / −13,640 |

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
Retired raw output (see retention notice) validates pairing/repeats and emits every Wire row, including
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
(Retired raw output (see retention notice)). The unmodified full run fails
`reusable_output_keeps_capacity_and_clears_all_failure_paths`: its
FailSecond serializer returns an error on the second call; research calls it
once and returns success. This expected failure is retained as a qualification
limit, not described as a passing production suite. Allocation failure injection
and complete custom-serializer/failure-path qualification were not performed.

The cold-growth prototype was also measured with the 87-case native allocation
suite (opt-3). This is one allocation-count observation per operation, not native
timing or peak-live-heap measurement. Retired raw output (see retention notice)
and Retired raw output (see retention notice) retain hashes and controls.

| Fresh encoding | Frame bytes | Output capacity before → after | Reallocations before → after |
|---|---:|---:|---:|
| Empty i16 list | 12 | 12 → 64 | 0 → 0 |
| Wide u16 list, 16,384 | 49,166 | 49,166 → 65,536 | 0 → 10 |
| Ordinary byte vector, 1 MiB | 1,048,590 | 1,048,590 → 2,097,152 | 0 → 15 |
| Byte buffer, 1 MiB | 1,048,590 | 1,048,590 → 1,048,590 | 0 → 1 |

All four fresh paths still make one initial allocation. Reuse with sufficient
capacity remains allocation-free in these controls. Geometric growth increases
reserved output memory and may recopy data; it is not a free removal of sizing.
