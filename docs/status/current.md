# Current handoff

## Current batch

Released 0.5.5 (`55efae7`) is the baseline. Pending 0.5.6 fixes the optional
TypeScript registry generator and includes its qualification in CI/release gates.
Package versions, index and Git refs remain untouched.

The generator rejects `JSON` as a declaration name, preventing shadowing of its
JSON parser. Transparent reference/newtype cycles are rejected while recursive
records, lists and enums remain supported. Method frame allowances must fit the
11-byte header, and node/depth allowances must be positive. Invalid minimums
return `GenerationError::Limits`; invalid aliases/names return `Schema`.
Fixed-array metadata requires a non-negative integer length, with values beyond
JavaScript's exact integer range rejected. Zero-length arrays remain supported.
Native decode now uses its existing admission macro for 11 identical visitor
methods, removing repeated bodies while preserving their admission/depth rules.

## Validation ownership

`check-typescript` now runs feature-enabled runtime/borrowed Rust tests, strict
Clippy, the Rust 1.91 library check and generated TypeScript conformance. The
main CI target includes this check, so maintainer release validation does too.
Hosted CI prepares Node from `.nvmrc`, npm from `packageManager`, and the locked
TypeScript package on Linux and both macOS architectures. Local releases need
the same preparation before offline validation; see [development](../development.md).
The official setup-node action is pinned to `249970729cb0ef3589644e2896645e5dc5ba9c38`.

## Qualification

Focused Linux checks pass: native and feature-enabled Rust tests, strict Clippy,
Rust 1.91 host/Wasm compilation,
90 Rust/TypeScript vectors, 17,524 differential mutations and 1 MiB boundaries.
The generated recursive fixture now traverses reference/newtype aliases, checking
that cycle rejection preserves valid recursion. Workflow actionlint, shellcheck,
Bash syntax and disposable local Make/hook qualification pass.
Full CI and hosted macOS execution were not run; workflow execution remains the
proof of hosted Node/npm preparation. No installed consumer was changed.

## Retained decisions

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Finite scratch reuse shipped in 0.5.5. Fresh encoding remains separate because
sharing its mutable-buffer path regressed small-message instruction cost.
The [scratch-buffer study](../evaluation.md#finite-scratch-buffer-follow-up)
retains byte parity, capacity/retry tradeoffs and measured evidence.
Caller-owned scratch capacity may outlive large messages; retain immutable retry
bytes independently. Text prevalidation and code-generation preferences remain
rejected experiments, documented in [evaluation](../evaluation.md).

Shared Tooling is 0.25.8. The release editor remains separately pinned to IC Build
0.24.0. Shared snapshots, dependency selections and runtime encoding are unchanged.
Staging, commits, versions, releases and publication remain maintainer-owned.
