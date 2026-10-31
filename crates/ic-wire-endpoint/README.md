# IC Wire Endpoint

Typed `query` and `update` entrypoints backed by IC Platform 0.15. Core codec,
Limits/Error and schema identities remain owned by `ic-wire` independently of
runtime integration releases. Platform owns execution and the singleton guard.

Replace Wire's removed `endpoint` feature with an `ic-wire-endpoint` dependency.
Move attributes/helpers from `ic_wire::endpoint` to `ic_wire_endpoint`; retain
codec and schema imports from `ic_wire`. Enable `typescript` here for native
Contract schema projection. Frame bytes, bounds, acceptance and rollback rules
are unchanged. See the repository endpoint guide for handler examples.
