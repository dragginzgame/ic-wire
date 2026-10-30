# Current handoff

## Released source 0.8.0; pending 0.9.0

The released baseline is 0.8.0 at `25a7212`; own package versions remain
0.8.0. Pending 0.9.0 coordinates the optional endpoint runtime on Platform 0.10.
Core-only users need no migration. Leave staging, commits, own versions,
publication and release execution to the maintainer.

## Platform 0.10 / IC Tooling 0.71 adoption

Workspace requirements select Platform 0.10.0 and Build/Testkit 0.71.0; the lock
selects their coordinated macros, runtime guard and Host family. Native Testkit
retains Platform 0.9.1 without an executor. The actual canister graph has one
Platform 0.10 runtime and excludes external executor/IC0, Candid and PocketIC.
Core-only Wire remains Platform-free. Rust floor remains 1.91.

The managed release-editor selection is 0.71.0. Existing setup and offline
admission passed; the actual editor passed the disposable release-adapter fixture.
Only temporary repositories saw fixture commits/tags/pushes; validation gates
were substituted. No real release or publication was performed.

Endpoint consumers must align runtime-consuming dependencies on Platform 0.10.
Handler syntax, wire bytes, identities, bounds, acceptance and retry bytes are
unchanged. Platform's prepared-call API moved into its facade; Wire's existing
transport/endpoint code is not rewritten to use it. See [endpoints](../endpoints.md).
The registry review finds 118 existing Platform source files and all five macro
files unchanged; only the root declaration and added call module differ.

The [matched adoption study](../evidence/platform010-tooling071/README.md)
measures 12 cases across seven exports, both profiles, eight repetitions after
warmup: 384 logical calls / 608 physical executions. Complete instructions,
observed cycle debits, frames, outcomes and transaction-state oracles match.
Total Wasm grows eight bytes: 99,939 → 99,947 at opt-z and 93,510 → 93,518 at
opt-3. Code sections stay 86,068 / 80,630 bytes. This is a focused comparison,
not a new full Candid/178-case matrix or a general performance claim.

Locked metadata, dependency boundaries, endpoint native/schema/diagnostic checks,
strict Clippy, benchmark driver compilation/roundtrips, MSRV, installed endpoint
limits/rollback, editor admission and the disposable release fixture pass.
No full CI, macOS or sibling adoption was run. Evidence and reproduction are
linked from the study; raw runs are in `/tmp/wire-platform010-tooling071/`.

The retained manifest uses `Cargo.toml.snapshot`, not an active Cargo filename,
so `check-pins` does not discover an incomplete evidence workspace. Reproduction
copies it back to `Cargo.toml` only in its isolated workspace. `make check-pins`,
restored-workspace locked metadata, script syntax and documentation links pass.
No runtime or protocol change is involved.

## Shared Tooling and earlier qualification

Shared Tooling remains reviewed 0.27.11 at
`62edfbd43087a718bf0283e9040676a49211976d` (116 selected files). No snapshot payload
was changed in this batch. Benchmark roots use ordinary modules under `drivers/`;
lint expectations are scoped, with explained compile-fail input exceptions.
The [publication adapter](../cargo-publication.md) retains attempts and exact
package selection. Partial-upload reconciliation remains manual.

The [Platform 0.9 study](../evidence/platform-090-adoption/README.md) and
[0.9.1 follow-up](../evidence/platform-090-adoption/platform-091/README.md) retain
their original dependency graphs, full endpoint matrix and size/instruction
tradeoffs. Their older server identities are not this adoption's baseline.
The [0.7.25 admission study](../evidence/tripwire-adoption-0725/README.md) retains
1/10/100-type footprints and early-header results, including regressions and
excluded stale-target attempts. Use separate targets for separate workspaces.

The [Candid matrix](../evidence/candid-comparison-0725/README.md) retains its
22-cohort graph and fresh-query controls. Ordinary vectors remain weak cases;
explicit Packed wins its 436 primary comparisons. The
[Tripwire comparison](../evidence/tripwire-comparison-0725/README.md) remains a
focused ingress probe, not general CDK-RSX or consumer qualification.

## Selective input remains research

The admission study's fixed-shape trailing-byte prototype retains unchanged
Wire frames and strict malformed controls. It saves roughly 96% of prefix-read
instructions against borrowed input but only 0.186% of observed ingress cycles;
opt-z Wasm grows 856 bytes. Do not publish it as a generic Serde reader.
General budgeting, arbitrary schemas, text validation and source lifetimes are
unqualified. Platform's current decode hook already receives copied input;
a Platform-owned pre-copy/scoped-range API remains the next integration step.
No sibling checkout was changed or upstream PR opened in this batch.

## Maintained integration boundaries

Platform owns exports/execution; consumers own authentication, dispatch, retries,
persistence and deployment. Retain original retry/hash bytes. Synchronous reply
failure traps and rolls back; after await only the current callback rolls back.
See [endpoints](../endpoints.md), the [SDK recipe](../sdk-integration.md) and
[#29 acceptance](https://github.com/dragginzgame/ic-wire/issues/29#issuecomment-6475585612)
for consumer-owned integration obligations. Actual Canic adoption remains separate.
