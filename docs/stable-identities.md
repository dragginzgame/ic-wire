# Stable Wire identities

The maintainer approved this boundary for [#36](https://github.com/dragginzgame/ic-wire/issues/36)
on 2026-10-31 and requested implementation after Platform publication. It is now
implemented locally, pending delivery/publication. The original prerequisite is
supplied by registry Platform 0.14 / Types 0.1 and IC Build 0.73. No sibling
checkout, own existing version, real index or release ref is changed.

## Owners and release groups

| Contract | Owner | Release group |
| --- | --- | --- |
| Codec, Limits/Error, framing, bounds and TypeScript schema trait | ic-wire | core |
| Metadata derive and endpoint attribute generation | ic-wire-derive | core |
| Principal/Account/Subaccount and Serde representation | ic-platform-types 0.1 | Platform-owned independent supplier |
| Principal/Account Wire schema implementations | ic-wire platform-types | core |
| Contract, attributes, admission, trapping replies and CDK reexport | ic-wire-endpoint | endpoint |
| Exports, executor and runtime singleton | ic-platform 0.14 | Platform runtime |
| Typed client transport | IC Testkit | Tooling |
| Measurement metadata | IC Metrics | Metrics |

The sole schema trait owner implements its trait directly for canonical owned
types. Platform reexports the same definitions; there is no wrapper or orphan-rule
workaround, Platform-to-Wire dependency, duplicate codec or compatibility lane.
The core/derive group keeps stable identity across endpoint/runtime-only upgrades.
Endpoint's attributes still use the existing derive package and default to the
new facade path; metadata derives continue to target `ic_wire`.

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
- Keep `ic-wire/platform-types`; align once on Platform 0.14 reexports or direct
  `ic-platform-types` 0.1 imports. Future facade upgrades can retain that owner.
- Metrics and typed Testkit callers align once on the supplying core minor;
  runtime-only Endpoint releases can subsequently retain those public identities.

Pending selections are Wire 0.12 and Endpoint 0.1.0. Existing package versions stay
0.11.0 until the maintainer release; Endpoint's new initial package is 0.1.0.
Endpoint initially inherits its core dependency from the root workspace, so core
release preparation updates that requirement before packaging the initial endpoint.

## Grouped maintainer flow

[Explicit policy](../ci/cargo-release-groups.json) declares every workspace member.
`core` selects the coordinated root version, core/derive, unpublished controls and
root changelog; `endpoint` selects only Endpoint's version and ledger. The real
adapter uses the shared group plan's exact files for metadata/source admission,
receipts and post-commit checks. Group-specific tags/plans prevent collisions.
The metadata editor is published IC Build 0.73 with `group-manifest`.

```sh
make release-version RELEASE_GROUP=core
make release-version RELEASE_GROUP=endpoint
make release-minor RELEASE_GROUP=core
# After the initial endpoint release exists, ordinary endpoint-only changes:
make release-patch RELEASE_GROUP=endpoint
# Publication is separately authorized; core before initial dependent endpoint:
make publish PACKAGE="ic-wire-derive ic-wire"
make publish PACKAGE=ic-wire-endpoint
```

These are maintainer commands, not actions performed by this implementation.
The initial Endpoint 0.1.0 ledger is pending; ordinary increment commands expect
notes for their selected next version and refuse mismatched headings. Prepare its
initial publication only after the supplying core release is delivered. Default
publication eligibility orders derive, core, endpoint; explicit package selection
handles independent attempts, with exact versions and the lint gate retained.

## Qualification and remaining acceptance

[Reproduction, raw measurements and checks](evidence/stable-identities-0120/README.md)
cover real current schemas, Rust/TypeScript frames/refusals, feature isolation,
Rust 1.91 host/Wasm, renamed macro imports, installed endpoint guards/rollback,
source-aligned Metrics/Testkit and independent release-group preparation/resume.
A maintained simulated next endpoint/runtime fixture proves codec/derive/stable
identity package IDs remain fixed. It does not claim a future runtime is released.

The combined Platform 0.13 → 0.14 and package move adds 714/555 total Wasm bytes
at opt-z/opt-3; instruction changes are at most +0.026%. It changes no message
contract. #36 remains open until the implementation is delivered and its required
release/consumer acceptance is established. Publication and full hosted/native
qualification are not implied by local checks.
