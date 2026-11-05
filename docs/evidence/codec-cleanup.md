# Codec cleanup, pending Core 0.17.1

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The kept changes erase `N` from the heavy owned-byte visitor, share packed
validation/allocation source while retaining specialized copy iterators, and
expand the same strict u32/u64 reader algorithm from one private macro. Browser
factoring is limited to decoder error translation; its framing and completion
remain explicit. The stale native-module description is corrected. No API,
codec bytes, field bounds, traversal charges or typed-error changes; no migration.

Baseline is committed `0c8d68d` (published Core 0.17 / Endpoint 0.7 plus delivery
proof). Supplier lock, package versions, Rust/Cargo 1.99.1 / LLVM 23.1.1 and
wasm32-unknown-unknown are identical across paired sources. Both profiles use
LTO, one codegen unit, abort and stripping, with optimization z or 3. Each source
has a separate Cargo target directory. Actual Wasms, fresh Cargo artifact records
and baked Core source fingerprints are checked. No gzip objective.

## Whole probe footprint

The probe instantiates **1/10/100 distinct bounds per family** for Bytes, Text
and PackedI16, plus generic packed i16/u64 and scalar u32/u64 controls. It is a
stress/control fixture, not a projected application saving. Bytes/Text bounds
are 131072+i and packed element bounds 65536+i.

| Profile | Bounds/family | Before total bytes | Final total bytes | Change |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 1 | 34,660 | 34,721 | +61 (+0.18%) |
| opt-z | 10 | 79,168 | 76,318 | -2,850 (-3.60%) |
| opt-z | 100 | 513,707 | 480,797 | -32,910 (-6.41%) |
| opt-3 | 1 | 39,827 | 39,845 | +18 (+0.05%) |
| opt-3 | 10 | 108,357 | 103,165 | -5,192 (-4.79%) |
| opt-3 | 100 | 858,048 | 795,511 | -62,537 (-7.29%) |

At 100 bounds, code sections fall **510,063 → 477,151** at opt-z and
**854,717 → 792,278** at opt-3. The individual byte-only variant has the same
sizes as the final combined Rust cleanup. Packed/varint source deduplication
is not claimed as an additional size win.

## IC instruction tradeoffs

The probe brackets encode, owned decode, truncated-input rejection and loop
control with IC counters. Setup, warmups and reply transport are excluded. Four
operations per sample; eleven alternating, unique-caller samples per case/lane
prevent query-cache reuse. Each row's samples are identical. Returned payloads
must match between lanes. The final pair covers 52 inputs/profile: bytes/text/
bounded-packed at three representative bounds, generic packed i16/u64, scalar
0/127/128/max controls, and lengths 0/1/32/65536. Values are 0xa5 bytes, ASCII x,
i16::MAX and u64::MAX. Limits: bytes 2097152, allocation 268435456, nodes 8388608,
depth 64. These counters are not physical cycles or whole application costs.

| Owned decode | opt-z before → final | opt-3 before → final |
| --- | ---: | ---: |
| Empty bounded bytes | 583.5 → 642.5 (+10.11%) | 284 → 296 (+4.23%) |
| 64 KiB bounded bytes | 66,722.5 → 66,781.5 (+0.088%) | 66,254 → 66,266 (+0.018%) |
| Empty bounded text | 643.5 → 702.5 (+9.17%) | 346 → 358 (+3.47%) |
| 64 KiB bounded text | 247,063.5 → 247,122.5 (+0.024%) | 246,595 → 246,607 (+0.005%) |

The tiny-input cost is explicit: shared allocation/copy code trades some
specialization for smaller multi-bound modules. Worst truncated-empty overhead
is **368.5 → 406.5 (+10.31%)**. Packed and scalar operation counts are **identical**
for every tested input/profile; byte/text encoding is unchanged. This establishes
one measured code-size tradeoff, not a universal speed improvement. Native Rust
timing and a fresh Candid comparison are not claimed.

## Rejected experiments and browser qualification

A fully merged packed runtime visitor made large bounded i16 decode at opt-z
**132,276.5 → 1,508,420.5 instructions (11.4×)**. It is not kept. The final private
macro shares mechanics while preserving the old `as_chunks::<2>()` bounded
iterator and generic iterator. Original bound checks still precede allocation.

Broader Writer/Reader completion helpers added indirection and produced unstable
Node timings, including large regressions. They are not kept. The final browser
change shares only error translation, preserving the original success paths,
try boundaries, identity checks and trailing-input checks. Eleven-sample host
error-path medians (Node 24.21.0, ES2022) span **-1.82% to +4.69%**: malformed
payload 4627.175 → 4542.859 ns, malformed frame 4490.545 → 4561.552 ns, foreign
callback failure payload 5987.029 → 6267.737 ns and frame 6515.541 → 6617.849 ns.
No browser speedup is claimed. Exploratory noisy valid-path samples remain raw.

Final Rust tests/lint, JS golden/malformed/endian conformance, standalone
host/Wasm feature/Platform schema and stable identity checks pass. Endpoint
native/diagnostic/Wasm and installed admission/reply/rollback qualification pass.
No full CI, dependency update, staging, commit, version preparation or release.
