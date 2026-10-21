# Current handoff

## Current batch

Released 0.5.11 (`9190e54`) is the baseline. Pending 0.5.12 optimizes the generated
TypeScript runtime and expands qualification. Package versions, index and Git
refs remain untouched. No wire-format or public API changes are introduced.

Small integer payloads (8/16/32 bits) use number arithmetic; 64/128-bit values
retain bigint arithmetic. The dedicated number varint path preserves unsigned
32-bit values, signed ZigZag boundaries, minimal encodings and overflow refusal.
Length prefixes and method identities retain their existing handling.

Text encoding remains unchanged. Both a JavaScript UTF-8 length scan and direct
frame encoding followed by an in-place prefix shift were investigated. The scan
regressed large text; the shift had inconsistent gains and repeated regressions.
The experiments are retained as evidence, not production paths.

## Qualification

Focused TypeScript qualification passes: 32 Rust runtime/borrowed tests, strict
all-target feature Clippy, Rust 1.91 generator compilation, strict TypeScript,
145 Rust vectors, 29,356 differential mutations and 1 MiB frame boundaries.
The corpus now includes all small-integer varint transitions, signed ZigZag
boundaries and explicit malformed prefixes. Extra checks cover nested node/depth
limits, both directions, text prefix growth and independent returned frames.
Owned and borrowed Rust byte/text leaves agree on mutated inputs and typed errors.

An isolated comparison of text prototypes covers 8,600 combinations of strings,
allowances and directions with identical bytes/errors. Existing allocation
checks continue to bound short-string scratch; no new text path is retained.
Native Node benchmark inputs, environment and results are recorded in
[evaluation](../evaluation.md#typescript-small-integer-and-text-encoding-study).
Raw variants, scripts and logs are under `/tmp/ic-wire-0512/`.
These measurements are not browser or IC instruction claims. Full CI, hosted
macOS and installed consumer checks were not run for this working-tree batch.

## Retained decisions

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
The frame format, canonical re-encoding and exact retry bytes remain unchanged.
Fresh Rust allocation and scratch reuse retain their separately qualified paths.
TypeScript buffer growth remains rejected; the writer still respects its selected
frame allowance. Historical experiments and their tradeoffs remain in
[evaluation](../evaluation.md).

Shared Tooling is 0.25.9. IC Build's release editor remains 0.63.0 with `cli`.
Dependency selections and Rust production code are unchanged by this batch.
Staging, commits, versions, releases and publication remain maintainer-owned.
