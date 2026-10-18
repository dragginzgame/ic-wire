# IC Wire derives

The `Wire` derive implements explicit-layout bounded contracts for
[`ic-wire`](https://crates.io/crates/ic-wire). Applications should depend on
`ic-wire` and use its re-exported `Wire` derive.

Fields and enum variants require explicit `#[wire(id = N)]` identities. Generic
and recursive derives, unbounded containers and unsupported field shapes are
rejected. See the [runtime guide](https://github.com/dragginzgame/ic-wire/blob/main/crates/ic-wire/README.md).

Endpoint qualification attributes live in the unpublished
`ic-wire-adapter-fixture` workspace package. Canic owns endpoint integration.

Licensed under the MIT License; see the packaged LICENSE.
