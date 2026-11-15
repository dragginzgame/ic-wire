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
- Tooling is installed through the authenticated Toko Tooling 0.44.4 exporter;
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

- Root `Cargo.toml`: `ic-wire-derive =0.24.1` is coupled to the product runtime's
  generated metadata API, including its private macro support. Update the pair
  together and qualify derived consumers and archives.
- All three workspace catalogs: `toko-tooling =0.44.4` is the required exact
  source-bundle selection. Change it only with the authenticated snapshot update
  and admission procedure above.

## Scoped instruction-workspace exception

Under the maintainer's 15 November 2026 adoption request to approve or remove the
extra workspace after checking resolution, retain `qualification/ic-instructions/`
for its sole unpublished member `ic-wire-instructions` and its own
`Cargo.lock`. This is a scoped exception to the single qualification workspace,
using the pinned baseline's [integration isolation rule](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/rules/rust-workspaces.md#core-and-integration-graph-boundary).
It grants no additional product or release workspace.

The resolver probe on that date merged the real driver into a scratch copy of
the parent qualification workspace. Resolution succeeded but replaced the codec
graph's current `thiserror` and `thiserror-impl` 2.0.21 with 2.0.18:
`ic-testkit 0.106.0 -> pocket-ic 16.1.0 -> thiserror =2.0.18` constrains the shared
`postcard 1.1.3 -> cobs 0.3.0 -> thiserror ^2` path. There is no incompatible
public type crossing here; isolation prevents the native simulator from forcing
that downgrade or blocking later codec dependency maintenance. The simulator's
older transitive selection is not a direct dependency freshness exception.

The root Makefile includes this workspace in formatting and dependency commands;
`measure-consumer-ic` admits its tooling selection before running it. Its member
inherits the same common Cargo, Rust, Rustdoc and Clippy policy as the product.
Focused validation is locked metadata, native-driver Clippy with `-D warnings`,
and Rustdoc with `RUSTDOCFLAGS="-D warnings"`. Installed instruction qualification
uses `POCKET_IC_BIN=... make measure-consumer-ic` when its runtime contract changes.
Ordinary codec checks do not start a simulator. The guest stays in the parent
qualification workspace and no native dependency enters product Wire.

Revisit this exception when a selected Testkit/PocketIC update removes or relaxes
the exact constraint. Consolidate only after a resolver probe preserves the
independently selected codec suppliers and focused driver/consumer checks pass.
