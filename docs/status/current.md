# Current handoff

## Current batch

Released 0.5.8 (`1894d0a`) is the baseline. PR #15 refreshes the small finite
schema reservation proposal (#14). Fresh encoders request the full
admitted ceiling when it is at most 128 bytes, before initializing the header.
Larger ceilings and scratch reuse keep their existing reservation policy.
Tiny values under small schemas may retain up to 64 additional requested bytes.
Wire identity, single-pass serialization and typed failures remain unchanged.
Package versions are unchanged; pending notes target 0.5.9. Merge and release
remain maintainer-owned. The separate TypeScript growth experiment is unrelated.

## Qualification

The new regression fails on released 0.5.8 and passes with the fix. Focused
library/runtime/native/borrowed tests pass on Rust 1.99.1 and 1.91, as do strict
all-target/all-feature Clippy and Rust 1.91 Wasm compilation. The regression
checks fresh initialization, byte parity and warmed scratch stability.
Existing checks retain oversized output and allocation-failure identities.

Matched native and isolated IC measurements are recorded in
[evaluation](../evaluation.md#small-finite-schema-reservation). Raw probes, locks,
compiler outputs and runs remain under `/tmp/ic-wire-pr15-probe/`.
No full local CI or sibling suites ran. Hosted checks must qualify the refreshed
PR commit independently of the original PR's green checks.

## Retained decisions

0.5.7 adopted IC Tooling 0.63.0, with the release editor selecting `cli` separately
from the diagnostic library. It also bounds registry enumeration to once per
operation and validates optional collection lengths. TypeScript qualification
runs in CI/release gates; see [development](../development.md) for local Node/npm
preparation before offline release validation.

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Finite scratch reuse shipped in 0.5.5. Fresh encoding remains separate because
sharing its mutable-buffer path regressed small-message instruction cost.
The [scratch-buffer study](../evaluation.md#finite-scratch-buffer-follow-up)
retains byte parity, capacity/retry tradeoffs and measured evidence.
Caller-owned scratch capacity may outlive large messages; retain immutable retry
bytes independently. Rejected experiments remain in [evaluation](../evaluation.md).

Shared Tooling remains 0.25.8. IC Build's release editor remains 0.63.0 with `cli`.
Shared snapshots, dependency selection and Rust runtime encoding are unchanged.
Staging, commits, versions, releases and publication remain maintainer-owned.
