# IC Wire Endpoint macros

Implements the `query` and `update` attributes re-exported by
[`ic-wire-endpoint`](https://crates.io/crates/ic-wire-endpoint). Import attributes
through that package, including its `crate = "renamed"` option for re-exports.
Platform retains IC exports and execution; these macros supply Wire codec hooks.

This proc-macro package is versioned and released with Endpoint. The separate
`ic-wire-derive` package owns only Core's `Wire` schema metadata derive. No
serialization, frame, identity, admission or retry semantics change in this move.

Licensed under the Toko Studios Network Licence 1.0; see the packaged LICENSE.md.
