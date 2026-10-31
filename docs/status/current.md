# Current handoff

## Delivered releases

The maintainer delegated Git releases and crates.io publication. Shared Tooling
**0.27.19** is selected at `54fc49e12f439dbe18d232f622a6d78e61773353` through
the canonical exporter (119 payloads, no snapshot-owned edits). Source adoption
is commit `b404462`.

- **ic-wire / ic-wire-derive 0.13.0**: `core-v0.13.0`, commit
  `65d1b91600d64a1e4255774f6bfc8ad9752a63cb`.
- **ic-wire-endpoint 0.2.0**: `endpoint-v0.2.0`, commit
  `dfd4978bf6792d12d3fdab6ee9a7b2affeabde1c`.

Both standard minor releases completed their full local `make ci` gate and
atomic main/tag push. Both publication dry runs and actual publication passed,
core before Endpoint. All three non-yanked registry checksums match the upload
archives. [Delivery/consumer evidence](../evidence/published-pair-0130/README.md)
retains identities, logs and reproducible checks. Full effect logs are retained
at `/tmp/wire-ship-02719/`; release plans remain in `.git/release-state/`.
Do not replay these effects. No sibling working checkout was modified.

## Published consumer acceptance

The registry-only standalone consumer passes Rust 1.91.0 schema-only (four tests)
and combined Endpoint (five tests), Wasm checks, native derives and exact
Rust/TypeScript frames/malformed refusals. It selects registry Core/Derive 0.13,
Endpoint 0.2, facade 0.15.0 and compatible Types **0.15.1**. Schema-only excludes
CDK/runtime; combined has one runtime guard. Candid/PocketIC/external executor
and host serde_json remain outside production Wasm.

Isolated copies of released Metrics 0.22 and Testkit 0.73 also pass a Rust 1.91
identity test with published Wire 0.13: all four Metrics schemas, Principal/
Account, Contract, public Limits/Error and typed query/update signatures. Only
copied manifests change; their Rust sources are identical. This is source-aligned
qualification, not published downstream adoption. Those producers still own
supplying their matching releases; Blob adoption is likewise consumer-owned.
Do not reopen Tooling #274, which completed the older Wire 0.11 scope.

[Endpoint-release hosted CI](https://github.com/dragginzgame/ic-wire/actions/runs/41212395765)
was still running at inspection; Linux and ARM macOS TypeScript passed while
remaining core/benchmark/Intel work was pending. Neither this nor the final
documentation push is claimed as a complete hosted matrix. #36 remains open for
its applicable hosted acceptance; fleet-wide adoption is not an extra producer
closure gate. #37 remains superseded, without a legacy integration lane.

## Ownership, migration and protocol

The [approved boundary](../stable-identities.md) keeps codec, schema trait,
Limits/Error and canonical Types schema impls in Core. Endpoint owns Contract,
attributes, admission and trapping replies; Platform owns exports/execution.
Core `platform-types` selects canonical Types without the runtime facade/CDK.
Move `ic_wire::endpoint` to `ic_wire_endpoint`, remove `ic-wire/endpoint` and
select Endpoint `typescript` for native Contract schemas. Types 0.1 to 0.15
requires the one-time schema/typed-client identity alignment. Future runtime-only
updates can retain Core identity when the Types supplier is compatible.

No codec bytes, bounds, budgets, primitive/malformed acceptance, field order,
handler transactions or original retry/hash inputs change. Principal remains
bounded bytes (max 29). Account remains owner then optional 32-byte subaccount:
absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes stay distinct.
Collection/custom-Serde normalization remains consumer-owned; retain retry bytes.

The [Platform 0.15 matched study](../evidence/platform015-adoption/README.md)
retains 384 logical calls / 608 executions with identical paired instruction and
cycle samples. Total Wasm stays 100,441 B (opt-z) / 94,073 B (opt-3); code sections
stay 86,544 / 81,127 B. This is recorded fixture qualification, not a new Candid
matrix or an optimization claim. Documentation/effect recording adds no runtime
or generator changes and requires no new performance benchmark.

## Release layout and retained evidence

Root Cargo is virtual with no workspace version; member versions are literal.
Explicit `core` and `endpoint` policies use Build 0.73 group manifests and shared
plans/receipts. Root CHANGELOG indexes canonical [core](../../releases/core/CHANGELOG.md)
and [endpoint](../../releases/endpoint/CHANGELOG.md) ledgers; coordinated releases
through 0.11 live in [history](../../releases/history.md). Missing, empty or unknown
release/resume/publication groups refuse before effects. No implicit whole-workspace
publication. Derive publishes before Core. Future releases need matching new notes
and authorization; this delivery does not delegate unrelated effects.

[Extraction qualification](../evidence/stable-identities-0120/README.md) retains
ownership, simulated future runtime identity, endpoint installed limits/rollback,
MSRV, diagnostic and group recovery proofs. The simulated fixture holds Types
fixed; it is not a promise about upstream future versions. Historical
[Blob 0.60 pairing](../evidence/published-pair-0110/README.md) is Wire 0.11 evidence,
not new Blob adoption. [Candid matrix](../evidence/candid-comparison-0725/README.md)
retains ordinary-vector controls and explicit Packed lanes; Candid stays dev-only.
Evidence manifests are snapshots, never active Cargo workspaces under docs.
