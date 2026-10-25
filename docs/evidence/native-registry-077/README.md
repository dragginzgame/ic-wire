# Native recursive registries and projected budgets (0.7.7)

This host-only addition addresses [recursive projection #20](https://github.com/dragginzgame/ic-wire/issues/20)
and [browser budget projection #21](https://github.com/dragginzgame/ic-wire/issues/21).
Publication remains maintainer-owned; local qualification does not close that gate.

`SchemaRegistry` collects native shapes through the existing describe interface;
no new generated helper couples a newer derive to a newer library. Concrete mutual
recursion also avoids redundant cyclic trait predicates. Names cannot overwrite
existing definitions; finish verifies encountered projections and rejects conflicts,
unresolved refs, alias-only cycles and excessive metadata. Inline native_schema
retains its established output and recursive Complexity refusal.

`method_limits` projects independent byte/node/depth budgets and shares the
renderer validation. Rust allocation is deliberately omitted, not approximated as
JS heap. Native/TS counters, byte encodings and envelopes remain unchanged.

## Qualification

The maintained conformance example covers 13 shapes: scalar, record, option,
newtype, transparent wrapper, empty enum, empty/nested recursive lists, recursive
option, recursive record, mutually recursive records, bytes and packed i16.
Both directions sweep node/depth allowances and exact byte boundaries, asserting
frozen admission thresholds independently for Rust encode/decode and TS encode/decode.
Generated recursive modules type-check; truncations, trailing bytes, wrong headers
and low/high-bit mutations are checked against native acceptance. The ordinary
159 vectors, 33,560 mutations and 24 native projection vectors remain controls.
[Recorded admission minima](admission.json) retain the differences, including the
fact that decode success does not guarantee encoding under the same limits.

Package tests/doctests, strict Clippy, native/Wasm feature forwarding, Rust 1.91
checks and compiler diagnostics qualify the local source. No sibling changes,
publication or installed-consumer acceptance are claimed. Consumers retain custom
Serde projections/validators and qualify their own budgets and real endpoint corpus.
An isolated check also compiled the new derive against the pre-change metadata
module, confirming it does not require the new registry API from older libraries.

## Host generation and artifact measurements

Baseline is 0.7.6 (`accf815`) plus the incoming cleanup batch, preserved as a patch
in the archive. The compared runtime is identical; only host metadata/derive code
changes. Same Cargo.lock, Rust 1.99.1 / LLVM 23.1.1, Linux x86_64, opt-3, LTO,
one codegen unit, panic abort and stripped symbols. Each host run alternates seven
samples per implementation; three runs before and three after. Projection/render
samples average 100 operations; limit projection samples average 1,000 operations.
Times include allocation/destruction. No CPU affinity; treat small changes as noisy.

| Host operation | Before / handwritten | After / typed | Change |
| --- | ---: | ---: | ---: |
| Inline Page<u64> projection + render, 1 named schema | 3,269 ns | 3,039 ns | −7.04% |
| Same shape under 100 names | 305,103 ns | 310,193 ns | +1.67% |
| Six budget properties, handwritten JSON versus validated helper | 146 ns | 189 ns | +29.45% (+43 ns) |

Values are pooled medians of 21 batch means. Per-run inline medians are
2,941/3,076/3,286 → 2,998/3,112/3,026 ns for one schema and
300,129/311,815/303,526 → 307,852/315,589/304,681 ns for 100. The budget control
uses identical black-boxed Limits; its extra validation is intentional. This is
not a codec speedup claim or a 100-distinct-type benchmark. The prior inline API
cannot successfully project recursive types, so it supplies no valid recursive
before/after timing baseline.

The one-type runtime Wasms are byte-identical in both profiles:

| Profile | Total bytes before = after | Code section before = after |
| --- | ---: | ---: |
| opt-z | 53,998 | 42,997 |
| opt-3 | 74,871 | 48,129 |

SHA-256: opt-z `b391e404581f917a4077e51f78367cad706f0ef54f71eba9fb0ad0e1c7fb9443`;
opt-3 `0bfbc069230d3e9b7f633d55dff7b81001ba31d41ead40c29f1ab35254d10ef6`.
No fresh IC instruction or native codec timing run was needed for these identical
artifacts. This does not requalify every consumer Wasm or the full Candid matrix.

[Source, raw samples, compiler identity and replay](raw.tar.gz) retain reproduction
inputs. Extract and run `bash replay.sh` with prepared Rust/dependencies; it uses
an isolated temporary tree and offline Cargo. Maintained `make check-typescript`
reproduces cross-runtime qualification with the documented Node/TypeScript setup.
Original artifacts/logs remain under `/tmp/wire-issues-20-21`.
