# Runtime optimization qualification

## Decoder tail screen (8 November 2026)

**Neither candidate is adopted.** This study starts from the current unrolled
u32 writer and Build 0.80 lock. Baseline parser SHA-256:
`67240609997fde1ccf2fe54544d77387015f1b67b812aa94000aeee28b3b84a8`;
lock SHA-256:
`385b5344224449a182554018443beffd23b98164756c42a0c4acf3eb4257c534`.
Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown, existing opt-z/opt-3 profiles,
LTO/one codegen unit and no post-link optimizer. Fresh separate Wasm targets
and rebuilt native drivers retain exact parser/lock identities.

- `tail` changes the existing u32/u64 reader macro: loop over intermediate
  groups only, then read the last byte separately and require
  `1..=max_of_last_byte::<T>()` before shifting it into the result.
- `unrolled32` replaces only the u32 macro invocation with five explicit reads.
  The first byte retains its early return; groups at shifts 7, 14 and 21 reject
  a terminating zero. The fifth byte must be in `1..=15`. u64 stays unchanged.

Whole-fixture uncompressed **total / code-section bytes**:

| Fixture/profile | Baseline | Split tail | Unrolled u32 |
| --- | ---: | ---: | ---: |
| Hotspots / opt-z | 83,384 / 72,441 | 83,560 / 72,612 | 83,665 / 72,713 |
| Hotspots / opt-3 | 104,128 / 93,834 | 104,218 / 93,924 | 104,391 / 94,097 |
| Standard / opt-z | 594,851 / 548,055 | 595,023 / 548,226 | 595,132 / 548,327 |
| Standard / opt-3 | 889,425 / 828,831 | 889,323 / 828,730 | 890,095 / 829,500 |

**Owned-decode IC instructions**, lists with 16,384 values:

| Case/profile | Baseline | Split tail | Unrolled u32 |
| --- | ---: | ---: | ---: |
| Wide u32 / opt-z | 7,083,521 | 7,493,113 (+5.8%) | 6,510,046 (−8.1%) |
| Wide u32 / opt-3 | 2,724,229 | 2,527,618 (−7.2%) | 2,560,384 (−6.0%) |
| Wide u64 / opt-z | 14,096,374 | 14,686,190 (+4.2%) | 14,096,339 |
| Wide u64 / opt-3 | 6,066,465 | 5,722,394 (−5.7%) | 6,066,457 |
| Wide u16 control / opt-z | 4,069,359.5 | 4,593,639.5 (+12.9%) | 4,593,612.5 (+12.9%) |

The independent cost screen rejects 18 and ten of 860 Wire operation pairs
respectively, including decode and truncated-input rejection. Each lane has
1,904 IC rows across 87 cases, with three identical samples and unique query
callers. Inputs, frames and Candid counters match. Changes outside the directly
edited integer method show why whole-module controls matter; these results do
not prove a particular compiler inlining or code-layout cause. Both candidates
also grow primary-profile Wasm. Native timing, memory profiling, application
measurements and a full comparison matrix were not run after rejection.

**Retained test coverage:** `wide_integer_endings_match_independent_postcard_oracle`
checks u32/i32/u64/i64 with all possible final bytes after zero/all-one
continuation groups, at lengths zero through eleven. Postcard supplies decoded
value and consumed prefix; its encoder independently determines shortest-form
acceptance. Tests distinguish malformed input from valid values with trailing
bytes and check that a following tuple field remains untouched. Baseline and
both candidates pass 48 native tests (one timing probe ignored). The retained
test and strict native-test Clippy pass in the normal checkout.

Raw evidence/reproduction: `target/evidence/decoder-tail-20261108/`, including
`prepare.mjs`, `screen.sh`, `prepare-ic.sh`, `prepare-unrolled32-ic.sh`,
`cost-screen.sh`, `analyze.mjs` and source snapshots. The first driver-setup
attempt reached unrolled32 before its screen artifact existed; only that lane's
setup was rerun after the artifacts were ready. Completed runs and source hashes,
not that incomplete setup, supply the results above. Production parser and lock
remain byte-identical to baseline. Bytes, identities, budgets and acceptance
stay unchanged: `80 00` remains malformed; u32 `ff ff ff ff 0f` remains valid.
No consumer migration or retry/hash change. Only the test is retained.

## Historical qualification for 0.7.16

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Baseline: released 0.7.15, `47f011702edd0fce99d4272fd80b02991ee5f94f`.
The retained Retired raw output (see retention notice) replaces the u16 tail loop with
explicit second/third byte-group checks. Signed i16 uses that parser after
zigzag conversion. One-byte values retain their early return. Encoding, shortest
representations, overflow rejection, typed errors and admission stay unchanged:
`ff ff 03` still means u16 65535; `80 00` and `ff ff 04` stay malformed.
No consumer migration is required. There is one production Serde/Postcard codec.

All runs use the frozen baseline lock: Postcard 1.1.3, Candid 0.10.39, Platform
0.6.16 and Build/Testkit/host 0.68.5. A concurrent checkout update to Platform
0.6.17/tooling 0.68.6 is preserved and excluded. Retired raw output (see retention notice) and
Retired raw output (see retention notice) record Rust 1.99.1 / x86_64 Linux. Wasm targets
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

Retired raw output (see retention notice) and
Retired raw output (see retention notice) retain every result.
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
artifacts are byte-identical. Retired raw output (see retention notice) retain
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

Retired raw output (see retention notice) include both run identities and pooled
summaries; Retired raw output (see retention notice) include improvements and
regressions. The Retired raw output (see retention notice) checks input/frame hashes, matched
settings and counts. Native benchmark Git observations describe the invocation
checkout; measured source is the frozen revision plus decoder patch and is bound
by each executable hash. No new allocation-count/heap savings are claimed.

## Rejected candidates

Each patch was measured separately against the same baseline. Table cells are
**total Wasm byte deltas**; corresponding code-section sizes/hashes are retained.

| Candidate | Standard z / 3 | 100 types z / 3 | Decision |
|---|---:|---:|---|
| Retired raw output (see retention notice) | +157 / +786 | −57 / +457 | Predominantly grows Wasm; size-only rejection. |
| Retired raw output (see retention notice) | −2,701 / +4,720 | −2,369 / +2,830 | Grows opt-3; size-only rejection. |
| Retired raw output (see retention notice) | −395 / −921 | 0 / 0 | Wide-u16 encoding regresses 51.3% IC instructions. |
| Retired raw output (see retention notice) | −66 / +2,372 | −451 / −2,796 | Mixed footprint; no instruction qualification or promotion. |

The writer's opt-3 wide-u16 fresh encode rises 1,852,251.25 → 2,802,520.25;
small-u16-list encode rises 1,246,043.25 → 1,557,336.25 (+24.98%). Scalar u16 zero
rises 647.25 → 673.25 (+4.02%). Retired raw output (see retention notice)
and Retired raw output (see retention notice) retain those regressions and
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

Historical reproduction used retired inputs; see the [retention policy](README.md).
