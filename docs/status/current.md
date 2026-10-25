# Current handoff

## Released baseline and pending 0.7.7

0.7.6 is released at `accf8158d0c932e1fed614a59d06e2f4cc751f78`; package
versions remain 0.7.6. Incoming 0.7.7 cleanup work is preserved. This batch adds
host-only recursive registry collection and typed browser-budget projection for
[#20](https://github.com/dragginzgame/ic-wire/issues/20) and
[#21](https://github.com/dragginzgame/ic-wire/issues/21), plus incremental browser
encode allocation for [#22](https://github.com/dragginzgame/ic-wire/issues/22).
These require publication;
local implementation does not complete that acceptance gate. No issue was closed.
Staging, commits, versions, publication and pushes remain maintainer-owned.
No sibling repository was changed.

## Browser writer

The TypeScript writer starts at at most 64 bytes and grows within its directional
ceiling. Outputs remain independently owned; bytes, admission rules and validator
visits are unchanged. Regenerate clients after publication; no contract migration
is needed. Growth can increase cumulative allocation and copying for tight limits.
The [writer evidence](../evidence/typescript-writer-077/README.md) retains matched
23-case Node timing/allocation measurements, source snapshots and replay commands.
The full TypeScript gate passes, including tiny/invalid allocation regressions,
exact byte ceilings, UTF-8, packed values, both float widths and callback counts.
This is a browser allocation change, with no Rust runtime or IC performance claim.

## Host metadata additions

`SchemaRegistry` collects concrete native shapes using existing describe calls and
ref grammar. It supports self/mutual recursion, distinct generic instantiations
and shared roots, rejects duplicate names, validates refs/alias cycles and bounds
metadata work. Encountered projections are rechecked so compiler diagnostic type
names cannot silently conflate conflicting layouts. Manual projections must remain
deterministic and verified against Serde. Inline native_schema output/refusals stay
unchanged. No new helper is required by generated derives; concrete mutual types
avoid redundant cyclic where-predicates. No new codec or serializer exists.

`method_limits` projects request/response byte/node/depth settings independently
through the generator's existing validator. It omits Rust allocation admission;
this cannot be translated into JS heap bounds. Equal numeric allowances do not
mean equal visitation charges. The packaged README includes migration examples,
observed differences and the consumer qualification recipe.

The [host evidence](../evidence/native-registry-077/README.md) retains 13 cross-runtime
shapes in both directions, frozen node/depth minima, byte boundaries and malformed
mutation acceptance. Recursive generated modules type-check. Runtime bytes,
envelopes and admission counters are unchanged; one-type opt-z/opt-3 Wasms remain
byte-identical to the incoming cleanup. No fresh IC measurement is claimed.
Three-run host sampling measures inline projection/render at 3,269 → 3,039 ns for
one schema and 305,103 → 310,193 ns for 100 names (+1.67%); validated limit projection
costs 189 ns versus 146 ns for handwritten JSON. These noisy host costs are separate
from codec performance. Raw source/samples and replay commands are retained.

Feature-enabled package tests/doctests (including 31 native tests), strict Clippy,
Rust 1.91 host/Wasm forwarding and nine compiler diagnostics plus a passing control
pass. The complete TypeScript gate passes 159 ordinary vectors, 33,560 mutations,
24 prior native vectors, 13 new admission shapes and 54 simulated endian cases.
Formatting, links and diff checks pass. Logs: `/tmp/wire-issues-20-21`.
Full CI, fresh macOS execution, package publication and consumer adoption remain
outside this local qualification. Sibling endpoint policy stays consumer-owned.

## Preserved cleanup batch

The private parser now takes only its existing Slice input; its unused Flavor
generic/PhantomData and an unused finite-float macro argument were removed.
Benchmark runners share a cohort catalogue and one matrix source snapshot;
standalone runs retain source capture. Historical evaluation moved to its archive.
The [parser evidence](../evidence/parser-cleanup-077/README.md) records 4,026 paired
IC rows: successful decode never regresses; the worst rejection costs two extra
instructions (+0.71%). Standard/1/100-type opt-z Wasms are byte-identical; opt-3
saves 20/5/5 bytes. Runner dry runs cover 21 cohorts and standalone hotspots.

Keep the separate packed implementations after the prior
[consolidation screen](../packed-consolidation-0611.md). Packed tags already share
TS runtime branches; the scaling macro, shared lint validation and borrowed privacy
helpers remain intentional boundaries. No hard cut was needed for this batch.

## Production direction and tooling

One Serde/Postcard codec remains. Metadata follows Serde declaration order without
field IDs. Consumers assign identities and coordinate layouts. Collections/custom
Serde may normalize; retain original frames for retries and raw-byte signatures.
Use ByteBuf/Bytes for binary data and Packed<T> for suitable bulk fields, preserving
field bounds and small-varint controls. See the [runtime](../runtime.md) and
[bulk guide](../packed-fields.md). Real big-endian execution remains unqualified.

Build/Testkit and the release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling remains 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
The [evaluation guide](../evaluation.md) indexes maintained and historical evidence.
