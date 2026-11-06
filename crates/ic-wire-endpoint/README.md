# IC Wire Endpoint

Typed `query` and `update` entrypoints backed by independent IC Platform Runtime. Core codec,
Limits/Error and schema identities remain owned by `ic-wire` independently of
runtime integration releases. Runtime owns execution and the singleton guard.
Query/update generation lives in `ic-wire-endpoint-macros`, versioned and
released with this package. Continue using `ic_wire_endpoint::{query, update}`;
Core's `ic-wire-derive` owns only metadata and no longer has an `endpoint` feature.
The one-time package cut aligns Core 0.17 / Endpoint 0.7; generated contracts,
attribute options, bytes and execution behavior do not change.

Replace Wire's removed `endpoint` feature with an `ic-wire-endpoint` dependency.
Move attributes/helpers from `ic_wire::endpoint` to `ic_wire_endpoint`; retain
codec and schema imports from `ic_wire`. Enable `typescript` here for native
Contract schema projection. Frame bytes, bounds, acceptance and rollback rules
are unchanged. See the repository endpoint guide for handler examples.

### Independent Runtime adoption

Rust 1.99.1 or newer is required.

Endpoint 0.7 selects published `ic-platform-runtime = "0.19.0"` directly,
with Core 0.17 schemas on canonical Types 0.18.1. Earlier Endpoint 0.5 / Core 0.15
used Runtime 0.18 and Types 0.17.
Applications that use low-level APIs select that package's `cdk` feature and use
`ic_platform_runtime::{cdk, executor, ic0}`. Handler macros continue to target the
runtime re-exported through Endpoint; applications need no new dependency solely
to use an Endpoint macro. Core and derive keep their independent release identity.

Select matching Timers and other runtime suppliers before upgrading applications.
The qualified Runtime/macros/exact guard 0.19.2 share canonical Types 0.18.1;
the compatible Runtime dependency range remains ^0.19.0.
Do not mix older execution generations with this singleton. Crypto/Candid facade
features may be selected separately with a facade that delegates to this owner.
Qualification uses the normal registry lock without supplier patches. The private
native fixture uses the matching Platform/Testkit client; no facade enters
the production Endpoint graph.

Types 0.18 changes Account's present subaccount from `01 + 32 bytes` to
`01 20 + 32 bytes`; None stays `00`. Match Core's bounded-byte schema,
regenerate Uint8Array clients, use fresh contract keys and requalify limits.
Current macros still use exported-name default IDs: schema/package changes do
not update them automatically. Resolve retained operations and unframed stored
data under consumer policy; preserve original retry/hash bytes. Automatic
fingerprints remain a separate prepared experiment, not an activated API.
