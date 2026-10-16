# Changelog

## [0.1.2]

- Enable maintainer-operated Git releases through Shared Tooling, with coordinated
  Cargo metadata, retained validation evidence and interrupted-delivery checks.
  Keep package publication disabled and separate from version/tag/push delivery.

- Select Postcard for controlled inter-canister calls and owned user endpoints.
  Add a private browser experiment with Rust-generated bindings/vectors, a
  bounded TypeScript client and signed Chromium/PocketIC updates. Verify u64
  precision, malformed input and certificate rejection; public API remains open.

- Add matched installed Candid/Postcard canister experiments with optional real
  management traffic. Record payloads, complete Wasm/code-section sizes and IC
  instruction counters with reproducible artifacts and rejection checks; keep
  the production library independent of the evaluation dependencies.
- Generate the native experiment's scalar record serialization from one explicit
  wire-order declaration, preserving its existing request bytes. Type retained
  request/reply buffers and add compile-fail checks for invalid callers and
  unsupported fields. This remains private evaluation syntax, not a runtime API.
- Synchronize the lockfile's local package entry with the existing 0.1.1 manifest
  so locked builds succeed; no dependency versions change.

## [0.1.1]

- Add a native typed request/reply experiment with explicit wire layout,
  contract/direction checks, bounded output and retained retry bytes. This remains
  an evaluation example, with no selected runtime codec or IC endpoint adapter.
- Fix the local Make fixture's path comparison for temporary directories with
  trailing slashes or symlink aliases, as encountered by macOS CI.

## [0.1.0]

- Initialize the maintainer-authorized private `dragginzgame/ic-wire` GitHub
  repository and activate the local formatting hook; package publication remains
  disabled.
- Add an evaluation-only comparison of Candid, Postcard, CBOR and an independently
  authored fixed-layout comparator across seven synthetic fixtures, including an
  IcyDB-shaped schema artifact. Record payload sizes/hashes, strict round trips,
  malformed-input tests and the positional field-identity limitation. Keep the
  runtime library dependency-free; no codec selection or adoption is implied.
- Establish an unpublished, dependency-free Rust workspace and no_std scaffold.
- Record scope, proposed endpoint ergonomics, codec evaluation criteria and
  preliminary Canic code-size attribution. No codec or protocol is implemented.
- Adopt reviewed Shared Tooling 0.15.4 for explicit developer tool setup,
  formatting, Git hooks, snapshot verification and workspace counts. Add local
  Make checks and Linux/macOS CI; release delivery remains disabled.
- Evaluate optional codec sharing with IcyDB. Identify reusable bounded byte
  handling and preserve database-specific format requirements; sharing remains
  a candidate to measure, with no codec or adoption selected.
