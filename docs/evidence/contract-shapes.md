# Contract-shape Wire/Candid qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

This private benchmark extension adds 58 cases, appending standard workload
indices 333–390 without changing existing cases. It also exposes the same cases
as the private `contract-shapes` cohort. Production codec, schema, frame bytes,
normalization policy and resource budgets are unchanged. This is new coverage,
not a measured production optimization or a rerun of the entire Candid matrix.

## Workloads and controls

- Actual Platform Accounts: absent, explicit zero and nonzero 32-byte subaccounts;
  scalar and 0/1/16/128-entry vectors. Account equality normalizes absent and zero,
  so independent tests compare the raw owner/subaccount fields and distinct frames.
- Principal-keyed maps and Account sets at 0/1/16/128 entries. Both set decoders
  construct BTreeSets. Distinct owners preserve cardinality and iteration order.
- Individual records with 8/32/128 wide integer fields, plus optional-field records
  with none, a few or all fields populated.
- Enums with 8/32/129 variants and first/last values; explicit Postcard indices
  127/128 cross the varint boundary. Invalid index 129 is rejected.
- Blob batches with equal total content lengths 0/1/4,096/65,536 bytes spread over
  1/16/128 parts, plus ragged nested batches with empty inner collections.

Wire uses the owned Platform Types identities. Candid uses a private external
Account DTO with Candid Principal and identical owner/subaccount fields. Conversion
and input identity are outside timed/counted codec bodies. No Candid trait is
implemented for owned Platform identities. External Candid setup is excluded from
Wire-only artifacts. These are complete benchmark modules including setup/reporting
support, not per-type attribution or predicted application linker savings.

## Results

Wire wins **216/232** primary successful IC instruction comparisons,
with 16 Candid wins and no ties. Wire wins **58/58 decodes in each profile**.
Encode wins are 49/58 at opt-z and 51/58 at opt-3. This unweighted workload count
is not a production traffic estimate. All 1,230 IC operation rows have three
identical repeated counters (3,690 samples), with exact guest/native frames paired.

Opt-3 absolute instruction counts per operation and complete frame bytes:

| Workload | Encode Wire / Candid | Decode Wire / Candid | Bytes Wire / Candid |
| --- | ---: | ---: | ---: |
| Account, no subaccount | 964.25 / 35,595.25 | 949.5 / 48,105.5 | 42 / 57 |
| 128 Accounts, zero subaccounts | 482,927.25 / 104,255.5 | 569,479.5 / 1,636,131.5 | 8,077 / 8,349 |
| Principal map, 128 entries | 73,835.25 / 86,136.25 | 241,169.5 / 860,775 | 4,109 / 5,139 |
| Account set, 128 entries | 359,897.25 / 107,161.25 | 575,686.5 / 1,563,713.5 | 6,701 / 6,930 |
| 128-field wide record | 67,926.25 / 365,318.25 | 62,160.5 / 520,212.5 | 1,291 / 1,802 |
| 128 optional fields, all absent | 25,896.25 / 478,535.25 | 21,022.5 / 865,073.5 | 139 / 908 |
| 129-variant enum, variant 128 | 1,135.25 / 361,585.25 | 793.5 / 311,014.5 | 23 / 794 |
| 64 KiB across 128 blobs | 121,131 / 203,465 | 163,063 / 378,700 | 65,805 / 65,805 |

The largest successful-operation gap is explicit-subaccount Account batch encoding:
**482,927.25 versus 104,255.5 instructions (4.63×)** at 128 entries, for both
zero and nonzero subaccounts. Account-set encoding at 128 entries is **3.36×**
Candid. At 16 explicit-subaccount Accounts the opt-3 gap is only **1.13×**;
empty/singleton controls remain in the data. Ordinary fixed 32-byte subaccounts
are a useful next isolation experiment; this suite does not establish which
serializer/admission step dominates, or authorize changing their bytes.

Truncation is a separate weak path: a ragged 64 KiB batch costs **68,982 versus
8,843 instructions (7.80×)** to reject at opt-3. Sixteen blobs totalling 64 KiB
cost **73,766 versus 11,471 (6.43×)**. A late failure can occur after valid earlier
elements have been processed. No new primary or rejection IC ratio reaches 10×;
this does not remove the larger ordinary-vector gaps in the historical matrix.
All primary ratios and percentage differences are in Retired raw output (see retention notice),
with separate Retired raw output (see retention notice) and Retired raw output (see retention notice).

## Uncompressed Wasm

Both encoding and decoding remain reachable. Total module bytes and independently
parsed section-10 payload sizes come from the same final artifacts. This measures
the 58-case cohort, not one module per individual type.

| Profile | Wire total / code bytes | Candid total / code bytes | Wire total / code reduction |
| --- | ---: | ---: | ---: |
| opt-z | 226,987 / 193,245 | 688,585 / 629,135 | 67.04% / 69.28% |
| opt-3 | 297,355 / 265,780 | 996,081 / 938,529 | 70.15% / 71.68% |

Retired raw output (see retention notice) bind the four retained raw modules.
Benchmark expansion adds no production runtime code and claims no reduction in
a consumer's existing artifact.

## Native time and allocation

The account hotspot also appears in both warmed native runs. Opt-3 medians, in
nanoseconds, for 128 explicit-zero-subaccount Accounts:

| Operation | Wire run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: |
| Encode | 14,165.77 / 10,230.23 | 3,275.06 / 2,456.25 |
| Decode | 13,993.75 / 10,516.37 | 60,008.09 / 40,612.03 |

Native encode is 4.33× / 4.16× Candid for these samples. Run-to-run host variation
is visible; native timing is not interchangeable with deterministic IC counters.
The same path uses 1 Wire allocation and zero reallocations for encode, versus
32 Candid allocations and 14 reallocations. Decode uses 1/5 versus 574/129
allocations/reallocations. Peak live heap is 8,077 versus 18,982 encode bytes and
8,064 versus 9,747 decode bytes. Separate instrumented builds include different
allocator machinery; these are observations, not peak application memory claims.
Retired raw output (see retention notice) and Retired raw output (see retention notice)
retain all cases, profiles and rejection controls.

## Method

The frozen source selects Core/derive 0.14.2 with the pending 0.14.3 fixtures,
Platform/PocketIC client 0.17.0, Types 0.16.0, Testkit 0.78.0, Metrics 0.25.1,
Build/Host 0.73.6 and thiserror 2.0.21. The pre-existing working lock was retained,
not updated by this experiment. Candid is 0.10.39 and Postcard is 1.1.3.

Rust 1.99.1 / LLVM 23.1.1 targets wasm32-unknown-unknown. `evaluation` uses opt-z;
`benchmark-speed` uses opt-3. Both use LTO, one codegen unit, aborting panic and
stripped symbols, with no post-link optimization. Measurements do not select a
consumer optimization profile. All settings and source/input identities remain
in the archived runner output.

Native times use two serial warm runs, each with eleven calibrated batch samples
per operation and a two-millisecond minimum batch. The shared Linux host is not
CPU-isolated; these are codec microbenchmarks, not request tail latency. Allocation
counts and peak live heap use separate instrumented builds and one operation
following warmup, not repeated timing samples.

IC Testkit owns the explicitly prepared PocketIC 16.0.0 server. Three distinct
callers per group prevent query-cache reuse. Warm operation counters include loop
work and result destruction; controls are retained without subtraction. Input
setup, ingress/reply transport, authentication, cold start, application work and
physical cycle debits are excluded. Bad-header/truncation costs are separate from
successful encode/decode. Wire budgets and Candid quotas have different semantics.
