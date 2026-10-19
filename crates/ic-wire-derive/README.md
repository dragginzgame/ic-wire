# IC Wire derives

The `Wire` derive implements explicit-layout bounded contracts for
[`ic-wire`](https://crates.io/crates/ic-wire). Applications should depend on
`ic-wire` and use its re-exported `Wire` derive.

Fields and enum variants require explicit `#[wire(id = N)]` identities. Records
and enums support lifetime parameters, including inline outlives bounds, for
bounded borrowed fields. Type/const parameters, where clauses, recursive derives,
unbounded containers and unsupported field shapes are
rejected. See the [runtime guide](https://github.com/dragginzgame/ic-wire/blob/main/crates/ic-wire/README.md).

Consumers own endpoint integration and its qualification.

Licensed under the MIT License; see the packaged LICENSE.
