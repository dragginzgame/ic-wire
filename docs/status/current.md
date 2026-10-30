# Current handoff

## Released source 0.9.0; pending 0.10.0

Released baseline: `edb5d1b`, Wire 0.9.0. Own versions remain unchanged. Pending
0.10.0 adds opt-in identity schemas and adopts published Platform 0.11. Leave staging,
commits, versions, publication and release execution to the maintainer.

## Principal/Account schema transfer

`platform-types` implements the existing host `typescript::Wire` trait for
Platform's owned Principal and Account. It enables `principal` with defaults
disabled and host TypeScript support. Only `endpoint` selects CDK; both features
share one optional Platform dependency. Core-only Wire remains Platform-free.
No wrappers, extra crates, serializer alternatives or second schema trait.

Consumers remove Platform `wire-schema` and select Wire `platform-types`.
Principal stays bounded bytes (max 29); Account stays owner then optional
32-byte subaccount. Absent and explicit zero remain distinct bytes despite
Account equality. Serialization, budgets, identities, accepted inputs and
retry bytes are unchanged. See the [migration](../../crates/ic-wire/README.md#optional-platform-identity-schemas).

The single root dependency and lock now select published Platform 0.11.0, macros
and runtime guard together. All 126 supplier source files match the isolated
preparation; registry archive checksums match the lock. Endpoint consumers must
align runtime-consuming dependencies on Platform 0.11. The coordinated runtime
identity change makes the pending Wire release 0.10.0. No sibling checkout was
modified and no publication, release or commit was performed.

## Qualification

[Prepared-source evidence](../evidence/platform-types-transfer/README.md) binds
Platform 421edbc plus its prepared removal, the Wire transfer, exact compiler,
locks, server and artifact hashes. Schema output and Rust frame vectors match
between registry 0.10 and prepared 0.11. Rust 1.91 standalone derives, host/Wasm
feature isolation, combined endpoint schemas, strict Clippy and Rust/TS
byte/malformed tests pass. `make check-platform-types` runs this focused consumer
proof and is included in the TypeScript CI group.

[Published-supplier qualification](../evidence/platform-types-transfer/published/README.md)
uses registry 0.11 for standalone schema/Serde consumers, Rust 1.91 host/Wasm,
combined endpoint graphs and strict Clippy. Exact Rust/TS bytes and malformed
cases pass. Endpoint native/schema tests, 11 compiler diagnostics and passing
control, strict lint, Wasm/MSRV and installed limits/rollback pass. The Candid
comparison corpus's three roundtrip/rejection tests pass on the new supplier.

Dependency checks match package names, avoiding a false Candid hit in local
candidate paths. The canister graph has one runtime; Testkit's native-only
Platform 0.9.1 stays outside it. Schema-only users select no Candid/executor;
Wasm has no serde_json and core-only Wire remains Platform-free.

The published-supplier comparison covers 12 cases/seven exports, two profiles
and eight repetitions after warmup: 384 measured logical calls / 608 physical
executions. Every instruction/cycle sample matches, as do frames and outcomes.
Code sections remain 86,068 B (opt-z) and 80,630 B (opt-3); total Wasm is
99,947 → 99,950 B (+3 B, +0.003%) and 93,518 → 93,518 B. Raw outputs are
`/tmp/wire-platform011-published/`. The prepared study stays separately scoped.

Focused pin/boundary checks, format, script syntax and documentation links pass. No full CI, macOS or
sibling suite was run. Original prepared outputs remain
`/tmp/wire-platform-types-transfer/`. Exact inputs and reproduction are in
the evidence; the prepared study's source-path size difference is not a saving.

## Maintained tooling and earlier studies

Build/Testkit/managed release-editor remain 0.71.0. Shared Tooling remains reviewed
0.27.11 at `62edfbd43087a718bf0283e9040676a49211976d`; no snapshot changes.
The [previous supplier study](../evidence/platform010-tooling071/README.md) retains
its own inputs and measurements. Evidence manifests use `.snapshot`, not active
`Cargo.toml` names. [Publication](../cargo-publication.md) remains explicit.

The [Candid matrix](../evidence/candid-comparison-0725/README.md) retains its
22-cohort controls; ordinary vectors remain weak cases and explicit Packed wins
its 436 primary comparisons. The [Tripwire comparison](../evidence/tripwire-comparison-0725/README.md)
is a focused ingress probe, not general CDK-RSX or consumer qualification.
The [admission study](../evidence/tripwire-adoption-0725/README.md) keeps selective
input as research: its copied-input, fixed-shape prototype is not a generic
Serde reader. No pre-copy API or consumer integration is added here.

Platform owns exports/execution; consumers own authentication, dispatch, retries,
persistence and deployment. Retain original retry/hash bytes. Synchronous reply
failure rolls back; after await only the current callback rolls back. See
[endpoints](../endpoints.md) and the [SDK recipe](../sdk-integration.md).
