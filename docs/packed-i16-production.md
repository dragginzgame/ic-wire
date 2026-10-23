# Packed i16 production qualification (0.6.8)

`ic_wire::PackedI16<N>` is an explicit bounded Serde field with matching
`Int16Array` TypeScript metadata. It promotes the selected
[research implementation](packed-i16-study.md) into the existing Postcard codec.
`new(Vec<i16>)`, `as_slice()` and `into_vec()` preserve owned storage without
copying. Little-endian serialization uses bytemuck 1.25.2's safe checked byte
view. Decoding validates parity and the element bound before reserving owned
storage, then reads little-endian words without alignment assumptions.

Ordinary Vec/List encoding is unchanged. Switching a field changes the count to
bytes and the words to fixed-width little-endian: `[32767]` changes from
`01 fe ff 03` to `02 ff 7f`; `[0]` grows from `01 00` to `02 00 00`.
Consumers must coordinate schemas, regenerate clients and resolve retained old
frames before switching. There is no automatic rewrite or alternate reader.
Envelope identity, canonical-byte verification and same-release retries remain.
Bulk byte admission replaces per-element traversal for this explicit field.

## Results

Packed encode and decode beat Candid for all twenty cases in both profiles
(80 comparisons). All three IC samples agree for every measured operation.
For 16,384 wide values, instruction counts are:

| Profile / operation | Ordinary Wire baseline | PackedI16 | Candid | Packed vs ordinary | Packed vs Candid |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z encode | 9,438,687.5 | 34,414.5 | 86,093.5 | -99.64% | -60.03% |
| opt-z decode | 9,050,320.5 | 82,130.5 | 1,186,279.5 | -99.09% | -93.08% |
| opt-3 encode | 4,014,993.25 | 33,489.25 | 80,320.25 | -99.17% | -58.31% |
| opt-3 decode | 5,067,454 | 79,960 | 1,408,211 | -98.42% | -94.32% |

The same wide input's complete frame shrinks from 49,166 to 32,782 bytes
(-33.32%); Candid is 32,780 bytes. Packed encode allocates one 32,782-byte output;
decode allocates one 32,768-byte vector, with no reallocations. Measured peak live
bytes match those amounts. Small values instead grow from 16,398 to 32,782 bytes
(+99.91%); fixed-width packing is a deliberate size/speed choice.

Native opt-3 run medians for wide encode are 85.72–86.74 µs ordinary,
0.520–0.529 µs packed and 1.506–1.578 µs Candid. Decode medians are
149.73–152.53 µs ordinary, 0.954–0.989 µs packed and 21.10–21.87 µs Candid.
These repeated-run ranges are observations, not confidence intervals.

Small-input controls expose the regressions:

| Profile / input / operation | Ordinary IC | Packed IC | Change |
| --- | ---: | ---: | ---: |
| opt-z empty encode | 1,322.5 | 1,420.5 | +7.41% |
| opt-z empty decode | 680.5 | 910.5 | +33.80% |
| opt-z one zero encode | 1,756.5 | 1,534.5 | -12.64% |
| opt-z one zero decode | 1,425.5 | 1,435.5 | +0.70% |
| opt-3 empty encode | 779.25 | 567.25 | -27.21% |
| opt-3 empty decode | 431 | 525 | +21.81% |
| opt-3 one zero encode | 1,006.25 | 678.25 | -32.60% |
| opt-3 one zero decode | 914 | 886 | -3.06% |

Native empty decode also regresses: opt-z 22.74–23.52 → 64.89–65.81 ns;
opt-3 12.70–12.88 → 48.98–50.26 ns. This is a bulk field, not a universal
replacement for small integer lists. Both forms remain much faster than Candid
in these controls; ordinary lists remain available with unchanged bytes.

| Complete fixture | Ordinary Wire total / code | With packed field total / code | Added total / code | Candid total / code |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 42,065 / 33,154 | 44,503 / 35,458 | +2,438 / +2,304 | 216,356 / 190,714 |
| opt-3 | 45,551 / 36,897 | 49,379 / 40,614 | +3,828 / +3,717 | 284,138 / 259,050 |

All sizes are uncompressed Wasm bytes. Added total/code costs are +5.80%/+6.95%
at opt-z and +8.40%/+10.07% at opt-3. Candidate Wire contains both ordinary and
packed benchmark paths. This is one element type with one packed bound, not
100 independently derived consumer types or an installed application's size.

## Measurement method

The baseline uses ordinary Wire and Candid lists; the candidate adds the public
packed field. Both use identical production source, dependency lock, inputs,
limits and compiler settings. This measures opting into the field, including
its incremental linked code; it is not a revision-to-revision optimization of
ordinary Vec<i16>. Prototype adapter/conversion implementations have been removed
from the maintained benchmark; their sources and results remain archived.

The twenty inputs have 0/1/128/1,024/16,384 elements, distributed as small 0–63,
wide i16::MAX, negative -1 through -64, or mixed varint boundaries/extrema.
Two native runs per profile each collect eleven calibrated samples, at least
2 ms per batch or 1,048,576 repetitions. Allocation counts and peak heap use
separate instrumented builds. Setup is excluded; output/value destruction is
included. Native timing is unpinned, warm-cache and not request latency.

Rust 1.99.1 / LLVM 23.1.1 targets x86_64 Linux (Threadripper 7970X) and
wasm32-unknown-unknown. Both profiles use LTO, one codegen unit, panic abort and
stripped symbols; `evaluation` is opt-z and `benchmark-speed` is opt-3.
Postcard is 1.1.3, IC Platform 0.6.10, Host/Testkit 0.66.0 and PocketIC 16.1.0.
The IC runner measures three samples of four operations with performance counter
0, without subtracting loop overhead. It verifies returned frames against the
host oracle. The evidence retains every sample, input/frame hash, codec quota,
compiler identity, source snapshot and lockfile.

## Reproduction and validation

Run each cohort into a new output directory, sequentially:

```sh
RUNS=2 SAMPLES=11 BATCH_MS=2 CORPUS_FEATURE=packed-i16-baseline \
  POCKET_IC_BIN=/path/to/prepared/pocket-ic bash scripts/ci/evaluate-codecs.sh BASELINE
RUNS=2 SAMPLES=11 BATCH_MS=2 CORPUS_FEATURE=packed-i16 \
  POCKET_IC_BIN=/path/to/prepared/pocket-ic bash scripts/ci/evaluate-codecs.sh CANDIDATE
```

[Measurements](evidence/packed-i16-production/measurements.csv),
[artifact sizes](evidence/packed-i16-production/artifacts.csv),
[identity](evidence/packed-i16-production/identity.json) and the
[source/sample archive](evidence/packed-i16-production/raw.json.gz) preserve this
run. The archive includes the runner and analysis script; compression is only
for evidence storage, never a Wasm optimization criterion.

Focused native tests exhaust all 65,536 i16 values and cover bounds, noncanonical
lengths, odd/truncated/trailing payloads, unaligned input and budget refusal.
TypeScript qualification passes 149 vectors in both directions and 30,172
Rust-oracle mutations, plus existing large-byte/native-schema fixtures. It checks
owned typed-array results, offset inputs, exact bytes and malformed lengths.
Strict Clippy, Rust 1.91 native/Wasm checks, benchmark roundtrips and doc examples
pass. Big-endian execution, installed consumers, TypeScript performance and host
metadata-generation timing are not qualified by this benchmark. Full CI was not run.
