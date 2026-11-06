# Latest recorded Candid comparison

Measured **6 November 2026**, from Core **0.17.4** with pending qualification
supplier updates. Wire retains one production Serde/Postcard codec; `Wire`
derives host metadata, and `Packed<T>` is an explicit field representation.
Candid remains in unpublished qualification packages.

The complete **28-cohort** roster contains **1,575 overlapping case entries**,
including all **391 standard cases**, and **6,300 primary encode/decode
comparisons**. Counts are not unique Rust types or a traffic weighting.

| Scope | Comparisons | Wire wins | Candid wins |
| --- | ---: | ---: | ---: |
| 27 production-representation cohorts | 5,924 | 5,360 | 564 |
| Private float/bool bulk research | 376 | 369 | 7 |
| Complete roster | 6,300 | 5,729 | 571 |

There are no ties. The research row is not a shipped capability. This refresh
changes no production codec, bytes, schemas, identities, limits or acceptance.

## Uncompressed Wasm footprint

Total / code-section bytes from the same complete module. Wire is smaller in
both metrics in **all 56 cohort/profile pairs**. These are library fixtures,
not full Toko builds, application linker savings or deployment-limit claims.

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 391 cases | opt-z | 594,548 / 547,752 | 1,450,851 / 1,375,031 |
| Standard, 391 cases | opt-3 | 891,464 / 830,841 | 2,099,419 / 2,008,920 |
| Toko-shaped, 10 cases | opt-z | 80,948 / 69,786 | 387,240 / 352,047 |
| Toko-shaped, 10 cases | opt-3 | 103,366 / 92,866 | 610,565 / 576,847 |
| 100 ordinary derived types | opt-z | 310,764 / 292,027 | 1,445,424 / 1,390,356 |
| 100 ordinary derived types | opt-3 | 396,160 / 361,734 | 2,111,726 / 2,041,548 |

The 100-type controls exercise Serde codec derives versus CandidType/Serde;
host-only Wire metadata generation is separate. Their stress results are not
projected consumer savings. No compression metric is used.

## Instructions, frame bytes and remaining gaps

Selected **opt-3 IC instructions/op**; encode means fresh output. Frames include
each codec's framing. Linked fixture context affects absolute instruction costs,
so compare labelled cohorts rather than treating one shape's count as universal.

| Cohort / payload | Wire encode / decode | Candid encode / decode | Wire / Candid frame bytes |
| --- | ---: | ---: | ---: |
| Standard: byte-oriented 1 MiB blob | 1,049,704 / 1,049,508 | 2,115,234 / 1,068,979 | 1,048,590 / 1,048,588 |
| Standard: 16,384 ordinary wide i16 | 3,113,807.25 / 1,904,910.5 | 80,397.25 / 1,412,137.5 | 49,166 / 32,780 |
| Standard: 1,024-hash blob manifest | 4,026,955.25 / 3,990,178.5 | 156,147.25 / 4,631,755.5 | 32,830 / 33,883 |
| Standard: 128 explicit-zero Accounts | 122,726.25 / 104,781.5 | 104,445.75 / 1,686,854.5 | 8,205 / 8,349 |

The standard corpus favors Wire on **319/391 encodes and 370/391 decodes** at
opt-3. Toko-shaped cases favor Wire in **37/40** primary comparisons. Accounts
above use the plain-array external Candid DTO; the expanded coverage also retains
byte-adapted Candid Account/hash controls. Owned Types remain on Wire's side;
external DTO conversion/setup is outside measured codec work.

Ordinary lists remain expensive. Across production cohorts, worst encode ratios
are **121.39×** Candid at opt-z (`standard/u16-list-wide-16384`) and **49.67×**
at opt-3 (`bulk-control/u16-wide-16384`). Worst decode ratios are **11.97× /
5.36×**. There are **91 / 75** encode and **4 / 0** decode comparisons at least
10× slower, for opt-z / opt-3 respectively; these include overlapping cases.
A 1 MiB bool list reaches **46.04× / 28.79×** encode in `coverage-shapes`.
Rejection costs remain separate: traversing an ordinary list's valid prefix can
be expensive before discovering truncation; Wire budgets and Candid quotas are
not equivalent rejection policies.

The 109-case generic Packed cohort wins **all 436** primary comparisons, and
bounded Packed-i16 wins **all 80**. Packing is not always smaller: in the paired
bulk cohorts, 128 small u16 values encode in **10,016.25 → 1,176.25 instructions**
while the frame grows **141 → 269 bytes**. This is an explicit representation
choice, not a transparent optimization of an existing consumer contract.

## Advice audit

Every production comparison at least 10× slower falls into ordinary byte,
supported integer/hash collections, or float/bool lists. Existing default
[bulk advice](packed-fields.md) covers those families when supported Wire
metadata is generated. A new end-to-end test derives 15 hotspot field families,
checks their exact default paths, verifies byte/Packed alternatives stay quiet,
and confirms acknowledgements leave generated source unchanged. The eight-entry
print cap remains intact; no new lint traversal or host cost is introduced.

This is not a universal Serde lint. Codec-only consumers need host schema
generation to receive advice. Raw floats, maps, pointer-sized integers and custom
layouts require an explicit supported schema projection; finite float wrappers
have metadata. Bool/float/128-bit/zero-sized lists receive cost/bounds advice,
not a recommendation for unsupported Packed words. No consumer Serde is rewritten.

## Private bulk research

The existing private byte-buffer fields still reduce large float/bool costs.
For 1 MiB of values at opt-3, bool encode changes **60,818,316 → 1,049,585**
instructions; f32 encode changes **12,583,841 → 1,049,588** and decode
**12,326,591 → 1,049,688**. Singleton bool encode instead grows
**520.25 → 672.25** (+29.22%, +152 instructions).

| Profile | Ordinary Wire total / code | Research total / code | Total / code delta |
| --- | ---: | ---: | ---: |
| opt-z | 199,233 / 165,580 | 198,294 / 164,642 | −939 / −938 |
| opt-3 | 245,756 / 214,493 | 244,192 / 212,931 | −1,564 / −1,562 |

Logical inputs and Candid frames match across these cohorts. Candid modules have
identical sizes but different hashes; three opt-z borrowed-record encodes differ
by +142 instructions. Float/bool Candid counters match. This is not a passing
unchanged-Candid-artifact regression gate, and no production promotion is made.
Float prefixes change from element counts to byte counts (one f32: `01` → `04`);
bool admission also differs even when bytes agree. The
[original research description](evidence/primitive-bulk.md) explains the
contract/client work that adoption would require.

## Shared-writer experiment: rejected

An isolated candidate mapped signed i32 through the existing unsigned writer,
following the already-shipped i64 approach. Its byte/budget tests pass, including
all signed width boundaries, but it fails the size objective:

| Fixture / profile | Total before → after (delta) | Code before → after (delta) |
| --- | ---: | ---: |
| Ordinary bulk / opt-z | 104,929 → 104,932 (+3) | 93,742 → 93,745 (+3) |
| Ordinary bulk / opt-3 | 136,727 → 136,815 (+88) | 126,242 → 126,330 (+88) |
| Small controls / opt-z | 94,263 → 94,263 (0) | 81,814 → 81,814 (0) |
| Small controls / opt-3 | 120,399 → 120,399 (0) | 108,957 → 108,957 (0) |
| Toko-shaped / opt-z | 80,948 → 80,948 (0) | 69,786 → 69,786 (0) |
| Toko-shaped / opt-3 | 103,366 → 103,366 (0) | 92,866 → 92,866 (0) |

The four unchanged modules are bit-identical. Six fresh guest builds and the
candidate's frozen source inventory verify; it used a separate Cargo target and
the matrix's compiler, lock, inputs and profiles. No candidate IC/native speed
claim is made: it stopped at the size screen. Production retains its existing
writer; `-64_i32` remains `7f`, `64_i32` remains `80 01`, with unchanged budgets.
The new boundary regression test is retained. Shared parser-tail extraction was
also reviewed; [previous rejection](evidence/parser-screen.md) already records
size growth, with [decode regressions](evidence/tail-and-budgets.md) in the earlier
study. Its isolated correctness check is not a new performance qualification.

## Native time, allocation and methodology

For the standard 1 MiB blob at opt-3, the two native run medians are:

| Operation | Wire run 1 / run 2, µs | Candid run 1 / run 2, µs |
| --- | ---: | ---: |
| Fresh encode | 20.348 / 18.551 | 978.142 / 759.335 |
| Owned decode | 17.996 / 18.382 | 21.180 / 19.281 |

Each run retains eleven calibrated samples with minimum 2 ms batches. The host
is shared, not CPU-isolated; these are observations, not application latency or
IC instruction ratios. Separate allocator probes record one allocation and zero
reallocations for each Wire blob operation, versus Candid's 17 + 3 reallocations
for encode and 31 + 0 for decode. Separate peak-live-heap probes record
1,048,590 / 1,048,576 bytes for Wire encode/decode, versus
2,098,807 / 1,049,380 for Candid. These are not allocation-admission allowances.

The frozen lock selects Candid **0.10.40**, Postcard **1.1.3**, Platform facade
**0.23.2**, Types **0.18.2**, Runtime **0.19.2**, Testkit **0.89.0** and Build
**0.75.0**. Rust/Cargo **1.99.1**, LLVM **23.1.1**, host
`x86_64-unknown-linux-gnu`, guest `wasm32-unknown-unknown`; prepared PocketIC
**16.1.0**. Both profiles use fat LTO, one codegen unit, panic abort and symbol
stripping; `evaluation` selects opt-z, `benchmark-speed` selects opt-3. No
post-link optimizer or consumer deployment transformation is applied.

All **33,480 IC operation rows** have three distinct-caller samples with equal
counters. All **112 modules** have verified hashes, fresh guest compiler records
and independently checked section-10 payload sizes. **108,072** comparisons
confirm native timing/allocation/heap records agree with IC input/frame hashes.
Counters include warm codec bodies, result destruction and loop overhead, but
exclude transport, authentication, ingress copying, application work and physical
cycles. Setup/conversion is excluded. Heap, time, instructions and bytes remain
separate measurements.

Raw evidence, frozen sources and recovery records remain under ignored
`target/evidence/codec-matrix-20261106-current/`; candidate sources/results are in
`target/evidence/signed32-20261106/`. The lock SHA-256 is
`63e09516e475fe59cf7af121c9defb58458c0bd81c4b99535107f490e8e65d1b`.
The sole mid-run source repair was the post-measurement reporter's handling of
native-run companion logs; original samples and runtime/fixture sources were
preserved. Subsequent checkout updates to Platform facade 0.23.3, Runtime
0.19.3 and Testkit 0.89.1 were not measured.
Older graphs/artifacts do not provide a matched regression comparison with this
refresh. No full repository CI, macOS run or full consumer build is implied.

Use the [maintained matrix command](development.md#candid-comparison-benchmarks) with a fresh
output directory and explicitly prepared PocketIC server to reproduce. The
[data-only matched cost gate](development.md#paired-cost-regression-check)
requires completed, matched baseline/candidate runs; `make check-benchmarks`
alone tests correctness and tooling, not measured performance.
