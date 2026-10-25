# Current handoff

## Released 0.7.10; pending 0.7.11

The measured release is 0.7.10
(`c2adbfb47da6c180bc470c0d5204a35d7b61d574`); own versions remain unchanged.
The full Candid comparison below qualifies the released runtime. The subsequent
host-only batch makes bulk advice automatic in TypeScript contract generation.
Runtime protocol, admission rules and generated client source are unchanged.
Staging, commits, version changes and publication remain maintainer-owned.

A concurrent checkout Cargo.lock update selected Build/Testkit/host support 0.68.3.
It is preserved and excluded from the full comparison and task change metrics.
The matrix and hotspot run use the frozen release lock: Build/Testkit 0.68.2,
Platform 0.6.16, Candid 0.10.39 and Postcard 1.1.3. Do not describe the report as
qualification of the later working-tree dependency update.

## Automatic host advice

`generate_registry` now requires an acknowledgement map and a diagnostic handler.
There is no silent three-argument compatibility API or extra feature flag. CLI
callers print the report; build scripts emit Cargo warnings. Exact-path reasons
silence intentional representations, and stale paths remain visible. Review
[consumer migration and scope](../packed-fields.md#host-side-bulk-advice-and-ci).
Canic and other installed consumers still need to adapt their host callers; no
sibling was edited. Ordinary Serde compilation outside metadata generation does
not emit this advice. The `typescript` feature remains a useful host dependency
boundary. The existing rooted traversal is reused without cloning schemas.

Focused native/runtime tests, TypeScript conformance and host/Wasm isolation,
Clippy, Rust 1.91 checks and four doctests pass. The matched host benchmark
measures the added work, including small controls: 1,000 scalar fields increase
from 315 to 389 microseconds (+23.5%); 1,000 advised byte-list fields from 349 to
676 microseconds (+93.6%). Generated outputs are byte-identical in all runs.
See [raw samples and reproduction](../evidence/default-bulk-advice-0711/README.md).
These are host generation costs, not codec/IC regressions. No new runtime
benchmark or full CI was run for this host-only change.

## Full comparison

All 21 main cohorts and the 87-case hotspot corpus completed: 22 cohorts,
1,208 overlapping case entries, both opt-z and opt-3. Wire wins 4,308 of 4,832
primary owned IC encode/decode comparisons, with no ties. Total Wasm and code
sections are smaller in all 44 cohort/profile pairs; total reductions range from
58.6% to 84.2%. These are complete fixtures, not installed application savings.
Frames are smaller in 758 case entries, equal in 15 and larger in 435.

At opt-3, 100 ordinary types use 398,739 total / 364,245 code bytes versus
Candid's 2,081,252 / 2,012,071. The 100 mixed Packed records use
650,829 / 615,774 versus 2,639,766 / 2,570,257.
All 25,670 IC result rows have three equal repeated counters. Native time,
allocation counts and peak heap are measured separately and retained as evidence.

Both 0.7.9 regression comparisons are complete. All 88 normalized artifacts are
byte-identical. Against the original release, all 13,590 Wire instruction rows
and exact frames are unchanged; all 44 Wire code-section sizes are unchanged.
Wire total Wasm has two −2-byte changes and one +2-byte change; the other 41
artifacts keep their sizes. Small Candid-control differences are retained too.
See the [full report](../full-comparison-0710.md) and
[evidence/reproduction](../evidence/full-comparison-0710/README.md).

The analysis validates every paired input/frame hash and baseline identity.
Source reconstruction, script syntax, documentation links and whitespace checks
pass. Full CI and fresh macOS execution were not run for this reporting batch.

## Remaining costs and field selection

Ordinary large vectors retain expensive paths: at opt-3, 1 MiB Vec<u8> encode
costs 37.23× Candid and wide i16 encoding costs 32.82×. Byte-oriented fields and
Packed are explicit representations within the one codec. Current Packed supports
i8; its 109-case generic corpus wins all 436 primary IC comparisons.

Rejection must be considered separately. A truncated 1 MiB ordinary Vec<u8>
costs 42,472,926 instructions at opt-3 versus Candid's 24,367. Its byte-oriented
blob counterpart costs 636 versus 24,861. Ordinary sequences can traverse a
long valid prefix before detecting truncation. Use byte-oriented binary fields,
Packed where appropriate and consumer-specific runtime budgets; do not infer
cheap rejection from successful-operation results. See [field guidance](../packed-fields.md).

## Maintained boundaries

One Serde/Postcard codec remains. Metadata follows declaration order without
field IDs. Consumers own identities, coordinated layout changes, transport and
application semantics. Keep original frames for retries and raw-byte signatures;
collection/custom Serde normalization remains consumer-owned.
See the [runtime contract](../runtime.md). Installed consumer behavior, fresh
macOS timing and real big-endian execution are not qualified by this matrix.

The preceding [host cleanup](../evidence/host-cleanup-0710/README.md) retains its
matched macro/registry generation measurements and unchanged generated outputs.
Rooted bulk review for [#25](https://github.com/dragginzgame/ic-wire/issues/25)
remains available; it advises on representations rather than rewriting fields.
No sibling checkout, remote issue or release state was changed in this batch.
