# Latest recorded Candid comparison

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

IC Wire has one production Serde/Postcard codec. Wire derives host layout
metadata; Serde owns serialization. `Packed<T>` is an explicit field representation.
Candid remains in the unpublished qualification package.

The 4 November comparison used Core 0.16.1 plus pending 0.16.2 qualification
changes with Platform 0.21 and Testkit 0.84.1. It reported **26 cohorts / 1,387
overlapping case entries / 5,548 successful encode/decode comparisons**, including
all **391 standard cases**, actual Platform identities and 1/10/100-type fixtures.
Wire won **5,005**; Candid won **543**, with no ties. Counts are not a traffic weighting or number
of unique Rust types. This adoption/benchmark batch changes no codec or protocol.

## Uncompressed Wasm footprint

Total / code-section bytes; complete modules, not projected application savings.

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 391 cases | opt-z | 591,251 / 544,459 | 1,429,335 / 1,353,653 |
| Standard, 391 cases | opt-3 | 891,840 / 831,250 | 2,086,822 / 1,996,418 |
| Toko-shaped, 10 cases | opt-3 | 103,644 / 93,144 | 608,968 / 575,325 |
| 100 ordinary derived types | opt-z | 310,606 / 291,870 | 1,435,140 / 1,380,148 |
| 100 ordinary derived types | opt-3 | 396,039 / 361,615 | 2,106,849 / 2,036,738 |

Wire is smaller in both metrics in **all 52** cohort/profile pairs. At 100
ordinary types, opt-3 total Wasm is **81.20% smaller**. The fixtures exercise
Serde codec derives versus CandidType/Serde derives; host-only schema generation
is separate. Compression is only evidence storage, never an optimization metric.

## Instructions, bytes and remaining losses

At opt-3 the standard corpus favors Wire on **318/391 encodes and 370/391
decodes**. The 109-case generic Packed cohort wins all **436** primary comparisons.
A byte-oriented 1 MiB blob uses **1,049,704 encode / 1,049,508 decode** instructions
versus Candid's **2,115,234 / 1,068,979**, with frames **1,048,590 / 1,048,588** bytes.

Ordinary vectors remain weak cases: worst encode is **119.50×** Candid at opt-z
and **49.67×** at opt-3. Ordinary bulk-control decode reaches **12.30× / 5.36×**.
The generic 1 MiB Vec<u8> encode is **35.75×** at opt-3; its truncated rejection
cost is **42,472,926** instructions versus **4,289**. Byte fields reject unavailable
ranges without traversing a long element prefix. Rejection policies differ.

Other encode hotspots include f32 lists (**8.68× / 2.17×** opt-z/opt-3), bool
lists (**4.67× / 3.43×**) and opt-z zero-sized unit lists (**9.47×**). Floats and
bools are not currently supported Packed words. Ordinary hash manifests also lose
badly; matched existing packed hash fields avoid that path. They add one frame
byte for 1,024 hashes, while small-u16 packing grows **141 → 269** bytes.
Existing layouts/bounds need coordinated adoption; Serde is never silently rewritten.

A 128 explicit-zero Account batch encodes at **1.175×** Candid at opt-3 and
decodes at **0.0624×** using the existing plain-array Candid DTO. That is not a
claim against every possible byte-adapted Candid DTO. Owned Types stay on the
Wire side; external Candid DTOs and conversion remain outside its codec boundary.

Ranked findings prioritize
hash-list advice and measured research into float/bool bulk fields. The
coverage audit identifies missing
Candid byte-adapted controls, standalone owned Principals, nested packed fields,
larger float/bool sweeps, deterministic hash collections, bounded external Nat
and standard-library types. Native codec correctness is not TypeScript schema
support or measured performance coverage.


The [94-case follow-up](evidence/primitive-bulk.md) adds two measured cohorts
without changing the preceding report. Its private float/bool byte-buffer fields
cut 1 MiB opt-3 encode to about 1.05M instructions versus Candid's 2.11M and reduce
whole-fixture uncompressed Wasm by 939 bytes opt-z / 1,564 opt-3. Tiny-input
regressions remain. This is research, with no public float/bool Packed words or
production byte/client change. Byte-adapted Candid Account/hash controls retain
identical frames and improve decode by about 27–28% in that linked context;
plain controls remain available. Do not combine its counts with the older full
run or treat every ordinary vector as automatically optimized.

## Measurement boundaries

The frozen source selects Candid 0.10.39, Postcard 1.1.3, Platform facade 0.21,
Runtime 0.19, Types 0.18, Testkit 0.84.1 and Rust/Cargo 1.99.1 / LLVM 23.1.1.
The explicit authenticated PocketIC server is standard 16.1.0. All **29,492 IC
operation rows** have three fresh-caller samples with identical counters; all
**104** module hashes, real code-section sizes and fresh compiler records verify.

Both optimization profiles retain two serial native runs of eleven batch samples
and separate allocator/heap builds. IC counts cover warm codec bodies and result
destruction/loop overhead, excluding transport, authentication, application work,
physical cycles and ingress copying. Native buffer-placement effects are separate.
Full Wasm, instructions, native time, allocations, heap and frame bytes must not
be combined into one speed/size claim. Older reports have different source,
supplier and linked contexts; this is a fresh baseline, not matched attribution
to the Platform adoption or a regression delta from the earlier matrix.

`make check-benchmarks` checks correctness, lint and compiler support. It does not
execute a measured performance gate. Full evidence and reproduction
retain all results and declared limits. No full repository CI or new native macOS
qualification is implied by this Linux experiment.

## Historical evidence

- 1 November full comparison: 333-case
  standard corpus and 22-cohort matrix on its original graph.
- [Contract-shape follow-up](evidence/contract-shapes.md): original
  58-case expansion, before the Account cut.
- [Platform/Account cut](evidence/account-platform-cut.md): matched supplier
  and consumer contract adoption, kept separate from this refresh.
- [Byte-array key study](evidence/byte-array.md): original explicit-key
  tradeoffs and empty controls on its frozen graph.
- [Wire/Candid/Tripwire ingress](evidence/tripwire-comparison.md):
  complete physical updates with a different compiler/server and focused blob shape.
- 0.7.25 full comparison,
  [0.7.10](full-comparison-0710.md), [0.6.13](full-comparison-0613.md) and
  [earlier experiments](history/candid-comparison.md) retain their original scopes.
