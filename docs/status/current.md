# Current handoff

## Current batch

Released 0.5.19 (`0381af2`) includes the qualification-crate rename and retirement
of alternative-codec suites. The follow-up audit changes documentation only:
released work is no longer described as pending, and the development guide links
to the recorded Shared Tooling pin instead of duplicating an outdated revision.
No production, test or benchmark code is removed in this follow-up.

The maintained crate is `ic-wire-qualification`; its CI target is
`check-qualification`. It retains the allocation/borrowing fixture, packed-projection
test and compiler diagnostics. Protocol and generator probes also remain.
The old crate/target names occur only in historical reports and evidence or
retirement explanations. No active build/CI references or compatibility aliases
remain. Bitcode, wincode and MessagePack dependencies are absent from the workspace.

## Evidence and qualification

The audit reviewed workspace manifests, Make/CI wiring, native and finite codec
boundaries, derives, bounded projections, qualification fixtures and benchmark
entry points. The remaining similar flows have distinct ownership, admission or
error contracts; no further safe code deletion was identified in this pass.

Offline locked dependency metadata, dependency-boundary checks, snapshot integrity,
documentation links and whitespace checks pass. Runtime tests and timing probes
were not rerun for documentation-only changes. Protocol bytes, resource limits,
public APIs, dependencies, versions, staging and Git refs are unchanged.

Historical benchmark evidence is retained. The last complete alternative-codec
comparison source/lockfile is `bd9e5bc0ac2c6a057570ae5aae55e0c1f431488b`; old
commands require a separate historical checkout. See
[retired comparisons](../performance.md#retired-alternative-codec-comparisons).

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
