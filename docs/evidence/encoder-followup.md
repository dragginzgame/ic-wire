# Encoder experiments

## U32 length thresholds (9 November 2026)

Do not adopt the threshold-count rewrite of `raw_u32`'s exact tuple length.
For values already known to be at least 128, replace the leading-zero/division
formula with `2 + (value >= 2^14) + (value >= 2^21) + (value >= 2^28)`.
This preserves the exact tuple length, bytes and admission; the signed-i32
writer remains untouched. Production encoding is unchanged.

Matched baseline/candidate use Core 0.18.0 (`df071a4`) plus the pending
Build 0.81/Testkit 0.95 adoption and preserved dependency updates. Lock SHA-256:
`39145cb64911b59ae8c1026d74bc32298912b550dd73496072eb4c61c4697389`.
Rust/Cargo 1.99.1, LLVM 23.1.1, wasm32-unknown-unknown, fat LTO, one codegen
unit, panic abort, stripped symbols; `evaluation` is opt-z and
`benchmark-speed` is opt-3. Each lane has its own source and Cargo target.
Fresh Wasm builds and rebuilt instruction drivers bind the lock/encoder hashes.

Uncompressed Wasm, **total / code-section bytes**:

| Fixture/profile | Baseline | Thresholds | Delta |
| --- | ---: | ---: | ---: |
| hotspots / opt-z | 82,973 / 72,062 | 82,966 / 72,055 | -7 / -7 |
| hotspots / opt-3 | 104,101 / 93,943 | 104,101 / 93,943 | 0 / 0 |
| standard / opt-z | 594,278 / 547,562 | 594,308 / 547,592 | +30 / +30 |
| standard / opt-3 | 893,678 / 833,247 | 893,666 / 833,235 | -12 / -12 |

PocketIC 16.1.0 runs 87 hotspot cases, three identical samples per operation,
1,904 rows per lane. Payload/input hashes, frame sizes and Candid counters
match; baked lock/encoder identities match the sources. All 860 Wire operation
pairs pass the `baseline * 1.05 + 64` screen, and the hotspot-only size screen
passes. Wide-u32 list encode (16,384 elements) drops from **6,702,812 to
6,637,276 IC instructions (-0.98%)** at opt-z; opt-3 stays **3,720,193**.
Wide-u32 decode stays **7,083,521 / 2,724,229** at opt-z/3. The selected
wide-u16/i32/u64/i64 and empty mixed-blob encode/decode controls are unchanged.

Both lanes pass five library and 48 native tests, with the existing timing
probe ignored. This includes exact integer boundary bytes/budgets and malformed
wide-integer coverage. Reject the candidate: the tiny targeted instruction gain
does not justify the broader primary-profile size increase. Native timing,
allocation/heap, Toko-shaped Wasm and the full comparison matrix were not run
after this decision; no application saving is claimed. Codec bytes, identities,
budgets and acceptance remain unchanged; no consumer action is required.

Raw inputs/results and reproduction scripts:
`target/evidence/u32-length-20261109/` (`prepare.mjs`, `screen.sh`,
`prepare-ic.sh`, `analyze.mjs`, `cost-screen.sh`, `sizes.csv`). Run each lane's
`build/debug/examples/codec-instructions` with its absolute `results` directory
and explicitly prepared `POCKET_IC_BIN`. Raw evidence remains outside Git.

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

## Isolating the u32 shortcut (8 November 2026)

**Research only; production remains unchanged.** The narrower follow-up avoids
routing i32 through u32's small-value branch. Two candidates were screened:

- `split32`: extract the existing i32 raw tuple loop into private
  `raw_u32<S: Serializer>(inner: S, value: u32)`. i32 zigzags then calls it
  directly; u32 keeps its `< 128` byte shortcut and otherwise calls the helper.
- `u32only`: use that helper only for u32, leaving the existing i32 method intact.
  No inlining attributes, decoder changes or u64 changes are introduced.

The helper computes `((32 - value.leading_zeros()).max(1) as usize).div_ceil(7)`,
opens an inner tuple of that width, emits `(value as u8) | 128` while shifting
right seven bits until below 128, then emits the final byte and ends the tuple.
It wraps Postcard output/sizing only; implementation bytes never acquire extra
admission charges. u32 128 remains `80 01`; i32 -64 remains `7f`. Bytes,
identities, budgets and acceptance are unchanged; no consumer migration.

Fresh matched source is 4f62dfb plus the pending Toko 0.34 migration. Lock SHA-256:
`8edac5b7c18891e830db716efb866ba77bf5bd96e0a0fa2010bc362d640aaf05`.
Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown, existing evaluation opt-z and
benchmark-speed opt-3 profiles, LTO/one codegen unit, no post-link optimizer.
Each source has a separate target. Candidate native targets seed dependency
caches only; touched product sources rebuild, and baked encoder/lock hashes
match the measured source. Wasm screens use fresh targets. Raw scripts, frozen
inputs, hashes and outputs: `target/evidence/u32-split-20261108/` (`prepare.mjs`,
`screen.sh`, `measure.sh`, `size-controls.sh`, `analyze.mjs`). The u32-only IC
control reuses exact baseline Candid artifacts and their build records.

Whole-module uncompressed **total / code-section bytes**:

| Fixture/profile | Baseline | Shared loop | u32 only |
| --- | ---: | ---: | ---: |
| Hotspots / opt-z | 83,508 / 72,547 | 83,236 / 72,291 | 83,207 / 72,264 |
| Hotspots / opt-3 | 103,969 / 93,675 | 103,826 / 93,532 | 103,826 / 93,532 |
| Standard / opt-z | 594,633 / 547,837 | 594,647 / 547,849 | 594,690 / 547,894 |
| Standard / opt-3 | 891,015 / 830,421 | 890,339 / 829,745 | 890,339 / 829,745 |

Three identical IC samples per operation, with distinct query callers, cover
87 hotspot cases and 1,904 rows per lane. Inputs/frame hashes match; Candid
counters are unchanged. **Encode instructions** (lists have 16,384 elements):

| Case/profile | Baseline | Shared loop | u32 only |
| --- | ---: | ---: | ---: |
| Wide u32 / opt-z | 12,207,902 | 8,242,906 (-32.5%) | 7,653,083 (-37.3%) |
| Wide i32 / opt-z | 7,948,064 | 8,193,753 (+3.1%) | 7,947,996 (−68) |
| Singleton wide u32 / opt-z | 2,286.5 | 1,976.5 | 1,941.5 |
| Empty-blob mixed record / opt-z | 5,197.5 | 4,857.5 | 4,857.5 |
| Wide u32 / opt-3 | 4,326,408 | 4,637,699 (+7.2%) | 4,637,699 (+7.2%) |
| Wide i32 / opt-3 | 4,752,384 | 4,752,384 | 4,752,384 |
| Singleton wide u32 / opt-3 | 1,062.25 | 1,088.25 | 1,088.25 |

Both candidates fail four of 860 Wire operation comparisons: opt-3 wide-u32
fresh/reused encoding at 128 and 16,384 elements. The hotspot footprint checks
pass, but standard opt-z grows 14/12 or 57/57 total/code bytes respectively.
The candidates produce byte-identical opt-3 hotspot modules. The u32-only
primary-profile result is better than sharing: source factoring is not proof of
better emitted code. This experiment does not prove a particular LLVM cause.

For the shared-loop candidate, both profiles have identical baseline/candidate
allocation counts and peak-live-heap measurements. Two native runs/profile use
11 samples and 2 ms batches. Wide-u32 encode medians in ns are
214,324 / 220,625 → 177,100 / 251,741 at opt-z, and
106,789 / 102,765 → 87,649 / 90,823 at opt-3. Wide-i32 opt-z is
150,762 / 160,224 → 175,403 / 177,144. These timings are variable and do not
replace IC counters; the native u32 opt-3 gain accompanies an IC regression.
u32-only native timing and memory are **not measured**; shared-loop results
must not be attributed to it. Both candidates pass 47 native codec tests, one
timing probe ignored, including Postcard parity and exact resource boundaries.
Initial sandboxed local-server failures are retained separately from completed
PocketIC 16.1.0 runs.

The initial decision preferred **u32-only for the next qualification**, not the shared-helper rewrite.
It is a plausible opt-z tradeoff, but not a passing no-regression result or an
adopted change. Before adoption, complete its native/memory and signed32 controls,
check a Toko-shaped footprint, and explicitly record the opt-3 cost and small
standard opt-z growth. No full matrix, installed application, macOS or application
Wasm saving is claimed. The standard fixture above is footprint-only.

## Bounded unrolled u32 writer (8 November 2026)

**Adopted locally for pending Core 0.18.0.** The loop-only candidates above are
not adopted. Their follow-up keeps the existing u32 `< 128` shortcut and i32
method, but unrolls `raw_u32`'s continuation steps: emit the first byte (the
caller guarantees value >= 128), shift seven bits, then use three nested
`>= 128` checks to emit/shift the remaining continuation groups. Finally emit
the last byte and end the exact-width tuple. At most five bytes are written.
One private helper is generic over the existing output/sizing serializers, not
over each consumer's record type. No codec option or per-type variant is added.

The compiler, profiles, frozen lock and input identities are those above.
Reproduction extends the same evidence directory with `prepare-unrolled.mjs`,
`screen-unrolled.sh`, `qualify-unrolled.sh`, `compare-final.mjs` and
`production-wasm.sh`. `controls.sh` completes the loop candidate's controls;
its initial invocation accidentally selected the argument-taking benchmark
binary as a test, failed before running tests, and was corrected to the existing
`codec-instructions` example test target. No fixture implementation changed.

Uncompressed whole-fixture **total / code-section bytes**:

| Fixture/profile | Baseline | Unrolled | Change |
| --- | ---: | ---: | ---: |
| Hotspots / opt-z | 83,508 / 72,547 | 83,384 / 72,441 | −124 / −106 |
| Hotspots / opt-3 | 103,969 / 93,675 | 104,128 / 93,834 | +159 / +159 |
| Standard / opt-z | 594,633 / 547,837 | 594,851 / 548,055 | +218 / +218 |
| Standard / opt-3 | 891,015 / 830,421 | 889,425 / 828,831 | −1,590 / −1,590 |
| Toko-shaped / opt-z | 80,916 / 69,754 | 80,732 / 69,588 | −184 / −166 |
| Toko-shaped / opt-3 | 103,366 / 92,866 | 103,093 / 92,592 | −273 / −274 |

The signed32 fixture is byte-identical in both profiles: 75,500/63,418 bytes
at opt-z and 91,229/80,005 at opt-3. Toko-shaped and standard results are
footprint-only, not measured application savings.

Encode **IC instructions**, with three identical samples per operation and
distinct callers (large lists contain 16,384 values):

| Case | opt-z before → after | opt-3 before → after |
| --- | ---: | ---: |
| Wide u32 list | 12,207,902 → 6,702,812 (−45.1%) | 4,326,408 → 3,720,193 (−14.0%) |
| Singleton wide u32 | 2,286.5 → 1,884.5 (−17.6%) | 1,062.25 → 1,020.25 (−4.0%) |
| Wide i32 list | 7,948,064 → 7,947,996 | 4,752,384 → 4,752,384 |
| Empty-blob mixed record | 5,197.5 → 4,857.5 (−6.5%) | 2,136.25 → 2,136.25 |
| 1 MiB byte buffer | 1,050,658 → 1,050,590 | 1,049,699 → 1,049,699 |

All **860 hotspot Wire operation pairs** pass the instruction margin; **all
1,070 signed32 rows**, including Candid, have exactly unchanged counters.
Frame/input hashes and Candid artifacts/counters match. The automated hotspot
cost screen still **fails** its opt-3 no-growth rule (+159 bytes); standard
opt-z also grows 218 bytes. Under the opt-z-first policy, these small fixture
increases are accepted for the large instruction reduction in both profiles.
This is an explicit tradeoff, not a passing no-growth claim.

Both profiles retain identical allocation counts and peak-live-heap observations
across 778 rows per measurement mode. Two serial native runs/profile, eleven
samples each, retain absolute encode medians (ns):

| Case/profile | Baseline runs | Unrolled runs |
| --- | ---: | ---: |
| Wide u32 / opt-z | 214,324 / 220,625 | 154,821 / 145,174 |
| Wide u32 / opt-3 | 106,789 / 102,765 | 68,965 / 67,495 |
| Singleton u32 / opt-z | 56.8 / 56.5 | 57.7 / 73.9 |
| Wide i32 / opt-z | 150,762 / 160,224 | 165,821 / 169,711 |
| Empty-blob mixed / opt-z | 117.6 / 120.0 | 145.5 / 135.5 |

Native small/mixed controls show regressions and variability; do not substitute
IC improvements for a universal native-speed claim. The loop-only candidate's
completed native/memory controls are retained in `u32only-native.json`, but it
remains superseded because the unrolled writer removes its opt-3 IC penalty.

Candidate tests pass: 47 native codec tests (one timing probe ignored), and six
signed32 corpus tests including boundaries, truncation and deque admission.
Production native/runtime tests and strict Core Clippy pass. Fresh production
hotspot/standard Wasm is compared against all four measured candidate modules;
baked encoder/lock identities bind the IC sources. No full matrix, native macOS,
new installed-endpoint study or downstream application qualification is claimed.

Codec bytes, schemas, identities, budgets and acceptance remain unchanged:
u32 128 stays `80 01`, and u32::MAX stays `ff ff ff ff 0f`. No migration,
client regeneration or retry/hash change. Only Core production changes;
Endpoint does not need a release for this optimization.

## Wider writer rejection screen (8 November 2026)

**None adopted.** A fresh baseline includes the retained unrolled u32 writer
and IC Build 0.80 adoption. Lock SHA-256 is
`385b5344224449a182554018443beffd23b98164756c42a0c4acf3eb4257c534`;
encoder SHA-256 is
`6db3b2b563a7e854080f4324bf6d15f301057c6bd8f1286f10908fa4d01723d4`.
Rust 1.99.1 / LLVM 23.1.1 and both existing profiles remain unchanged. Separate
fresh Wasm targets and rebuilt native drivers bind each candidate to its source
and lock. Native dependency caches are seeded, but product inputs are touched
and baked source hashes checked. Baseline Candid artifacts are reused exactly.

Three isolated changes were tested, retaining the original small-value shortcut:

1. `unrolled64`: route wider u64 through a ten-byte raw tuple writer. Compute
   width from leading zeros, emit/shift the first continuation byte, then use
   eight nested `>= 128` checks before the final byte. Existing i64 zigzag
   delegation uses this writer too; u32/i32 remain untouched.
2. `direct16`: emit two or three raw u8 tuple elements for wider u16 using the
   same branching structure as the existing i16 writer, without zigzag.
3. `reuse16`: change only wider u16 forwarding to
   `raw_u32(self.inner, u32::from(value))`; no additional writer implementation.

Whole-module uncompressed **total / code-section bytes**:

| Fixture/profile | Baseline | Unrolled u64 | Direct u16 | Reused u32 for u16 |
| --- | ---: | ---: | ---: | ---: |
| Hotspots / opt-z | 83,384 / 72,441 | 83,609 / 72,689 | 83,201 / 72,276 | 83,208 / 72,280 |
| Hotspots / opt-3 | 104,128 / 93,834 | 104,331 / 94,036 | 104,193 / 93,899 | 103,988 / 93,692 |
| Standard / opt-z | 594,851 / 548,055 | 595,082 / 548,303 | 594,676 / 547,898 | 594,556 / 547,775 |
| Standard / opt-3 | 889,425 / 828,831 | 890,418 / 829,824 | 889,101 / 828,506 | 889,146 / 828,551 |

Selected **encode IC instructions** for 16,384-element ordinary lists:

| Candidate / case | opt-z before → after | opt-3 before → after |
| --- | ---: | ---: |
| Unrolled u64 / wide u64 | 17,811,161 → 13,076,185 (−26.6%) | 6,767,606 → 9,749,494 (+44.1%) |
| Unrolled u64 / wide i64 | 18,024,153 → 13,289,177 (−26.3%) | 6,996,982 → 9,978,870 (+42.6%) |
| Direct u16 / wide u16 | 9,422,394.5 → 4,408,892.5 (−53.2%) | 1,770,327.25 → 2,343,767.25 (+32.4%) |
| Reused u32 / wide u16 | 9,422,394.5 → 5,162,554.5 (−45.2%) | 1,770,327.25 → 2,818,902.25 (+59.2%) |
| Reused u32 / wide u32 control | 6,702,812 → 7,194,330 (+7.3%) | 3,720,193 → 4,473,859 (+20.3%) |

The 87-case hotspot corpus retains empty/singleton, byte-buffer, ordinary-vector
and mixed-record controls. For example, unrolled u64 singleton encode changes
2,559.5 → 2,270.5 at opt-z but 1,207.25 → 1,389.25 at opt-3. Direct u16 singleton
changes 2,049.5 → 1,745.5 and 931.25 → 966.25. These are not universal gains.

Each candidate has 1,904 rows with three identical IC samples and unique query
callers; input/frame hashes and Candid counters match baseline. The independent
cost screen rejects 12, four and 12 of 860 Wire operation pairs respectively.
The reuse candidate passes footprint checks but regresses the already retained
u32 path even at opt-z: sharing a source helper does not guarantee better emitted
code. These measurements do not isolate an LLVM inlining or layout cause.

All candidates pass 47 native codec tests (one timing probe ignored), including
exhaustive u16 byte/budget checks and signed/unsigned width boundaries. Since the
instruction screen rejects each candidate, native timing, allocation/heap,
Toko-shaped and expanded signed32 measurements were not run. Standard modules
are footprint-only. No application savings or full matrix are claimed.

Reproduction and raw evidence: `target/evidence/wide-writers-20261108/`, including
`prepare.mjs`, `screen.sh`, `prepare-ic.sh`, `screen-reuse16.sh`,
`prepare-reuse16-ic.sh`, `cost-screen.sh` and `analyze-all.mjs`. The reuse candidate
is reproduced from baseline by the single substitution above. Production source
and lock hashes remain equal to this study's baseline. Bytes, schemas, identities,
budgets and acceptance are unchanged; u16/u64 128 remains `80 01`, i64 -64 `7f`.
No consumer migration, regeneration or retry/hash change. Preserve the retained
u32 optimization; do not extend it mechanically to these widths.

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
