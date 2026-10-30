# Current handoff

## Released source 0.7.25; pending 0.8.0

HEAD is the maintainer's 0.7.25 release at `e026eb4`; own package versions remain
0.7.25. Pending 0.8.0 coordinates the optional endpoint runtime on Platform 0.9.
Core-only users need no migration. Leave staging, commits, versions, publication
and release execution to the maintainer.

## Shared Tooling and Platform adoption

Shared Tooling is 0.27.11 at `62edfbd43087a718bf0283e9040676a49211976d`,
refreshed through its canonical exporter (116 files). Only governance payloads
changed; installer catalogs, release mechanics and publication routing are intact.
Benchmark target roots now share ordinary directory modules under `drivers/`,
without module-path overrides or new production dependencies. Scoped lint
expectations replace allowances; compile-fail inputs retain a documented exception.

Platform 0.9.1 and its macros own the sole canister executor/IC0 implementation.
The endpoint graph rejects external executor/IC0 suppliers and remains Candid-free;
the core codec graph remains Platform-free. Testkit 0.70.5 retains native-only
Platform 0.8.3 without a second executor. Build/Testkit manifest minima and the
release-editor CLI remain 0.70.3; their selected lock family stays 0.70.5.
The Rust floor stays 1.91. Endpoint users must coordinate application runtime
consumers (including timers) on Platform 0.9. Handler syntax, wire bytes, limits
and acceptance rules are unchanged. See [endpoints](../endpoints.md).

The [matched 0.9.0 adoption](../evidence/platform-090-adoption/README.md) passes all
17 endpoints / 178 cases: 5,696 logical calls / 6,048 physical executions with
matching frames, outcomes and rollback. Total Wasm falls 3,671 bytes (3.54%) at
opt-z / 5,121 (5.19%) at opt-3; code falls 3,002 / 4,561 bytes. Largest successful
instruction regression is 0.0264%; largest overall is 0.1855% on malformed input.
Snapshot/boundary, endpoint/diagnostic, benchmark-corpus, strict Clippy, MSRV,
formatting and link checks pass. No full CI/macOS or consumer adoption was run.
The [0.9.1 follow-up](../evidence/platform-090-adoption/platform-091/README.md)
scopes the Cargo guard to runtime features. Registry runtime/macro sources and
both artifact sizes are unchanged; focused graph/lint/schema/MSRV checks pass.
Physical instruction measurements were not rerun for this packaging-only patch.

## Retained Tripwire-inspired changes

Generated endpoint guards now call `Contract::admit_request()`: check size and
the existing eleven-byte identity/direction header before copying full ingress.
The standalone `admit_request(Limits)` remains the size-only primitive; both use
one size check. The existing decoder still owns full validation. A private cold
error helper shares formatting across admission, decode and response encoding.
No bytes, limits, accepted inputs, error messages or rollback rules change.
For ID 7 / u8 42 the frame remains `49 57 07 00 00 00 00 00 00 00 00 2a`.
No migration, alternate codec or compatibility lane is introduced.

The [0.7.25 admission study](../evidence/tripwire-adoption-0725/README.md) retains
costs and 1/10/100-type footprints on its original Platform graph, including
regressions and excluded stale-target results. Use separate targets per workspace.

## Selective input remains research

The study also retains a known-shape trailing-byte-field prototype using Wire's
strict parser and unchanged encoder. Native boundaries and over 10,000 mutations
match owned/borrowed typed failures. All 1,536 measured physical updates pass,
including rejection without reading the body. Opt-3 one-MiB prefix instructions
fall about 96% against borrowed input, but cycle debit falls only 0.186%.
Selective total Wasm grows 856 bytes at opt-z and shrinks 25 at opt-3.
The safe initialized-buffer owned-range control improves on ordinary owned input
but remains slower than borrowed input. No unsafe allocation helper was added.

Do not promote this DTO-specific locator as a generic Serde reader. It uses fixed
nonbinding traversal allowances and raw synchronous exports. General budget
accounting, arbitrary schemas, text validation, source lifetime and actual endpoint
integration remain unqualified. Platform's `decode_with` already receives a copied
Vec; a Platform-owned pre-copy hook and safe scoped/ranged input API are the next
integration requirements. No sibling checkout was changed or upstream PR opened.

## Earlier qualification and tooling

The [publication adapter](../cargo-publication.md) retains attempts and the exact
two-crate roster; partial-upload reconciliation stays manual. No real publication,
release, sibling adoption or new tool installation was executed in this batch.

The [Candid matrix](../evidence/candid-comparison-0725/README.md) retains its
22-cohort graph and fresh-query controls. Ordinary vectors remain weak cases;
explicit Packed wins its 436 primary comparisons. This is not a matched regression
comparison with older dependency/server graphs or this endpoint adoption.

The [actual Tripwire comparison](../evidence/tripwire-comparison-0725/README.md)
retains 1,280 physical calls on one pinned nightly/wasm32 graph. Its headline uses
Platform's real direct-initialization ingress copy; the older zero-filled control
is separate. It is a focused blob probe, not full CDK-RSX/consumer qualification.

## Maintained integration boundaries

See [endpoints](../endpoints.md) for generated owned/borrowed sync/async/composite
entrypoints, stable default IDs/budgets and schema projection. Platform owns exports
and execution; consumers own auth, dispatch, retries, persistence and deployment.
Retain original retry/hash bytes. Synchronous reply failure traps and rolls back;
after a real await only the current callback rolls back, not earlier transactions.

The [prior endpoint matrix](../evidence/endpoint-matrix-0724/README.md),
[matched adoption](../evidence/upstream-adoption-0724/README.md) and
[borrowed endpoint study](../evidence/borrowed-endpoints-0724/README.md) retain their
original graphs and scope. The [SDK recipe](../sdk-integration.md) and
[#29 acceptance](https://github.com/dragginzgame/ic-wire/issues/29#issuecomment-6475585612)
retain consumer-owned integration obligations. Actual Canic adoption and native
macOS qualification were not performed in this batch.
