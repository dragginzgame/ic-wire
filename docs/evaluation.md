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

Private raw Postcard, bitcode and wincode controls have different admission and
canonicality guarantees. They are not production codec modes. Optional
[native timings](performance.md) must retain compiler, target, profile, inputs,
lock selection and codec parameters. Native nanoseconds do not establish IC
instruction counts, deployment size or end-to-end latency.

Run focused checks under the shared [validation policy](../DRAGGINZGAME.md#validation-and-evidence).
The complete `make ci` scaffold gate is reserved for an explicit local request
or the maintainer's release flow. Wire does not qualify sibling endpoints,
authentication, storage formats or delivery policy.
