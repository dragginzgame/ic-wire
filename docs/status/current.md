# Current handoff

## Current batch

Pending 0.5.21 improves owned byte/text decoding on Wasm. A separate bulk
comparison helper processes 64-byte blocks while preserving exact canonical
validation, bounds, error precedence and the existing borrowed-input shortcut.
Native slice equality and protocol bytes are unchanged; no consumer migration
is required. Package versions, staging and Git refs are untouched.

Against rebuilt release 0.5.20 (`a5b80e6`), owned 1 MiB decode uses 8.7% fewer
IC instructions at opt-z and 6.1% fewer at opt-3. Whole-fixture Wasm grows
200/327 bytes, and gzip grows 109/201 bytes. Some small decodes regress: the
worst percentage case adds 73 instructions (3.48%) for a 127-byte blob. Candid
still wins owned 1 MiB decoding. Two alternative loops were rejected.
See [results, tradeoffs and reproduction](../candid-comparison.md#owned-blob-follow-up--pending-0521).

## Evidence and qualification

The complete 114-case IC corpus has three agreeing counts per measured operation
and exact native-frame equality for all 2,838 rows. Of these, 75 improve, 34
regress and 2,729 are unchanged. Baseline/candidate builds use identical inputs,
compiler settings and lockfiles, with separate opt-z/opt-3 Wasm and gzip sizes.
Native benchmark binaries are byte-identical before/after in both profiles.
Runtime tests, strict Clippy, native/Wasm Rust 1.91 and benchmark corpus checks
pass. Mismatch coverage extends through 257-byte unaligned inputs, and a bulk
mismatch still checks subsequent output allowance. Full CI/macOS were not run.

## Allocation-budget follow-up

The pending batch also stores native heap allowance in whole eight-byte charge
units. Independent boundary/overflow tests preserve the old admission semantics.
Relative to the owned helper above, Wasm shrinks 326/796 bytes and gzip shrinks
89/222 bytes at opt-z/opt-3. IC rows: 419 improve, two regress (under 0.025%),
2,417 unchanged. Allocations and peak heap are unchanged. Native timing controls
show host drift, so no causal native speedup is claimed. Focused runtime, Clippy,
MSRV and benchmark checks pass. Protocol bytes and consumer limits are unchanged.

Generic byte-vector encoding remains expensive. Byte-oriented Serde encoding
uses about 48× fewer IC instructions for 1 MiB at opt-3 with identical bytes;
corpus assertions cover matching 4 KiB/1 MiB frames. Consumer guidance explains
the representation and accounting tradeoff. See [results and reproduction](../candid-comparison.md#allocation-budget-units--pending-0521).

## Released foundation

0.5.20 shipped derive name-resolution fixes, shared TypeScript safe-integer
bounds, the redundant finite-frame check removal and the first Wasm comparison
optimization. It also added the unpublished Candid/PocketIC benchmark package.
Published crates remain independent of Candid and simulators. The benchmark
runner follows Cargo artifact paths and rejects incomplete/duplicate comparisons.
Its retained complete rerun is `target/codec-comparison-2026-10-22-rerun/`;
[methodology and earlier results](../candid-comparison.md) distinguish native
latency, IC instructions, allocations, peak heap and deployment sizes.

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
Shared Tooling is 0.26.6 at `4cb9bc9`; IC Build/Testkit and transitive Host
libraries select 0.64.0. The release editor is 0.64.0 with `cli`.
Platform imports already satisfy the 0.64 ownership changes. Host/Cargo-only
setup needs no IC executable selection. Runtime and protocol are unchanged.
Snapshot, tool admission, pin/boundary, formatting/link, disposable Make/release,
diagnostic, qualification and benchmark lint/MSRV checks pass on Linux. The 0.64
Host/Testkit driver also passes the complete PocketIC frame check against the
existing candidate Wasm: all 2,838 rows and repeated counts are identical.
Evidence: `target/tooling-064-adoption/` and `/tmp/ic-wire-tooling-adoption/`.
This validates host adoption, not rebuilt Wasm performance; full CI/macOS remain
unrun. Earlier benchmark compiler/lock identities remain historical evidence.
Ordinary agent work does not stage, commit or release.
