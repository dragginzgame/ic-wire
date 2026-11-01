# IC Wire Endpoint

Typed `query` and `update` entrypoints backed by IC Platform 0.16. Core codec,
Limits/Error and schema identities remain owned by `ic-wire` independently of
runtime integration releases. Platform owns execution and the singleton guard.

Replace Wire's removed `endpoint` feature with an `ic-wire-endpoint` dependency.
Move attributes/helpers from `ic_wire::endpoint` to `ic_wire_endpoint`; retain
codec and schema imports from `ic_wire`. Enable `typescript` here for native
Contract schema projection. Frame bytes, bounds, acceptance and rollback rules
are unchanged. See the repository endpoint guide for handler examples.

Endpoint 0.3 adopts Platform 0.16. Align all runtime suppliers, including timers,
before adopting it; Endpoint 0.2.1 retains Platform 0.15. Canonical Principal/Account remain Types 0.15.1, so the Core 0.13
codec/schema identity and retained Wire bytes do not change.

### Independent Runtime adoption

The pending Endpoint 0.4 selects `ic-platform-runtime = "0.17"` directly.
Applications that use low-level APIs select that package's `cdk` feature and use
`ic_platform_runtime::{cdk, executor, ic0}`. Handler macros continue to target the
runtime re-exported through Endpoint; applications need no new dependency solely
to use an Endpoint macro. Core and derive keep their independent release identity.

Publish Runtime 0.17 (macros and exact guard included), then this Endpoint change
and the matching Timers adoption, before upgrading applications. Legacy embedded
Platform 0.16 executors cannot coexist with Runtime 0.17. Crypto/Candid facade
features may be selected separately with a facade that delegates to this owner.
Source/archive qualification uses harness-root patches for unpublished suppliers;
refresh the registry lock after publication. No local dependency patch is shipped.
