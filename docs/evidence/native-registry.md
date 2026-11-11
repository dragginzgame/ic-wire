# Native recursive registries and projected budgets

## Registration-independent names — 11 November 2026

Generated definitions now use `Wire_TypeName_<16-hex-suffix>` from the complete
Rust type spelling rather than `WireType{index}`. The suffix includes namespace
and generic arguments and is always present: adding a namesake must not rename
an existing export. A collision rejects rather than allocating another number.
Explicit roots consistently alias these definitions, including when registered
before another root encounters the type. Projection-collision checks remain.

Consumers regenerate schemas/clients and update old positional imports once.
Names are stable under root reordering and unrelated additions, not Rust module
renames or changes to compiler type spellings. They are metadata labels, not
contract fingerprints. Bytes, method identities, limits and runtime acceptance
are unchanged. The implementation is host-only; no runtime Wasm saving is claimed.

Matched opt-z host study: Rust 1.99.1, identical locks, isolated baseline/candidate
sources and targets, eleven samples per run, two runs each. The synthetic fixture
registers many roots sharing recursive definitions; it is **not** Toko's unavailable
1,657-distinct-type schema. Generation includes collection, validation, advice and
TypeScript rendering. Median milliseconds (first run / serial repeat):

| Roots | Before ms | After ms | Generated TS bytes before → after |
| --- | ---: | ---: | ---: |
| 1 | 0.074 / 0.067 | 0.080 / 0.078 | 2,442 → 3,580 |
| 298 | 2.280 / 2.215 | 2.520 / 2.569 | 21,828 → 36,034 |
| 1,657 | 14.616 / 14.157 | 15.158 / 16.568 | 112,836 → 186,838 |

This accepts extra host generation time and longer TypeScript identifiers for
stable imports. Native timings are shared-host observations, not IC instructions;
the initial runs could overlap and the serial repeat is the stronger timing check.
The full Toko build and its generated JavaScript bundle have not been measured.
Raw sources, compiler/lock/source hashes and logs are retained under
`target/evidence/toko-feedback-20261111/`. Reproduce with the ignored native test
`measure_named_registry_generation` using separate source/target directories.

Regression coverage includes root order, unrelated insertion, namespace clashes,
generic instantiations, explicit-name collisions, recursive schemas, projection
conflicts and unchanged rooted acknowledgement paths. Rust/TypeScript byte and
malformed-input conformance passes after regeneration.

## Historical 0.7.7 qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

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
Retired raw output (see retention notice) retain the differences, including the
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

Retired raw output (see retention notice) retain reproduction
inputs. Extract and run `bash replay.sh` with prepared Rust/dependencies; it uses
an isolated temporary tree and offline Cargo. Maintained `make check-typescript`
reproduces cross-runtime qualification with the documented Node/TypeScript setup.
Original artifacts/logs remain under `/tmp/wire-issues-20-21`.

## Reusable lint paths — 11 November 2026

The rooted advisory walker now appends to one traversal-owned path and truncates
it when returning from each child. Only warnings retain full path copies. This
removes per-edge path allocation without memoizing use sites, dropping warnings,
changing acknowledgement escaping or increasing work/depth allowances. Early
errors still discard the whole report. Codec bytes, schema output, names,
identities, limits and acceptance are unchanged; no consumer migration is needed.

Matched sources under `target/evidence/lint-paths-20261111/{before,after}/source`
use Rust 1.99.1, identical locks and separate Cargo targets. The existing
`schema-cost` example now hashes both metadata and bulk-lint sources into its
identity. Two runs/profile, 11 calibrated batches/run (at least 10 ms/batch),
opt-z evaluation and opt-3 benchmark-speed. Timing includes stats_alloc and
output destruction on a shared host. Input and generated-output hashes match.

Generation medians across 22 batches; allocations exclude reallocations:

| Shape | opt-z before → after | Allocations before → after | opt-3 before → after |
| --- | ---: | ---: | ---: |
| One-field record | 2.174 → 2.173 µs | 24 → 23 | 1.535 → 1.539 µs |
| 4,096-field record | 2.400 → 2.268 ms | 25,228 → 21,132 | 1.505 → 1.362 ms |
| 128 fields × 32 options | 1.413 → 1.211 ms | 8,997 → 4,773 | 1.024 → 0.751 ms |
| Invalid unused definition | 1.191 → 1.208 ms | 12,929 → 12,929 | 0.867 → 0.883 ms |

Large-record reallocations fall 12,218 → 4,127; nested reallocations fall
12,083 → 7,835. The invalid-definition path fails before linting and has unchanged
allocation counts; its small timing regression is recorded, not erased.
Small-input timing is essentially flat. Host-only code is excluded on Wasm:
there is no canister size or IC instruction saving claim. Existing recursive,
shared-use-site, escaped-name, tuple/result/enum and budget-boundary tests pass.

Reproduce from each frozen source with `cargo run --locked --offline
--manifest-path qualification/Cargo.toml -p ic-wire-qualification --example
schema-cost --features schema-cost --profile evaluation`, then benchmark-speed,
selecting a distinct CARGO_TARGET_DIR for each source. Raw samples are retained
beside the source directories as `<profile>-<run>.jsonl`.


## Borrowed lint names — 11 November 2026

The advisory traversal now borrows reference names in its active recursion set
and ordinary field names during JSON-pointer escaping. Names containing slash or
tilde retain the existing owned escaping path. This removes temporary strings;
it does not memoize references or drop repeated use-site warnings. Generated
source, warning/acknowledgement paths, codec bytes, identities, budgets and
acceptance remain unchanged. No consumer action is required.

Matched sources and raw samples are retained under
`target/evidence/lint-borrows-20261111/{before,after}`. Reproduce with the same
schema-cost commands above and separate targets. Rust 1.99.1, matching root and
qualification locks, compiler pin and harness; two runs/profile, eleven calibrated
batches/run of at least 10 ms, with stats_alloc instrumentation. Baked metadata +
lint source hashes were independently checked against each source; all 48 paired
input/output/validation comparisons match. The two new controls cover escaped
names and a shared recursive definition. These are synthetic host measurements,
not a full application registry or Candid comparison.

Generation medians for runs 1 / 2 (allocations exclude reallocations):

| Shape | opt-z before → after (µs) | Allocations before → after | opt-3 before → after (µs) |
| --- | ---: | ---: | ---: |
| One field | 2.253 / 2.274 → 2.205 / 2.226 | 23 → 20 | 1.533 / 1.545 → 1.509 / 1.519 |
| 4,096 plain fields | 2,655 / 2,342 → 2,572 / 2,330 | 21,132 → 12,939 | 1,397 / 1,400 → 1,312 / 1,265 |
| 128 fields × 32 options | 1,263 / 1,354 → 1,395 / 1,276 | 4,773 → 4,516 | 799 / 835 → 828 / 837 |
| Invalid unused definition | 1,406 / 1,344 → 1,266 / 1,273 | 12,929 → 12,929 | 947 / 923 → 882 / 941 |
| 4,096 shared recursive references | 4,711 / 4,696 → 4,462 / 4,215 | 50,196 → 17,427 | 2,965 / 3,018 → 2,626 / 2,616 |
| 4,096 escaped fields | 2,685 / 2,541 → 2,691 / 2,579 | 21,123 → 21,122 | 1,514 / 1,474 → 1,554 / 1,531 |

Retained for deterministic allocation reduction and shared-reference improvement.
Plain-field allocated bytes fall 2,511,826 → 2,440,224; shared-reference bytes fall
3,492,161 → 3,305,871. Nested timings vary in direction; escaped-field opt-3
medians regress about 2.6–3.8%. No universal latency improvement is claimed.
The invalid definition fails before linting and serves as a host-noise control.
This code is host-only: IC instructions and uncompressed Wasm savings are not
applicable and were not measured. Existing native tests cover recursive/shared
references, escaped paths, acknowledgements and work/depth limits. All 87 active
native tests and strict Core/private-harness Clippy pass.


## Empty-acknowledgement shortcut — rejected 11 November 2026

Tested returning the complete warning report immediately after the bounded walk
when the acknowledgement map is empty, avoiding the temporary warning-path index.
The existing nonempty acknowledgement path was unchanged. The shortcut is removed;
the borrowed-name improvement above remains. No codec or generated-output change.

Sources/samples: `target/evidence/lint-empty-20261111/{before,after}`; same
schema-cost command and separate targets as above. Rust 1.99.1, matching compiler
pin, locks and harness. Baked source hashes verified. Four runs/profile, eleven
calibrated batches/run; runs 1–2 before then after, runs 3–4 reversed using the
fresh binaries. All 96 input/output/validation comparisons match. Native tests
(87 active) and strict Core Clippy passed for the candidate. Acknowledgement
coverage includes empty, partial, complete, stale and invalid reasons.

The 4,096 shared-reference fixture removes 377 allocations (17,427 → 17,050)
and 107,704 allocated bytes (3,305,871 → 3,198,167), with no allocation change in
the five controls. These are allocation traffic, not peak heap measurements.
Shared-reference generation medians, before → after in milliseconds:

| Run | opt-z | opt-3 |
| --- | ---: | ---: |
| 1 | 4.381 → 4.261 | 2.609 → 2.603 |
| 2 | 4.236 → 4.617 | 2.645 → 2.621 |
| 3 (reverse order) | 4.218 → 4.328 | 2.416 → 2.482 |
| 4 (reverse order) | 4.087 → 4.124 | 2.405 → 2.445 |

The escaped-name control regresses in all eight comparisons (about 2–19%);
small-record opt-z medians regress about 1–4%. Even the unchanged invalid-definition
control varies, so host noise prevents causal attribution of every timing loss.
There is no repeatable latency win to justify retaining this branch for a 2.2%
allocation-count reduction in the warning-heavy fixture. Reject rather than claim
an across-the-board improvement. No IC or uncompressed Wasm measurement applies:
this is host-only advisory generation. No runtime change or new release note.


## Direct registry append — 11 November 2026

Append the serialized registry and its fixed declaration prefix/suffix directly
to the output module instead of formatting one temporary string containing the
whole registry. JSON serialization and computed __proto__ escaping remain the
same. No API, generated bytes, warning paths, identity, budget or acceptance
change; no consumer migration. This removes a copy rather than adding a cache,
new mode or specialization.

Matched sources/samples are under
`target/evidence/registry-append-20261111/{before,after}`, with aggregate checks
in `verification.json`. Use the schema-cost reproduction command above. Rust
1.99.1, identical compiler pin, both locks and harness, separate fresh targets;
baked metadata/lint source hashes match their frozen files. Four runs/profile,
eleven calibrated batches/run, at least 10 ms/batch; runs 3–4 reverse source
order. All 96 input/output/validation comparisons match. Both lanes include the
pending borrowed-name change; the rejected empty-acknowledgement branch is absent.

Median of 44 batches; native generation time includes stats_alloc and output
destruction. Allocated bytes are allocation traffic, not measured peak heap:

| Shape | Allocated bytes before → after | opt-z µs before → after | opt-3 µs before → after |
| --- | ---: | ---: | ---: |
| small | 2,914 → 2,734 | 2.15 → 2.12 | 1.45 → 1.43 |
| large | 2,440,224 → 2,000,178 | 2266.47 → 2230.92 | 1204.46 → 1247.03 |
| nested | 4,845,668 → 4,655,066 | 1220.84 → 1206.53 | 768.33 → 759.76 |
| invalid-unused | 1,200,057 → 1,200,057 | 1245.07 → 1210.68 | 852.23 → 873.39 |
| shared-ref | 3,305,871 → 2,636,015 | 4204.41 → 4145.61 | 2450.86 → 2546.99 |
| escaped | 2,728,488 → 2,272,058 | 2605.51 → 2503.72 | 1407.20 → 1424.92 |

Successful generation removes one allocation; invalid unused definitions reject
before this append and have unchanged allocation traffic. Retain for 18%/20%
less allocated traffic in the plain/shared-reference fixtures and straightforward
code. Opt-z timing differences are small; no universal speedup is claimed.
Opt-3 large/shared-reference medians regress 3.5%/3.9%, and escaped names 1.3%.
The unchanged invalid-input control also varies, demonstrating host noise.
No runtime Wasm size or IC instruction result applies to this host-only change.
87 active native tests, strict Core Clippy, strict generated TypeScript and the
Rust/TypeScript exact-frame, malformed-input and admission checks pass, including
computed __proto__ fixtures. Formatting and documentation links also pass.

## Direct declarations and conditional key escaping — 11 November 2026

Write type, method and payload declarations directly into the output String with
writeln!, removing temporary formatted declarations. Reuse the serialized registry
String when it contains no literal `"__proto__":` key; keep the existing computed-key
replacement when it does. This changes neither the escaping algorithm nor generated
bytes. Runtime codec bytes, identities, limits, acceptance and diagnostics remain
unchanged; consumers need no migration. No cache or additional public API is added.

Matched study: target/evidence/generator-output-20261111, with paired source trees,
run.sh/final-run.sh, fresh build records and analyze.mjs/verification.json. Rust
1.99.1; identical root/qualification locks and harness; separate targets. Source
hashes baked into every row match metadata/lint files, and all 128 paired input,
output and validation results agree. Final candidate uses writeln!; final timings
were repeated after correctness builds finished. Four runs per profile, eleven
batches of at least 10 ms/run; runs 3–4 reverse lane order. Times include allocator
instrumentation and output destruction. Allocation traffic is not peak heap.
The baseline already includes the earlier borrowed lint names/direct registry append.

| Shape | Allocations before → after | Allocated bytes before → after | opt-z µs before → after | opt-3 µs before → after |
| --- | ---: | ---: | ---: | ---: |
| small | 19 → 16 | 2,734 → 2,163 | 2.10 → 2.07 | 1.40 → 1.29 |
| large | 12,938 → 12,935 | 2,000,178 → 1,585,198 | 2227.18 → 2257.46 | 1187.59 → 1217.06 |
| nested | 4,515 → 4,512 | 4,655,066 → 4,339,106 | 1208.96 → 1200.31 | 732.93 → 717.39 |
| invalid-unused | 12,929 → 12,928 | 1,200,057 → 1,078,392 | 1187.28 → 1214.29 | 825.84 → 851.54 |
| shared-ref | 17,426 → 17,422 | 2,636,015 → 2,183,754 | 4169.43 → 4159.66 | 2365.93 → 2417.32 |
| escaped | 21,121 → 21,118 | 2,272,058 → 1,832,502 | 2471.05 → 2500.01 | 1370.50 → 1401.00 |
| 256 additional types | 829 → 570 | 108,912 → 93,859 | 101.28 → 102.81 | 58.48 → 58.38 |
| prototype key | 422 → 419 | 67,674 → 59,954 | 65.04 → 65.73 | 32.66 → 34.03 |

Retain for straightforward removal of temporary allocations: 20.7%/17.2% less
allocated traffic on large plain/shared-reference registries, and 31.2% fewer
allocations in the many-type control. The prototype-key case exercises the fallback;
invalid unused definitions still reject, but preceding declaration rendering now
allocates less. Timing is mixed: opt-3 large/shared regress 2.5%/2.2%, prototype-key
4.2%; opt-z large regresses 1.4%. Do not claim universal latency improvement or
attribute all variation on this shared host to the candidate. These two changes
were measured together; the table does not isolate their individual contributions.

109 active borrowed/native/runtime tests pass (four ignored probes), strict Core
Clippy passes, and generated TypeScript compiles with strict settings. Conformance
passes 178 bidirectional vectors, 39,092 differential mutations, 1 MiB buffers,
42 scalar vectors, payload codecs, admission and incremental writers. Simulated
big-endian packed checks pass 78 cases. The maintained schema-cost harness adds
many-type and prototype-key controls. This is host-only code; runtime Wasm size
and IC instruction measurements are not applicable and no such saving is claimed.

## Direct composite rendering — 11 November 2026

Render tuple elements and record/enum fields into one String instead of collecting
formatted fragments in a Vec, joining them, then formatting the wrapper. Child
validation, duplicate-name/variant-ID checks, traversal order and work/depth
allowances remain unchanged. Validation-only traversal still emits no strings.
Exact generated bytes, diagnostics, runtime bytes, identities, limits and acceptance
are unchanged; no consumer migration. No cache or second traversal is introduced.

The baseline includes direct declarations and conditional prototype escaping above.
Paired evidence: target/evidence/schema-joins-20261111, with frozen sources,
run.sh, final-evaluation.sh, raw rows/build records, analyze.mjs and verification.json.
Rust 1.99.1, identical root/qualification locks and harness, separate targets.
Four runs/profile, eleven calibrated batches of at least 10 ms/run; reverse order
for runs 3–4. Baked metadata/lint hashes match the final sources; all 128 paired
input/output/validation comparisons pass. Fresh library records are verified.
The first opt-z candidate captured an unused Vec import removed during compilation;
the source verifier rejected that run. It remains under superseded-evaluation.
Final opt-z results were rebuilt in a fresh target and remeasured on both lanes;
no stale or identity-mismatched sample supports the retained result.

Native generation medians over 44 batches, including stats_alloc and destruction:

| Shape | Allocations before → after | Allocated bytes before → after | opt-z µs before → after | opt-3 µs before → after |
| --- | ---: | ---: | ---: | ---: |
| small | 16 → 13 | 2,163 → 1,976 | 2.07 → 1.98 | 1.36 → 1.17 |
| large | 12,935 → 8,837 | 1,585,198 → 1,072,434 | 2233.34 → 2083.02 | 1239.73 → 1045.66 |
| nested | 4,512 → 4,382 | 4,339,106 → 4,115,804 | 1181.38 → 1149.67 | 762.82 → 711.39 |
| invalid-unused | 12,928 → 8,830 | 1,078,392 → 565,628 | 1216.89 → 1061.62 | 879.55 → 687.11 |
| shared-ref | 17,422 → 13,320 | 2,183,754 → 1,695,196 | 4158.45 → 3909.43 | 2507.97 → 2266.28 |
| escaped | 21,118 → 17,020 | 1,832,502 → 1,295,162 | 2510.68 → 2373.71 | 1443.57 → 1220.93 |
| many-types | 570 → 567 | 93,859 → 93,672 | 103.25 → 99.61 | 55.93 → 54.71 |
| prototype key | 419 → 289 | 59,954 → 44,542 | 65.41 → 59.58 | 34.36 → 28.01 |

Retain: the large-record fixture removes 31.7% of allocations and 32.3% of allocated
traffic, with 6.7%/15.7% lower measured opt-z/opt-3 time. Shared references remove
23.5% of allocations and 22.4% of traffic, with 6.0%/9.6% lower measured time.
No aggregate case median regresses; shared-host timings remain observational,
not an application guarantee. Allocation traffic is not peak heap. These record-heavy
fixtures do not isolate enum/tuple speedups; those shapes have conformance coverage.

109 active borrowed/native/runtime tests and strict Core Clippy pass. Strict
TypeScript compilation, 178 bidirectional vectors, 39,092 differential mutations,
1 MiB byte buffers, 42 scalar vectors, payload/recursive-enum/tuple checks, admission
and incremental writers pass. Simulated big-endian packed checks pass 78 cases.
Host-only rendering does not enter the canister codec: IC instructions and runtime
Wasm measurements are not applicable, and no Wasm saving is claimed.
