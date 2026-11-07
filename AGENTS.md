# IC Wire contributor instructions

Apply the reviewed [Shared 0.31.5 policy](https://github.com/dragginzgame/shared-tooling/blob/b2026a118e38de4af1555c527af327cb604eacdf/DRAGGINZGAME.md)
and its revision-bound references. The installed [baseline](DRAGGINZGAME.md) and
[manifest](.shared-tooling.snapshot) own executable snapshot provenance; refresh
through Shared's exporter, never by editing payloads or following a moving sibling.
Shared owns action authority, delivery, compiler support, validation and reporting.
Do not add local approval gates or duplicate those rules here.

Read [the current handoff](docs/status/current.md) and this overlay first.
README.md provides orientation. Read docs/design.md for architecture changes,
docs/runtime.md for runtime contracts, docs/evaluation.md for codec qualification
and docs/development.md for local commands. These are task-specific references.

## Product boundaries

- Keep the Core codec Wasm-safe and independent of SDKs and native tooling.
  `ic-wire`/`ic-wire-derive` own one Serde/Postcard codec and host schema generation;
  the finite Wire API is retired. Alternative codec/derive strategies are research.
- Endpoint and its proc macro form one independent release group. They select
  Platform Runtime directly for exports/execution; do not introduce an executor.
  Canic policies, transport, authentication, retries and artifact admission stay
  downstream. No browser transport is added here.
- Core's optional `platform-types` schemas select canonical Platform Types.
  Keep schema-only hosts free of CDK/executor/endpoint machinery. Endpoint alone
  selects CDK; require one Runtime guard where execution is selected.
- Candid remains outside published packages. The unpublished benchmark package
  may compare external Candid DTOs through Platform and use Testkit's maintained
  PocketIC client, types and lifecycle as native dev dependencies. The unpublished
  endpoint fixture qualifies integration through those same owners.
- Keep qualification packages `publish = false`; their publication needs its own
  explicit scope. Keep current supplier selections/publication state in the
  handoff and root manifests/lock, not a second catalog here. No supplier patches
  or downstream wrapper/new-schema-trait requirements.
- Canic, Relay and IcyDB retain their contracts and runtime semantics. IcyDB's
  database storage/key encodings are outside this repository's initial scope.
- Establish a reusable Rust experience and a substantial measured win before
  broad adoption. Postcard is selected for bounded Rust integration and controlled
  RPC/owned endpoints; preserve the explicit docs/runtime.md contract.

## Implementation

Use the virtual root workspace, inherited package metadata/lints and root-owned
workspace.dependencies. Keep existing codec, schema-derive and Endpoint owners;
no speculative crate splits or placeholder public APIs. Production modules use
directories with mod.rs; tests follow Shared's tests.rs / *_tests.rs / tests/
conventions and ordinary module discovery. Declare private package LOC roles.

Rust **1.99.1** is both minimum and pinned compiler (maintainer decision #42).
Admit the exact workspace floor through the Shared reader and require equality
with rust-toolchain.toml and Cargo/rustc identities. Preserve one-compiler host,
Wasm, macro, schema and Endpoint proofs; 1.91 evidence is historical. The
maintainer approved a separate dated nightly for the opt-in sanitizer fuzzer
only; its pin belongs to scripts/ci/fuzz-runtime.sh. It changes neither the
package floor nor the development/release lane.

Inherit the workspace unsafe-code prohibition. The benchmark package instead
uses denial plus its narrowly justified IC-export attribute; exports call safe
system-call wrappers. Preserve native codec, malformed-input, allocation/borrowing,
compile-fail and standalone Rust/TypeScript conformance owners.

## Protocol and performance

- Keep limits checked before allocation and malformed-input errors typed. Custom
  Serde work outside visited values remains trusted, per docs/runtime.md.
- Native metadata has no field IDs and follows Serde declaration order; changing
  it requires coordinated consumers. Preserve exact retained retry bytes and
  encoder hash inputs. Primitive Postcard validation is one pass; collection and
  custom-Serde normalization is consumer-owned, so re-encoding may differ.
- Pre-1.0 hard cuts must not create legacy/v2 readers or automatic migration.
  Every protocol-affecting change needs an inline old/new byte or acceptance-rule
  explanation, consumer action, changelog and final handoff example. Explicitly
  state when bytes, identities, limits and acceptance remain unchanged.
- Default binary examples to byte fields and large supported integer/hash arrays
  to Packed<T>. Keep ordinary collections for records and small-value varints;
  preserve field bounds and visible ordinary-vector controls. Never silently
  rewrite consumer Serde. Stress fixture gains are not application savings;
  keep heavy codec mechanics shared and avoid added per-type codec variants.
- Runtime/generator-cost changes need a reproducible matched before/after study:
  compiler, target/profile, lock, input identity, parameters, small controls,
  repeated absolute results and regressions. Label native time, IC instructions,
  code-section and total uncompressed Wasm separately. No gzip objective or
  symbol-attribution-as-saving claims. State blocked/not-applicable measurements.
- Give isolated paired sources separate Cargo targets and verify fresh artifacts
  and baked source hashes; copied timestamps can admit stale builds. Check for
  active builds sharing target/ before compilation. Do not run broad sibling suites.

## Local workflow

Use focused checks from docs/development.md; ordinary work does not run make ci.
Select public release groups using the
[local impact decision](docs/development.md#decide-whether-a-public-release-is-needed),
not private qualification ownership or the presence of a pending changelog entry.
The maintainer-operated direct delivery destination is origin/main through Shared
Tooling. Core and Endpoint groups use literal versions and releases/<group>/
ledgers. Pass RELEASE_GROUP explicitly for release/resume/publication and preserve
qualification-only publication exclusions. Shared owns delegated effects.

One roster in ci/release-validation.json owns CI and release plans. Core retains
20 stages; qualified Endpoint-only Linux/default changes use 11. Shared/uncertain
inputs and other hosts broaden to full. Stage reuse/final-candidate execution
stay disabled until Wire qualifies those opt-ins. Local adapter fixtures use
only disposable repositories and never invoke release effects in this checkout.
