# Current handoff

## Current batch

Released 0.5.12 (`8562529`) is the baseline. Pending 0.5.13 reuses the generated
TypeScript runtime's number-based varints for enum tags and numeric ID lookup.
Schema IDs are u32, so their complete domain is exactly representable as numbers.
No helper, cache, public API or wire-format change is introduced. Wide integer
payloads, lengths and method identities retain bigint handling.

Package versions, staging and Git refs remain untouched. The numbered pending
changelog is ready for maintainer review; ordinary agent work does not release.

## Qualification

Focused TypeScript qualification passes: 32 Rust runtime/borrowed tests, strict
all-target feature Clippy, Rust 1.91 generator compilation, strict TypeScript,
146 Rust vectors, 29,560 differential mutations and 1 MiB frame boundaries.
The corpus adds a Rust-derived maximum enum ID. Independent bigint byte oracles
cover all u32 tag width transitions, both directions, unknown tags, truncation,
nonminimal encodings and overflow. Existing traversal and allocation limits stay
in force. Rust production code and dependency selections are unchanged.

Two isolated native Node benchmark runs cover arrays of 1,024 unit enum values
with four selected IDs and an unchanged u64 control. Enum encoding improves
5–35% and decoding 31–71% in these workloads; generated JavaScript size is
approximately neutral. These are not browser, IC instruction or Wasm savings.
See [evaluation](../evaluation.md#typescript-enum-tag-arithmetic) for parameters,
source identities, limitations and retained CSV results. Raw sources, scripts
and samples are in `/tmp/ic-wire-0513/` and `target/typescript-enum-tags/`.

Full CI, hosted macOS and installed consumer checks were not run for this batch.

## Retained decisions

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Canonical bytes, exact retries, typed errors and pre-allocation bounds remain
unchanged. Generated metadata and scalar validators remain trusted declarations.

The 0.5.12 small-integer optimization remains selected. Text scanning and direct
frame text encoding were rejected after measured regressions. The earlier
bounded-growth writer also remains rejected; buffer allowances are unchanged.
Historical measurements and tradeoffs remain in [evaluation](../evaluation.md).

Fresh Rust allocation and scratch reuse retain their separately qualified paths.
Shared Tooling is 0.25.9. IC Build's release editor remains 0.63.0 with `cli`.
Staging, commits, versions, releases and publication remain maintainer-owned.
