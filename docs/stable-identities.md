# Stable Wire identities

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The maintainer approved this boundary for [#36](https://github.com/dragginzgame/ic-wire/issues/36)
on 2026-10-31 and requested implementation after Platform publication. It is now
delivered and published as core 0.13.0 / Endpoint 0.2.0. The original prerequisite is
supplied by registry Platform/Types 0.15 and IC Build 0.73.
The root dependency catalog and [current handoff](status/current.md) own the
selected supplier cohort. Core owns codec/schema identity; Endpoint independently
selects Runtime. Crypto facade releases can advance without changing those
identities when canonical Types and Runtime remain compatible.
Owned codec fixtures select Types directly. Simulator clients stay outside
production graphs. The Build CLI is selected independently at 0.74.0.

## Owners and release groups

| Contract | Owner | Release group |
| --- | --- | --- |
| Codec, Limits/Error, framing, bounds and TypeScript schema trait | ic-wire | core |
| Serde layout metadata derive | ic-wire-derive | core |
| Query/update attribute generation | ic-wire-endpoint-macros | endpoint |
| Principal/Account/Subaccount and Serde representation | ic-platform-types | Platform-owned independent supplier |
| Principal/Account Wire schema implementations | ic-wire platform-types | core |
| Contract, attributes, admission, trapping replies and CDK reexport | ic-wire-endpoint | endpoint |
| Exports, executor and runtime singleton | ic-platform-runtime | Platform Runtime |
| Typed client transport | IC Testkit | Tooling |
| Measurement metadata | IC Metrics | Metrics |

The sole schema trait owner implements its trait directly for canonical owned
types. Platform reexports the same definitions; there is no wrapper or orphan-rule
workaround, Platform-to-Wire dependency, duplicate codec or compatibility lane.
The core/derive group keeps its identity across endpoint changes that retain the
Types supplier. An incompatible Types package version still requires core schema alignment. The
facade simulation holds Types/Runtime fixed to prove execution isolation;
an actual Types change still requires Core schema alignment.
Endpoint's attributes are implemented by `ic-wire-endpoint-macros`, selected and released
with `ic-wire-endpoint`. Endpoint is the public import
surface; metadata derives continue to target `ic_wire`. Endpoint generator edits
no longer change the Core derive package. Core selects that metadata derive only
with `typescript` (also selected by `platform-types`); ordinary codec consumers
need no Wire metadata dependency. The old `ic-wire-derive/endpoint`
feature and direct `ic_wire_derive::{query, update}` route are removed; use
`ic_wire_endpoint::{query, update}`. This one-time API cut needs Core 0.17 /
Endpoint 0.7; attribute options, generated contracts and protocol bytes stay the
same. Future Endpoint macro-only changes retain Core's package identity.

Core exposes `native::REQUEST_HEADER_BYTES` and `validate_request_header` for
pre-copy admission. The latter delegates to the existing frame validator. Endpoint
still performs byte admission first and complete decoding afterwards; it does not
copy the frame layout or decode a synthetic value. Protocol bytes, budgets,
malformed acceptance, sync/async rollback and original retry/hash inputs are unchanged.

## Consumer migration

- Keep codec, Limits/Error, DTO Serde derives and native `ic_wire::Wire` imports.
- Remove `ic-wire/endpoint`; add `ic-wire-endpoint` and replace
  `ic_wire::endpoint::query`/helpers with `ic_wire_endpoint::query`/helpers.
  Renamed endpoint imports use the existing `crate = "renamed"` option.
- Select Endpoint's `typescript` feature for native Contract schema methods.
- Keep `ic-wire/platform-types`; Core 0.17 supports direct Types 0.18.1
  imports and matching Platform re-exports. Earlier supplier identities do not
  implement this Core schema trait; align consuming schema declarations.
- Endpoint 0.7 selects Runtime 0.19. Adopt matching Timers and other
  execution suppliers together; legacy embedded executors are refused.
- Metrics and typed Testkit callers align once on the supplying core minor;
  runtime-only Endpoint releases can subsequently retain those public identities.

Use Core/derive 0.17 and Endpoint 0.7 for Types 0.18.1 / Runtime 0.19 alignment.
Present Account subaccounts now use exact Serde bytes: `01 + 32 bytes` becomes
`01 20 + 32 bytes`; None stays `00`. Regenerate Uint8Array clients, choose fresh
contract keys and requalify budgets. Current default IDs hash names only and do
not fence changed schemas. Preserve original retry/hash bytes and handle stored
unframed data separately; no legacy lane is supplied.
Endpoint initially inherits its core dependency from the root workspace, so core
release preparation updates that requirement before packaging the dependent endpoint.

## Grouped maintainer flow

[Explicit policy](../ci/cargo-release-groups.json) declares every workspace member.
Each member declares its existing literal version; root Cargo metadata has no
workspace version. `core` takes its authority from ic-wire and covers core/derive
plus private qualification; `endpoint` takes its authority from ic-wire-endpoint
and includes its versioned proc-macro package.
Root `[workspace.dependencies]` is the complete member catalog, including private
qualification packages. Each release group has one contiguous block with its
changelog link; the policy lists every alias so preparation also updates unused
catalog entries. No extra dependency edges or duplicate header map are added.
Both ledgers live at `releases/<group>/CHANGELOG.md`; root CHANGELOG is an index,
with coordinated releases through 0.11.0 preserved once in `releases/history.md`.
No package paths, numeric versions, tags or historical scope were changed. The real
adapter uses the shared group plan's exact files for metadata/source admission,
receipts and post-commit checks. Group-specific tags/plans prevent collisions.
The metadata editor is published IC Build 0.74.0 with `group-manifest`.

```sh
make release-version RELEASE_GROUP=core
make release-version RELEASE_GROUP=endpoint
make release-minor RELEASE_GROUP=core
make release-minor RELEASE_GROUP=endpoint
# Subsequent ordinary endpoint-only changes:
make release-patch RELEASE_GROUP=endpoint
# Publication is separately authorized; core before initial dependent endpoint:
make publish RELEASE_GROUP=core PACKAGE="ic-wire-derive ic-wire"
make publish RELEASE_GROUP=endpoint PACKAGE="ic-wire-endpoint-macros ic-wire-endpoint"
```

The delegated delivery executed both minor releases and published core 0.13.0
before Endpoint 0.2.0. Future increments require their own matching notes. Release, resume and publication reject missing,
empty or unknown groups before setup/effects. Publication selects only the chosen
group (derive before Core, endpoint macros before Endpoint); PACKAGE subsets
must belong to that group. Exact
versions and the lint gate remain enforced. There is no implicit group or whole-
workspace publication selection.

## Qualification and remaining acceptance

[Reproduction, raw measurements and checks](evidence/stable-identities.md)
cover real current schemas, Rust/TypeScript frames/refusals, feature isolation,
Rust 1.91 host/Wasm, renamed macro imports, installed endpoint guards/rollback,
source-aligned Metrics/Testkit and independent release-group preparation/resume.
A maintained simulated next Endpoint/generator and facade fixture proves
codec/derive/stable identity package IDs remain fixed. The facade dependency has
defaults disabled and selects no features: this checks package identity without
activating unused hashing. Both Wasm graphs reject BLAKE3 and retain one Runtime
guard. It does not claim a future runtime is released.

The combined Platform 0.13 → 0.14 and package move adds 714/555 total Wasm bytes
at opt-z/opt-3; instruction changes are at most +0.026%. It changes no message
contract. [Published-pair evidence](../releases/receipts/README.md) records
the delivered releases and registry-only Rust/MSRV/Wasm/TypeScript consumer.
Hosted CI status and producer-owned adoption are tracked in the current handoff.
