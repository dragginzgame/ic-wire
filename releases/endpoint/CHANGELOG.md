# Endpoint group changelog

Covers ic-wire-endpoint. Its independent series begins at 0.1.0; pre-extraction
repository releases remain in [shared history](../history.md). The published
0.1.0 heading was undated and is preserved as recorded.

## [0.2.0]

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
