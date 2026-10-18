# Current performance evidence

IC Wire remains Postcard-based. These private valid-input measurements compare
current finite and dynamic Wire costs with raw codec controls; they do not
establish the fastest protocol with equivalent admission/canonicality guarantees.
No Candid comparison or production codec-selection API is introduced.

## Reproduce

Prepare the developer tools and a PocketIC server as described in
[IC setup](ic-tools.md). Then, with `POCKET_IC_BIN` pointing to the admitted server:

```bash
make evaluate-performance
# Or retain a separate output directory for an experiment:
bash scripts/ci/evaluate-performance.sh "$PWD/target/performance-trial"
```

The existing IC Build retained-evidence runner builds four independent Wasm
artifacts, verifies their sole callable export, runs the installed fixtures and
checks source freshness before finalizing `report.json`. Testkit owns server
custody; IC Host owns inspection/hashing. This is an explicit experiment, not a
benchmark on every CI run. The native roundtrip/control tests run with the existing
`check-canister-evaluation` target.

## Conditions and interpretation

- Rust 1.99.1 (`58cf8f9c3`), Wasm32 unknown, PocketIC server 16.1.0 on local Linux.
  The shared `canister-evaluation` profile selects `opt-level="z"`, LTO, one codegen
  unit, aborting panics and stripped debug information.
- Locked Postcard 1.1.3 (`alloc`, no defaults), bitcode 0.6.9 (default derived
  codec), wincode 0.6.3 (defaults and derive, `deserialize_exact`). Source hashes,
  lock/build receipts, flags, compiler identity and artifact digests accompany the
  [before](evidence/performance-before.json) and [after](evidence/performance-after.json)
  reports. Package metadata stays at the maintainer's prepared 0.4.3.
- Each sample performs one reference encode/decode before timing, then batches
  32 encodes, 32 owned decodes and (for dynamic Wire) 32 standalone sizing passes.
  Tables show the median of seven batch instruction counts divided by 32, rounded
  to whole instructions. Allocation, destruction, loop and counter overhead are
  included without subtraction. Value construction, correctness assertions,
  argument copying, response delivery and transport latency are excluded.
- Native x86-64 nanosecond batches are retained separately, under the same named
  optimization profile. They are small local microbenchmarks, not statistically
  qualified CPU comparisons or substitutes for IC instruction counts.
- Wire uses an 11-byte method/direction frame, exact consumption, canonical
  reserialization and resource admission. The fixture's limits are 16,384 bytes,
  8,388,608 allocation units, 65,536 nodes and depth 32. Raw codecs are payload-only
  controls with their own default validation, not implementations of these limits.
- `wire-native` uses ordinary `Vec`/`String` DTOs. Finite Wire uses bounded
  `Bytes`/`Text`/`List`; their payload bytes are checked against ordinary Postcard.
  Additional bounded-DTO controls isolate the effect of Serde representation.
- Inputs are source-defined: a patterned 4 KiB blob, 768 bytes of repeated UTF-8,
  256 small integers, 256 **adjacent descending** integers near `u64::MAX`, and
  32 text-bearing rows. The wide-integer case is correlated, not random: bitcode
  compresses it particularly well. These cases are not a fleet workload model.

## Encoding and owned decoding

Each cell is **encode / decode IC instructions per operation**, after optimization.

| Case | Finite Wire | Dynamic Wire | Raw Postcard | Raw bitcode | Raw wincode |
| --- | ---: | ---: | ---: | ---: | ---: |
| scalar-zero | 1,849 / 932 | 2,343 / 1,149 | 1,011 / 255 | 2,022 / 1,452 | 665 / 94 |
| scalar-max | 2,155 / 2,006 | 2,946 / 2,223 | 1,335 / 883 | 2,256 / 552 | 665 / 94 |
| blob-empty | 1,936 / 1,409 | 2,497 / 1,570 | 1,029 / 535 | 2,917 / 925 | 761 / 643 |
| blob-4096 | 6,177 / 19,642 | 1,665,530 / 2,560,253 | 175,535 / 324,488 | 161,748 / 7,820 | 4,956 / 5,147 |
| text-utf8 | 2,849 / 23,461 | 3,443 / 23,388 | 2,396 / 19,633 | 7,348 / 25,134 | 1,688 / 19,581 |
| integers-small | 89,232 / 126,094 | 217,090 / 221,786 | 72,051 / 52,393 | 31,036 / 11,606 | 2,913 / 3,020 |
| integers-wide | 167,568 / 401,038 | 371,458 / 496,730 | 156,183 / 213,161 | 36,758 / 16,936 | 2,913 / 3,020 |
| page-empty | 1,904 / 1,317 | 2,505 / 1,577 | 1,033 / 568 | 4,285 / 1,717 | 806 / 471 |
| page-32 | 50,982 / 135,389 | 122,265 / 179,399 | 45,843 / 76,394 | 28,316 / 51,068 | 12,758 / 42,783 |

## Encoded bytes

Wire columns include the frame; controls contain payloads only. Finite and dynamic
Wire frames are byte-identical for every subject.

| Case | Wire | Postcard | bitcode | wincode |
| --- | ---: | ---: | ---: | ---: |
| scalar-zero | 12 | 1 | 2 | 8 |
| scalar-max | 21 | 10 | 9 | 8 |
| blob-empty | 12 | 1 | 1 | 8 |
| blob-4096 | 4109 | 4098 | 4101 | 4104 |
| text-utf8 | 781 | 770 | 772 | 776 |
| integers-small | 269 | 258 | 262 | 2056 |
| integers-wide | 2573 | 2562 | 270 | 2056 |
| page-empty | 12 | 1 | 1 | 8 |
| page-32 | 674 | 663 | 361 | 1054 |

## Changes retained

Canonical comparison now checks Wasm bulk chunks with exact unaligned eight-byte
loads and handles one-byte writes directly. Native production builds retain their
normal slice comparison. There is no hash shortcut, second output allocation or
relaxed equality. Native unit tests exercise the Wasm chunk algorithm at every
byte position through length 65, including tails and refusal after a mismatch.
Public finite/native tests also reject custom-deserializer normalization.

Dynamic encoding combines its four node charges and two value-size charges into
one admission operation. Accepted resource budgets and typed refusal remain the
same; checked arithmetic and sticky refusal precede consumer serialization.

| Case | Finite decode before | After | Before / after |
| --- | ---: | ---: | ---: |
| scalar-zero | 948 | 932 | 1.02× |
| scalar-max | 2,091 | 2,006 | 1.04× |
| blob-empty | 1,365 | 1,409 | 0.97× |
| blob-4096 | 112,667 | 19,642 | 5.74× |
| text-utf8 | 40,774 | 23,461 | 1.74× |
| integers-small | 130,130 | 126,094 | 1.03× |
| integers-wide | 422,738 | 401,038 | 1.05× |
| page-empty | 1,333 | 1,317 | 1.01× |
| page-32 | 141,849 | 135,389 | 1.05× |

Empty bounded-blob decoding regresses by about 3% (44 instructions); this is not a
universal speedup. Dynamic ordinary-blob encoding falls from 3,394,464 to 1,665,530
instructions (2.04×). All three raw-control artifacts and their instruction
counts are identical between the final matched before/after runs.

The wire fixture's full Wasm changes from 267,774 to 266,573 bytes.
Raw Postcard, bitcode and wincode fixtures are respectively
151,370, 159,663 and 130,986 bytes.
These fixtures link different numbers of lanes and common measurement/correctness
code. They establish fixture artifact sizes, not production-code attribution or
compressed deployment savings. No postprocessing/compression measurement is made.

An intermediate baseline restored with preserved timestamps reused a cached
optimized artifact. It was discarded. The retained baseline used fresh source
writes; final optimized source hashes were checked after restoration. Trial and
failed logs remain under `/tmp/ic-wire-performance-*`; retained Wasm is under
`target/performance-baseline-fresh` and `target/performance-final`.

## Practical decisions

Keep Postcard and the existing contract. This evidence does not justify claiming
that Wire is the fastest non-self-describing codec: fixed-width wincode wins
these numeric throughput cases, and bitcode wins several batch sizes.

Use `Bytes<N>` for owned bounded blobs when the contract permits it. In this
4 KiB fixture, dynamic decoding of ordinary `Vec<u8>` costs
2,560,253 instructions;
the same dynamic API with `Bytes<4096>` costs
19,916.
The payload bytes are identical, but ordinary Serde sequences visit each element.
Raw Postcard with the same bounded DTO decodes in
5,337 instructions.
The reports retain all three representations; the plain-Vec control alone would
misrepresent the overhead of canonical checking.

Keep canonical reserialization. A strict varint parser alone cannot replace it:
trusted custom Serde deserializers may normalize otherwise valid bytes. Removing
the pass would require a stronger public type contract or weaker canonicality.
Neither is introduced. Dynamic encoding also retains its preallocation admission
pass; its measured standalone sizing cost is available in each report.

Borrowed decoding remains a separate public API decision. No lifetimes, endpoint
contracts, storage formats or wire bytes change in this batch. Consumer-shaped
random/high-entropy and mixed-distribution workloads, equal resource guarantees
and native macOS/installed-IC qualification are still needed before a broader
ranking or codec replacement decision.
