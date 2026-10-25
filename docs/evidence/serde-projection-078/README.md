# Serde projection qualification, pending 0.7.8

Baseline: release 0.7.7, `6d1f8a1bbde6ad16d24d1d857b1a30292d5b26b0`.
Candidate: the source patch in [raw evidence](raw.tar.gz). Addresses
[#23](https://github.com/dragginzgame/ic-wire/issues/23) and
[#24](https://github.com/dragginzgame/ic-wire/issues/24); publication remains pending.

## Contract

Previously optional serde_bytes adapters and symmetric Serde renames failed
metadata derivation. They now project their existing Serde layout. There is no
runtime codec, byte, identity, framing or admission change. Optional bytes use
`null | { some: Uint8Array }`, not number arrays; they preserve these payloads:

| Value | Payload bytes |
| --- | --- |
| None | `00` |
| Some(empty) | `01 00` |
| Some([0,128,255]) | `01 03 00 80 ff` |

`Int64(-7)` renamed `Int` retains variant zero and payload `00 0d`.
`List([Int64(-7)])` in the recursive fixture retains `01 01 00 0d`.
Field labels do not reorder fields; variant labels do not reorder ordinals.
Regenerate clients and preserve existing consumer labels when adopting the derive.
Generated registry names are not stable contract identities, as already documented.
Ambiguous labels, asymmetric renames, rename_all, alias, custom serializers and
non-byte adapters such as Option<Vec<u16>> remain rejected.

## Validation

The TypeScript gate passes 159 ordinary vectors, 33,560 mutations, 29 native
projection vectors, 16 admission shapes and 54 simulated-endian cases. It includes
strict generated-module compilation, strict Clippy and Rust 1.91 host/Wasm schema
forwarding. Native tests pass 33 cases, including alias/generic/reference/nested
option metadata and a renamed recursive enum. Doctests reject non-byte adapter
inputs; macro tests reject duplicate names and unsupported Serde attributes.

Both request and response tests exercise empty/nonempty optional buffers, exact
byte limits, node/depth limits, malformed input and numeric-array rejection.
The adapter's Rust decode node charge differs from a bare option, and TS remains
separately qualified. Frozen minima for a record with one optional byte field:

| Value | Rust encode/decode nodes | TS encode/decode nodes | Rust encode/decode depth | TS encode/decode depth |
| --- | --- | --- | --- | --- |
| None | 8 / 3 | 2 / 2 | 4 / 2 | 2 / 2 |
| Some(empty or bytes) | 12 / 4 | 3 / 3 | 5 / 3 | 3 / 3 |

No full CI, fresh macOS execution or real big-endian run was performed.

## Host generation controls

Same locked dependencies, Rust 1.99.1 / LLVM 23.1.1, Linux x86_64 Threadripper
7970X, Cargo dev profile (unoptimized, debug info), no extra RUSTFLAGS or CPU
pinning. Separate build target directories prevent path-independent Cargo
freshness from accidentally reusing the other source tree. Three alternating
before/after runs; seven samples per run, 100 operations per sample. Each operation
projects Page<u64> and renders it under one or 100 names, including allocations.
Reported values are medians of 21 batch means. Small differences are noisy.

| Projection + rendering | Before ns | After ns | Change |
| --- | ---: | ---: | ---: |
| One schema | 23,102 | 23,454 | +1.52% |
| Same schema under 100 names | 2,222,775 | 2,189,005 | −1.52% |

Per-run medians: one schema 23,102/22,815/23,551 → 23,417/24,279/23,047;
100 names 2,215,313/2,232,258/2,222,775 → 2,145,899/2,221,994/2,189,005.
These are host regression controls, not codec improvements or macro-expansion
benchmarks. The new declarations cannot derive on the baseline, so no successful
before/after generation timing exists for them. No IC instruction or Wasm size
claim is made: all production changes are native metadata or macro expansion;
Rust/TypeScript codec implementations are unchanged.

Extract raw evidence to a disposable directory and run `bash replay.sh` with
prepared dependencies/toolchain. It retains baseline sources, candidate patch,
raw samples, generated admission evidence and logs. The maintained
`make check-typescript` reproduces conformance with the repository's prepared
Node 24.21.0 and TypeScript 7.0.2 tools.
