# Evaluation scope

The maintained qualification exercises the selected Postcard codec, its
generated TypeScript implementation and an isolated Candid comparison suite.
Endpoint adapters and browser SDKs remain consumer-owned. The unpublished codec
benchmark package may use PocketIC for isolated IC instruction measurements;
it does not qualify consumer endpoint integrations.

The released 0.6.0 Serde hard cut removed the finite Wire API. Archived sections recording
finite admission and WireSchema experiments are historical research; the
[current production comparisons](candid-comparison.md)
owns shipped-codec IC, frame and Wasm measurements. The retired protocol-study
runner is available only in the archived finite revision linked by those studies;
it is no longer maintained alongside the production Serde suite.

## Maintained checks

| Target | Evidence |
| --- | --- |
| `test` | Runtime/native bounds, canonical bytes, typed failures and retry buffers |
| `check-typed-diagnostics` | Actual compile-fail subjects and a passing control |
| `check-typescript` | Generated types, Rust vectors and differential malformed-input checks |
| `check-benchmarks` | Matched Candid/Wire corpus, malformed-frame checks, lint and MSRV |
| `check-consumer-benchmarks` | Toko-shaped and 1/10/100-type derive corpus, lint and MSRV checks |
| `evaluate-codecs` | Repeated native timing, allocator counts, peak live heap and Wasm footprint |
| `check-qualification` | Allocation/borrowing fixture tests, strict linting and compiler-floor checks |
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.91 floor |

The unpublished `ic-wire-qualification` package owns the maintained allocation,
borrowing and buffer-reuse fixtures plus the IC Build compiler-diagnostic runner.
Earlier alternative-codec suites were retired in 0.5.19; their reports and raw
evidence remain historical. The new [Candid suite](candid-comparison.md) is owned
by the unpublished `ic-wire-benchmarks` package. See [retired comparisons](performance.md#retired-alternative-codec-comparisons)
for the last source revision and reproduction boundary. Raw Postcard controls in
the retained protocol probes omit Wire admission/canonicality and are not runtime
modes. Native timings, IC instructions and artifact sizes remain distinct evidence.

Run focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` scaffold gate is reserved for an explicit local request
or the maintainer's release flow. Wire does not qualify sibling endpoints,
authentication, storage formats or delivery policy.

See [borrowing, buffers and bounded layouts](allocation-comparison.md) for the
44-case native/IC experiment and its consumer-schema tradeoffs.

## Historical evidence

The [historical experiments](history/evaluation.md) preserve earlier measurements,
source revisions and rejected designs. They are not supported runtime APIs.
