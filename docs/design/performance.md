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

Each guest encodes and decodes 4,096 u32 values, either all 1, one u32::MAX per
16 elements, or all u32::MAX. Wire uses payload encoding. Candid encodes one
argument. The packed guest starts with an owned packed field. Input construction,
round-trip assertions and raw reply handling occur outside the counter intervals.
Counts include dropping the output. Each table cell reports the median encode /
decode instructions over three query calls from distinct callers. Wasm bytes are
whole uncompressed guest modules including the probe. These numbers describe this
fixture on Rust 1.100.0 and Candid 0.10.40.

| Profile | Representation | Wasm bytes | Small | Sparse wide | Wide |
|---|---|---:|---:|---:|---:|
| opt-z | Ordinary Wire | 21,108 | 332,767 / 512,305 | 461,279 / 556,593 | 2,388,959 / 1,220,913 |
| opt-z | Packed Wire | 20,677 | 17,623 / 111,424 | 17,623 / 111,424 | 17,623 / 111,424 |
| opt-z | Candid | 258,732 | 52,555 / 321,902 | 53,066 / 321,902 | 53,066 / 321,902 |
| opt-3 | Ordinary Wire | 22,801 | 291,517 / 236,640 | 341,693 / 261,472 | 1,094,333 / 633,952 |
| opt-3 | Packed Wire | 21,225 | 17,159 / 29,302 | 17,159 / 29,302 | 17,159 / 29,302 |
| opt-3 | Candid | 358,278 | 47,113 / 354,991 | 47,549 / 354,991 | 47,549 / 354,991 |

Ordinary Wire payloads were 4,098 / 5,122 / 20,482 bytes in distribution order.
Packed payloads were 16,387 bytes throughout and Candid messages were 16,395.
Use observed value widths and the application's byte and instruction budgets
when selecting a field layout. Changing an existing field changes its contract.

The separate warmed Linux allocator probe observed one encode allocation for
both Wire representations. Ordinary decode took one allocation and ten growth
reallocations, while packed decode took one allocation of 16,384 bytes. Candid
took 16 encode allocations and three reallocations, then 23 decode allocations.
The runner retains native allocation details, all counter samples, artifact
hashes and process cleanup evidence in its printed output directory.
