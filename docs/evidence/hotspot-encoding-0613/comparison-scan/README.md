# Exact comparison scan: cost isolation and mixed records

No runtime optimization is retained. The current 64-byte/u128 reduction remains
selected. Two exact alternatives reduce Wasm slightly but regress decoding.
Five mixed-record cases remain in the hotspot corpus, bringing it from 82 to 87.
Production bytes, limits, canonical acceptance and retry semantics are unchanged.

## Cost isolation

All rows below are IC instructions per owned decode at opt-3, including result
destruction and loop overhead. The diagnostic variants are **not valid codecs**.
They exist only in frozen scratch sources, not as production features.

| Workload | Baseline | Omit bulk equality scan | Omit canonical validation entirely |
| --- | ---: | ---: | ---: |
| Empty blob | 636.5 | 629.5 | 402.5 |
| 64-byte blob | 1,203.5 | 979.5 | 772.5 |
| 1 MiB blob | 2,524,535 | 1,049,826 | 1,049,491 |
| 64-byte mixed record | 7,192.5 | 6,460.5 | 4,715.5 |
| 1 MiB mixed record | 2,550,066 | 1,063,513 | 1,061,640 |

For the large blob, omitting only bulk comparisons removes 58.42% of total
instructions; removing all canonical validation removes 58.43%. This locates the
large-buffer overhead in the byte scan, with little additional traversal cost.
Small and mixed records retain more serializer work. The no-scan diagnostic
preserves scalar, pointer, length and admission work but skips comparisons of
buffers at least eight bytes long; it can accept modified contents. The other
diagnostic bypasses the entire reserialization and its admission checks.

These are matched counterfactual builds, not independently timed pipeline stages:
optimizer interactions mean their deltas are not an exact additive decomposition.
In particular, the no-scan build regresses generic byte-vector decoding by 97.70%
at opt-3 even though those scalar comparisons are retained. No diagnostic result
is claimed as a shippable saving or evidence that checks can safely be removed.

## Exact candidates

Both retain the 64-byte block size, 8-byte tail, pointer fast path, bounds and
full serializer traversal. Neither changes bytes or the final acceptance result.
`array-eq` compares fixed arrays using Rust equality and stops scanning on a
mismatch. `word64` reduces eight u64 XORs into one u64 accumulator instead of
four u128 XORs into a u128 accumulator. Existing exhaustive mismatch/tail tests
pass for both in native test builds, which enable the Wasm comparison branch.

| Opt-3 decode | Baseline | array-eq | Change | word64 | Change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 64-byte blob | 1,203.5 | 2,805.5 | +133.11% | 1,201.5 | -0.17% |
| 1 MiB blob | 2,524,535 | 28,870,001 | +1,043.58% | 2,557,299 | +1.30% |
| 64-byte mixed record | 7,192.5 | 12,004.5 | +66.90% | 7,190.5 | -0.03% |
| 1 MiB mixed record | 2,550,066 | 29,101,350 | +1,041.20% | 2,583,082 | +1.29% |

Empty-blob costs are unchanged for the exact candidates. At opt-z, 1 MiB blob
decode goes from 2,574,293 to 28,870,599 (+1,021.50%) or 2,607,057 (+1.27%).
The mixed large record similarly regresses 985.38% or 1.23%. All boundary controls
and unchanged Candid controls remain in the CSV; there is no universal win.

Uncompressed whole-probe total / code-section bytes:

| Profile | Baseline | array-eq | word64 |
| --- | ---: | ---: | ---: |
| opt-z | 88,623 / 77,603 | 88,478 / 77,458 | 88,618 / 77,598 |
| opt-3 | 113,275 / 102,926 | 113,136 / 102,787 | 113,266 / 102,917 |

The tiny size savings do not justify these instruction regressions. No further
loop unrolling, alternate production codec, relaxed validation or unsafe code is
introduced. Native codec time was not rerun: exact candidates alter a Wasm/test
branch and were rejected on IC evidence; no native timing claim is made.

## Corpus, identity and reproduction

The added records combine a byte-oriented blob, packed i16 samples, string labels,
an optional cursor and an integer ID. Blob lengths are 0/1/64/4096/1048576; sample
counts are capped at 4096. Candid uses its native integer vectors for samples.
The JSON semantic input is common, while each codec keeps its own frame layout.
Every driver response is checked against the native oracle's exact frame.

Baseline is f470bcd9 plus the incoming dirty 0.6.13 batch, including the i16 writer,
ByteBuf and the 82-case boundary corpus, plus the identical five new cases in
all builds. The frozen source is authoritative, not the pristine Git commit.
Rust 1.99.1 / LLVM 23.1.1; wasm32-unknown-unknown; evaluation opt-z and
benchmark-speed opt-3; LTO, one codegen unit, abort panics and stripped symbols.
Same Cargo.lock: Serde 1.0.229, Postcard 1.1.3, Candid 0.10.39, Platform 0.6.14,
Host/Testkit 0.67.0. PocketIC 16.1.0 is explicit and Testkit owns its lifecycle.

Five builds each produce 1,904 rows with three equal samples: 9,520 rows total,
7,616 baseline/variant pairs. Repetitions are four through 64 KiB and one above;
setup/transport is excluded, loop overhead remains included. Input/frame hashes,
lengths and repetition counts agree in all pairs; Candid counters are unchanged.
Guest binaries and native oracle were freshly built from the frozen source.
Local Rust files were touched between source copies in the shared target.

[All comparisons](comparison.csv), [artifact sizes](wasm.json) and
[raw evidence](raw.tar.gz) retain sources, parameters, samples, identities,
artifacts, build commands and logs. Extract and run `bash replay.sh /absolute/fresh`
with prepared `POCKET_IC_BIN`, Rust targets and cached dependencies. The replay
was syntax-checked; constituent builds/runs were executed, but the consolidated
replay was not executed a second time. Archive compression is storage only.

The retained 87-case corpus passes roundtrip/damaged-frame checks and strict
Clippy. Both exact candidate comparison unit suites pass. Formatting and local
document links pass. Broader matrix, native timing, macOS and installed-consumer
qualification were not rerun; rejected candidates are not release-qualified.
