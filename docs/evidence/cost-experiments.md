# Focused admission cost investigations

## Budget predicate screen (8 November 2026)

**Keep the existing admission implementation.** Two alternatives preserve exact
charges, sticky refusal and no partial debit, but neither improves the measured
tradeoff. The baseline includes the retained u32 writer and Testkit 0.94.0.
Lock SHA-256:
`b36a8d0fdeee188f98c2d47a34e3502ba775ee8c850f1e9bbddd8bc205a39968`;
native admission source SHA-256:
`20c405500acf6a98ce1331a524e904e994af62a604861f7ae9ea998cc6a0676e`.
Rust 1.99.1 / LLVM 23.1.1, existing opt-z/opt-3 profiles, LTO/one codegen unit,
wasm32-unknown-unknown, no post-link optimizer; separate fresh Wasm targets.

- `predicates` loads node/heap allowances and combines the four pure refusal
  predicates using Boolean bitwise OR, replacing the initial sticky-refusal
  return and subsequent short-circuit comparisons. Debits still occur only on success.
- `subtract` computes both `overflowing_sub` results, combines their underflow
  flags with sticky/depth refusal, and commits the results only on success.

Whole-fixture uncompressed **total / code-section bytes**:

| Fixture/profile | Baseline | Combined predicates | Underflow flags |
| --- | ---: | ---: | ---: |
| Hotspots / opt-z | 83,384 / 72,441 | 83,438 / 72,495 | 83,442 / 72,499 |
| Hotspots / opt-3 | 104,128 / 93,834 | 104,156 / 93,862 | 104,158 / 93,864 |
| Standard / opt-z | 594,851 / 548,055 | 595,005 / 548,209 | 595,047 / 548,251 |
| Standard / opt-3 | 889,425 / 828,831 | 889,570 / 828,976 | 889,605 / 829,011 |

Each candidate has 1,904 matched IC rows over the 87-case hotspot corpus, with
three identical samples and distinct callers. Candid counters, frames and logical
inputs match; baked admission-source/lock hashes match the frozen sources.
Both pass all **860 Wire operation comparisons** within the 5% + 64 margin, but
both fail the no-growth Wasm screen. Passing the instruction margin is not proof
of a useful optimization. Combined predicates improve 20 rows, leave 663 equal
and worsen 177; underflow flags improve 20, leave 641 equal and worsen 199.

Selected **IC instructions** (large lists have 16,384 elements):

| Operation/profile | Baseline | Combined predicates | Underflow flags |
| --- | ---: | ---: | ---: |
| Wide-u32 encode / opt-z | 6,702,812 | 6,735,580 | 6,735,580 |
| Wide-u64 decode / opt-z | 14,096,374 | 14,112,758 | 14,112,758 |
| Empty-blob mixed decode / opt-z | 4,764.5 | 4,784.5 | 4,790.5 |
| Wide-u32 encode / opt-3 | 3,720,193 | 3,720,193 | 3,720,193 |
| Wide-u64 decode / opt-3 | 6,066,465 | 6,066,465 | 6,066,465 |
| Empty-blob mixed decode / opt-3 | 2,844.5 | 2,864.5 | 2,868.5 |

Retain `admission_refusal_is_sticky_and_never_partially_debits`, an independent
byte-denominated reference test for zero/exhausted allowances, reserved bytes,
maximum-sized charges, multiplication overflow, depth refusal and no partial
state update. All lanes pass five library and 48 native tests (one timing probe
ignored). The retained test and strict library/test Clippy pass in the normal
checkout. Native timing, allocation/heap, Toko/application measurements and a full
matrix were not run: the IC/footprint screen provides no reason to promote either.

Reproduction: `target/evidence/budget-predicates-20261108/`, with `prepare.mjs`,
`screen.sh`, `prepare-ic.sh`, `cost-screen.sh`, `analyze.mjs` and frozen inputs.
One analysis invocation preceded the last identity file and failed; only the
completed rerun supplies the results above. Native dependency caches were seeded,
product inputs rebuilt, and exact baseline Candid modules reused. Production
admission source and lock remain equal to baseline. Bytes, identities, budgets
and acceptance are unchanged; no consumer migration or retry/hash change.
Only the strengthened test is kept.

## Historical qualification for Core 0.13.4

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Measured 2026-11-01 against published Core 0.13.3 runtime source, with the same
new private 20-case harness on both sides. Keep the simpler `enter_many`
allowance/debit cleanup; reject the other four candidates. This is a small
footprint improvement with native timing tradeoffs, not a fix for every Candid
loss. Shared Tooling 0.27.24 was adopted canonically in the same working batch.

## Uncompressed Wasm

All values are bytes from the final raw module. Code bytes are the section-10
payload length, independently parsed from that same module; section headers
and other sections remain in total bytes. These complete fixtures retain both
encoding and decoding. They do not establish application linker savings or
remaining application code-limit headroom.

| Wire fixture / profile | Total before → after | Total delta | Code before → after | Code delta |
| --- | ---: | ---: | ---: | ---: |
| 20 controls, opt-z | 94,219 → 94,193 | −26 (−0.0276%) | 81,768 → 81,742 | −26 (−0.0318%) |
| 20 controls, opt-3 | 121,110 → 121,096 | −14 (−0.0116%) | 109,667 → 109,653 | −14 (−0.0128%) |
| 100 derived types, opt-z | 310,706 → 310,637 | −69 (−0.0222%) | 291,971 → 291,902 | −69 (−0.0236%) |
| 100 derived types, opt-3 | 398,675 → 398,548 | −127 (−0.0319%) | 364,245 → 364,120 | −125 (−0.0343%) |

The matched Candid smoke modules remain identical: opt-z **402,762 total /
367,085 code** and opt-3 **624,858 total / 590,483 code**. The 100-type follow-up
measures Wire before/after only; it is not another complete Candid matrix.
The [earlier full matrix](../benchmarks/candid-comparison.md) retains its original
source/dependency graph and cannot substitute for this matched baseline.

## IC instructions and bytes

All 192 paired Wire operations pass the explicit `baseline × 1.05 + 64`
instruction screen. The largest increase is the 128-entry byte-key-map opt-3
encode: **62,120.25 → 62,376.25**, +256 instructions (**+0.4121%**).
Optional-unit-list opt-3 decode improves **256,857.5 → 240,473.5** (−6.3786%);
truncated-input refusal improves **256,820.75 → 240,436.75** (−6.3795%).
All paired frames, input hashes and Candid counters remain identical. All
three fresh-caller IC samples agree per row. Loop overhead and result destruction
are included; loop controls are retained rather than subtracted. Setup and reply
transport are excluded. No endpoint cycle measurement is claimed here.

Small controls and the remaining large-unit gap, instructions per operation:

| Input / profile / operation | Wire before → after | Change | Candid control |
| --- | ---: | ---: | ---: |
| u64 0, opt-z encode | 1,214.5 → 1,212.5 | −0.1647% | 10,916.75 |
| Empty blob, opt-z encode | 1,666.5 → 1,664.5 | −0.1200% | 22,273.5 |
| Empty unit list, opt-3 encode | 756.25 → 756.25 | 0% | 13,395.25 |
| Singleton unit list, opt-3 decode | 258.5 → 258.5 | 0% | 18,756 |
| 4,096 units, opt-z encode | 181,764.5 → 181,764.5 | 0% | 19,167.75 |
| 4,096 units, opt-z decode | 66,163.5 → 66,163.5 | 0% | 1,535,925.5 |
| 4,096 units, opt-3 encode | 778.25 → 778.25 | 0% | 13,129.5 |
| 4,096 units, opt-3 decode | 295.5 → 295.5 | 0% | 1,066,776 |

The opt-z unit encode remains about 9.48× Candid. Serde's default `collect_seq`
visits references; admission work survives this compiler setting even though
unit values emit no element bytes. Opt-3 can fold that loop. Hints on unit
serialization alone do not remove it. Automatically skipping Serde visits could
change admission or custom serialization semantics, so no unit-specific codec,
representation or unchecked path is added. This remains a separate research
target, with ordinary controls visible.

## Native timing and allocations

Native medians in nanoseconds, two serial runs of eleven calibrated batch
samples (minimum 2 ms per batch). The shared host is not CPU-isolated; small
changes and inconsistent directions are not established wins. Result destruction
is included. These are native timings, separate from IC instructions.

| opt-3 path | Run 1 before → after | Change | Run 2 before → after | Change |
| --- | ---: | ---: | ---: | ---: |
| Empty unit list encode | 39.504 → 34.685 | −12.20% | 39.368 → 38.857 | −1.30% |
| Singleton unit list encode | 39.628 → 36.941 | −6.78% | 38.997 → 34.977 | −10.31% |
| Singleton unit list decode | 10.875 → 9.836 | −9.56% | 11.045 → 10.053 | −8.98% |
| 4,096 units encode | 1,791.995 → 259.266 | −85.53% | 1,838.643 → 260.855 | −85.81% |
| 4,096 units decode | 664.783 → 869.400 | +30.78% | 674.460 → 868.843 | +28.82% |
| 4,096 empty records decode | 668.932 → 816.806 | +22.11% | 657.010 → 869.539 | +32.35% |

Keep the decode regression visible: approximately 0.2 µs per operation. The
largest native increase is 32.35%, on empty-record decoding. Opt-z unit-list
encode-reuse varies **4,325.869 → 5,219.422 ns (+20.66%)** in run 1 and
**5,188.691 → 4,327.082 ns (−16.61%)** in run 2; do not select a preferred run.
All allocation and peak-live-heap observations are identical before/after for
both profiles and every recorded operation. Those probes are separate warmed,
single-operation instrumented runs, not eleven allocator samples.

The selected cleanup is justified by simpler control flow and smaller raw/code
modules at both profiles, with modest IC movement. It is not a universal native
speed improvement. Retired raw output (see retention notice) retain regressions,
Candid controls and allocator/heap rows.

## Candidates and rejected work

Each patch is independent against the retained baseline. The corrected screen
measures three stable IC samples per operation and retains all physical modules.

| Candidate | opt-z total / code delta | opt-3 total / code delta | Decision |
| --- | ---: | ---: | --- |
| Outline `serialize_bytes` on Wasm | 0 / 0 | +15 / +13 | Reject: byte-key-map encode 62,120.25 → 124,522.25 at opt-3 |
| Inline unit/unit-struct serializers on Wasm | 0 / 0 | 0 / 0 | Reject: identical artifacts and counters |
| Skip zero-size heap charges | 0 / 0 | 0 / 0 | Reject: no size win, tiny instruction changes |
| Combine all budget checks, change check/commit ordering | −503 / −496 | +6 / +6 | Reject: opt-z unit decode 66,163.5 → 74,356.5 (+12.38%) |
| Direct checks retaining refusal/debit ordering | −26 / −26 | −14 / −14 | Keep, then repeat full native/IC smoke and 100-type size qualification |

The selected helper checks existing node/heap allowances before subtraction,
then commits the same heap and node debits. Sticky refusal, depth/overflow
rejection, allocation accounting, malformed acceptance, Postcard bytes and exact
retry/hash inputs are unchanged. Existing contracts require no migration. The
working source adds an explanatory comment to the exact measured patch; no
additional runtime tokens differ. No profiles, dependencies or own versions
change, and one production Serde/Postcard codec remains.

An earlier copied-workspace screen reused newer cached artifacts because copied
source retained older mtimes. It is **invalid** and contributes no result above.
Retired raw output (see retention notice) record that failure. The
corrected runs use one fixed worktree, copy changed files immediately before
building, and require observed library/guest compilation. The final paired gate
also refuses a cached candidate. Retired raw output (see retention notice) covers
both sides, both profiles and both fixtures.

## Shared budget outlining — rejected, 11 November 2026

Do not promote the isolated change from Wasm inline(always) to inline(never) on
Budget::enter_many. It preserves the source-level checks and two-pass encoding,
but the observed instruction/size tradeoff does not justify replacing production.
The normal checkout's codec/admission source is unchanged.

Both frozen sources use Rust 1.99.1, identical locks and the maintained 154-case
coverage-shapes suite. Separate targets retain opt-z/opt-3 LTO modules, two native
runs/profile with 11 batches at a 2 ms minimum, allocation/heap probes, and three
identical IC samples per operation. The existing explicitly prepared PocketIC 16
server supplies execution. Native code is unchanged by this Wasm-only attribute;
its timings are controls, not evidence of a native optimization.

| Wire fixture | Before total / code bytes | Candidate total / code bytes |
| --- | ---: | ---: |
| opt-z | 263,829 / 228,715 | 261,817 / 226,700 |
| opt-3 | 348,674 / 315,897 | 351,256 / 318,334 |

Observed IC instructions/op (not a passing paired-cost qualification):

| Profile / 4,096-entry operation | Before | Candidate |
| --- | ---: | ---: |
| opt-z tuple encode | 13,956,856 | 15,046,523 |
| opt-z tuple decode | 13,091,998 | 14,607,571 |
| opt-z page encode | 10,429,732 | 11,051,219 |
| opt-z mixed enum decode | 4,199,621.5 | 4,874,105.5 |
| opt-3 tuple encode | 5,333,976 | 8,279,131 |
| opt-3 tuple decode | 3,870,359 | 6,962,898 |
| opt-3 page encode | 5,126,831 | 5,626,973 |

Qualification limits are explicit: the strict cost checker stops on cached-build
records because the exact sources/targets were first compiled for the size
screen. No input changed between those builds, but this is not a passing fresh-
build gate. Candid modules also differ (opt-z +11 total/+10 code bytes, opt-3
same sizes but changed hash), and 40 Candid counter rows differ. Do not label
this a controlled oracle/no-growth pass or use it as a published speedup claim.
Independent verification checks recorded admission hashes against source, all
IC artifact hashes, 6,404 repeated-sample rows, 10,280 matched native input/frame
rows and 5,140 identical allocation/heap observations. Simple threshold arithmetic
finds 609 Wire rows above baseline × 1.05 + 64; it does not override the failed
provenance/control gate. The evidence warrants rejecting this candidate, not
spending more qualification effort to ship a 2 KB opt-z saving with these losses.

Evidence and executable verification: target/evidence/admission-outline-20261111/,
including before/after/source, results, analyze.mjs, verification.json and
cost-screen.log. The source delta is solely the named attribute. Reproduction
uses each frozen source's evaluate-codecs.sh with CORPUS_FEATURE=coverage-shapes,
CODEC_SOURCE_SNAPSHOT pointing to that source, a separate CARGO_TARGET_DIR per
lane and an explicitly prepared POCKET_IC_BIN. Fresh qualification would require
fresh build records and matching Candid controls. Nothing from this runtime
prototype is applied to production; bytes, limits and identities stay unchanged.


## Bulk-write inlining — rejected, 11 November 2026

Do not add Wasm inline(always) or inline to contract::Output::try_extend.
This shared bounded buffer-write method is used by Postcard buffers and wider
integer writers. Both attributes reduce the linked coverage-shapes fixture but
worsen important encoding operations. Both candidates remain only in isolated
sources; normal production source was never edited.

Whole uncompressed Wasm, total / code-section bytes:

| Profile | Baseline | Forced inline | Inline hint |
| --- | ---: | ---: | ---: |
| opt-z | 263,824 / 228,710 | 263,633 / 228,516 | 263,596 / 228,483 |
| opt-3 | 348,674 / 315,897 | 343,830 / 311,058 | 343,830 / 311,058 |

Selected IC encode instructions/op:

| Profile / case | Baseline | Forced inline | Inline hint |
| --- | ---: | ---: | ---: |
| opt-z / 128 Accounts (principal length 1) | 248,489.5 | 262,858.5 | 270,548.5 |
| opt-z / 4,096 packed records | 10,226,206 | 10,788,215 | 11,101,471 |
| opt-z / 4,096 mixed-width tuples | 13,874,950 | 14,087,975 | 14,227,249 |
| opt-3 / 128 Accounts (principal length 1) | 72,166.25 | 90,211.25 | 90,211.25 |
| opt-3 / 4,096 packed records | 5,126,831 | 4,762,386 | 4,762,386 |
| opt-3 / 4,096 mixed-width tuples | 5,333,976 | 5,333,976 | 5,333,976 |

The strict cost checker reaches numeric comparison: source/lock/fixture identities,
fresh candidate library/guest builds, frame/input hashes, artifact hashes and
Candid controls pass. It **fails** 61 forced-inline pairs (22 opt-z, 39 opt-3) and
74 hinted pairs (35 opt-z, 39 opt-3) out of 1,338 Wire operation pairs each, using
the unchanged baseline × 1.05 + 64 instruction screen. No size growth occurs.
The opt-3 page improvement does not justify ~25% Account encode regression or
the opt-z page losses of 5.5% / 8.6%. These are matched local comparisons, not
regression claims against a different earlier matrix artifact.

Rust 1.99.1, identical qualification locks (Types 0.19.1, Candid 0.10.40),
wasm32-unknown-unknown, maintained evaluation/benchmark-speed profiles, 154
coverage-shapes cases and three identical samples per IC operation on prepared
PocketIC 16. Each lane has a separate source and Cargo target. The same retained
Candid Wasm from the verified Core 0.21 matrix is used on every lane and is
remeasured; its hashes and counters agree. Wire artifacts are freshly compiled
from each lane. No alternative comparator, missing case or relaxed threshold.
Native borrowed/native/runtime tests pass (6/87/16, four ignored probes). The
attributes are Wasm-only; native timing and allocation studies were not run and
no native improvement is claimed. Toko and hundred-type size expansion was not
pursued after the instruction screen rejected both candidates. Bytes, identities,
limits and acceptance stay unchanged; no consumer action or release is needed.

Evidence: `target/evidence/extend-inline-20261111/` with before/after/hint sources,
`screen.sh`, `instructions.sh`, `hint-instructions.sh`, raw builds/Wasms/IC rows,
`sizes.csv`, both cost-screen logs and `analyze.mjs`. Reproduce size builds with
`cargo build --locked --offline --manifest-path SOURCE/qualification/Cargo.toml
-p ic-wire-benchmarks --example codec-wasm --features wire,coverage-shapes
--target wasm32-unknown-unknown --profile evaluation` (then benchmark-speed),
using a fresh target per lane. The retained drivers explicitly select the
prepared server; `analyze.mjs after` / `analyze.mjs hint` validate source hashes
and fresh builds before reporting failed thresholds. No cleanup is authorized.

## Shared framed decode admission — not promoted, 11 November 2026

An isolated candidate splits native::decode into its public payload-length check
and a private decode_payload body. decode_frame already admits the whole frame,
validates its header and subtracts the header from the byte allowance; it can
therefore call that body without repeating the implied payload-length check.
All allocation/node/depth admission and Postcard validation remain in the body.
Bytes, identities, limits, acceptance and error precedence are unchanged.

| Profile | Baseline total / code bytes | Candidate total / code bytes |
| --- | ---: | ---: |
| opt-z | 263,829 / 228,715 | 263,491 / 228,377 |
| opt-3 | 348,674 / 315,897 | 348,983 / 316,205 |

Of 669 Wire operation pairs per profile, opt-z changes 324 by only −4 to −6 IC
instructions; the rest are identical. Opt-3 changes 335 by −14 to +24 instructions.
For example, empty Principal decode is 625.5 → 621.5 instructions at opt-z and
459.5 → 455.5 at opt-3. Encode costs are unchanged in the selected Account,
4,096-record page and 4,096-tuple controls. All 1,338 instruction comparisons pass
the unchanged baseline × 1.05 + 64 screen. The overall strict checker **fails**
on opt-3 total/code growth (+309/+308 bytes); no threshold was relaxed.

This is a small, credible opt-z fixture saving, not a substantial runtime win.
It is not promoted: the secondary-profile growth and unmeasured application
footprint do not yet justify changing production decoding. Toko/hundred-type
sizes and native timing/allocation effects were not measured. A later decision
would need those proofs; do not describe this fixture result as application savings.

Qualification: Rust 1.99.1, wasm32-unknown-unknown, evaluation/benchmark-speed
profiles, identical qualification locks (Types 0.19.1, Candid 0.10.40), 154 cases,
three identical IC samples per operation on PocketIC 16. Separate paired sources
and targets produced fresh library/guest records. Source, admission, contract,
lock and artifact hashes are checked; exact frame/input and matching retained
Candid artifacts/counters pass. Candidate borrowed/native/runtime tests pass
6/87/16 active tests, with four ignored probes, including the existing frame-budget
boundary and error-precedence test. Production runtime source remains untouched.

Evidence: target/evidence/frame-admission-20261111 contains before/after sources,
screen.sh, instructions.sh, sizes.csv, analyze.mjs, verification.json and the
strict cost-screen.log. The scripts reproduce the fresh size/IC comparison with
separate targets and the prepared PocketIC binary; analyze.mjs checks baked source
hashes and fresh build records. Preserve these artifacts; no release is implied.

## Fused nonzero packed decoding — rejected, 11 November 2026

The current nonzero_i64 adapter decodes Packed<i64> and validates NonZeroI64
through an owned iterator collection. An isolated candidate instead uses a byte
visitor, validates whole 8-byte lengths, reserves once, then reads/checks/pushes
each word. It avoids the second source-level traversal but loses important opt-3
optimization. Keep the current implementation; do not promote the explicit loop.

| Profile / elements | Baseline decode IC instructions | Candidate |
| --- | ---: | ---: |
| opt-z / 0 | 766.5 | 575.5 |
| opt-z / 1 | 1,197.5 | 1,004.5 |
| opt-z / 128 | 7,260.5 | 6,813.5 |
| opt-z / 4,096 | 181,495.5 | 173,112.5 |
| opt-3 / 0 | 355.5 | 328.5 |
| opt-3 / 1 | 723.5 | 613.5 |
| opt-3 / 128 | 4,064.5 | 6,621.5 |
| opt-3 / 4,096 | 94,928.5 | 180,813.5 |

Whole uncompressed Wasm total/code bytes: opt-z 262,828/227,715 →
262,947/227,834 (+119/+119); opt-3 347,219/314,441 → 347,244/314,467
(+25/+26). The large opt-z improvement is 4.6%, but opt-3 regresses 90.5%.
The unchanged strict checker reaches numeric comparison and fails two instruction
pairs plus four total/code size checks. All 1,338 operation pairs are compared;
no screen threshold is relaxed. These are fixture results, not application savings.

Matched Rust 1.99.1, wasm32-unknown-unknown, evaluation/benchmark-speed profiles,
identical qualification locks (Types 0.19.1, Candid 0.10.40), 154 borrowed-containers
cases, three identical IC samples per operation on PocketIC 16. Each source has
its own fresh Cargo target. Baked adapter/admission/contract/lock hashes, fresh
library/guest records, exact frames/inputs and matching retained Candid controls
are verified. The initial coverage-shapes build was discarded from the comparison:
it selects ordinary nonzero vectors, not this adapter. Its artifacts remain in
unused-coverage directories; only borrowed-containers results support this finding.

Candidate borrowed/native/runtime checks pass 6/87/16 active tests, four ignored,
including adapter bytes/budgets, early/late zero rejection, truncation, trailing
bytes and subsequent valid decoding. Accepted values, bytes, identities and
admission limits remain unchanged. No native latency/allocation study or expanded
application size study was run after the IC rejection. Production code is untouched.

Evidence: target/evidence/nonzero-decode-20261111 contains paired sources,
screen.sh, instructions.sh, analyze.mjs, sizes.csv, verification.json and
cost-screen.log. The scripts select borrowed-containers, fresh separate targets,
and the prepared PocketIC server. The analysis independently verifies the changed
public_nonzero_adapter_sha256. No release or consumer action is needed.

## Block validation for packed booleans — rejected, 11 November 2026

The bool adapter currently uses bytemuck's checked byte-to-bool slice cast,
then copies into owned storage. An isolated candidate validates eight bytes per
u64 with mask 0xfefe_fefe_fefe_fefe, checks the remainder, then safely constructs
the output with bytes.iter().map(|byte| *byte != 0). No unchecked conversion is
introduced. Validation still precedes allocation; bytes, identities, budgets
and malformed-input acceptance remain unchanged.

| Profile / elements | Baseline decode IC instructions | Candidate |
| --- | ---: | ---: |
| opt-z / 0 | 597.5 | 612.5 |
| opt-z / 1 | 1,111.5 | 1,041.5 |
| opt-z / 128 | 3,720.5 | 4,220.5 |
| opt-z / 16,384 | 329,154.5 | 404,838.5 |
| opt-z / 1,048,576 | 20,973,155 | 25,822,727 |
| opt-3 / 0 | 316.5 | 334.5 |
| opt-3 / 1 | 655.5 | 592.5 |
| opt-3 / 128 | 3,215.5 | 2,139.5 |
| opt-3 / 16,384 | 328,594.5 | 199,502.5 |
| opt-3 / 1,048,576 | 20,972,595 | 12,714,991 |

Whole uncompressed Wasm total/code bytes: opt-z 262,914/227,800 →
263,002/227,888 (+88/+88); opt-3 346,927/314,153 → 347,251/314,477
(+324/+324). The large opt-3 gain is 39.4%, but primary opt-z loses 23.1%.
Reject this candidate. The unchanged strict checker compares 1,338 operation
pairs and fails three opt-z decode thresholds plus all four total/code size
checks. Source brevity or fewer validation iterations does not establish a win
for the complete validate-and-copy operation.

Matched Rust 1.99.1, wasm32-unknown-unknown, evaluation/benchmark-speed profiles,
identical qualification locks (Types 0.19.1, Candid 0.10.40), 154 research-bulk
cases and three identical IC samples per operation on PocketIC 16. Separate
paired source/target directories, fresh library/guest builds, baked bool-adapter,
admission/contract/lock identities, exact frames/inputs and matching retained
Candid artifacts/counters are verified. No threshold is relaxed.

Candidate borrowed/native/runtime checks pass 6/87/16 active tests, four ignored.
An additional isolated test checks all invalid byte values 2..=255 at every
position in buffers of lengths 0..=33, plus valid alternating values; it passes.
This covers all eight-byte lanes, block boundaries and remainders. No native
latency/allocation or application-size study followed the primary-profile
rejection. No production runtime changes, release or consumer action follow.

Evidence: target/evidence/bool-blocks-20261111 contains before/after sources,
screen.sh, instructions.sh, analyze.mjs, sizes.csv, verification.json and
cost-screen.log. Scripts select research-bulk and separate fresh targets, with
the prepared PocketIC server. The analyzer independently checks the changed
public_bools_adapter_sha256. Keep the current safe checked-copy implementation.

## Packed finite-f64 exponent masks — rejected, 12 November 2026

Tested eight exponent-mask additions followed by one sign-bit test, replacing
individual `is_finite` calls in the existing shared validator. This mirrors the
retained f32 idea but does not improve f64. Validation still precedes allocation;
exact float bits, finite-only acceptance, limits and framing are unchanged.
No production Rust change is retained.

| Profile / 131,072 f64 values | Before instructions | Candidate | Change |
| --- | ---: | ---: | ---: |
| opt-z encode | 4,130,880 | 4,360,289 | +5.55% |
| opt-z decode | 5,080,618 | 5,195,306 | +2.26% |
| opt-3 encode | 4,097,092 | 4,326,470 | +5.60% |
| opt-3 decode | 2,966,558 | 3,081,247 | +3.87% |

Total/code-section uncompressed Wasm: opt-z 262,938/227,824 →
263,014/227,899 (+76/+75); opt-3 346,927/314,153 → 347,031/314,257
(+104/+104). Empty/singleton encode also regresses slightly. The unchanged cost
screen checks 1,338 operation pairs and reports eight failures: four size checks
and two encode thresholds in each profile. No threshold was relaxed.

Matched Rust 1.99.1, wasm32-unknown-unknown, evaluation/benchmark-speed profiles,
154 research-bulk cases, identical frozen locks and separate fresh Cargo targets.
The current qualification lock includes Testkit 0.101.0 and Build 0.83.0.
Three identical IC samples per operation, exact frames and inputs, source hashes
(including public_finite_adapter_sha256), fresh library/guest artifacts and
unchanged retained Candid controls are checked by the existing paired verifier.
Candid guests are reused controls from the prior matrix, not newly built supplier
qualification. The prepared PocketIC 16.0.0 server is used for both lanes.
Five finite-related native tests pass, including a new isolated reference test
covering all 2,048 exponents in each of 17 positions (two blocks and remainder).
No native latency or application Wasm saving is claimed.

Reproduction/evidence: `target/evidence/lengths-f64-20261112/` contains paired
sources, screen.sh, instructions.sh, analyze.mjs, sizes.csv, per-lane build logs,
raw instruction samples and after-verification.json. Run the two shell scripts
from the repository root, followed by `node` on analyze.mjs; use fresh source and
target/output locations for another measurement. Initial instruction startup
stopped on a missing cached errno package; locked fetch prepared it before any
measurements. The completed retry retains identical locks.


## Derive adapter prefix cleanup — 13 November 2026

Keep the host-only `LitStr::value().trim_prefix("::")` replacement. The former
`strip_prefix(...).unwrap_or(path.value().as_str())` eagerly decoded the same
literal twice. Supported adapter names, single optional root-prefix acceptance,
generated tokens, codec bytes, identities and limits remain unchanged. No consumer
migration is required; repeated `::::` prefixes remain rejected.

The study calls the actual copied `native_schema::expand` plus the unchanged
`fresh_ident` helper, with pre-parsed one-field and 64-field structs. Adapter fields
use `::serde_bytes`; the control is one ordinary `Vec<u8>` field. Rust 1.100.0
(`0ef1bc386`), x86_64-unknown-linux-gnu, identical locked syn 3.0.10,
proc-macro2 1.0.107, quote 1.0.47 and stats_alloc 0.1.10; LTO and one codegen unit.
Each lane has a separate source/target directory. Fresh binaries embed the schema
source digest; the runner verifies those identities and exact generated tokens.
Eleven alternating paired process runs each time 50 expansions per case.

| Profile / case | Allocations before → after | Allocated bytes before → after | Instrumented median ns before → after |
| --- | --- | --- | --- |
| opt-z / control | 168 → 168 | 15,771 → 15,771 | 14,571 → 14,443 |
| opt-z / 1 adapter | 188 → 186 | 16,827 → 16,796 | 15,548 → 14,809 |
| opt-z / 64 adapters | 3,346 → 3,218 | 247,749 → 245,765 | 247,010 → 235,644 |
| opt-3 / control | 167 → 167 | 15,747 → 15,747 | 8,256 → 8,308 |
| opt-3 / 1 adapter | 187 → 185 | 16,803 → 16,772 | 8,560 → 8,200 |
| opt-3 / 64 adapters | 3,345 → 3,217 | 247,725 → 245,741 | 149,774 → 144,920 |

The firm saving is two allocations and 31 allocated bytes per adapter in these
inputs. Timing includes allocator instrumentation and overlaps between lanes:
opt-z one-adapter ranges are 15,215–20,144 / 14,579–32,705 ns; opt-3 control median
regresses about 0.6%. Do not claim a reliable wall-clock build-speed improvement.
This is host expansion, not a complete downstream build; IC instructions and
linked application Wasm savings are not applicable claims. No allocator, codec
or per-type Wasm specialization was introduced.

Raw paired source, locks, source/binary identities, generated tokens, all samples
and summary remain under `target/evidence/derive-prefix-1100/`. Reproduce by
building each lane with `cargo build --locked --offline --manifest-path
<lane>/Cargo.toml --profile release` and `--profile speed`, then running
`python3 target/evidence/derive-prefix-1100/measure.py`. Preserve separate targets.
