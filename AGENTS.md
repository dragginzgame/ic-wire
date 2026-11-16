# IC Wire

Read TOKO-TOOLING.md, README.md and docs/DESIGN.md before implementation.
The installed Toko Tooling baseline is adopted; this overlay retains Wire's
product scope and stricter safety constraints.

- This is an authorized local replacement evaluation, not a deployed replacement.
  Qualify the current implementation independently; do not restore historical
  implementation dependencies or paired comparison lanes.
  Production deployment and consumer migration remain outside this local scope.
- Reduce independent responsibilities and execution paths. Do not transplant the
  old implementation or its release system merely to obtain a working workspace.
  Reuse requires a destination contract, provenance and focused qualification.
- Preserve one production Serde/Postcard codec unless a separately justified
  design decision changes it. No private Postcard fork, parallel codec lanes or
  automatically rewritten consumer Serde representations.
- Keep codec code Wasm-safe and independent of SDKs, native tools and Candid.
  Endpoint execution, transport, authentication, retries and deployment belong
  downstream. Do not restore Wire Endpoint or an executor.
- Platform Types owns Principal/Account identities and Serde implementations.
  Wire owns optional host schema integration; keep codec-only graphs small.
- Preserve pre-allocation admission and typed malformed-input errors. Account for
  trusted custom Serde work and consumer-owned collection normalization explicitly.
- The user authorized a hard cut: legacy API, wire, schema, acceptance and budget
  compatibility are not requirements. Keep one current implementation without
  versioned compatibility lanes. Record intentional contract changes and qualify
  safety and cross-language behavior. Actual deployment and persisted-state
  operations remain separately authorized actions.
- Avoid new abstraction layers and per-type codec specialization without measured
  justification. Prioritize total uncompressed Wasm at opt-z; report opt-3 losses,
  IC instructions, allocation and payload bytes separately. Do not optimize gzip.
- Keep product packages in a virtual root under crates/ and real qualification
  packages in a separate qualification/ workspace. Add no placeholder packages.
  Candid is an optional research comparator in private dev-only qualification.
  Use published Host, Build and Testkit for their owned native responsibilities.
- Tooling is installed through the authenticated Toko Tooling 0.44.11 exporter;
  `.toko-tooling.snapshot` records its source and owned bytes. Detailed reference
  policy remains pinned at revision 626da7cd9626cc2e152594862a2d82ea82ef51da;
  do not follow a moving sibling baseline. The compiler is Rust 1.100.0.
  `make toko-tooling-check` admits the member declaration, Cargo.lock and snapshot;
  `make snapshot-check` verifies just the installed bytes.
- Keep lasting design decisions small and current; do not copy evidence trees,
  historical journals or broad orchestration before there is a real caller.
- This initialization authorizes no commit, remote creation, publication,
  deployment, consumer migration or deletion of the original repository.

## Dependency selection

Use ordinary Cargo-compatible requirements for registry libraries; each workspace's
committed lockfile freezes its qualified selection. The 15 November 2026 adoption
review checked ordinary direct Rust suppliers against the official registry index;
those selections were already the latest stable non-yanked releases. Compatible
requirements retain the selected versions until an explicit lockfile update.
Review future lock updates as dependency changes, not automatic new benchmark proof.

Retained exact requirements have these specific owners and reasons:

- Root `Cargo.toml`: `ic-wire-derive =0.24.3` is coupled to the product runtime's
  generated metadata API, including its private macro support. Update the pair
  together and qualify derived consumers and archives.
- Both workspace catalogs: `toko-tooling =0.44.11` is the required exact
  source-bundle selection. Change it only with the authenticated snapshot update
  and admission procedure above.

## Qualification layout

All qualification packages are immediate children of qualification/:
ic-wire-compatibility, ic-wire-consumer and ic-wire-instructions. They share one
virtual qualification/Cargo.toml and Cargo.lock. The maintainer's single-tier
layout correction supersedes the former separate instruction-workspace exception.
The initial consolidation preserved both previous graphs' selections. Subsequent
supplier updates are qualified independently: product thiserror is 2.0.21;
PocketIC requires 2.0.18 in qualification, where generic-array stays at 0.14.7.
No simulator dependency enters product Wire.

The parent qualification workspace denies unsafe code for the narrowly expected
Rust 2024 guest export attribute. The native instruction driver retains a local
forbid. Product packages continue to forbid unsafe code.
