# Bounded host schema measurement

This probe answers whether candidate named collection hides obvious host-cost
regressions on shared scalar and recursive registries. It does not select a
replacement or qualify Wasm, IC instructions, browser throughput or JS heap use.
The existing reference remains production owner.

Frozen before the first run: both implementations use the same concrete bool or
recursive Chain type, 1/128/1024 public aliases, prebuilt root names, the same
compiler and Cargo bench profile (opt-level 3), and their selected locked sources.
Candidate defaults and the reference's built-in ceilings must both admit the
workload. Each pair must produce structurally identical JSON with zero tolerance.
Three warmup pairs precede 21 measured pairs; execution order alternates. Report
minimum, median and maximum nanoseconds per phase, without subtracting overhead.
Do not assert timing thresholds in CI or treat sub-microsecond differences as
stable evidence.

Collection includes construction and all registrations, including disposal of the
reference register API's returned root reference. Finalization includes projection
rechecks and validation; the candidate also constructs its typed graph. Export
converts the finished representation to JSON (candidate metadata_json versus
serde_json for the reference map). Total is the sum of these phases. Equality
checks, parsing, output printing and finished-graph destruction are outside the
timed regions. This is the caller cost of the two APIs, not identical internal
work. It excludes declaration/client rendering and allocation instrumentation.

The initial comparison criterion is candidate median total <= reference median
total for each workload (zero accepted regression). Exceeding it is an
investigation finding, not permission to accept a tradeoff. Phase results explain
where costs lie; improvements elsewhere do not erase a failing workload. A single
host run neither establishes statistical significance nor authorizes adoption.
Keep raw samples' summary output and record the compiler, host, lock hash and
source state with any reported observation. Do not change workloads or thresholds
in response to results without documenting a new experiment.

Reproduce with `make bench-schema`. This private harness has no production callers
or new dependencies, and does not resume the legacy benchmark matrix.

## First observation — 2026-11-15

[Per-phase summaries](schema-measurement.jsonl) record the first successful run.
Host: Linux 7.0.0-42-generic x86_64, AMD Ryzen Threadripper 7970X; shared host,
frequency boost enabled, no CPU affinity or frequency pinning. Compiler:
Rust 1.100.0 (`0ef1bc386059596dada8fb65b50425aab2ea637a`), LLVM 23.1.5.
Candidate product code is the 0.23.6 checkpoint `3a9614d`, relabelled pending
0.23.7; this local increment adds scale tests and the probe without changing its
product algorithms. The published reference remains 0.22.4, source-equivalent to
the pinned design revision. Cargo bench uses its default optimized profile;
no opt-z, Wasm or IC measurement is included.

| Shape | Roots | Candidate median total (µs) | Reference (µs) | Initial criterion |
| --- | ---: | ---: | ---: | --- |
| Scalar | 1 | 1.352 | 0.971 | Investigate regression |
| Recursive | 1 | 8.753 | 6.630 | Investigate regression |
| Scalar | 128 | 113.140 | 105.919 | Investigate regression |
| Recursive | 128 | 208.664 | 194.773 | Investigate regression |
| Scalar | 1024 | 919.975 | 970.240 | Below reference in this run |
| Recursive | 1024 | 1622.603 | 1627.249 | Effectively close; no win claimed |

All 144 paired schema comparisons passed; exported byte lengths also matched.
At 128 roots collection was faster for the candidate, while finalization and
export were slower. Candidate export medians were 25.779/28.733 µs versus
7.011/7.751 µs for scalar/recursive reference maps. The next investigation is the
bounded graph-to-JSON writer, while preserving output admission and typed-graph
ownership. This run fails the initial all-workload criterion; no performance
tradeoff or replacement is accepted. Clock overhead is material for the smallest
case and the observed maxima show scheduler noise.

Reproduction SHA-256 inputs:

- `qualification/Cargo.lock`: `d215e3150d65eeec0a4c038f5e597939a0deb86863a864947680eb51302b4e64`
- `qualification/compatibility/benches/schema.rs`: `a74e3848ecbd898bcbec3fc78e8b11cd84d2af175da1963b2941f5630a9d5439`
- `crates/ic-wire/src/typescript/native/mod.rs`: `770b3bb0b23b718cc3804c8916cc111654af8a3c99a9989000f6434e50ae0ede`
- `crates/ic-wire/src/typescript/native/registry.rs`: `2f3b9c8b369ec307c7c2a627a3a15951f9eedb337376ae47aba97c1fb1d2431e`
- `crates/ic-wire/src/typescript/mod.rs`: `b96e1fee7792011c6f798e57d630979c872269a0a8c3ca76186aefbad72a084f`
- `crates/ic-wire/src/typescript/render.rs`: `ea04561b2ddf075a352ea4df475731844aab36bad2418131d726b5747398f774`

## UTF-8 span append observation — 2026-11-15

The exporter previously charged and appended every unescaped character separately.
It now scans the same characters but admits/appends each ordinary UTF-8 span as a
whole, splitting only at escape boundaries. No alternate metadata graph, output
cache, dependency or scalar specialization was added. The admitted output and
escape grammar remain unchanged. A focused test covers all control characters,
quotes/backslashes, multibyte text, supplementary characters and U+2028/U+2029,
checking every output limit through exact success for both JSON and declarations.
All 11 host and 7 cross-language tests (including strict TS checks) pass.

The same probe, workloads, warmups, paired order and criterion were rerun on the
same host/compiler. [Per-phase summaries](schema-measurement-spans.jsonl) retain
this observation separately from the baseline. Only the product renderer changed;
its SHA-256 is now
`11018a6fc530fa23465155db8b889684f23963f7a2dd3b54da21715bdc60d528`.
Other reproduction hashes above remain applicable.

| Shape | Roots | Candidate median total (µs) | Reference (µs) | Initial criterion |
| --- | ---: | ---: | ---: | --- |
| Scalar | 1 | 1.172 | 0.981 | Investigate regression |
| Recursive | 1 | 8.082 | 6.660 | Investigate regression |
| Scalar | 128 | 100.731 | 108.784 | Below reference in this run |
| Recursive | 128 | 184.157 | 187.502 | Below reference in this run |
| Scalar | 1024 | 791.590 | 948.976 | Below reference in this run |
| Recursive | 1024 | 1445.103 | 1572.874 | Below reference in this run |

Candidate 128-root export medians decreased from 25.779/28.733 µs to
14.251/14.472 µs for scalar/recursive cases; corresponding reference medians were
7.311/7.751 µs in this run. Export is still slower than the reference map writer.
All 144 paired schema comparisons passed, with unchanged output sizes. Single-root
regressions remain, so the all-workload criterion is still unmet. This is a focused
host optimization observation, not a statistically qualified performance guarantee
or an accepted tradeoff. Wasm and IC costs remain unmeasured.
