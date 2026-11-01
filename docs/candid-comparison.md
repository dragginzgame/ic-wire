# Current Candid comparison

IC Wire has one production Serde/Postcard codec. Wire derives host layout
metadata; Serde owns serialization. `Packed<T>` is an explicit field representation.
Candid and PocketIC remain in the unpublished benchmark package.

The [1 November full comparison](evidence/candid-comparison-0133/README.md)
qualifies Core 0.13.2 with pending 0.13.3 benchmark/type-coverage additions.
It covers **22 cohorts / 1,243 case entries / 4,972 primary comparisons** across
both optimization profiles, including the expanded **333-case** standard corpus.
IC instructions, wire bytes, repeated native timing, allocation/heap observations
and raw Wasm/code-section sizes remain separate. Production codec and protocol
are unchanged by this benchmark refresh.

## Wasm footprint

Opt-3 whole benchmark modules, in bytes:

| Fixture | Wire total / code | Candid total / code |
| --- | ---: | ---: |
| Standard, 333 cases | 662,430 / 606,280 | 1,610,988 / 1,527,857 |
| Toko-shaped, 10 cases | 103,809 / 93,308 | 608,980 / 575,323 |
| 100 ordinary derived types | 398,675 / 364,245 | 2,106,847 / 2,036,736 |

Wire has smaller raw Wasm and code sections in **all 44** cohort/profile pairs.
At 100 ordinary types, total Wasm is **81.08% smaller**. These complete-module
comparisons are not projected net savings in an application retaining external
Candid endpoints. Both profiles are retained in the full report.

## IC instructions and frame bytes

Wire wins **4,462 / 4,972** primary successful instruction comparisons; Candid
wins 510, with no ties. Overlapping controls are not weighted by production
traffic. Standard opt-3 favors Wire on **264 / 333 encodes and 312 / 333 decodes**.
All 35 new cases decode faster with Wire at both profiles; 31 / 35 new encodes
win at opt-3 and 29 / 35 at opt-z.

A byte-oriented 1 MiB blob encodes in **1,049,704 instructions** versus Candid's
**2,115,234**. Its owned decode uses **1,049,508** versus **1,068,979**. Wire's
frame is two bytes larger: 1,048,590 versus 1,048,588 bytes. Borrowed counters
begin with an already-owned input slice and exclude bringing ingress into memory.

Large ordinary vectors remain explicit weak cases. The worst opt-z successful
encode is wide u16 at **121.39× Candid**; the opt-3 maximum is the ordinary u16
bulk control at **51.09×**. Generic 1 MiB `Vec<u8>` encode remains **37.23×**
Candid at opt-3. Ordinary bulk-control decode reaches **12.30×** at opt-z and
**5.36×** at opt-3. The new 128-entry binary-key map also has a **4.34×** opt-3
encode loss, despite a smaller frame and faster decode.

Byte-oriented fields and explicit packed representations avoid many collection
losses. The 109-case generic Packed cohort wins all **436** primary comparisons
across both profiles. Its matching wide-i16 fixture uses **33,489.25 encode /
33,476 decode instructions**, versus Candid's **80,320.25 / 1,411,787**, with two
more frame bytes. Packed fields can expand small values; select representations
for the workload, coordinate existing contracts and retain retry bytes. See the
[field selection guide](packed-fields.md).

Rejection cost is separate. At opt-3, rejecting a truncated generic 1 MiB
`Vec<u8>` costs **42,472,926** instructions versus Candid's **4,289**. The report
retains paired bad-header and truncation costs. Ordinary sequence parsing can
traverse a long valid prefix before finding EOF; byte-oriented fields can reject
unavailable ranges before copying them.

The follow-up [standard byte-array key study](evidence/byte-array-0133/README.md)
keeps the ordinary controls and adds explicit `ic_wire::ByteArray<32>` keys.
At opt-3 its matched 128-entry map reduces encode instructions 78.50% and decode
68.32%; both beat Candid. It adds 128 frame bytes and 1,526 raw Wasm bytes (2.11%)
in that focused fixture; opt-z adds 1,990 Wasm bytes. Empty opt-3 decode regresses
18 instructions (4.80%). This is a speed/size tradeoff with coordinated contract
adoption, not a fix for the other ordinary-vector losses. That paired study uses
its own frozen Metrics 0.24.4 graph; the full matrix above remains unchanged.

## Measurement boundaries

The frozen run selects Candid 0.10.39, Postcard 1.1.3, Platform 0.15.3,
Testkit 0.75.2 and Metrics 0.24.3, with Rust 1.99.1 / LLVM 23.1.1 and the explicit
PocketIC 16.0.0 server. A later checkout update to Metrics 0.24.4 is recorded
separately and was preserved. The report binds measurements to the frozen graph.
All 26,440 IC operation rows have three unique-caller samples with identical
counters; input/frame pairing and all 88 module hashes pass strict analysis.

Two serial native runs per profile retain eleven warm batch samples per
operation. Allocation counts and peak live heap use separate instrumented builds
and one measured operation after warmup. Native time, IC instructions, allocator
metrics, wire bytes and artifact sizes are not interchangeable. IC counters cover
warm codec bodies, excluding transport, authentication, business logic, cold start
and physical cycle debits. Gzip size is not an optimization objective.

`make check-benchmarks` checks correctness, lint and compiler-floor support;
it does **not** execute a performance regression gate. Reproduction commands,
frozen source, raw samples, per-case CSVs and raw Wasm archives are in the
[full evidence](evidence/candid-comparison-0133/README.md). Resource policies and
schema-evolution guarantees differ. This is a fresh paired Wire/Candid baseline,
not a matched regression test against the previous dependency/server/corpus graph.
Codec roundtrips do not qualify every Rust type for generated TypeScript metadata.

The separate [Wire/Candid/Tripwire ingress study](evidence/tripwire-comparison-0725/README.md)
retains complete physical-update measurements using its own nightly compiler and
focused blob shape. Do not subtract absolute values across these studies.

## Historical evidence

- [0.7.25 full comparison](evidence/candid-comparison-0725/README.md): previous
  298-case standard corpus and 22-cohort study, retaining its original inputs.
- [0.7.10 full comparison](full-comparison-0710.md): matched previous-release
  qualification and its different dependency/server graph.
- [One-pass decoding cut](evidence/one-pass-070/README.md): consumer-owned
  collection/custom-Serde normalization.
- [Expanded 0.6.13 matrix](full-comparison-0613.md) and
  [hotspot follow-up](hotspot-encoding-0613.md): earlier encoder/decoder results.
- [Packed promotion](packed-promotion-0611.md), [design study](bulk-fields-0611.md)
  and [sharing study](packed-sharing-0611.md): field representation decisions.
- [Earlier comparisons](history/candid-comparison.md): retired finite-codec and
  rejected experiments, reproducible at their recorded revisions.
