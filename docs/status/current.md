# Current handoff

## Current batch

Released 0.5.13 (`752a308`) is the baseline. Pending 0.5.14 expands TypeScript
length-prefix qualification, records a rejected short-length fast path, and fixes
byte ownership for Node Buffer inputs. Decoded bytes now use an explicit
Uint8Array copy instead of the input subtype's slice method. Package versions,
staging and Git refs remain untouched; ordinary agent work does not release.

The prototype peeked at one-byte length prefixes and used number arithmetic,
falling back to the existing u64 decoder for longer prefixes. It preserved bytes
and typed errors, but longer-text performance regressed in repeated controls.
Do not restore it solely on the strength of its short-blob/list improvements.

## Qualification

Focused TypeScript qualification passes with the retained runtime: 32 Rust
runtime/borrowed tests, strict all-target feature Clippy, Rust 1.91 generator
compilation, strict TypeScript, 146 Rust vectors, 29,560 differential mutations
and 1 MiB frame boundaries.

New checks cover every one-byte length for bytes/text/lists, exact and exceeded
schema bounds in both directions, and malformed u64 prefixes. An external probe
compares 150,000 deterministic inputs between the rejected prototype and baseline,
including typed errors. Existing traversal and allocation limits remain intact.

Every Rust vector is also decoded from an offset Buffer view and checked after
overwriting the input, in both directions. Separate checks ensure mutating decoded
bytes cannot change a Buffer frame. The baseline aliases storage; the fix does not.

Native Node measurements use two initial runs and two longer control runs.
Longer runs improve short blobs/text and nested lists, but 4 KiB text regresses
13–32%. See [evaluation](../evaluation.md#typescript-short-length-decoding-experiment)
for parameters, source identities, all results and limitations. Raw sources,
scripts and samples are in `/tmp/ic-wire-0514/` and
`target/typescript-length-prefixes/`. The length fast path remains rejected.

The byte ownership fix improves 4 KiB blob decoding about 24% and 1 MiB decoding
6–12% in two native Node runs; 32-byte blobs regress 2–6%. This is retained for
correctness, not as a universal speedup. See
[evaluation](../evaluation.md#typescript-byte-ownership-correction) for inputs and
limits. Raw evidence is in `/tmp/ic-wire-blob-owned/` and
`target/typescript-byte-ownership/`.

Full CI, hosted macOS and installed consumer checks were not run for this batch.

## Retained decisions

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Canonical bytes, exact retries, typed errors and pre-allocation bounds remain
unchanged. Generated metadata and scalar validators remain trusted declarations.

Released small-integer and enum-tag optimizations remain selected. Wide integers,
length prefixes and method identities retain bigint handling. Text scanning,
direct frame text encoding and bounded-growth writers remain rejected following
measured regressions. Historical evidence stays in [evaluation](../evaluation.md).

Fresh Rust allocation and scratch reuse retain their separately qualified paths.
Shared Tooling is 0.25.9. IC Build's release editor remains 0.63.0 with `cli`.
Rust production code and dependency selections are unchanged. Staging, commits,
versions, releases and publication remain maintainer-owned.
