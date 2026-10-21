# Current handoff

## Current batch

Released 0.5.17 (`f8e43cf`) is the baseline. Pending 0.5.18 consolidates six
native decoder methods into the existing admission macro. Methods with names,
lengths and field/variant lists now share the same budget check and visitor-depth
setup as scalar and collection methods. Seven compound encoder implementations
now share one local macro for child-value forwarding and completion. Container
setup stays explicit; child admission and depth are unchanged. Six owned/borrowed
byte/text visitor methods also share their existing conservative byte-charge
check, with no ownership conversion or new runtime mode.
Protocol bytes, identities, resource charges, error precedence and accepted
inputs are unchanged. Package versions, dependencies, staging and Git refs remain
untouched.

## Evidence and qualification

The before/after expanded crate is textually identical after excluding the edited
macro definitions themselves. Generated methods, signatures and bodies are unchanged.
This is source maintenance, not a runtime cost optimization; no timing or artifact
size saving is claimed and a performance benchmark is not applicable.

Reproduce the expansion inspection with the baseline and working-tree sources:
`RUSTC_BOOTSTRAP=ic_wire cargo rustc --offline --locked -p ic-wire --lib -- -Zunpretty=expanded`.
The temporary compiler override enables inspection only, not ordinary builds.
Rust 1.99.1 expansion files and their diff are retained locally in
`/tmp/ic-wire-admission-cleanup/`. Compare after removing the `admit_visitor`
macro definition, leaving all generated implementations intact. The encoder
follow-up uses the preceding working tree as its baseline; its files and diff are
in `/tmp/ic-wire-compound-cleanup/`. Excluding the new `compound` definition
also leaves a textually identical expanded crate. The byte/text follow-up repeats
this comparison against the preceding working tree, excluding only `visit_buffer`;
its identical expansion and check logs are in `/tmp/ic-wire-buffer-cleanup/`.

`make test` passes runtime/native coverage and doctests. Focused checks also cover
strict Clippy, Wasm compilation, Rust 1.91 native/Wasm compilation, formatting,
documentation links and whitespace. No new tests mirror the unchanged expansion.
Full CI, browser conformance and sibling suites are outside this source-only batch.

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
Shared Tooling is 0.25.9; IC Build's release editor is 0.63.0 with `cli`.
Ordinary agent work does not stage, commit or release.
