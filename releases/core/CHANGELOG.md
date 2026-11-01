# Core group changelog

Covers ic-wire, ic-wire-derive and the private ic-wire-benchmarks,
ic-wire-qualification and ic-wire-endpoint-fixture packages. This independent
series begins at 0.12.0. Earlier coordinated releases end at 0.11.0 and remain
in [shared history](../history.md).

## [0.13.3] - 2026-11-01

### ➕ Added

- Expose standard `serde_bytes::ByteArray<N>` as `ic_wire::ByteArray<N>` with
  exact-width TypeScript metadata, including standard fixed-array byte adapters
  and optional fields. Existing arrays are unchanged; explicitly adopting the
  bulk representation changes `[0_u8,255]` from `00 ff` to `02 00 ff`, requiring
  coordinated contracts, regenerated clients and qualified limits.

### 🧪 Testing

- Qualify standard byte arrays across Rust/TypeScript, malformed inputs, budgets,
  standalone derives and Rust 1.91 host/Wasm forwarding. The
  [matched key-map study](../../docs/evidence/byte-array-0133/README.md) retains
  small controls and Candid: at 128 keys opt-3 instructions fall 78.50% encode /
  68.32% decode, with 128 more frame bytes and 1,526 more raw Wasm bytes.

- Refresh the complete Wire/Candid comparison: 22 cohorts, 1,243 case entries,
  both optimization profiles, repeated native samples, allocation/heap probes,
  IC instructions, wire bytes and raw Wasm/code-section sizes. Retain the frozen
  source and all 88 modules; strict pairing, coverage and provenance checks pass.
  Include ordinary-control losses and the 35 new cases. This is measured codec
  qualification, not a production optimization or protocol change.
  [Full results](../../docs/evidence/candid-comparison-0133/README.md).

- Add 35 matched Wire/Candid workloads for zero-sized collections, binary-key
  maps, nested blob results, optional arrays, ownership wrappers and enum shapes.
  Append cases to preserve existing IC indices and retain empty/singleton controls.
  Add native Serde checks for all nonzero integer types, network addresses,
  ranges and exact IEEE float bits, with malformed-value/truncation/trailing-byte
  coverage. These are codec tests, not new TypeScript schema support or fresh
  performance measurements. Production bytes, budgets and acceptance are unchanged.

### ⬆️ Dependencies

- Select Platform 0.16.0 for private codec/endpoint qualification while preserving
  canonical Types 0.15.1 and the independent Core codec/schema identity. Keep
  Testkit's native-only Platform 0.15 selection isolated from production runtime;
  the endpoint integration's coordinated adoption belongs to Endpoint 0.3.0.

- Adopt Build/Host 0.73.3, including the managed release metadata editor, and
  Testkit 0.75.2. Preserve canonical Types 0.15.1; the retained benchmark and
  adoption reports identify their own frozen Platform/Host/Metrics selections.
  These tooling-only changes require no Core consumer migration; they preserve
  production codec/schema bytes, budgets and malformed-input acceptance.

### 🔧 Changed

- Refresh Shared Tooling to committed revision `619d194` (recorded 0.27.23)
  through the canonical exporter. Keep Cargo's independent release groups and
  package-version preparation unchanged; standalone pending-VERSION automation
  does not apply to this workspace.

## [0.13.2] - 2026-10-31

### ⬆️ Dependencies

- Adopt Build/Host 0.73.2, including the managed release metadata editor, and
  Testkit 0.75.1 with arithmetic-only Metrics 0.24.1. Preserve the production
  codec/Endpoint dependency graph, protocol bytes, resource budgets and Rust 1.91
  support. No consumer migration or runtime performance saving is implied.

## [0.13.1] - 2026-10-31

### ⬆️ Dependencies

- Require canonical Platform Types 0.15.1 and adopt Build/host tools 0.73.1,
  including the explicitly installed release metadata editor. Native controls use
  Testkit 0.75.0 / Metrics 0.23.2, removing the old Platform 0.13 dependency.
  Canonical Rust type identity, schema bytes, limits and codec acceptance remain
  unchanged; no consumer migration or production codec change is required.

### 🔧 Changed

- Adopt Shared Tooling revision `143a9ab` through the canonical exporter
  (upstream's committed VERSION records 0.27.21). Present all six workspace members
  once in root dependency catalog blocks, with group changelog links and complete
  release-policy alias rosters. Remove the duplicate header map; admission checks
  coverage, paths and full member versions across all groups. Preserve package
  versions, dependency resolution, Cargo cleanup and independent release flows.
- Qualify the published Testkit 0.75 typed APIs and Metrics 0.24 metadata against
  registry Wire 0.13 on Rust 1.91, without source-aligned manifests or patches.
  The workspace's arithmetic-only Metrics selection remains 0.23.2.

### 🧹 Cleanup

- Move the unchanged coverage-guided fuzz target into private qualification under
  its explicit native fuzz-runtime feature. Make native heap/allocation profilers
  optional through the benchmark's existing heap/allocations features. Ordinary
  codec tests and default benchmark graphs no longer select these tools; graph
  checks enforce that boundary. Keep explicit profiler/fuzz coverage, mutual
  exclusion and Rust 1.91 qualification. No codec API, bytes, budgets or malformed
  acceptance changes; no build-time or Wasm-size saving is claimed.
  [#39 — Opt-in qualification tools](https://github.com/dragginzgame/ic-wire/issues/39).

## [0.13.0] - 2026-10-31

### ⚠️ Breaking

- Align published Platform, macros, runtime guard and Types on 0.15.0.
  Types moves from 0.1 to 0.15, changing Principal/Account Rust identity even
  though its implementation is unchanged. Schema consumers align direct
  suppliers and typed/metadata clients on this Wire minor; endpoint consumers
  select the supplying Endpoint minor. No wrappers or compatibility lanes.
  Codec bytes, bounds, budgets, malformed acceptance and retry/hash inputs stay
  unchanged. Absent Account `01 04 00` and explicit zero `01 04 01` plus 32 zero
  bytes remain distinct.

### 🥾 Governance

- Adopt the reviewed Shared Tooling 0.27.19 snapshot at `54fc49e`. Use literal versions for every member, symmetric group ledgers
  under `releases/`, a root changelog index and one preserved coordinated history.
  Require explicit core/endpoint selection for release, resume and publication;
  keep crate paths, numeric versions, lock selections and the full gate intact.
  Protocol/runtime behavior and codec performance are unchanged by this tooling
  and metadata migration. Delegated release/ship/deploy requests now use the
  existing delivery workflow without repeated per-command confirmation.

### 🧪 Testing

- Qualify current/MSRV host/Wasm schemas, exact Rust/TypeScript frames/refusals,
  renamed endpoint diagnostics and feature isolation. Make the prospective
  identity fixture resolve its current supplier and next simulated version from
  workspace metadata rather than fixed historical versions. Matched
  [installed qualification](../../docs/evidence/platform015-adoption/README.md) retains
  identical instruction/cycle samples over 384 logical calls / 608 executions.
  Total Wasm stays 100,441 B opt-z / 94,073 B opt-3; code stays 86,544 / 81,127 B.

### 📚 Documentation

- Consolidate obsolete Platform 0.13 issue #37 into #36 without claiming the old
  pair was published. Record delivered core 0.12.1's successful hosted matrix,
  pending current supplier delivery and Platform's prepared independent-group
  restoration; retain published/source-aligned consumer distinctions.

## [0.12.1] - 2026-10-31

### 🩹 Fixed

- Reconcile the earlier release-group adapter contribution with the stable
  identity/endpoint split. Preserve current groups and receipt paths, retain
  literal metadata exclusions and the validated fixture-editor override, and
  exercise both real groups instead of recreating an obsolete synthetic policy.
  Protocol bytes, budgets and runtime behavior are unchanged.

## [0.12.0] - 2026-10-31

### ⚠️ Breaking

- Implement the approved [stable identity boundary](../../docs/stable-identities.md)
  for [#36](https://github.com/dragginzgame/ic-wire/issues/36): move endpoint
  helpers and attributes into independently versioned `ic-wire-endpoint` 0.1.0.
  Remove `ic-wire/endpoint`; migrate `ic_wire::endpoint` imports to
  `ic_wire_endpoint`. Keep codec/schema/limits ownership with ic-wire/derive.
- Select published Platform Types 0.1 for `platform-types` schemas and Platform
  0.14 for Endpoint. Principal/Account trait implementations stay with Wire;
  Platform reexports the canonical stable definitions. Consumers align facade
  and native typed/metadata suppliers once on this Wire minor. This supersedes
  the pending 0.13-only pairing in [#37](https://github.com/dragginzgame/ic-wire/issues/37).
  Bytes, budgets, declaration order, malformed acceptance, retry/hash inputs and
  rollback behavior stay unchanged. Absent Account `01 04 00` remains distinct
  from explicit-zero `01 04 01` followed by 32 zero bytes.

### ⬆️ Dependencies

- Adopt Shared Tooling 0.27.14 at `b8d2c6e27a755a7bae46ef25ccd2b5b4f53e8e57`
  through the canonical exporter. Include explicit publication options,
  core/integration boundary guidance, opt-in Cargo release-group support and the
  narrow `ic-platform-types -> ic_principal` owner exception. Wire's direct
  dependency restrictions remain intact.
  Keep Wire's current coordinated release setup; group activation still needs
  the supplying IC Build editor and local integration qualification. Runtime
  dependencies, codec bytes, budgets and acceptance rules are unchanged by this
  tooling adoption.

- Adopt IC Build/Testkit 0.73.0 and the matching managed `cli` editor. Use its
  `group-manifest` renderer with explicit core/endpoint release groups and separate
  ledgers/receipts. No package publication or existing version changes are performed.

### 🧪 Testing

- Qualify the [stable identity/endpoint migration](../../docs/evidence/stable-identities-0120/README.md)
  with current/MSRV host/Wasm, exact Rust/TypeScript schemas/bytes/refusals,
  diagnostics, installed guards/rollback and prospective identity isolation.
  Align released Metrics/Testkit source copies through manifest-only changes for
  one trait/Limits/Error identity; no downstream working checkout edits or Cargo
  patches. Qualify both release groups and exact publication selections in
  disposable repositories. Matched 384 logical calls / 608 executions retain
  identical frames/outcomes; total Wasm grows 714 B (opt-z) / 555 B (opt-3), and
  the largest instruction regression is 0.026%. Retain raw traces and inputs.
- Qualify the published Wire 0.11 / Platform 0.12 / Metrics 0.22 pairing with
  Blob's actual schema example in an isolated released-source copy, standalone
  Rust 1.91 host/Wasm consumers and Rust–TypeScript exact-byte/malformed checks.
  Retain [reproduction and evidence](../../docs/evidence/published-pair-0110/README.md)
  for [#34](https://github.com/dragginzgame/ic-wire/issues/34); sibling working
  checkouts are untouched. No codec bytes, budgets or acceptance rules change.

### 📚 Documentation

- Document the one-time schema/endpoint migration, group commands and initial
  Endpoint publication ordering. Published consumer qualification remains pending
  the maintainer delivery; #36/#37 are not closed on local evidence alone.
- Refresh the handoff for released 0.11.0 and close completed #31/#34 outcomes.

