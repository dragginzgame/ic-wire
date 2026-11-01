# IC Wire Endpoint

Typed `query` and `update` entrypoints backed by independent IC Platform Runtime. Core codec,
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

The pending Endpoint 0.4 selects published `ic-platform-runtime = "0.17.1"` directly,
with Core 0.14 schemas on canonical Types 0.16.
Applications that use low-level APIs select that package's `cdk` feature and use
`ic_platform_runtime::{cdk, executor, ic0}`. Handler macros continue to target the
runtime re-exported through Endpoint; applications need no new dependency solely
to use an Endpoint macro. Core and derive keep their independent release identity.

Publish the aligned Core/Endpoint changes and matching Timers adoption before
upgrading applications. Runtime/macros/exact guard 0.17.1 are already published. Legacy embedded
Platform 0.16 executors cannot coexist with Runtime 0.17. Crypto/Candid facade
features may be selected separately with a facade that delegates to this owner.
Qualification uses the normal registry lock without supplier patches. The private
native fixture retains its older qualified facade/Testkit client; no facade enters
the production Endpoint graph.
