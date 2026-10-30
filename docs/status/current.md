# Current handoff

## Released 0.7.24; pending 0.7.25

HEAD is the maintainer's 0.7.24 release at `254f687`. Own package versions remain
0.7.24; the undated 0.7.25 changelog is a compatible-addition/optimization draft.
Leave staging, commits, versions, publication and releases to the maintainer.
Incoming Shared Tooling adoption and benchmark evidence are preserved.

## Retained Tripwire-inspired changes

Generated endpoint guards now call `Contract::admit_request()`: check size and
the existing eleven-byte identity/direction header before copying full ingress.
The standalone `admit_request(Limits)` remains the size-only primitive; both use
one size check. The existing decoder still owns full validation. A private cold
error helper shares formatting across admission, decode and response encoding.
No bytes, limits, accepted inputs, error messages or rollback rules change.
For ID 7 / u8 42 the frame remains `49 57 07 00 00 00 00 00 00 00 00 2a`.
No migration, alternate codec or compatibility lane is introduced.

The [admission study](../evidence/tripwire-adoption-0725/README.md) qualifies all
17 endpoints / 178 cases under both profiles: 5,696 measured logical calls /
6,048 physical executions, with matching outcomes, frames and state oracles.
A 64 KiB wrong-ID query falls about 37.6% in instructions; successful-call overhead
is at most 0.257% in this matrix. Full fixture total Wasm changes −32 bytes at
opt-z / +128 at opt-3; code changes −42 / +217. At 100 generated endpoints/types,
total size falls 6,196 / 6,960 bytes. One endpoint grows 147 bytes in each profile.
Both sizes and regressions are retained; these are not application linker deltas.

Focused endpoint native/schema/diagnostic checks, strict Clippy, dependency
boundaries and Rust 1.91 Wasm checks pass. Full CI/macOS were not run. Reproduction,
source patches, raw traces, scale artifacts/hashes and host-generation samples are
linked from the study. Host timings are noisy because sibling builds were active;
no native generation speedup is claimed. Raw runs: `/tmp/wire-tripwire-adoption-0725/`.
An initial shared-target attempt reused stale Cargo artifacts and is excluded;
all reported candidates use separate targets for separate source workspaces.

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

## Other pending work preserved

Shared Tooling is 0.27.10 at `61a173990683597c1c6a25f0145a2c87200c3cee`, adopted via
the canonical exporter (116 files). The [publication adapter](../cargo-publication.md)
replaces the local recipe: exact public roster/version selectors, lint gate and
retained attempts; `make publish-check` is dry-run and subsets use `PACKAGE`.
`PUBLISH_PACKAGES` fails with guidance. Partial-upload reconciliation stays manual;
there is no automatic retry, registry readback claim or release-triggered upload.
Snapshot/tool admission, disposable Make/publication/release-resume fixtures and
links pass. Uploads were substituted; no real publication or release was executed.
Adoption evidence: `/tmp/wire-shared-02710/`. Snapshot files were not patched manually.

Platform stays 0.8.3. Tool catalog/manifest selections stay 0.70.3; the incoming
lock already resolves Build/Testkit/Host 0.70.4. Cargo.lock, Rust floor 1.91 and
runtime dependency selections are unchanged by the current experiments.

The [fresh Candid matrix](../evidence/candid-comparison-0725/README.md) covers 22
cohorts / 1,208 case entries / 4,832 primary comparisons on the unchanged core codec.
All 25,670 instruction rows match cache controls after the fresh-caller sampler
fix. Snapshotting includes the endpoint workspace member; overlapping native
samples were replaced serially and retained separately. Ordinary vectors remain
weak cases; explicit Packed wins its 436 primary comparisons. This is not a matched
regression comparison with the older 0.7.10 dependency/server graph.

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
