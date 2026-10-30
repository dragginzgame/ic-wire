# Current Wire versus Candid and Tripwire: scoped ingress probe

Measured 30 October 2026; current Wire 0.7.24 source, pending 0.7.25 evidence.
Production code, dependencies and protocol are unchanged. This is a standalone
qualification fixture, not an additional production codec or SDK adoption.

Tripwire is the actual unmodified library from
[CDK-RSX `d0e53afc`](https://github.com/Neutrinomic/cdk-rsx/tree/d0e53afc7b7fdc4b7cf4e2ccbe1e55a386133c4b/ver1/crates/cdk-rsx-tripwire).
The [harness's twenty-case study](https://github.com/Neutrinomic/cdk-rsx-harness/blob/62fd6147169994cdac06109de2c2ecfeb9c6e6e4/research/benchmarks/bench-2026-10-29-tripwire-20.md)
already compares published IC Wire 0.7.21 with Tripwire and an optimized Candid
fork through the full CDK-RSX runtime on wasm64. Its Wire lane is IC Wire, not a
different serializer. Those historical numbers are not the measurements below.
This fresh probe isolates codec entry/ownership choices on wasm32, includes
current borrowed Wire, and uses upstream Candid 0.10.39 through Platform 0.8.3.

## Complete-call instructions

All **1,280 measured physical updates** passed complete response checks: five
lanes × sixteen workloads × eight varied inputs × two profiles. Ten initial
empty-call warmups are excluded. The spread describes these varied inputs, not
a statistical confidence interval. Means below include argument copying, decoding,
selected business work, destruction, reply and metered heap-page charges.
They are not body-only performance counters. Page charges are already included.

One MiB body, `opt-level=3`; means rounded to whole instructions:

| Work | Wire owned | Wire borrowed | Candid owned | Tripwire owned | Tripwire selective |
| --- | ---: | ---: | ---: | ---: | ---: |
| Read and sum all bytes | 15,848,756 | 12,239,515 | 15,892,265 | 12,239,426 | 10,111,508 |
| Read first 256 bytes | 7,704,145 | 4,094,977 | 7,747,654 | 4,094,889 | 250,590 |
| Business rejection from scalar fields | 7,702,120 | 4,092,952 | 7,745,629 | 4,092,864 | 163,335 |
| Generate, encode and return full body | 70,092,010 | 70,112,081 | 74,041,928 | 71,159,808 | 71,159,876 |

Compared with borrowed Wire, selective Tripwire uses 17.39% fewer instructions
for the full scan, 93.88% fewer for the prefix, and 96.01% fewer for business
rejection. Generate/encode/reply is **1.49% more expensive**. At opt-z that last
regression is **19.66%**: 85,319,790 versus 102,094,481 instructions. Generation
and reply transport are included, so this is not isolated encoder throughput.

Small controls are retained, not extrapolated from the large case. At opt-3 a
32-byte full scan costs 484,625 instructions for borrowed Wire, 528,547 for Candid
and 248,511 for selective Tripwire. Included page charges are 480,000, 480,000
and 240,000 respectively. For the one MiB full scan those page charges are
3,060,000, 5,620,000 and 240,000. These are execution charges, not allocated-byte
counts; the report does not present them as heap allocation measurements.

Instruction savings are not proportional cycle savings. The one MiB prefix
case's observed debit is **2,107,749,477 → 2,103,884,590 cycles** for borrowed
Wire → selective Tripwire, a **0.183%** reduction. Every lane still submits the
whole incoming payload. Cycle balances reflect this pinned simulator/configuration,
not a general mainnet price estimate. Per-input samples, min/max, included page
charges and cycle debits are in [opt-3](benchmark-speed.json) and [opt-z](evaluation.json).

## Uncompressed Wasm

Each separate module has the same two raw update methods and one request shape.
Both decoding and encoding are reachable. The fixture uses raw synchronous
exports; it does not exercise an executor, persistent root or deployment adapter.
These are whole module sizes, not application
linker deltas or a 100-independent-type footprint claim.

| Lane | Opt-z total | Opt-z code section | Opt-3 total | Opt-3 code section |
| --- | ---: | ---: | ---: | ---: |
| Wire owned | 30,991 | 27,872 | 22,936 | 20,570 |
| Wire borrowed | 30,721 | 27,603 | 22,806 | 20,440 |
| Candid owned | 310,741 | 275,759 | 407,210 | 371,662 |
| Tripwire owned | 26,114 | 23,108 | 21,883 | 19,231 |
| Tripwire selective | 25,981 | 23,071 | 21,024 | 18,620 |

Selective Tripwire is 1,782 bytes smaller than borrowed Wire at opt-3 (7.81%)
and 4,740 bytes smaller at opt-z (15.43%). Gzip size is not measured or used.

## Request bytes and acceptance

The logical request is `mode: u8, nonce: u64, body: bytes`. Nonces cover 0, 127,
128, 16,383, 16,384, u32::MAX, u64::MAX−1 and u64::MAX. Body byte i is
`(i * 17 + nonce) mod 256`. Wire's varints explain its size range:

| Body bytes | Wire frame | Candid frame | Tripwire frame |
| --- | ---: | ---: | ---: |
| 0 | 14–23 | 39 | 9 |
| 32 | 46–55 | 71 | 41 |
| 32,768 | 32,784–32,793 | 32,809 | 32,777 |
| 1,048,576 | 1,048,592–1,048,601 | 1,048,617 | 1,048,585 |

Wire uses ID 77 and its existing eleven-byte request header; Tripwire uses a
one-byte mode, little-endian eight-byte nonce, then the body, without a
transmitted identity or length. Fixed-width integers can cost more than Wire
varints in other shapes with many small numbers; this blob probe does not measure
that tradeoff. This describes existing formats; none changes.
The analyzer independently reconstructs Wire and Tripwire frame hashes.

Wire and Candid copy the entire bounded ingress through Platform’s production
`cdk::api::msg_arg_data()` helper, which initializes final storage directly
without a preceding zeroing pass. Wire owned and Candid then
materialize an owned body and drop the raw input before business work. Borrowed
Wire validates the complete frame and lends its body without that second copy.
Tripwire owned copies the selected body directly into final owned storage.
Selective Tripwire reads the fixed area, then uses 4 KiB stack scratch for a
full scan, reads 256 bytes for the prefix, or skips the body on business rejection.
It does not promise eager validation of unused fields. These successful-work and
business-rejection comparisons do not establish equivalent malformed-input rules.

Consume replies are the same independently checked 24 raw bytes for every lane.
The separate encode method receives a thirteen-byte size/nonce/mode recipe,
generates the body and returns that codec's actual encoded request bytes. This
keeps response framing separate from the input-ownership comparison. It is not
a test of generated endpoint integration, async calls, arbitrary Serde types,
nested Tripwire lists, UTF-8, schema evolution or the complete SDK.

## What this suggests for Wire

The strongest measured opportunity is selective access to large ingress bodies,
not a universally faster serializer. Tripwire's owned full-scan path is already
almost identical in instruction cost to borrowed Wire, while selective prefix
reads avoid bringing the remaining body into Wasm memory. A similar Wire design
would need an explicit input-source and validation contract; it is not a drop-in
replacement for Serde deserialization from a complete slice. This study does not
justify changing Wire's existing full-input validation or adding a second codec.

The existing borrowed endpoint path remains useful without that redesign. Before
pursuing selective input, measure a real consumer's mix of full reads, early exits
and replies: the reply-generation regression and small observed cycle savings
can matter more than the prefix-read instruction ratio.

## Reproduction and evidence

All five lanes use the same locked graph, compiler and profile: Tripwire's pinned
nightly-2026-10-15 (Rust 1.101.0-nightly), wasm32, LTO, one codegen unit, panic abort,
stripped symbols, with opt-z and opt-3. This compiler differs from the stable
compiler in the broad [Candid matrix](../candid-comparison-0725/README.md);
absolute results from those studies must not be subtracted from each other.

[receipt.json](receipt.json) records source identities and runtime hashes;
[upstream-files.json](upstream-files.json) records verified upstream Git blobs.
The original run downloaded only four transitive SDK crates and narrowed workspace
membership; runtime and derive files matched the upstream blobs exactly.
[fixture](fixture/lib.rs), [host driver](fixture/measure.rs), frozen dependency
lock and [analyzer](analyze.mjs) retain the experiment. [samples.jsonl.gz](samples.jsonl.gz)
retains all call/physical-execution receipts; compression is evidence storage only.
The analyzer rejects missing/truncated traces, missing counters, wrong artifacts,
wrong results, duplicate executions and incomplete sample counts.

Prepare the pinned nightly and wasm32 target, cached locked dependencies, a
complete SDK checkout at the recorded revision, frozen Wire source, the existing
`codec-report` tool and the instrumented PocketIC server recorded in the JSON.
Then run serially, outside other native timing work:

```sh
POCKET_IC_BIN=/path/to/prepared/instrumented-pocket-ic \
CODEC_REPORT=/path/to/prepared/codec-report \
bash docs/evidence/tripwire-comparison-0725/reproduce.sh \
  /absolute/fresh-output /absolute/frozen-wire /absolute/pinned-cdk-rsx
```

Raw original builds, logs and per-canister trace responses remain in
`/tmp/wire-tripwire-0725-production/`. No sibling checkout was changed, no alternative codec
was added to Wire, and nothing was published.

The initial zero-filled ingress probe is retained under [zeroing-control](zeroing-control/README.md).
It is not the headline comparison: current Platform already avoids that extra pass.
