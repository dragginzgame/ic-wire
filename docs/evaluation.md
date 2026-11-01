# Evaluation scope

The maintained qualification exercises the selected Postcard codec, its
generated TypeScript implementation and an isolated Candid comparison suite.
The optional typed endpoint integration has a standalone Platform/Testkit fixture;
framework policies and browser SDKs remain consumer-owned. The unpublished codec
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
| `check-endpoint` | Typed endpoint/native schema checks, compiler diagnostics, lint, Wasm/MSRV and graph isolation |
| `check-endpoint-ic` | Explicit installed endpoint/rollback and async qualification using a prepared PocketIC server |
| `evaluate-endpoints` | Every exported fixture endpoint: repeated complete IC instructions, observed cycle debits, frame sizes and Wasm; explicitly prepared instrumented server |
| `check-wasm`, `check-msrv` | Runtime compilation for Wasm and the Rust 1.91 floor |

The unpublished `ic-wire-qualification` package owns the maintained allocation,
borrowing and buffer-reuse fixtures plus the IC Build compiler-diagnostic runner.

The standard Wire/Candid corpus includes empty and singleton controls, zero-sized
unit/record collections, nested empty lists, binary-key maps, blob-valued results,
fixed optional arrays, boxed values, Unicode `Cow<str>` and tuple/record enum
variants. New workloads append to the corpus so existing IC case indices stay
fixed. Binary-key maps use ordered pairs for their JSON input identity only;
both measured codecs still serialize the map. These cases use the existing native,
IC instruction and Wasm-footprint runners; adding coverage is not a new performance
measurement. Historical reports retain their original corpus and source identity.

The focused `byte-keys` and `byte-keys-control` cohorts compare standard
`ByteArray<32>` keys against unchanged `[u8;32]` map keys at 0/1/16/128 entries.
Both keep identical Candid maps and logical inputs. The
[matched study](evidence/byte-array-0133/README.md) retains repeated native time,
IC instructions, frame bytes, allocation/heap probes and raw Wasm/code sizes.
This explicit representation improves nonempty-map instructions while increasing
Wasm and wire size; ordinary controls and existing corpus indices remain intact.

Rust-only native tests additionally qualify Serde's standard nonzero integers,
network addresses and ranges, including invalid values, every truncated frame
prefix and trailing-byte refusal. Raw floating-point checks compare IEEE bits,
including signed zero and NaN payloads, rather than float equality. These codec
checks do not imply generated TypeScript schema support for every Rust type.

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
