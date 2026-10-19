# Evaluation scope

The maintained qualification exercises the selected Postcard codec and its
generated TypeScript implementation. Endpoint adapters, simulators, browser SDKs
and installed canister drivers are outside this repository.

## Maintained checks

| Target | Evidence |
| --- | --- |
| `test` | Runtime/native bounds, canonical bytes, typed failures and retry buffers |
| `check-typed-diagnostics` | Actual compile-fail subjects and a passing control |
| `check-typescript` | Generated types, Rust vectors and differential malformed-input checks |
| `check-evaluation` | Native roundtrips against private raw-codec controls and strict linting |
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.91 floor |

The maintainer-requested [MessagePack comparison](messagepack-comparison.md) adds
an optional raw-codec experiment with retained native, external IC instruction
and linked Wasm size evidence. `make evaluate-messagepack` reruns its native
portion; external installed probes remain outside the maintained workspace.

Private raw Postcard, MessagePack, bitcode and wincode controls have different admission and
canonicality guarantees. They are not production codec modes. Optional
[native timings](performance.md) must retain compiler, target, profile, inputs,
lock selection and codec parameters. Native nanoseconds do not establish IC
instruction counts, deployment size or end-to-end latency.

Run focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` scaffold gate is reserved for an explicit local request
or the maintainer's release flow. Wire does not qualify sibling endpoints,
authentication, storage formats or delivery policy.

See [borrowing, buffers and bounded layouts](allocation-comparison.md) for the
44-case native/IC experiment and its consumer-schema tradeoffs.

The 0.5 borrowed-contract [allocation probe](evidence/borrowed-contracts-0.5/allocations.csv)
compares owned and borrowed finite decoding of the same name/chunk frames. On
Linux x86_64 with Rust 1.99.1, opt-z/LTO/one codegen unit/panic abort, a single-thread
counting allocator observes zero requests for borrowed views. Owned decoding of
a 1 MiB chunk plus the five-byte name requests 1,048,581 bytes in two allocations.
Construction of the source/input frame is outside the count. These are allocation
request counts/bytes, not peak heap or timing measurements. Source, lock, compiler
identity and logs are retained at `target/borrowed-contracts-0.5/`; no consumer
repository or deployed endpoint is qualified by this probe.

## Final 0.5 borrowed contract measurements

The final lifetime-aware derive and finite decode API were measured against the
owned form of the same contract on 2026-10-19. These are within-candidate
owned/borrowed comparisons, not a 0.4-to-0.5 release speedup. Both forms use field
IDs 1/2, a five-byte `chunk` name, and 0/32/4,096/1,048,576 bytes of value 42.
Bounds are Text 64/Bytes 1 MiB; method ID 1 has limits of 2 MiB frame, 4 MiB
allocation, 16 nodes and depth 8. Frame hashes match between both IC probes.

| 1 MiB operation | opt-z owned → borrowed | opt-3 owned → borrowed |
| --- | --- | --- |
| Native decode/drop | 34.28 → 10.03 µs | 33.92 → 10.02 µs |
| Native decode/full scan/drop | 285.45 → 257.77 µs | 144.77 → 120.71 µs |
| IC decode/drop instructions | 3,281,082.5 → 2,005.5 | 2,754,987 → 966 |
| IC decode/full scan/drop instructions | 19,009,864.5 → 15,730,787.5 | 10,881,549 → 8,127,528 |
| Linked probe Wasm bytes | 41,383 → 40,232 | 32,331 → 31,380 |
| gzip-9 probe bytes | 14,638 → 14,468 | 12,466 → 12,167 |

[All timing/instruction samples summarized by case](evidence/borrowed-contracts-0.5/performance.csv)
and [linked/compressed sizes](evidence/borrowed-contracts-0.5/wasm-size.csv) retain
both profiles. Native results are medians of 11 alternating owned/borrowed
batches; repetition counts double until borrowed batches take at least 2 ms or
reach 1,048,576 iterations. Min/max batch means are included to expose noise.
Input construction is outside measurement; decode includes canonical admission,
black-boxed values and owned destruction. The full scan sums every payload/name
byte. Neither operation measures encoding, transport or persistence.

Rust 1.99.1/LLVM 23.1.1 targets Linux x86_64 and wasm32-unknown-unknown with LTO,
one codegen unit, panic abort and stripped debug information. `release` uses
opt-z; `speed` uses opt-3. PocketIC 16.1.0 runs each separately linked decoder in
five queries of four decodes, using `ic0.performance_counter(0)` around the loop;
the table divides the median counter delta by four without overhead subtraction.
Both Wasm probes include the same owned input encoder/setup and reply handler;
sizes describe these complete probes, not isolated decoder symbol attribution.
Gzip uses level 9 and mtime 0. No production dependency was added.

Borrowed opaque payload admission avoids a copy and, on Wasm, the existing
same-slice canonical comparison avoids scanning those bytes again. UTF-8 text
still needs validation. The full-scan results show the smaller benefit when the
consumer actually reads every byte. Borrowed views retain their input frame;
zero allocation requests do not mean zero resident input memory. Native and IC
numbers are distinct, and neither establishes installed Blob endpoint latency.
Source, lockfiles, compiler identity, raw batches, frame hashes and Wasm files
are retained locally at `target/borrowed-contracts-0.5/final-performance/`.
