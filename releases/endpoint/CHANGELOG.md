# Endpoint group changelog

Covers ic-wire-endpoint. Its independent series begins at 0.1.0; pre-extraction
repository releases remain in [shared history](../history.md). The published
0.1.0 heading was undated and is preserved as recorded.

## [0.4.0]

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
