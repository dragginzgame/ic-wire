# Endpoint group changelog

Covers ic-wire-endpoint and ic-wire-endpoint-macros. Its independent series begins at 0.1.0; pre-extraction
repository releases remain in [shared history](../history.md). The published
0.1.0 heading was undated and is preserved as recorded.

## [0.13.0]

### ⚠️ Breaking

- Adopt Platform Runtime/macros/guard 0.22 and canonical Types 0.20. Align
  downstream Runtime/Types dependencies together; frame bytes, identities,
  limits and transaction semantics remain unchanged.

- Require Rust 1.100.0 for Endpoint and its macros and apply the common lint
  policy ([#60](https://github.com/dragginzgame/ic-wire/issues/60)). Upgrade the
  consumer compiler; frame bytes, method identities, budgets, dispatch and
  rollback semantics remain unchanged.

## [0.12.2] - 2026-11-12

### Maintenance

- Qualify Platform Runtime/macros/guard 0.21.2 and Types 0.19.4 with Core 0.21.3,
  using Testkit 0.101.0 for installed calls, limits and rollback checks.
  Endpoint source, frame bytes, identities and budgets remain unchanged.

## [0.12.1] - 2026-11-11

### Maintenance

- Qualify the current Core 0.21 integration with Platform Types 0.19.3 and
  Toko Tooling 0.39.11. Endpoint execution, frame bytes, identities and budgets
  remain unchanged.

## [0.12.0] - 2026-11-11

### ⚠️ Breaking

- Align Endpoint with Core 0.21's generated TypeScript naming contract. Update
  Core and Endpoint together, regenerate schemas/clients and replace positional
  type imports. Endpoint execution, encoded frames, method IDs, budgets and
  rollback behavior remain unchanged.

### 🔧 Changed

- Qualify Platform Runtime 0.21.1 with its matching macros/guard and Types
  0.19.1. Runtime owns the transitive IC system binding; Core and schema-only
  consumers remain independent of execution machinery.

## [0.11.0] - 2026-11-10

### ⚠️ Breaking

- Adopt Core 0.20's upstream Postcard decoder. Encoded frames, method identities,
  resource budgets, handler APIs and rollback behavior stay unchanged. Redundant
  varints now decode successfully (`80 00` becomes zero); consumers must review
  strict-rejection assumptions, regenerate browser clients and retain original
  retry/hash bytes. Align Core and Endpoint to this supported release line; no
  compatibility reader or stored-data rewrite is introduced.

## [0.10.0] - 2026-11-09

### 🔧 Tooling

- Adopt Toko Tooling 0.38.0 and its independent qualification layout. Flatten
  private package paths under `qualification/` and migrate all active callers;
  public package identities, codec bytes, limits and acceptance are unchanged.

### ⚠️ Breaking

- Select Runtime/macros/guard 0.21 for canonical Types 0.19 composition with
  Core's pending 0.19 integration. Align consumer package generations; old
  published Endpoint/Types pairs are not supported mixed compositions. Handler
  syntax, exports, frame bytes, identities, limits and rollback remain unchanged.
  Qualify raw Testkit transport through explicit native principal bytes without
  requiring its private infrastructure to adopt the product's Types generation.
  [#58](https://github.com/dragginzgame/ic-wire/issues/58)

## [0.9.0] - 2026-11-08

### ⚠️ Breaking

- Select Core 0.18 alongside its rooted metadata-lint API hard cut. Endpoint
  handlers and macro syntax are unchanged. Frame bytes, method identities,
  limits, acceptance and rollback behavior remain unchanged; retained retry
  bytes and stored records need no migration. Host lint callers must migrate
  to Core’s `lint_bulk_roots` API and rooted acknowledgement paths.

## [0.8.1] - 2026-11-08

### ⬆️ Dependencies

- Select Platform Runtime 0.20.1 with matching macros and exact guard, preserving
  its entrypoint and option-parsing semantics. Retain one execution generation.
  Frame bytes, schemas, method identities, limits, acceptance, exports and
  rollback behavior are unchanged; no client regeneration or migration is needed.

## [0.8.0] - 2026-11-07

### ⚠️ Breaking

- Select published Platform Runtime/macros/exact guard 0.20.0. Align Timers and
  other execution suppliers before upgrading a shared canister graph; the exact
  singleton guard rejects mixed runtime generations. Wire already uses Runtime's
  retained ordinary `cdk` macro family, so its attributes need no syntax change.
  Direct Runtime callers must replace removed `runtime_*` aliases with ordinary
  `cdk` macros. Core's codec/schema identity, frame bytes, method identities,
  budgets, acceptance, exports and rollback rules are unchanged. No client
  regeneration or data migration is required by this supplier adoption.

## [0.7.3] - 2026-11-07

### 🔧 Changed

- Require published Platform Runtime 0.19.5 with its matching macros and exact
  singleton guard. Adopt its task-removal and initialized-buffer copy fixes;
  Wire codec bytes, schemas, method identities, budgets and rollback rules
  remain unchanged. No consumer migration is required.

## [0.7.2] - 2026-11-06

### 🩹 Fixed

- Fully qualify successful generated guard results so a consumer's imported
  `Ok` cannot capture query/update expansion. Cover owned and borrowed handlers,
  disabled items, generics and receiver diagnostics with the real macro.
  Public APIs, bytes, identities, admission and rollback rules are unchanged.

### 🧪 Testing

- Include mixed published/current consumers in Endpoint qualification and run
  frozen byte/admission assertions on the installed Wasm test fixture.

## [0.7.1] - 2026-11-05

### 🔧 Changed

- Adopt published Platform Runtime/macros/exact guard 0.19.2 together. Endpoint
  remains the execution integration owner; codec bytes, schemas, identities,
  budgets and rollback rules are unchanged.

### 🧹 Cleanup

- Use standard-library paths in the SDK-backed Endpoint runtime and generated
  entrypoints. Public APIs, expansion behavior, codec bytes, budgets and rollback
  rules are unchanged; Core retains its no_std support.

### 🧪 Testing

- Keep full history secret scanning enabled while recognizing exact historical
  benchmark source-digest fingerprints. Preserve the existing exception and
  qualify unrelated token-shaped input rejection; no files or rules are skipped.
- Keep the invalid-UTF-8 filename fixture Linux-only; macOS filesystems reject
  that test input before the report scanner can inspect it.

## [0.7.0] - 2026-11-04

[Migration and release contract](0.7.md).

### ⚠️ Breaking

- Align the coordinated Core 0.17 package cut that retires endpoint attributes
  from Core's derive crate. Existing `ic_wire_endpoint::{query, update}` imports
  and attribute options remain unchanged; no wire or retained-byte migration.

### 🧹 Cleanup

- Own query/update expansion in `ic-wire-endpoint-macros`, versioned and
  released with Endpoint. Macro-only edits no longer require a Core release.
  Keep Platform's export/executor ownership and one production Serde/Postcard
  codec. Publish endpoint macros before the Endpoint facade.

## [0.6.1] - 2026-11-03

### 🔧 Changed

- Use Shared-verified release plans and sealed recovery evidence for the qualified
  Endpoint-only gate. Retain full validation for shared, Core, dependency or
  uncertain changes, all native CI jobs and existing source/publication checks.
  Old internal validation receipts require a fresh gate; handler APIs, frame
  bytes, admission and rollback semantics are unchanged.
  [#45](https://github.com/dragginzgame/ic-wire/issues/45).

### 🧪 Testing

- Compile the ignored endpoint cost caller against the canonical fallible
  PocketIC/Testkit client, including its test-only code path.

## [0.6.0] - 2026-11-03

### ⚠️ Breaking

- Adopt Core 0.16's public Error, Limits, codec and schema identities, with
  Runtime/macros/exact guard 0.19 and canonical Types 0.18. Align direct imports,
  Timers and other execution adapters; do not mix runtime generations. Platform
  retains exports/execution and the existing synchronous/async rollback rules.
- Account endpoints must coordinate `01 + 32 bytes` → `01 20 + 32 bytes`
  for present subaccounts; None stays `00`. Regenerate Uint8Array clients,
  requalify budgets and choose fresh contract keys. Default method-name IDs do
  not change with schema or package versions. Preserve original retry/hash bytes
  and handle unframed persisted data separately; there is no fallback reader.

### 🧪 Testing

- Align qualification callers with fallible Platform/PocketIC 0.19 and Testkit
  0.81. Treat only typed canister errors as trap/rejection evidence; fail client
  and transport errors instead of admitting false negative-test/cost samples.
- [Runtime comparison and installed evidence](../../docs/evidence/account-platform-cut.md)
  cover all 17 exports / 356 case-profile pairs with no instruction/cycle/page
  cost regressions. Total/code Wasm falls 224/222 bytes at opt-z and is unchanged
  at opt-3. Keep request/reply bytes, admission and rollback qualification distinct
  from the separate Account payload cut.

### 🔬 Research

- Automatic contract fingerprints and removal of manual macro IDs remain in
  [isolated preparation](../../docs/evidence/contract-artifact.md).
  This release retains the production macro API and one Serde/Postcard codec.

## [0.5.0] - 2026-11-02

### ⚠️ Breaking

- Require Rust 1.99.1 for Endpoint consumers, matching the workspace compiler
  policy. Upgrade older toolchains. Handler APIs, endpoint bytes, budgets and
  rollback semantics are unchanged.
- Select published Runtime/macros/exact guard 0.18.0. Align other executor
  suppliers and Core's Types 0.17 schemas; preserve accepted Wire frames,
  admission, exports and transaction behavior without a compatibility lane.
  Align packaged and repository migration guidance with this exact supplier cohort.

### 🧪 Testing

- Keep the full pinned history secret scan, allowing only the exact historical
  fingerprint for a benchmark fixture's public SHA-256. Independently reproduce
  the digest and retain the redacted scan; no path, rule or history is excluded.

## [0.4.0] - 2026-11-01

### ⚠️ Breaking

- Select published `ic-platform-runtime` 0.17.1 directly for typed entrypoints and execution.
  Endpoint and canister runtime identity no longer follow crypto facade releases.
  Align Core schemas with Types 0.16; handler syntax, Wire bytes and budgets are unchanged.
  Migrate Timers and other executor suppliers together; the exact singleton guard
  intentionally rejects the legacy embedded Platform 0.16 executor.

## [0.3.0] - 2026-11-01

### ⚠️ Breaking

- Adopt Platform 0.16.0 for typed entrypoints; align other application runtime
  suppliers, including timers, rather than mixing 0.15/0.16 executor owners.
  Keep canonical Types 0.15.1 and Core/derive identity, handler syntax, Wire bytes,
  budgets, malformed-input acceptance and transaction/rollback rules unchanged.

### 🩹 Fixed

- Use Platform's crate-scoped native entrypoint symbols, allowing overlapping
  method names from different canister libraries in one host process while
  retaining the exact Wasm export names. Qualify installed admission, async
  calls, reply failures and rollback with the supplying runtime.

## [0.2.1] - 2026-10-31

### ⬆️ Dependencies

- Require compatible Platform 0.15.2 and qualify installed endpoint behavior with
  Testkit 0.75.0. Keep canonical Types on 0.15.1. Handler API, frame bytes, resource
  budgets, malformed-input acceptance and transaction/rollback rules are unchanged.

## [0.2.0] - 2026-10-31

- Move this group's canonical ledger to `releases/endpoint/CHANGELOG.md` and
  require explicit endpoint selection for release/resume/publication, including
  recovery attempts. Retain the existing literal version, package path and
  runtime semantics; shared layout details are in the [core ledger](../core/CHANGELOG.md).

- Align Platform/Types on published 0.15 and the supplying Wire 0.13 core.
  This changes exposed Rust identities; align codec and typed/schema consumers.
  Handler syntax, frame bytes, budgets, admission and rollback rules are unchanged.

## [0.1.0]

- Extract typed endpoint integration from ic-wire, using Platform 0.14.0.
  Keep codec/schema/limits identities with ic-wire and Platform identities with
  ic-platform-types. Bytes, admission, budgets and rollback rules are unchanged.
