# Current handoff

## Current batch

Released 0.5.6 (`3a6efba`) is the baseline. Pending 0.5.7 avoids repeated
TypeScript registry enumeration and rejects malformed optional collection bounds.
It also adopts published IC Tooling 0.63.0: the private diagnostic fixture uses
IC Build with defaults disabled, resolving Host Artifacts/FS/Process 0.63.0.
The separately installed release editor is pinned to 0.63.0 with `--features cli`.
The adoption was reviewed against IC Tooling release `c342bb28`; sibling dirty
work was not used. No Testkit or new runtime dependency was added.
Package versions, index and Git refs remain untouched.

Alias resolution lazily computes the registry size once per encode/decode
operation using the existing traversal budget. It still traverses each alias
and runs reference/concrete validators in the original order. Separate calls
observe registry changes; missing references and cycles remain typed failures.
No global registry cache or new public option was introduced.

Collection bounds use one generator validation path. `max` is required for
arrays and optional for lists/bytes/text. Present bounds must be non-negative
integers within JavaScript's exact range. Optional byte `min` is validated too,
and cannot exceed `max`. Invalid shape/ranges return `Schema`; integers outside
the exact range return `Complexity`. Custom scalar checks remain consumer-owned.

## Qualification

Focused Linux checks pass: 29 feature-enabled Rust runtime/borrowed tests,
strict Clippy, Rust 1.91 generator compilation, 90 Rust/TypeScript vectors,
17,524 differential mutations and 1 MiB boundaries. The new bounds regression
failed before the fix and passes afterward. The registry fixture encodes and
decodes 100 referenced values with 1,000 registered types, asserting one registry
enumeration per operation, with mutation/cycle and existing validator checks.
This is operation-count evidence, not a browser timing or IC instruction claim.
IC Tooling adoption also passes 16 expected diagnostics plus the valid control,
evaluation tests, strict evaluation Clippy and Rust 1.91 checks. The selected
0.63.0 editor passes offline admission and disposable release/Make fixtures.
Full CI, hosted macOS and installed consumers were not run.

## Retained decisions

The 0.5.6 release added TypeScript qualification to CI/release gates and fixed
reserved names, transparent alias cycles and structural method/array budgets.
Hosted CI prepares pinned Node/npm; local release operators must prepare them
before offline validation. See [development](../development.md).

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Finite scratch reuse shipped in 0.5.5. Fresh encoding remains separate because
sharing its mutable-buffer path regressed small-message instruction cost.
The [scratch-buffer study](../evaluation.md#finite-scratch-buffer-follow-up)
retains byte parity, capacity/retry tradeoffs and measured evidence.
Caller-owned scratch capacity may outlive large messages; retain immutable retry
bytes independently. Rejected experiments remain in [evaluation](../evaluation.md).

Shared Tooling remains 0.25.8; its snapshot was not modified. Rust runtime
encoding and its dependency selection are unchanged. Staging, commits, versions, releases and publication are maintainer-owned.
