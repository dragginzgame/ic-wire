# Latest recorded Candid comparison

Measured **7 November 2026**, Core **0.17.6 working source** with uncommitted dependency updates.
The complete **36-cohort** run contains **2,105 overlapping case entries** and
**8,420 primary encode/decode comparisons**, not unique types or traffic weights.
Wire wins **7,295/7,924** non-research comparisons and **471/496** private research
comparisons; Candid wins 629 and 25, with no ties. There is one production
Serde/Postcard codec; Packed/adapters select fields explicitly. Candid is test-only.

## Uncompressed Wasm

Wire is smaller in total and code bytes in **all 72 cohort/profile pairs**.
Below: total / code-section bytes, without a post-link optimizer. These are
complete fixture modules, not full Toko builds or projected application savings.

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 391 cases | opt-z | 594,543 / 547,747 | 1,450,722 / 1,374,903 |
| Standard, 391 cases | opt-3 | 891,463 / 830,841 | 2,099,418 / 2,008,919 |
| Toko-shaped, 10 cases | opt-z | 80,916 / 69,754 | 387,103 / 351,919 |
| Toko-shaped, 10 cases | opt-3 | 103,366 / 92,866 | 610,572 / 576,846 |
| 100 ordinary derived types | opt-z | 310,732 / 291,995 | 1,445,295 / 1,390,228 |
| 100 ordinary derived types | opt-3 | 396,160 / 361,734 | 2,111,725 / 2,041,547 |

The 100-type controls keep all Serde/CandidType codec paths reachable. Wire's
metadata derive is host-only. Stress-fixture differences are not application savings.

## Instructions and frame bytes

Selected **opt-3 IC instructions/op**; encode allocates fresh output. Each row
uses its labelled linked fixture context. Frame bytes include codec framing.

| Cohort / payload | Wire encode / decode | Candid encode / decode | Wire / Candid bytes |
| --- | ---: | ---: | ---: |
| Standard: owned 1 MiB byte buffer | 1,049,704 / 1,049,508 | 2,115,234 / 1,068,979 | 1,048,590 / 1,048,588 |
| i16 control: 16,384 wide values | 2,638,738.25 / 1,905,088 | 80,320.25 / 1,411,845 | 49,166 / 32,780 |
| Packed-i16: same values | 33,489.25 / 33,476 | 80,320.25 / 1,411,845 | 32,782 / 32,780 |
| Standard: 128 explicit-zero Accounts | 122,726.25 / 104,781.5 | 104,445.75 / 1,686,854.5 | 8,205 / 8,349 |
| Byte-key maps: 128 keys | 62,377.25 / 221,495 | 87,093.25 / 1,360,194.5 | 6,477 / 6,485 |
| Public adapter: 4,096 nonzero i64 | 33,480.25 / 94,943.5 | 81,318.75 / 510,916.5 | 32,782 / 32,779 |
| Public adapter: heap of 4,096 u64 | 33,900.25 / 169,198.5 | 81,318.75 / 506,819.5 | 32,782 / 32,779 |

Standard opt-3 favors Wire on **319/391 encodes and 370/391 decodes**; Toko-shaped
cases win **37/40** comparisons. Losses are two opt-z ID filters (1.19× / 1.39×)
and one opt-3 projection (1.02×). The Account row uses plain-array Candid DTOs;
coverage also measures byte-adapted DTOs. External DTO setup is outside timing.

Generic Packed wins **all 436** primary comparisons and bounded Packed-i16 wins
**all 80**. Packing can cost bytes: 128 small u16 values encode in
**10,016.25 → 1,176.25 instructions**, while frames grow **141 → 269 bytes**.
Byte-key maps reduce encode **236,900.25 → 62,377.25** and decode
**699,614 → 221,495**, but add 128 frame bytes and **1,543 total / 1,545 code**
Wasm bytes at opt-3. Coordinate consumers and requalify bounds when changing
representations; retain original retry/hash bytes.

Ordinary-list encode still reaches **119.50× / 49.67× Candid** at opt-z/3.
There are **105 / 81** encode and **4 / 0** decode comparisons at least 10× slower
among non-research cohorts. These include overlapping ordinary controls.
Default [bulk advice](packed-fields.md) requires supported host metadata;
codec-only/custom layouts are not universally linted. Float/bool advice warns
about cost, not unsupported Packed types. Consumer Serde is never rewritten.
Rejection is separate: truncated 1 MiB ordinary bytes cost **42,472,926** opt-3
instructions versus Candid **4,289**; a byte field costs **636** versus **4,290**.
Wire limits and Candid quotas are not equivalent rejection policies.

## Expanded container coverage

The **124-case** follow-up includes nested options/results, hash lists, nonzero
lists and heaps, with empty/singleton controls. Public adapters cut encode from
**1,337,370.25 / 1,349,956.25** to the table's **33,480.25 / 33,900.25** instructions.
They use one allocation per operation; the copying Serde control uses three on
encode. Frames grow from **28,688 / 24,525** to **32,782 bytes**. Both domain
families beat Candid at 0/1/128/4,096 values in both profiles and directions.

## Packed domain conversion follow-up

`packed-containers` copies inside Serde during sizing and writing; the earlier
one-copy DTO study included conversion once. `adapter-dto-1/10/100` constructs
Packed DTOs **before timing**. Keep these controls distinct. Candid vectors do
not perform Wire's nonzero validation or heap reconstruction.

## Private bool/float research

For 1 MiB of values at opt-3, encode changes **60,818,316 → 1,049,585**
instructions for bool and **12,583,841 → 1,049,588** for f32; Candid uses
**2,112,270 / 2,112,195**. Float decode becomes **1,049,688**, versus ordinary
f32's **12,326,591** and Candid's **21,515,241**. Bool decode remains
**33,031,148** versus ordinary **44,570,576** and Candid **208,686,341**.
The subsequent [checked-copy study](evidence/primitive-bulk.md#checked-bool-copy-follow-up-7-november)
reduces private bool decode to **20,972,610** at opt-3, with **173 fewer** total
and code-section Wasm bytes. Singleton decode adds 73 instructions and native
opt-3 large decode is 1.8–4.0% slower. This remains private research.
Singleton bool encode regresses **520.25 → 672.25** in the original bulk study.

Whole-fixture total / code bytes change **216,815 / 182,344 → 215,963 / 181,492**
at opt-z and **272,618 / 240,547 → 271,183 / 239,113** at opt-3. These are
private representations, with no public Packed/schema/client support. Float
prefixes change from element counts to byte counts (`01` → `04` for one f32);
bool's valid bytes match but admission changes. Consumer coordination, malformed
input tests, finite/IEEE policy and generated-client qualification remain required.

## Borrowed adapter investigation

Public `serde_packed::nonzero_i64` / `heap_u64` shipped in 0.17.6. The
[6 November matched study](https://github.com/dragginzgame/ic-wire/blob/bcbcec53ce9eaadf5a839efdb95595b834d2583b/docs/candid-comparison.md#borrowed-adapter-investigation)
halved large-case encode versus the original one-copy DTO, reduced allocations
2 → 1 and saved 198/579 total and code bytes at opt-z/3. Its raw inputs remain
under `target/evidence/public-adapters-20261106/`. Current 100-type DTO → adapter
total/code bytes are **338,091/319,115 → 346,308/327,327** at opt-z and
**551,576/516,943 → 567,725/533,090** at opt-3. Already-packed data stays smaller;
these controls omit domain conversion. Forced non-inlining previously failed
the small/mixed-profile screen. Adoption changes bytes/budgets; matching Packed
DTO bytes can still have different record-field allowances. Follow the
[consumer guidance](../crates/ic-wire/README.md#representation-and-adoption-details).

## Next experiments

1. **Bool bulk adapter:** checked copying now has a measured IC/Wasm win and
   exhaustive invalid-byte tests; qualify public schema, bounds and clients before
   promotion. Float promotion additionally needs a finite/IEEE policy.
2. **Narrow bound-wrapper sharing:** varied bounds add **16,810 / 25,305 total**
   and **16,671 / 25,185 code bytes** at opt-z/3 in the 100-type control. Heavy
   loops already share code; test the remaining wrapper without changing bounds.
   This is a diagnostic gap, not an achieved or guaranteed application saving.
3. **Packed 16-byte identifiers:** test a single shared representation against
   Toko-shaped ID filters, including conversion costs. Current Packed supports
   32-byte hashes, not 16-byte IDs. This is unmeasured; database keys stay outside scope.

Do not repeat blanket one-pass encoder or parser-tail rewrites without a new
approach: [earlier experiments](evidence/encoder-followup.md) failed small-value,
allocation, semantic or Wasm tradeoffs. Whole-list bound erasure also regressed
zero-sized lists; retain the [qualified specialized paths](evidence/bound-sharing.md).

## Native time, memory and reproduction

Standard 1 MiB blob, opt-3, two native run medians in **µs/op**:

| Operation | Wire run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: |
| Fresh encode | 17.064 / 17.199 | 759.928 / 754.623 |
| Owned decode | 17.282 / 17.153 | 18.298 / 18.949 |

The shared host is not latency-isolated. Separate probes record Wire **one
allocation, zero reallocations** per operation;
Candid encode uses **17 + 3 reallocations**, decode **31 + 0**. Peak live bytes
are Wire **1,048,590 / 1,048,576**, Candid **2,098,807 / 1,049,380** for encode/decode.

Rust/Cargo **1.99.1**, LLVM **23.1.1**; x86_64 native, wasm32-unknown-unknown;
opt-z/opt-3, fat LTO, one codegen unit, abort, stripped symbols. Candid **0.10.40**,
Postcard **1.1.3**, Platform **0.23.5**, Types **0.18.2**, Runtime **0.19.4**,
Testkit **0.89.4**, Build **0.76.0**, Host **0.74.4**. Prepared PocketIC **16.1.0**.
Lock SHA-256: `43bde528b6e4216f5eaf6f98d81e051fb8707406c658b84951a266fba9525cb9`.

Reproduce with the [maintained matrix command](development.md#candid-comparison-benchmarks):
two native runs, eleven samples, minimum 2 ms batches, three distinct-caller IC
samples, fresh target and explicit prepared server. IC counters include codec
work, result destruction and loop overhead; exclude transport, authentication,
ingress copies and physical cycles. Domain work inside adapters is included.
No full CI, installed Endpoint, consumer build or real big-endian proof is implied.
Older graphs are not regression baselines; feature-paired Candid modules can differ.

Raw evidence: `target/evidence/codec-matrix-20261107/`, source base **bcbcec5** plus
recorded working edits. All **171 source hashes**, **144 fresh module hashes/code
sizes**, **44,308 IC rows** with three equal repeats and **142,776 native/IC
input/frame checks** verify, including 12 representation pairs. Cleanup recovery
retained probes and rebuilt missing guests using the identical server; see `recovery-1/`.
No production bytes, schemas, identities, budgets or acceptance changed.
