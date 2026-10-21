# Maintained performance probes

The unpublished `ic-wire-qualification` package retains the 44-case allocation,
borrowing and buffer-reuse fixture. `make check-qualification` checks its behavior;
`make evaluate-allocation` measures it into a fresh output directory. See
[local development](development.md#allocation-timing-modes) for codec-only timing.
The [protocol probe](protocol-study.md) and the generator probe below remain
available for matched runtime and host-generation experiments. These timings do
not establish deployed endpoint latency or IC instruction costs.

## Retired alternative-codec comparisons

The nine-case Postcard/bitcode/wincode suite and the 27-workload MessagePack suite
were retired in 0.5.19. Their feature flags, dependencies and
`evaluate-performance` / `evaluate-messagepack` targets are removed. Postcard
remains the selected encoding; future optimization work focuses on that contract.
For the last maintained source and lockfile, use revision
`bd9e5bc0ac2c6a057570ae5aae55e0c1f431488b` in a separate checkout. Commands in old
reports belong to their recorded source revisions, not the current workspace.

The [MessagePack comparison](messagepack-comparison.md), its raw evidence and the
subsequent [runtime optimization study](runtime-optimization.md) remain historical
records of measured tradeoffs. The [0.4 allocation study](allocation-comparison.md)
and [bounded allocation study](bounded-allocation-study.md) likewise retain their
original results; current qualification is described in [evaluation](evaluation.md).

## TypeScript generator comparisons

The host-only [generator probe](../scripts/benchmarks/generator/mod.rs) measures
`generate_registry` for records, enums and method modules at 8, 128 and 1,024
entries. Copy it to `crates/ic-wire/examples/generator_bench.rs` in disposable
baseline/candidate snapshots, then build each with:

```sh
CARGO_NET_OFFLINE=true cargo build --locked --release -p ic-wire \
  --features typescript --example generator_bench
mkdir -p probe-output
./target/release/examples/generator_bench probe-output > samples.csv
```

Use the same lockfile and probe for both builds and retain each executable before
changing sources. The output directory receives generated TypeScript for byte
comparison/hashing. CSV columns are kind, entry count, batch, repetitions, output
bytes and nanoseconds per generation. Setup, output writes and input construction
are outside timing; generation allocations and destruction are included. Each
case warms 100 times, calibrates to at least 20 ms (4,096-call cap), and records
11 batches. Compare medians, repeat runs, and report small-case regressions too.

The released 0.5.16 native-registry validation change was compared with 0.5.15
(`39535ba`), using the same working-tree lockfile in both isolated builds.
[Median results](evidence/typescript-generator/native.csv) and
[all samples](evidence/typescript-generator/samples.csv) retain two runs ordered
baseline/candidate/candidate/baseline. Linux x86_64, AMD Threadripper 7970X,
Rust 1.99.1 (`58cf8f9c3`, LLVM 23.1.1), default Cargo release profile (opt-level 3,
no LTO), no RUSTFLAGS overrides, and CPU 0 affinity were used. No other benchmark
ran concurrently. The measurements are native host generation time, not codec
latency, IC instructions, Wasm size or allocation counts.

For 1,024 entries, record generation improves 82–87%, enum generation 89%, and
method-module generation 82–84%. Eight-entry controls range from 7% faster to 5%
slower. Generated output is identical for all nine cases. The finite-schema
projection cleanup and early oversized-enum refusal are qualified by tests but
are not timed by this registry probe; no measured savings are attributed to them.

Fields/variants are named `field0` onward, carry sequential IDs from 0 and unit
payloads. Methods are named `method0` onward, have matching decimal-string IDs,
query mode, unit request/response, 11-byte allowances and node/depth allowances
of 1. The runtime module is `./wire-runtime.js`; records/enums use type `Payload`.
Sources, binaries, compiler identity and logs remain in
`/tmp/ic-wire-generator-bench/`. SHA-256 identities:

- Shared dependency lock: `5662d8e01c56e0dcebab91167976f6163bf543fe89bbf620bd065d893832a27c`.
- Probe: `1ffcd3388af6f967570db15927312abbd2a7ddeeff1c3531652ac31b2467e6ef`.
- Candidate `typescript/mod.rs`: `ce7ad944dd26bea115ac2861a7f4334c94537863485858f95302fedafc1ad65e`.
- Candidate `typescript/metadata/mod.rs`: `21196bd6f4938436d8a4b556b70007040502bf08f8e55a4a14816b626ee93294`.

See the [layout and canonical-decoding study](protocol-study.md) for schema byte
tradeoffs, stage timings and the native borrowed-buffer comparison shortcut.
