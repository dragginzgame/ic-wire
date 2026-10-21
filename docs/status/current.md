# Current handoff

## Current batch

Released 0.5.7 (`32a343a`) is the baseline. Pending 0.5.8 fixes TypeScript float
metadata bounds and removes per-float encoding/decoding scratch allocations. The packaged
README now recommends `ic-wire = "0.5"`, matching the documented API.
Package versions, index and Git refs remain untouched.

Float `min`/`max` metadata is now enforced inclusively on both encode and decode.
Out-of-range outgoing values fail with `value`; incoming values fail with
`malformed`. Registry generation refuses nonnumeric or inverted bounds with
`GenerationError::Schema`. Existing finite-number and exact f32 admission remain,
including preservation of negative zero. Consumer callbacks still run normally.
Integer and float metadata bounds now share validation. Integer bounds must be
integral and within ±9,007,199,254,740,991, preventing JavaScript rounding before
BigInt conversion. Wider constraints use named consumer checks; bigint payload
ranges are unchanged. Invalid shapes/inversion return `Schema`; out-of-exact-range
integer bounds return `Complexity`.

The writer retains its existing header DataView and writes float bytes directly
into the admitted output buffer. It checks remaining bytes before writing.
A four-f64 probe retains the identical 43-byte frame while explicit Uint8Array
constructions fall from five to one and DataView constructions from five to one.
The returned slice still allocates independent storage. These are constructor
counts, not measured execution time, peak memory or deployment-size savings.
The reader also shares the envelope DataView: four floats need one view instead
of five and no per-float subarrays. Input byte offsets remain respected, and
remaining input is checked before each read.

## Qualification

Focused Linux checks pass: 31 feature-enabled Rust runtime/borrowed tests, strict
Clippy, Rust 1.91 generator compilation, 90 Rust/TypeScript vectors, 17,524
differential mutations and 1 MiB boundaries. The float-bound regression failed
before the fix and passes afterward. New checks cover both widths and directions,
inclusive endpoints, negative zero, exact/insufficient frame allowance, malformed
metadata and absence of per-float explicit buffer/view construction. Decoder
checks cover both widths, nonzero frame offsets and every payload truncation.
Integer-bound checks cover both signs, all five widths, both directions, inclusive
endpoints, invalid metadata and exact-range limits. The new generator regression
failed before the correction.
Sources and local check output are in `/tmp/ic-wire-float-058/` and
`/tmp/ic-wire-reader-058/`; numeric-bound checks are in `/tmp/ic-wire-numeric-058/`.
Full CI, hosted macOS and installed consumers were not run.

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
