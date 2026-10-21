# Current handoff

## Current batch

Released 0.5.14 (`42ecc9b`) is the baseline. Pending 0.5.15 rejects `as` as a
TypeScript registry declaration name and rejects non-string scalar-check
annotations. The existing shared declaration-name
check now returns `GenerationError::Schema` for that name, rather than producing
`export type as = ...`, which TypeScript rejects. Escaped field and variant
names are unaffected; runtime behavior and wire bytes are unchanged.

The regression extends the existing invalid-name tests for both registry types
and method declarations. Package versions, staging and Git refs remain untouched;
ordinary agent work does not release.

Scalar-check validation now requires a string name on every schema node, including
named references and nested values. Null, numeric, boolean, array and object
annotations return `GenerationError::Schema`; named consumer validators remain
accepted. This changes generator admission only, without adding runtime checks.

## Qualification

The invalid type declaration was independently reproduced with the installed
TypeScript 7.0.2 compiler (`TS1005`). Other contextual names were probed without
broadening the refusal list beyond the demonstrated failure. Local probe files
and the focused-check log are under `/tmp/ic-wire-0515/`.

The malformed scalar-check regression failed against the prior generator, which
returned source containing `check:null`. It now passes for five invalid annotation
types across leaf, reference and nested schemas. Before/after check logs are in
`/tmp/ic-wire-0515-checks/`.

Focused TypeScript qualification passes: 32 Rust runtime/borrowed tests, strict
all-target feature Clippy, Rust 1.91 generator compilation, strict TypeScript,
146 Rust vectors, 29,560 differential mutations and 1 MiB frame boundaries.
Formatting, documentation links and whitespace checks pass. Full CI, hosted
macOS and installed consumer checks were not run for this batch.

## Retained decisions

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Canonical bytes, exact retries, typed errors and pre-allocation bounds remain
unchanged. Generated metadata and scalar validators remain trusted declarations.

Released small-integer and enum-tag optimizations remain selected. Wide integers,
length prefixes and method identities retain bigint handling. The 0.5.14 explicit
Uint8Array copy preserves decoded-byte ownership for Node Buffer inputs.

Short-length fast paths, text scanning, direct frame text encoding and bounded
writer growth remain rejected following measured regressions. Historical evidence
and retained script locations stay in [evaluation](../evaluation.md).

Fresh Rust allocation and scratch reuse retain their separately qualified paths.
Shared Tooling is 0.25.9. IC Build's release editor remains 0.63.0 with `cli`.
Dependency selections are unchanged. Staging, commits, versions, releases and
publication remain maintainer-owned.
