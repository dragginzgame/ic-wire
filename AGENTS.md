# IC Wire contributor instructions

Apply the reviewed [Toko Tooling 0.38.0 policy](https://github.com/dragginzgame/toko-tooling/blob/b171c182dbab832c74c97e978f70a6a0e3b8fb7c/TOKO-TOOLING.md)
and its revision-bound references. The installed [baseline](TOKO-TOOLING.md) and
[manifest](.toko-tooling.snapshot) own executable snapshot provenance; refresh
through the Cargo-selected registry installer, never by editing payloads or following a moving sibling.
Toko Tooling owns action authority, delivery, compiler support, validation and reporting.
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
  may select Candid directly as an unconditional dev-only comparison oracle
  under the reviewed tooling exception and use Testkit's maintained
  PocketIC client, types and lifecycle behind native-only `ic` features. The unpublished
  endpoint fixture qualifies integration through those same owners.
- Keep qualification packages `publish = false`; their publication needs its own
  explicit scope. Keep current supplier selections/publication state in the
  handoff and the owning workspace manifests/locks, not a second catalog here. No supplier patches
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
directories with mod.rs; tests follow Toko Tooling's tests.rs / *_tests.rs / tests/
conventions and ordinary module discovery. Declare private package LOC roles.

The independent virtual workspace `qualification/Cargo.toml` owns the three
unpublished packages in `qualification/` and `qualification/Cargo.lock`.
It isolates Build/Testkit/Platform-facade resolution from the four public root
members under the tooling core/integration boundary. Native corpus/allocation
checks do not enable `ic` or `diagnostics`; these features select Testkit and
Build only when needed. Both workspaces retain the same compiler, formatting
and tooling admission. Use the focused commands in docs/development.md; retain
explicit integration proofs and the existing release roster. No public package
version or protocol change is implied by this private workspace split.

Rust **1.99.1** is both minimum and pinned compiler (maintainer decision #42).
Admit the exact workspace floor through the Toko Tooling reader and require equality
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
- Use opt-z as the primary optimization decision profile for our size-constrained
  deployments. Keep opt-3 as a secondary regression check and report its losses;
  a candidate need not win both profiles. Small measured uncompressed Wasm growth
  may be accepted for substantial instruction savings with an explicit recorded
  tradeoff. Automated cost-screen failures remain visible; do not call an approved
  tradeoff a passing no-growth check or silently relax its thresholds.
- Give isolated paired sources separate Cargo targets and verify fresh artifacts
  and baked source hashes; copied timestamps can admit stale builds. Check for
  active builds sharing target/ before compilation. Do not run broad sibling suites.

## Local workflow

Use focused checks from docs/development.md; ordinary work does not run make ci.
Select public release groups using the
[local impact decision](docs/development.md#decide-whether-a-public-release-is-needed),
not private qualification ownership or the presence of a pending changelog entry.
The maintainer-operated direct delivery destination is origin/main through Toko
Tooling. Core and Endpoint groups use literal versions and releases/<group>/
ledgers. Pass RELEASE_GROUP explicitly for release/resume/publication and preserve
qualification-only publication exclusions. Toko Tooling owns delegated effects.

One roster in ci/release-validation.json owns CI and release plans. Core retains
20 stages; qualified Endpoint-only Linux/default changes use 11. Toko Tooling/uncertain
inputs and other hosts broaden to full. Stage reuse/final-candidate execution
stay disabled until Wire qualifies those opt-ins. Local adapter fixtures use
only disposable repositories and never invoke release effects in this checkout.
