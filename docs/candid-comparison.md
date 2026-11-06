# Latest recorded Candid comparison

Full matrix measured **6 November 2026**, from Core **0.17.4** with pending qualification
supplier updates. Wire retains one production Serde/Postcard codec; `Wire`
derives host metadata, and `Packed<T>` is an explicit field representation.
Candid remains in unpublished qualification packages.

A subsequent **Core 0.17.5** source follow-up adds 30 container cases; see
[expanded container coverage](#expanded-container-coverage). Its newer supplier
graph and larger fixture are separate from the full-matrix results below.
The [packed domain conversion study](#packed-domain-conversion-follow-up) then
qualifies an explicit transport choice for its nonzero-list and heap hotspots.
The [borrowed adapter investigation](#borrowed-adapter-investigation) removes
its encode copy privately; public derive/client support is not yet supplied.

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

## Expanded container coverage

The follow-up cohort now has **124 cases**, including 30 additions at indices
94–123. Existing names, indices, logical inputs and encoded frames are preserved.
New cases cover `Result<Packed<u64>, ByteBuf>`, nested optional bytes, lists of
`ByteArray<32>` versus `Packed<[u8;32]>`, `Vec<NonZeroI64>` and `BinaryHeap<u64>`.
Collections have 0/1/128/4,096-element controls; nested byte options distinguish
outer absent, inner absent and present-empty. This adds qualification only;
production bytes, schemas, identities, budgets and acceptance are unchanged.

Among the **120 new primary comparisons**, Wire wins **50/60 at opt-z** and
**52/60 at opt-3**. Selected **opt-3 IC instructions/op**, with full frame bytes:

| New payload | Wire encode / decode | Candid encode / decode | Wire / Candid bytes |
| --- | ---: | ---: | ---: |
| Success with 4,096 packed u64 | 33,630.25 / 33,945.5 | 105,154.25 / 534,614.5 | 32,783 / 32,793 |
| Error with 4,096 blob bytes | 4,819.25 / 4,904.5 | 47,808.25 / 52,078.5 | 4,110 / 4,121 |
| Nested present option, 4,096 bytes | 4,843.25 / 4,896.5 | 40,910.25 / 46,030.5 | 4,111 / 4,113 |
| 4,096 individually byte-adapted hashes | 365,458 / 1,031,735 | 418,449 / 3,896,738 | 135,181 / 135,181 |
| 4,096 packed hashes | 132,084 / 132,208 | 527,302 / 3,896,731 | 131,086 / 135,181 |
| 4,096 nonzero i64 values | 1,337,370.25 / 1,198,222.5 | 80,934.25 / 511,084.5 | 28,688 / 32,779 |
| Heap of 4,096 u64 values | 1,349,956.25 / 1,175,292.5 | 81,318.75 / 506,819.5 | 24,525 / 32,779 |

Nonzero-list and heap encode are **16.52× / 16.60×** Candid at opt-3 and
**38.25× / 45.55×** at opt-z. None of the new decode comparisons reaches 10×.
Candid uses external integer-vector DTOs: Wire additionally validates nonzero
values and reconstructs a heap. These are not equivalent invalid-value or
reconstruction policies. Bulk transport would require explicit integer fields,
retained nonzero validation and heap reconstruction; those costs are not measured
as a proposed fix here. Codec-only custom types do not automatically receive
schema advice, and this extension adds no metadata implementations.

Both hash controls have identical Candid types, logical inputs and encoded bytes.
Their separately exercised encode counters differ, so the table retains each
control's actual result; no unchanged-Candid-artifact optimization claim is made.
The packed representation is already shipped. Choosing it instead of a list of
byte fields changes the consumer's representation and requires coordinated
adoption; the benchmark does not rewrite any consumer.

| Expanded 124-case fixture | Wire total / code bytes | Candid total / code bytes |
| --- | ---: | ---: |
| opt-z | 216,791 / 182,336 | 710,048 / 648,409 |
| opt-3 | 272,389 / 240,334 | 1,054,195 / 994,719 |

These are complete uncompressed fixture modules, not application savings or a
matched size delta against the old 94-case fixture. For packed hashes at opt-3,
two native run medians are **2.104 / 2.076 µs encode** and **2.263 / 2.111 µs
decode**, versus Candid **10.668 / 10.266 µs encode**, **84.255 / 79.141 µs decode**.
Wire uses one allocation and no reallocations per operation. Candid uses
20 allocations + 15 reallocations for encode and 39 + 1 for decode. Separate
peak-live-heap probes record Wire **131,086 / 131,072 bytes**, versus Candid
**280,277 / 132,072 bytes**. The shared host is noisy; native heap-control Candid
encode medians varied from 8.462 to 1.604 µs, while all repeated IC counters agree.

The frozen source uses Platform **0.23.4**, Runtime **0.19.4**, Types **0.18.2**,
Testkit **0.89.2**, Candid **0.10.40**, Postcard **1.1.3**, Rust/Cargo **1.99.1**
and PocketIC **16.1.0**, with the same two profile settings described below.
All **2,594 IC rows** have three equal fresh-caller samples; **8,328** native,
allocation and heap records match IC input/frame hashes. Four guest artifacts
are fresh, with verified total/code sizes and source hashes baked into host
evidence. The 12 focused corpus tests, affected all-target strict Clippy and
format checks pass. This is not Endpoint supplier qualification or full CI.

Raw results, frozen source, inventory and verifier stay under ignored
`target/evidence/container-types-20261106/`; lock SHA-256:
`2f34d31bfee0a84d801a064bcf90a82cfeba355226ee2a6ce9e8c5917b5a52ec`.
Reproduce with the [codec runner](development.md#candid-comparison-benchmarks),
`CORPUS_FEATURE=coverage-shapes`, two native runs, eleven samples, 2 ms minimum
batches, a fresh Cargo target and an explicit prepared PocketIC server. The
retained `verify.py` checks source inventory, fresh artifacts and measurement
identities; it is not a passing baseline/candidate regression gate.

## Packed domain conversion follow-up

**Keep the explicit consumer pattern**, using existing `Packed<i64>` and
`Packed<u64>` DTO fields. No production codec implementation, public API or
automatic Serde rewrite is added. A paired `packed-containers` qualification
cohort replaces only the eight nonzero-list/heap cases from `coverage-shapes`.
All 124 inputs and Candid DTOs match; ordinary controls remain maintained.
Both sides start from domain values. Wire's measured operations include
temporary encode copies, zero validation, heap reconstruction and destruction.
Candid continues to decode external integer vectors without those domain checks.

For **4,096 values**, **opt-3 IC instructions/op** are:

| Domain payload | Ordinary Wire encode / decode | Packed + conversion encode / decode | Candid encode / decode |
| --- | ---: | ---: | ---: |
| Nonzero i64 list | 1,337,370.25 / 1,198,222.5 | 67,399.25 / 95,366.5 | 81,318.75 / 510,916.5 |
| u64 heap | 1,349,956.25 / 1,175,292.5 | 67,252.25 / 169,068.5 | 81,318.75 / 506,819.5 |

Encode drops **94.96% / 95.02%** and decode **92.04% / 85.61%**, respectively.
Both now beat Candid, including domain conversion. At opt-z, packed encode/decode
is **68,670.5 / 181,887.5** for nonzero lists and **68,632.5 / 273,881.5** for
heaps, versus Candid **87,278 / 549,872.5** and **87,279 / 549,879.5**.
All **32** target comparisons (0/1/128/4,096 values, both operations/profiles)
beat Candid. Across all 30 appended cases, the packed selection wins **118/120**;
there are no ≥10× losses among those additions. Earlier ordinary-list hotspots
elsewhere in the corpus remain; this is not a transparent global codec fix.

| Complete fixture | Ordinary total / code | Packed total / code | Total / code delta |
| --- | ---: | ---: | ---: |
| opt-z | 216,815 / 182,344 | 216,155 / 181,685 | −660 / −659 |
| opt-3 | 272,586 / 240,515 | 271,785 / 239,713 | −801 / −802 |

All sizes are **uncompressed Wasm bytes**. These are matched linked fixtures,
not predicted Toko savings. Candid code-section sizes are unchanged; total size
changes by +8 bytes at opt-z and 0 at opt-3. Candid artifacts are not bit-identical.
Across 496 unchanged Wire encode/decode controls, the largest regression is
**7 instructions** (+0.43%); Candid's largest is **105 instructions**, with no
relative regression above 0.28%. None exceeds the comparison policy's 5% + 64
instruction allowance. This is a representation study, not a passing unchanged-
frame regression gate: the selected fields deliberately change payload bytes.

Tradeoffs remain explicit:

- The 4,096-value nonzero frame grows **28,688 → 32,782 bytes**; the heap grows
  **24,525 → 32,782**. Inputs are mixed full-width values, not uniformly small
  integers. Small varints may have a much larger size advantage.
- Empty-container decode increases **302.5 → 356.5** instructions for nonzero
  lists and **304.5 → 485.5** for heaps at opt-3. At opt-z, empty encode increases
  by 205 / 335 instructions and decode by 170 / 113. Singleton heap encode adds
  25 instructions at opt-3; the selected singleton nonzero encode improves.
- Borrowed-domain encode uses **two allocations instead of one**. At 4,096
  values, peak temporary heap grows from **28,688 / 24,525** to **65,550 bytes**.
  Decode retains a **32,768-byte** peak, one allocation and no reallocations,
  instead of ten reallocations. These observations include the conversion code;
  they are not a promise about Rust's future allocation optimizations.

Two opt-3 native run medians, ordinary → packed, in µs:

| Domain / operation | Ordinary run 1 / run 2 | Packed run 1 / run 2 |
| --- | ---: | ---: |
| Nonzero encode | 27.686 / 26.286 | 1.232 / 1.229 |
| Nonzero decode | 17.702 / 18.348 | 1.466 / 1.371 |
| Heap encode | 29.653 / 30.079 | 1.190 / 1.161 |
| Heap decode | 15.095 / 15.268 | 1.925 / 1.767 |

The [compiled consumer example](../crates/ic-wire/README.md) shows the boundary.
Nonzero admission must run before application logic; decoding a `Packed<i64>`
alone does not reject zero. Rebuild heaps from untrusted word order and preserve
duplicates. Keep element limits and account for conversion memory/work outside
Wire's native admission. Adopting `[-1_i64]` changes `01 01` to
`08 ff ff ff ff ff ff ff ff`; a heap containing `1_u64` changes `01 01` to
`08 01 00 00 00 00 00 00 00`. Coordinate clients/contracts, requalify budgets and
retain original retry/hash bytes. Existing consumers remain unchanged.

The two isolated sources use the expanded follow-up's compiler, lock and suppliers,
with separate fresh Cargo targets, two native runs of eleven ≥2 ms samples per
profile and three distinct-caller IC samples. All **5,188 IC rows** have equal
repeats; **16,656** native/allocation/heap records match guest input/frame hashes.
Eight guest modules have fresh compiler records and verified total/code sizes.
The 13 corpus tests pass in both selections, strict affected Clippy passes, and
the new codec-only guide example passes Rustdoc without metadata features.
Raw source inventories, commands in the retained runners, verifier and paired
results live under ignored `target/evidence/packed-containers-20261106/`.
Use `CORPUS_FEATURE=coverage-shapes` and `CORPUS_FEATURE=packed-containers` with
the maintained codec runner to repeat the study. No consumer deployment,
Endpoint supplier qualification or full CI is implied.

## Borrowed adapter investigation

The private prototype is now promoted to explicit public Serde field adapters:
`serde_packed::nonzero_i64` and `serde_packed::heap_u64`. They reuse Packed's
codec directly from domain storage, with owned decode, automatic Wire metadata
and browser nonzero validation. No second codec or schema trait is introduced.

Fresh **2026-11-06** paired runs compare the original one-copy Packed DTO against
the public adapters, including domain conversion, validation and reconstruction
inside timing. This avoids overstating savings against a copying Serde adapter
that visits its temporary vector during sizing and writing. At 4,096 values,
**opt-3 IC instructions/op** are:

| Payload | Original DTO encode / decode | Public adapter encode / decode | Candid encode / decode |
| --- | ---: | ---: | ---: |
| Nonzero i64 | 67,399.25 / 95,366.5 | 33,480.25 / 94,943.5 | 81,318.75 / 510,916.5 |
| u64 heap | 67,252.25 / 169,068.5 | 33,900.25 / 169,198.5 | 81,318.75 / 506,819.5 |

At opt-z, encode is **68,670.5 → 34,539.5** and **68,632.5 → 34,985.5**.
Both target families beat Candid at 0/1/128/4,096 values in both directions and
profiles. All frames match the DTO control (32,782 bytes at 4,096 elements).
Encode allocations fall **2 → 1**, peak live bytes **65,550 → 32,782**; decode
remains one allocation / 32,768 peak bytes. Empty nonzero encode adds 2 IC
instructions at opt-3; heap decode adds up to 172 at opt-z / 130 at opt-3.
All **1,090 Wire operation pairs**, including rejection, stay within 5% + 64.
The largest increase across 496 unchanged encode/decode controls is one instruction.
All Candid operation counters match. Candid artifact hashes differ (code sizes
match), so this is **not** a passing bit-identical-artifact cost gate.

| Complete 124-case fixture | DTO total / code bytes | Adapter total / code bytes | Delta, both metrics |
| --- | ---: | ---: | ---: |
| opt-z | 216,155 / 181,685 | 215,957 / 181,487 | −198 |
| opt-3 | 271,785 / 239,713 | 271,206 / 239,134 | −579 |

A separate stress comparison against **already-packed records**, with their DTOs
constructed outside timing, shows a cost for retaining domain fields. These are
final linked **uncompressed total / code-section bytes**, not application savings:

| Types | opt-z Packed → adapters (total / code) | opt-3 Packed → adapters (total / code) |
| --- | ---: | ---: |
| 1 | 58,505 / 47,299 → 59,152 / 47,946 | 86,764 / 59,817 → 87,104 / 60,157 |
| 10 | 109,612 / 97,558 → 113,657 / 101,595 | 164,399 / 136,657 → 173,385 / 145,641 |
| 100 | 338,123 / 319,147 → 346,340 / 327,359 | 551,576 / 516,943 → 567,725 / 533,090 |

Use adapters selectively for domain fields; existing Packed DTOs remain smaller.
Forcing decoder non-inlining left the 10/100-type artifacts identical and made
one type larger. Forcing encoder non-inlining reduced the 100-type opt-3 total
to 557,011, but grew the single-type total to 87,989 and did not improve the larger
opt-z fixtures. Neither annotation is retained; no generic codec policy changed.

Native medians (two runs, µs/op) remain separate observations:

| Payload | DTO encode; decode | Adapter encode; decode |
| --- | ---: | ---: |
| Nonzero i64 | 1.173 / 1.251; 8.214 / 1.448 | 0.542 / 0.565; 1.440 / 1.425 |
| u64 heap | 1.188 / 1.142; 1.888 / 1.835 | 0.536 / 0.541; 1.779 / 1.840 |

The host is shared, with some background compilation; the nonzero decode spread
is not a proven speedup. IC counters, not native timing, drive the comparison.
Rust/Cargo **1.99.1**, LLVM **23.1.1**, host x86_64 and wasm32-unknown-unknown;
opt-z/opt-3, fat LTO, one codegen unit, panic abort, stripped symbols. Both sources
use Candid 0.10.40, Postcard 1.1.3, facade 0.23.4, Types 0.18.2, Runtime 0.19.4,
Testkit **0.89.3**, Build 0.75.0 and Host 0.74.4; lock SHA-256
`04623e37662c068b19fba770c5be5170b3a906a8c7548fd21004b74f4f648192`.
PocketIC 16.1.0 supplies three distinct-caller samples with equal counters.
No transport, authentication, physical cycles or real big-endian execution claim.

Raw evidence remains under ignored `target/evidence/public-adapters-20261106/`:
5,188 verified IC rows, 16,656 matching native/allocation/heap records, eight fresh
comparison modules and 24 verified footprint/annotation experiment modules.
Separate targets, source inventories, baked host source hashes, artifact hashes
and independent code-section parsing bind the measurements. After freezing Rust runtime sources, only documentation, extra test assertions
and a TypeScript type refinement changed; the emitted runtime JavaScript remains
byte-identical.

Adoption from ordinary nonzero `[-1]` changes `01 01` to
`08 ff ff ff ff ff ff ff ff`; heap `[1]` changes `01 01` to
`08 01 00 00 00 00 00 00 00`. Coordinate identities/consumers, regenerate clients
and requalify limits. Existing consumers remain unchanged. Existing Packed DTO
bytes match, but adapter record encoding can charge a smaller reference wrapper:
64-bit `{tag:1, words:[-1]}` needs 842 vs 1,098 allowance bytes; decode needs 842
for both. These are traversal allowances, not heap measurements or portable Wasm
thresholds. Rust rebuilds heap order; browser arrays retain received order and
duplicates. Retain exact retry/hash bytes instead of assuming canonical re-encoding.

## Full-matrix native time, allocation and methodology

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
