# Current handoff

## Released 0.7.15; pending 0.7.16

Baseline is `47f011702edd0fce99d4272fd80b02991ee5f94f`; the checkout started clean.
One production change is retained: explicit three-group u16 parsing, also used
by i16 zigzag decoding. Runtime encoding, shortest-form/overflow rejection,
resource admission, typed errors, metadata and protocol bytes are unchanged.
No consumer migration, version edit, staging, commit, push or release was performed.

A concurrent lock update to Build/Testkit/host 0.68.6 and Platform 0.6.17 is
preserved. The retained decoder's benchmarks use frozen baseline dependencies:
Build/Testkit/host 0.68.5, Platform 0.6.16, Candid 0.10.39 and Postcard 1.1.3.
The later encoder research uses 0.68.6/0.6.17 in both variants; do not attribute
its results to the older graph or call it full dependency-update qualification.

## Retained result and tradeoffs

The u16 decoder's bounded tail loop becomes explicit second/third group checks.
One-byte values retain their fast path; redundant terminal zero groups and
out-of-range third bytes remain malformed. An independent Postcard decode/
canonical-re-encode oracle covers 131,072 three-byte cases, alongside existing
exhaustive valid u16/i16 tests.

For 16,384 wide values, IC owned decode improves 7.13% for u16 and 6.44% for i16
at opt-3, and 9.99%/11.21% at opt-z. Small-value-list counters are unchanged.
Five measured Wire rows regress: empty-list decode adds two opt-3 instructions,
and truncated three-byte scalars add four. These small regressions are retained.
Standard opt-3 Wasm total/code shrink 218 bytes; opt-z grows seven bytes. Both
100-type artifacts are byte-identical. No installed application saving is claimed.

Native wide-list pooled medians improve 8.93%/7.35%. Empty/singleton u16 controls
regress 3.06%/6.57%, and the large small-i16 control regresses 5.62%, with materially
different run medians. Report these as host observations, not universal speedups.
Native timing, IC instructions, frame sizes and Wasm sizes remain separate.
See [full evidence and reproduction](../evidence/runtime-0716/README.md).

## Rejected experiments

Combined budget predicates and outlined admission helpers give mixed/worse Wasm
sizes. Sharing the i16/u16 writer saves under 1 KiB but regresses wide-u16-list
IC encoding 51.3%; it is not retained. Forcing Output helpers inline reduces
100-type Wasm by 2,796 bytes at opt-3 but grows standard Wasm 2,372 bytes, so it
remains research. No alternate production codec, mode or feature was added.

The [encoder follow-up](../evidence/encoder-followup-0716/README.md) isolates
try_push/try_extend inlining and three bounded one-pass growth prototypes.
Inline-extend saves Wasm but regresses wide-u16 encoding 96.42%. Outlined
one-pass reserve saves 16,657 total / 16,646 code bytes in 100-type opt-3 Wasm,
but regresses wide-i16 encoding 69.96%. Cold-growth one-pass improves wide
lists yet regresses unit/nested-blob encoding 40.75%/46.75%, reserves more memory
and changes the documented two-pass contract. All five remain research; no
additional production change was retained. Matched standard/100-type IC frames
and Candid controls agree; native allocation counts are separate from timing.

## Validation and scope

Focused checks pass: 2 unit, 6 borrowed, 40 feature-enabled native and 15 runtime
tests; strict Clippy, Rust 1.91 native tests, TypeScript conformance/feature-boundary
checks, formatting and documentation links. Standard IC runs contain 298 cases /
7,006 rows per variant with three equal repeated counters and matching frame/input
hashes. Candid controls are unchanged. Native timing covers 87 hotspot cases in
two paired runs with seven samples per operation. The 100-type comparison is
size/equivalence evidence, not separately measured IC counters. Full CI, the full
Candid matrix, installed consumers and fresh macOS execution were not run.

Consumers still choose byte buffers/Packed for bulk fields and bind schemas to
deployed identities. Preserve original frames for exact retries and signatures;
collection normalization remains consumer-owned. See the [runtime contract](../runtime.md).
