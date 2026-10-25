# Derive and registry host cleanup

Baseline: released 0.7.9, `00b1686a80cd7a3068be0a24aaee0dc8239009c7`.
Candidate: the source patch and before/after source copies in [raw evidence](raw.tar.gz).
Both changes are host work. Runtime bytes, admission, method identity, metadata
acceptance and production codecs are unchanged. No client regeneration is required
for correctness; generated tokens and TypeScript source match the baseline.

## Retained changes

Field attributes were parsed for their value projection, again for record labels,
and again for generic bounds. The derive now collects each field with its validated
Projection once and shares that result. It removes the later field-list collection
without introducing another type, parsing mode or public API.

Generation previously converted the registry to an owned serde_json::Value tree
before serializing it. Serializing the borrowed map directly removes that entire
temporary tree. JSON escaping, computed __proto__ keys and validation stay intact.

## Traversal audit

| Existing work | Disposition |
| --- | --- |
| Rechecking encountered projection callbacks | Keep: diagnostic type_name labels are not unique type identities; conflicting/dynamic metadata must reject. Function pointers are not a safe replacement identity. |
| Validating the completed registry | Keep: hand-authored metadata can contain invalid shapes, unresolved refs and alias cycles. |
| Validating input to rendering/linting | Keep: these public APIs accept mutable caller-owned maps, not an immutable validated handle. |
| Transparent-alias cycle traversal | Keep: the existing checked set already avoids revisiting completed chains. |
| Rooted lint expansion | Keep: field-specific acknowledgements require independent sibling visits; active-path and work limits bound recursion/DAG expansion. |
| Copying the registry before JSON serialization | Remove: no validation or ownership obligation requires this temporary value. |

No cache, parallel validator or trusted-input bypass was added.

## Measurements

[Full numeric results](summary.csv), including per-run medians. Rust 1.99.1,
LLVM 23.1.1, Linux x86_64 Threadripper 7970X, standalone Cargo release profile
(opt-3, default codegen units, no LTO), identical locked dependencies and inputs.
No CPU pinning; small differences are noise. Three runs, 11 batch samples each,
alternating before/after order, with 30 warmups. Values below are pooled medians
of 33 batch means; allocation/destruction is included.

| Host operation | Before µs | After µs | Change |
| --- | ---: | ---: | ---: |
| Expand plain one-field record | 9.079 | 9.035 | −0.48% |
| Expand attributed one-field record | 11.660 | 10.778 | −7.56% |
| Expand plain 100-field record | 191.844 | 190.432 | −0.74% |
| Expand attributed 100-field record | 444.681 | 352.476 | −20.74% |
| Expand generic 100-field record | 561.464 | 479.313 | −14.63% |
| Expand 100-variant enum | 774.467 | 729.902 | −5.75% |
| Generate one recursive named record + method | 5.072 | 3.015 | −40.56% |
| Generate 100 recursive named records + method | 323.461 | 120.342 | −62.80% |
| Generate 1,000 recursive named records + method | 6,241.752 | 1,322.744 | −78.81% |

Expansion samples use 500 iterations for one field and 25 for 100 fields/variants.
They time the actual expand function plus AST cloning, not source parsing or a
complete Rust build. Generation samples use 1,000/100/10 iterations at the three
registry sizes. Records contain scalar/byte fields and recursive optional refs;
the method includes a result/error response and asymmetric budgets. Output checks
include quote/Unicode field labels and __proto__ handling. All paired token/source
strings match exactly; four rejection diagnostics also match. There are no pooled
median regressions; the last plain-100 expansion run regresses 0.41%, consistent
with its small noisy overall difference. These are not codec/IC speedups or Wasm
savings. No fresh IC-instruction or Wasm-size measurement was needed: runtime code
is untouched and the paired derive output is identical.

## Verification and replay

The full TypeScript gate covers strict generated modules, 159 vectors, 33,560
mutations, 29 native projections, 16 admission shapes and 54 simulated-endian
cases, plus native/Wasm schema forwarding and Rust 1.91. Native registry/admission
tests and unsupported-attribute derive tests remain the behavior checks. Compiler
diagnostics qualify public refusal behavior. Full CI and fresh macOS execution
are outside this local batch.

Extract raw evidence into a disposable directory and run `bash replay.sh` with
prepared Rust/dependencies. The harness directly compiles archived before/after
implementation modules with the unchanged helper/dependency interfaces. It retains
input constructors, emitted tokens/source, raw samples, Cargo.lock and logs.
It does not install tools or modify the repository.
