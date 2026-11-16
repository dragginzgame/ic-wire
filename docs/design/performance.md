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
