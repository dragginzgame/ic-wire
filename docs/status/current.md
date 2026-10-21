# Current handoff

## Current batch

Released 0.5.18 (`bd9e5bc`) is the baseline. Pending 0.5.19 renames the unpublished
`ic-wire-canister-evaluation` package/directory to `ic-wire-qualification` and the
CI target to `check-qualification`. No compatibility aliases are retained.
The allocation fixture, its packed-projection test and compiler diagnostics remain.
The standalone nine-case codec comparison and MessagePack comparison suites are
removed along with their features, drivers and bitcode/wincode/rmp-serde dependencies.
`evaluate-performance` and `evaluate-messagepack` are retired; allocation, protocol
and generator probes remain. Historical reports and raw evidence are preserved,
with old reproduction commands explicitly tied to historical source checkouts.

Production Rust code, protocol bytes, identities, admission and borrowed/owned
semantics are unchanged. This reduces maintained test/tooling scope, not payload
or Wasm size. Package versions, staging and Git refs remain untouched; Cargo.lock
is pruned for the renamed package and removed dependency graph.

## Evidence and qualification

Focused validation covers the renamed package's tests, all-target Clippy and Rust
1.91 check, compile-fail diagnostics, dependency boundaries, documentation links
and the disposable release adapter. No runtime performance claim is made: production
code and retained allocation workloads are unchanged. Full CI and fresh timing
experiments are outside this tooling-retirement batch.

The last complete comparison source/lockfile is revision
`bd9e5bc0ac2c6a057570ae5aae55e0c1f431488b`; use a separate checkout to rerun retired
commands. Historical hashes, data and original package paths in evidence are not
rewritten to imply they came from the renamed crate. Details are in
[performance](../performance.md#retired-alternative-codec-comparisons).

## Retained decisions

Production remains bounded Postcard. Native owned content uses slice equality;
Wasm retains its word comparison. Both skip exact same-input-region comparisons
for borrowed values after length/allowance checks. Existing borrowing and the
released native shortcut remain documented in the
[protocol/layout study](../protocol-study.md).

Released 0.5.17 rejected word-reduction and early-exit native comparisons after
owned-blob regressions; see [native comparison results](../native-comparison-study.md).
Neither [bulk-reservation policy](../bulk-reservation-study.md) replaces current
growth: capacity savings came with multi-chunk or timing regressions.
[Native borrowed-view qualification](../native-views-study.md) compares existing
owned/borrowed APIs, not a new optimization. It finds 21–25% lower 1 MiB decode
plus byte-scan time and 45–46% lower sixteen-text-pair time. Blob-storage already
borrows download descriptor text; its chunk transport remains provider-specific.

Hypothetical layout cuts remain experiments. Generic single-pass canonical
decoding is unimplemented; normalized maps and arbitrary Serde semantics require
more than varint checks. Exact retries, canonical hashing and typed errors remain
required. Metadata and scalar validators are trusted declarations. Earlier
runtime evidence is in [evaluation](../evaluation.md); generator scaling evidence
is in [performance](../performance.md#typescript-generator-comparisons).
Shared Tooling is 0.25.9; IC Build's release editor is 0.63.0 with `cli`.
Ordinary agent work does not stage, commit or release.
