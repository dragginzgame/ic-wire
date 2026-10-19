# Wasm admission and canonical-comparison optimization

Measured on 2026-10-19 against the working tree from the completed
[MessagePack experiment](messagepack-comparison.md), including its earlier
canonical XOR/OR optimization. This batch preserves that incoming work.

Two changes are retained:

- Force inline the three small budget-admission helpers **only on wasm32**.
  This exposes constant element sizes and charges to the compiler and removes
  repeated helper calls from hash-array traversal. The arithmetic, sticky refusal,
  check order and error mappings are unchanged.
- Compare canonical byte/text writes of at least 64 bytes in 16-byte groups,
  using exact u128 XOR/OR accumulation. Wasm lowers these to integer operations;
  no SIMD feature, unsafe code, hashing or scratch allocation is required.
  Writes of 8–63 bytes retain the previous 8-byte path. Native release builds
  retain ordinary slice comparison and their existing inlining decisions.

The native-unit equality test now covers every length through 97 and every
single-byte mismatch position. This exercises both sides of the 64-byte threshold,
multiple wide groups and every possible wide tail length, as well as the existing
short/long input and exhausted-output-allowance cases.

## Results

IC instructions per operation, medians of five identical query samples, each
containing four operations. These are codec costs, not endpoint latency or bills.
The final Wasm binaries are byte-identical to the measured combined candidate.
All 27 workload payload lengths and SHA-256 values agree with the baseline in
both profiles and every experiment.

| Profile | Workload | Operation | Before | After | Reduction |
| --- | --- | --- | ---: | ---: | ---: |
| Size (`z`) | Small RPC | encode | 3,371.5 | 2,921.5 | 13.3% |
| Size (`z`) | Small RPC | decode | 3,147.5 | 2,437.5 | 22.6% |
| Size (`z`) | Manifest, 32 array hashes | encode | 277,988.5 | 175,948.5 | 36.7% |
| Size (`z`) | Manifest, 32 array hashes | decode | 502,730.5 | 333,012.5 | 33.8% |
| Size (`z`) | Manifest, 1,024 array hashes | encode | 7,899,656.5 | 4,788,880.5 | 39.4% |
| Size (`z`) | Manifest, 1,024 array hashes | decode | 14,442,566.5 | 9,262,257.5 | 35.9% |
| Size (`z`) | Opaque 1 MiB bytes | decode | 4,458,244.25 | 3,278,491.25 | 26.5% |
| Speed (`3`) | Manifest, 32 array hashes | encode | 130,981 | 95,738 | 26.9% |
| Speed (`3`) | Manifest, 32 array hashes | decode | 238,138.75 | 221,415.75 | 7.0% |
| Speed (`3`) | Manifest, 1,024 array hashes | encode | 3,778,613 | 2,654,156 | 29.8% |
| Speed (`3`) | Manifest, 1,024 array hashes | decode | 6,804,814.75 | 6,296,059.75 | 7.5% |
| Speed (`3`) | Opaque 1 MiB bytes | decode | 2,753,749.75 | 2,753,743.75 | essentially unchanged |

All size-optimized measured operations improve. Speed optimization has tradeoffs:
the alternative binary-hash manifest's encode cost increases 9.1% at 1,024 hashes
and 4.5% at 32; 255-byte buffer decode increases 7.3%. These cases remain in the
raw results. Small-operation changes below a few percent should not be treated
as a universal win. The selected change favors the current array-hash Blob shape;
binary-hash DTOs remain a separate schema experiment.

| Complete Wire benchmark probe | Before linked | After linked | Before gzip | After gzip |
| --- | ---: | ---: | ---: | ---: |
| Size (`z`) | 121,640 | 123,854 | 34,584 | 35,394 |
| Speed (`3`) | 149,529 | 151,950 | 42,928 | 43,323 |

Sizes are bytes: linked growth is 2,214 / 2,421 bytes, and deterministic gzip
growth is 810 / 395 bytes. These are complete 27-workload probes, not projected
production Blob deployment sizes. The extra instructions saved in array traversal
justify this measured size tradeoff for this use case.

## Experiments rejected or narrowed

The retained evidence includes six measured code combinations and the final
wasm32-only scope refinement:

- Budget helpers alone supplied the main manifest gain. Inlining only the combined
  admission helper saved less decode work than inlining all three helpers.
- Forcing scalar output writes inline reduced the main manifest gain. Forcing
  the serialization wrapper inline almost erased the speed-profile manifest gain.
  Neither additional hint is retained.
- Switching all eight-byte comparisons to sixteen-byte comparisons made short
  varint writes fall back to bytewise equality; random-u64 decode regressed by
  about 20% in the speed profile. The retained 64-byte threshold preserves the
  established scalar-write path.
- Unrestricted native inlining improved the 1,024-array-hash manifest in adjacent
  native runs (55.45 to 37.95 microseconds encode; 103.01 to 91.27 decode), but the
  alternative binary-hash decode increased from 30.98 to 36.97 microseconds.
  Raw controls also exposed host timing drift. The final hints are wasm32-only:
  those native numbers describe a **rejected candidate**, not final improvements.

No sizing pass, admission check or canonical reserialization was removed. Skipping
those passes would require proving custom Serde behavior and failure semantics;
this measurement does not establish that such a shortcut is valid.

## Reproduction and limits

The [metadata](evidence/runtime-optimization-2026-10-19/metadata.json),
[raw IC experiments](evidence/runtime-optimization-2026-10-19/ic-experiments.json)
and [full delta CSV](evidence/runtime-optimization-2026-10-19/summary.csv) retain
all candidates, inputs and results. Native raw controls and samples are retained
with an explicit rejected-candidate label.

The compiler, profiles, codec versions, resource allowances, fixture, host and
PocketIC methodology are the same as the linked MessagePack report: Rust 1.99.1,
LLVM 23.1.1, PocketIC 16.1.0, opt-level `z`/`3`, LTO, one codegen unit, aborting
panics, stripped debug information, no wasm-opt, gzip `-n -9`. The external
harness measures codec operations between `ic0.performance_counter(0)` reads;
construction, roundtrip assertions and response transport are outside the clock.
Native runs used CPU 8 and 15 rotating calibrated batches per lane; the host was
not exclusively reserved. No native speed-up is claimed for the final change.

Exact baseline sources, per-candidate overlays, linked Wasms, logs and commands
are retained locally under `target/runtime-optimization-2026-10-19/`. The external
harness source/locks remain in the preceding experiment's
`target/messagepack-2026-10-19/external-harnesses.tar.gz`. These archives must
accompany the reports for a full IC rerun; platform dependencies remain outside
the maintained workspace. Commands in the local runner use the disposable paths
from this session and can be adjusted to a fresh checkout.

Focused qualification passes: 24 runtime/unit/integration tests, seven doctests,
strict runtime Clippy, Rust 1.91 host/Wasm checking, 90 TypeScript vectors and
17,524 differential mutations, including 1 MiB frame equality and boundary
refusals. The probe builds qualify Rust 1.99.1 Wasm. Bytes, runtime APIs, versions,
retry semantics and resource charges are unchanged. No full CI, native macOS
benchmark or actual consumer endpoint/deployment qualification is claimed.
