# Performance measurement scope

Measure the current implementation with the real standalone consumer. Report
opt-z total uncompressed Wasm first, then opt-3, code-section bytes, payload bytes,
allocation behavior and IC instructions separately. No historical implementation
is installed or used as a baseline. Correctness checks establish no speed claim.

## Standalone Wasm

Run `make measure-consumer-wasm` from the repository root with locked dependencies,
Rust 1.100.0, the Wasm target and Node prepared. It builds the upload consumer at
`wire-size` (opt-z) and `wire-speed` (opt-3) in fresh target directories, checks
zero host imports and executes each workload's assertions. Reports under
`qualification/target/wasm-measurement-*/` retain compiler identity, input hashes,
artifact hashes and total/code/other byte counts. Profile and compiler flag
overrides are rejected to keep runs reproducible. Gzip size is not measured.

The separate finite-f32 workload uses:

```sh
node qualification/ic-wire-consumer/measure-wasm.mjs wire-float-consumer
```

It checks finite values by bits, including signed zero and subnormals, and rejects
nonfinite words. Its size does not stand in for the upload workload or IC costs.

## Installed instructions

`POCKET_IC_BIN=/absolute/path/to/pocket-ic make measure-consumer-ic` measures the
upload consumer in a local simulator at both profiles. The
[native runner](../../qualification/ic-wire-instructions/README.md) uses published
Testkit for original PocketIC types and explicit bounded server/instance
ownership. It records three batches of eight repetitions for
request/response encode/decode, bad headers, truncation and loop control. Reports
retain samples and median batches without subtracting loop overhead. Every query
also checks independently specified 34-byte request and 14-byte response frames.

The runner reports measurements without a historical performance threshold.
Any future regression gate needs a separately reviewed workload and explicit
budget. These runs do not deploy a service or measure network/consensus latency.

## Allocation and limits

`make test-allocation` measures selected allocation boundaries in an isolated
process. It checks preflight refusal, output reservation, borrowed spans, field
bounds and domain validation. It is not a total-heap benchmark or a guarantee
about arbitrary custom Serde. See [qualification](../../qualification/README.md).
Native timing, full schema generation cost, cross-engine browser speed and actual
big-endian execution require their own evidence.

## Numeric list comparison

Run the private installed comparison from the repository root with a prepared
PocketIC 16.1.0 executable:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic bash qualification/ic-wire-instructions/run.sh --numeric
```

Each guest round-trips lists of 1, 16, 127, 128 and 4,096 u32 values in that
order within each query. For each length it measures all ones, one maximum
value every 16 elements starting with the first, and all maxima. Wire encodes
payloads and Candid encodes one argument. Input construction and round-trip
assertions precede the counters. Intervals include destruction of the result.
Allocator and Wasm linear-memory state carry over within the query. Measurements
are medians of three calls from distinct callers on PocketIC 16.1.0, Rust
1.100.0 and Candid 0.10.40.

At 4,096 elements the whole uncompressed guest sizes include the multi-length
probe. Instruction pairs are encode / decode.

| Profile | Representation | Wasm bytes | Small | Sparse wide | Wide |
|---|---|---:|---:|---:|---:|
| opt-z | Vec | 21,592 | 332,767 / 512,321 | 461,279 / 556,609 | 2,388,959 / 1,220,929 |
| opt-z | List | 22,115 | 332,825 / 525,126 | 464,409 / 569,414 | 2,438,169 / 1,233,734 |
| opt-z | Packed | 21,211 | 17,623 / 111,424 | 17,623 / 111,424 | 17,623 / 111,424 |
| opt-z | Candid | 259,180 | 133,658 / 322,057 | 53,508 / 321,932 | 53,285 / 321,902 |
| opt-3 | Vec | 23,172 | 291,517 / 236,640 | 341,693 / 261,472 | 1,094,333 / 633,952 |
| opt-3 | List | 24,158 | 291,517 / 257,220 | 341,693 / 282,056 | 1,094,333 / 654,536 |
| opt-3 | Packed | 21,874 | 17,159 / 29,302 | 17,159 / 29,302 | 17,159 / 29,302 |
| opt-3 | Candid | 358,649 | 128,141 / 355,146 | 47,984 / 355,020 | 47,762 / 354,991 |

For all-ones input, the bounded types have the following opt-z costs.

| Elements | List bytes | Packed bytes | List instructions | Packed instructions |
|---:|---:|---:|---:|---:|
| 1 | 2 | 5 | 1,075 / 996 | 1,121 / 805 |
| 16 | 17 | 65 | 2,290 / 4,220 | 1,181 / 1,210 |
| 127 | 128 | 510 | 11,281 / 19,331 | 1,680 / 4,228 |
| 128 | 130 | 514 | 11,429 / 19,500 | 1,684 / 4,255 |
| 4096 | 4,098 | 16,387 | 332,825 / 525,126 | 17,623 / 111,424 |

At 4,096 elements, Vec and List payload sizes matched across distributions
(4,098 / 5,122 / 20,482 bytes), versus 16,387 for Packed and 16,395 for Candid.
On Linux, each Wire encoder allocated once. Decoding 4,096 ones yielded one
allocation followed by 10 reallocations for Vec, 12 for List and zero for Packed.
For this input Candid used 16 allocations and three reallocations on encode,
then 23 allocations on decode. See the runner's allocations.txt for every length
and instructions.json for all samples and artifact hashes.

### Bounded list initial allocation

A matched run compared the current first growth of one element with an initial
capacity of up to four elements, targeting 16 bytes and capped by the field
bound. Both grew only after admitting an element. For 4,096 small u32 values
the candidate reduced native growth reallocations from 12 to 10. A singleton
requested 16 bytes instead of 4. Payload lengths matched across the probes.

| Bounded u32 probe | Current | Candidate |
|---|---:|---:|
| opt-z whole Wasm bytes | 22,115 | 22,115 |
| opt-z decode instructions, 16 ones | 4,220 | 3,577 |
| opt-z decode instructions, 4,096 ones | 525,126 | 524,483 |
| opt-3 whole Wasm bytes | 24,158 | 24,366 |
| opt-3 decode instructions, 4,096 ones | 257,220 | 264,952 |

Retain one-element first growth. The candidate improved medium-length decode,
but the long-list opt-z gain was small relative to the singleton memory increase
and the opt-3 regression. Local artifacts are in
`qualification/target/ic-counter-FW107X/` and `qualification/target/ic-counter-ycDm7K/`.

## Packed reads and allowance diagnostics

The standalone upload and finite-float workloads ran under Rust 1.100.0 on
wasm32-unknown-unknown. Each cell gives uncompressed module bytes / code-section
payload bytes. The middle build isolates exact-width reads. The last build adds
the first refused allowance and shares the meter debit across callers.

| Workload and profile | Before | Packed reads | With diagnostics | Combined change |
| --- | ---: | ---: | ---: | ---: |
| Upload, opt-z | 46,856 / 39,551 | 46,814 / 39,509 | 46,397 / 39,063 | −459 / −488 |
| Upload, opt-3 | 48,140 / 41,531 | 48,385 / 41,775 | 49,619 / 42,953 | +1,479 / +1,422 |
| Finite float, opt-z | 21,632 / 18,620 | 21,612 / 18,600 | 23,181 / 20,099 | +1,549 / +1,479 |
| Finite float, opt-3 | 22,160 / 19,356 | 22,154 / 19,350 | 23,879 / 21,004 | +1,719 / +1,648 |

The size tradeoff buys typed diagnostics on a refused traversal. It depends on
which codec operations and error paths the consumer retains. The measurement
runner executed every artifact. Its source digests for the three columns start
with `e8335fa30e95`, `dcaa858f0a7c` and `c2b472d62ae6`.

The upload probe on PocketIC 16.1.0 reported the following median IC
instruction counts for batches of eight operations across three calls. Counts
include loop overhead. The bare request and reply payloads were 23 and 3 bytes
respectively (34 and 14 bytes framed). The isolated native allocation probe
passed its eight existing groups, including refusal before output allocation.

| Profile | Encode request | Decode request | Encode reply | Decode reply | Loop control |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z | 48,222 | 45,758 | 18,854 | 5,926 | 606 |
| opt-3 | 27,192 | 28,624 | 9,096 | 4,464 | 432 |
