# IC Wire

A bounded Postcard binary protocol for controlled Rust endpoints on the Internet
Computer. IC Wire owns encoding, resource admission and typed
codec errors. The independently released `ic-wire-endpoint` package connects typed handlers to IC
Platform entrypoints. Consumers own authentication,
application semantics and retry policy.

The production runtime uses ordinary Serde derives with one Postcard codec.
Decode validates primitives in one pass and requires only Deserialize; collection
and custom-visitor normalization is consumer-owned. Keep original bytes for retries.
Bounded byte/text/list and finite-float fields remain optional Serde types.
Runtime traversal supplies byte, allocation, node and depth admission; host-only
`Wire` metadata generates matching TypeScript without field IDs. The metadata
derive is selected by `typescript` (also via `platform-types`); codec-only
consumers do not compile `ic-wire-derive`.
The former finite API is retired; `Wire` now generates metadata only. Alternative strategies
remain research rather than selectable production modes.

Browse the [documentation index](docs/README.md), or start with the
[runtime guide](docs/guides/runtime.md), [design](docs/guides/design.md),
[evaluation scope](docs/benchmarks/evaluation.md) and [current handoff](docs/status/current.md).
The runtime and derive crates were first published at 0.3.0; the private
`ic-wire-qualification` package remains unpublished.

Standalone canisters can use bare `ic_wire_endpoint::{query, update}` attributes
with bounded defaults and automatic method IDs; explicit overrides remain available.
Their generator lives in `ic-wire-endpoint-macros` within the Endpoint release
group, independently of Core's metadata derive.
See the [endpoint guide](docs/guides/endpoints.md)
and [standalone consumer](qualification/ic-wire-endpoint-fixture/src/lib.rs). `ic-platform-runtime` owns IC exports
and the executor; the core codec retains its runtime-independent default graph.

Enable `platform-types` for host `Wire` schemas on Platform's owned `Principal`
and `Account`, independently of endpoints. This replaces Platform's `wire-schema`
selection without changing bytes or budgets; see the
[migration and representation rules](crates/ic-wire/README.md#optional-platform-identity-schemas).

For new contracts, default binary fields to `Bytes<N>`, `ic_wire::ByteBuf`
or the `serde_bytes` adapter, and large homogeneous integer/hash arrays to
`Packed<T>`. Keep ordinary collections for records and small values where compact
varints matter. These field types share the same Serde/Postcard codec; choosing
packed fields is explicit and requires coordinated consumers for existing layouts.
See the [field selection and adoption guide](docs/guides/packed-fields.md).

## Development

The virtual root workspace owns package metadata, dependency selections and the
lockfile. See [local setup](docs/guides/development.md) and [contributor instructions](AGENTS.md).
With the documented prerequisites prepared:

```sh
make install-tools
make tools-check
make install-hooks
make check-rust-policy
make fmt-check
```

Then select the package/test or named contract target for the changed behavior;
see [local commands](docs/guides/development.md). `make lint` spans all public packages.
`make ci` is the complete gate for explicit broad validation or release qualification.
Release commands deliver Git versions
and tags; registry publication is a separate maintainer action.

The release policy requires `RELEASE_GROUP=core` or `RELEASE_GROUP=endpoint`,
using each group's ledger, exact metadata files and receipts while retaining the
complete gate. The [stable identity and release guide](docs/guides/stable-identities.md)
describes the implemented boundary, migration and publication order. Shared
Tooling owns the [group mechanism](https://github.com/dragginzgame/toko-tooling/issues/328).

## License

Licensed under the [Toko Studios Network Licence 1.0](LICENSE.md).
See [contribution requirements](CONTRIBUTING.md).
