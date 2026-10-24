# Sharing packed-i16 decoding across bounds (0.6.11)

The retained change uses one private byte visitor carrying the field's element
limit, instead of specializing that visitor for every PackedI16<N>. Its allocation
and copy method stays out of line so different bounds share it. The visitor returns
an owned Vec<i16>, which is wrapped in the same public PackedI16<N> type.

Each exact bound and byte-length parity is still checked before allocation.
Encoding, malformed-input acceptance, resource charges and canonical verification
are unchanged. For example, [32767] still emits payload 02 ff 7f. Consumers need
no schema changes or migration; metadata, envelope identity and exact retry bytes
are unchanged. This remains one production Serde/Postcard codec.

## Wasm size

Paired builds use the existing six-field 1/10/100-record fixtures with eight wide
i16 values per record. Shared bounds are 16,384 throughout. Varied bounds cycle
through 1,024–16,384, giving 1/10/16 distinct bounds in the three cohorts.
These are complete uncompressed Wasm artifacts, not symbol-attributed savings.

| Bounds / DTOs / profile | Total bytes before → after | Code-section bytes before → after |
| --- | ---: | ---: |
| Shared / 1 / opt-z | 63,089 → 63,197 | 51,196 → 51,303 |
| Shared / 10 / opt-z | 85,626 → 85,698 | 73,001 → 73,072 |
| Shared / 100 / opt-z | 263,834 → 263,906 | 244,453 → 244,524 |
| Shared / 1 / opt-3 | 93,851 → 93,986 | 66,256 → 66,389 |
| Shared / 10 / opt-3 | 121,953 → 122,108 | 93,676 → 93,830 |
| Shared / 100 / opt-3 | 393,641 → 393,797 | 358,597 → 358,751 |
| Varied / 1 / opt-z | 63,087 → 63,195 | 51,194 → 51,301 |
| Varied / 10 / opt-z | 101,050 → 99,187 | 88,371 → 86,505 |
| Varied / 100 / opt-z | 292,476 → 288,122 | 272,957 → 268,601 |
| Varied / 1 / opt-3 | 93,850 → 93,985 | 66,255 → 66,388 |
| Varied / 10 / opt-3 | 145,467 → 143,817 | 117,134 → 115,482 |
| Varied / 100 / opt-3 | 437,288 → 430,402 | 402,094 → 395,236 |

At 100 DTOs with sixteen bounds, total/code savings are 4,354/4,356 bytes
(-1.49%/-1.60%) at opt-z and 6,886/6,858 (-1.57%/-1.71%) at opt-3.
Shared-bound fixtures instead grow 72–156 total bytes. This is a targeted saving
for varied bounds, not a universal Wasm reduction. The twenty-input, single-bound
control fixture grows 100/163 total bytes and 98/162 code bytes at opt-z/opt-3.
The remaining varied-bound overhead is not eliminated.

A second screening variant also outlined the whole deserializer call. It saved
only another 52 total bytes in the opt-z hundred-type varied-bound build
(288,070 versus 288,122). The simpler visitor-only variant is retained. The wider
variant received size screening only, not full performance qualification.

## Instruction and native-time tradeoffs

The small corpus has 0/1/128/1,024/16,384 elements, distributed as small 0–63,
wide i16::MAX, negative -1 through -64, and mixed boundaries/extrema. Decode
includes canonical verification, owned allocation and destruction.

| Packed decode control | IC instructions before → after | Change |
| --- | ---: | ---: |
| Empty / opt-z | 910.5 → 942.5 | +3.51% |
| One element / opt-z | 1,435.5 → 1,470.5 | +2.44% |
| 128 elements / opt-z | 2,275.5 → 2,310.5 | +1.54% |
| 16,384 wide / opt-z | 82,130.5 → 82,165.5 | +0.043% |
| Empty / opt-3 | 525 → 564 | +7.43% |
| One element / opt-3 | 886 → 924 | +4.29% |
| 128 elements / opt-3 | 1,670 → 1,710 | +2.40% |
| 16,384 wide / opt-3 | 79,960 → 80,004 | +0.055% |

All encode counts and ordinary Wire/Candid control counts in this corpus remain
unchanged. At 100 DTOs, record_000 shared-bound decode changes from
7,057.5 to 7,081.5 at opt-z and 3,782.5 to 3,830.5 at opt-3. With varied bounds,
it changes from 7,076.5 to 7,142.5 and 3,782.5 to 3,798.5 respectively.
Across all hundred records, decode regressions range from 0.34% to 1.27%,
depending on bound/profile configuration; encode counts remain unchanged.

Native opt-3 empty decode run medians are 48.96–49.69 ns before and
50.68–51.58 ns after. One-element medians are 31.63–32.50 versus 30.61–32.68 ns.
The large wide case is 0.993–0.995 µs versus 0.957–0.988 µs. These are unpinned
microbenchmark observations, not a claimed native speedup or confidence interval.
Opt-z large medians overlap: 0.969–1.033 µs versus 1.001–1.028 µs.
All allocation counts, reallocation counts, requested bytes and peak-live
measurements match. Large packed decode still requests one 32,768-byte vector;
large encode requests one 32,782-byte output frame.

## Validation and measurement identity

Baseline is released 0.6.10, 31bcef2. Both sides use identical inputs,
Cargo.lock, compiler, profiles and codec budgets. Rust 1.99.1 / LLVM 23.1.1
builds x86_64 Linux (Threadripper 7970X) and wasm32-unknown-unknown, with LTO,
one codegen unit, panic abort and stripped symbols. evaluation is opt-z and
benchmark-speed is opt-3. Dependencies include Postcard 1.1.3, IC Platform
0.6.10 and Host/Testkit 0.66.0; PocketIC is 16.1.0.

Small-corpus native timing uses two runs/profile, eleven calibrated samples,
at least 2 ms/batch or 1,048,576 repetitions. Allocation and peak-heap runs are
separate. Setup and semantic equality checks are outside measurement; result
destruction is included. IC records contain three samples with four operations
per sample, without subtracting loop overhead. All repeated IC samples agree.
Candidate scaling runs reuse the matching baseline Candid binaries as fixed
controls and remeasure them; their hashes and instruction counts stay identical.

Input and frame hashes match across variants and measurement modes. An admission
probe sweeps byte, allocation, node and depth limits for empty, one-word, two-word
and mixed-bound tuple values: 16,976 configurations / 33,952 encode/decode outcomes
are identical, including typed errors. Each axis includes successful cases as
well as refusals. A regression decodes a tuple with different bounds, then swaps
the declared bounds and verifies refusal of the now-oversized field.

The exhaustive 65,536-word test, native runtime tests, packed/scaling roundtrips,
strict Clippy, TypeScript conformance (149 vectors / 30,172 mutations), and native
and Wasm Rust 1.91 checks pass. No full CI, installed consumer or big-endian
execution is claimed. The big-endian serialization path is unchanged.

[Artifact sizes](evidence/packed-sharing-0611/artifacts.csv),
[small-corpus measurements](evidence/packed-sharing-0611/small.csv),
[scaling measurements](evidence/packed-sharing-0611/scale.csv) and
[identity](evidence/packed-sharing-0611/identity.json) retain the summaries.
The [source/sample archive](evidence/packed-sharing-0611/raw.json.gz) contains
sources, both candidate variants, all samples, the admission probe and scripts.
For reproduction, restore a small-corpus source snapshot, then replay small.sh,
admission.sh and scale.sh with adjusted absolute source/cache/output paths and
new output directories. Scale measurements cover Wasm/IC costs, not native DTO
throughput. Archive compression is storage only; gzip is not an optimization goal.
