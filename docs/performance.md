# Native codec controls

The unpublished evaluation package retains nine logical cases and seven private
lanes for finite/dynamic Wire, raw Postcard, bitcode and wincode. Inputs include
scalar extremes, empty/4 KiB blobs, UTF-8, integer distributions and bounded rows.
Every sample checks a roundtrip. Wire representations match exact frames;
Postcard controls match the payload after the eleven-byte header.

`make check-evaluation` runs the native correctness checks. For an optional
timing experiment, prepare the tools in [local development](development.md), then:

```sh
make evaluate-performance
```

The command requires a fresh `target/native-performance` directory and refuses
to overwrite an existing experiment. The retained report records the current
compiler identity, native architecture/OS, evaluation profile, flags, lock/source
hashes, explicit codec parameters, payload hashes and seven batches of 32 operations.
Setup and correctness assertions are outside the timed loops; allocation,
destruction and clock/loop overhead remain included.

These are native nanoseconds. No simulator, installed instruction-count or Wasm
size experiment is maintained here. Raw controls do not implement Wire's resource
admission and canonical validation, so the results cannot establish an equivalent
fastest-codec claim or an end-to-end deployment saving. No timing result is claimed
without a report bound to its actual inputs.

## MessagePack comparison

See the [Blob-shaped MessagePack/Postcard experiment](messagepack-comparison.md)
for 27 workloads, two optimization profiles, raw samples, external IC instruction
counts and independently linked Wasm sizes. `make evaluate-messagepack` runs the
optional native portion into a fresh `target/messagepack-comparison` directory.

The subsequent [Wasm runtime optimization](runtime-optimization.md) retains
per-candidate instruction and size evidence, including regressions and rejected
native inlining attempts. Its baseline is the completed comparison above.

See [borrowing, buffers and bounded layouts](allocation-comparison.md) for the
44-case native/IC experiment and its consumer-schema tradeoffs.
