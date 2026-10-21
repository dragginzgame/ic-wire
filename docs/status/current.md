# Current handoff

## Current batch

Released 0.5.15 (`39535ba`) is the baseline. Pending 0.5.16 includes generator
validation cleanup and a measured native borrowed-buffer optimization.
Protocol bytes, identities, limits and malformed-input acceptance are unchanged.
Package versions, staging and Git refs remain untouched; releases are maintainer-owned.

Native metadata duplicate checks use ordered sets rather than repeated prefix
scans. Finite projection shares name/tag validation, checks adjacent record IDs,
and refuses oversized enum variant lists before copying/sorting them.

The canonical comparator now recognizes identical input regions on native hosts,
as it already did on Wasm. It checks allowance/length first, then avoids comparing
a slice with itself. Other regions still receive exact content comparison.
Borrowed 4 KiB decode improves about 65%; 1 MiB decode falls from about 9.8 us to
14 ns in paired native runs. These timings exclude processing/hashing the decoded
blob and do not imply additional Wasm or IC instruction savings.

Consumer layout experiments remain unadopted. Compact enum IDs save two bytes in
the example; mandatory/fixed-length keys save one byte but change valid values.
Fixed-key decoding regresses about 24–25% in the experiment. The probe uses new
identities and inline byte/cutover descriptions for incompatible layouts. No
consumer schema or automatic migration is changed. Generic single-pass canonical
decoding is not implemented: varint minimality alone cannot cover normalized maps
or arbitrary Serde semantics.

## Evidence and qualification

[Generator results](../performance.md#typescript-generator-comparisons) show
82–89% gains for 1,024-entry registries; small controls vary from 7% faster to 5%
slower. Those timings do not cover finite-projection/refusal changes. All 149
generated TypeScript files were unchanged against the released generator.

[Protocol/layout study](../protocol-study.md) records canonical stage timings,
small/owned/integer controls, frame identities and the reproducible isolated
runner. Exploratory evidence is in `/tmp/ic-wire-protocol-study/`; confirmation
builds and samples are in `/tmp/ic-wire-protocol-confirm/`. Baseline/candidate
builds use the same working-tree dependency lockfile; its independently arriving
edit was preserved. No universal speedup is claimed; integer controls include a
5% regression in one run and a 1% improvement in the other.

Focused Rust runtime, borrowed and native tests pass. The TypeScript gate passes
32 runtime/borrowed tests, strict feature Clippy, Rust 1.91 compilation, strict
TypeScript, 146 vectors, 29,560 differential mutations and 1 MiB boundaries.
The probe asserts raw Postcard's acceptance and Wire's rejection of a nonminimal
integer. The isolated runner compiled both variants and compared all frame files.
Full CI, hosted macOS and installed consumer checks were not run for this batch.

## Retained decisions

AGENTS.md requires benchmark reporting for cost changes and inline old/new behavior
plus compatibility notes for protocol changes. Production remains bounded Postcard;
JSON is generation/tooling metadata. Exact retries and canonical hashing remain.
Generated metadata and scalar validators remain trusted declarations.
Small-integer/enum-tag optimizations and Node Buffer byte ownership remain selected.
Rejected runtime experiments remain documented in [evaluation](../evaluation.md).
Shared Tooling is 0.25.9; IC Build's release editor is 0.63.0 with `cli`.
