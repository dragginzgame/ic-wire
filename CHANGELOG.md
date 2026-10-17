# Changelog

## [0.2.3]

- Share the TypeScript codec with native Serde consumers through concrete recursive metadata, tuples, transparent newtypes, explicit traversal limits and caller-owned scalar validation. Finite generated contracts use the same encoder, decoder and envelope. Rust derives still require finite metadata.

## [0.2.2] - 2026-10-16

- Reduce codec instruction costs by writing only emitted bytes and checking
  canonical serialization directly against input. Preserve wire bytes, admission
  limits and typed failures. The [matched measurements](docs/runtime-comparison.md)
  record instruction savings alongside the modest increase in linked Wasm.

## [0.2.1] - 2026-10-16

- Include the runtime benchmark modules in the disposable formatting-hook fixture,
  fixing `check-tooling` failures caused by missing source files.

- Add a reproducible installed comparison of the production runtime against
  strict Candid, covering bounded blobs, UTF-8 text and finite doubles, with
  separate internal-only and management-enabled Wasm measurements. See the
  [measured results and limitations](docs/runtime-comparison.md).

## [0.2.0] - 2026-10-16

- Isolate the patch-release test changelog from the real pending release, so
  minor/major release notes do not prevent validation/recovery checks from running.

- Add finite f32/f64 wire wrappers and matching TypeScript number projections,
  preserving finite bits while rejecting NaN and infinity. Qualify scalar edges
  and bounded SQL-shaped pages with Rust/TypeScript vectors. Recursion remains
  unsupported and sibling/storage contracts are unchanged.
- Add float metadata and make the public Schema enum non-exhaustive. Downstream
  schema matches must include a fallback; this source API change selects a minor
  release. Existing wire layouts remain unchanged. Workspace versions stay at
  0.1.6 until maintainer release preparation.

- Add evaluation-only Postcard query/update attributes and a local Canic-shaped
  fixture, without changing siblings or the default runtime API. Check real Wasm
  exports, sync/async handler types, stage ordering, typed errors and rollback
  when response encoding fails after a local state change. Keep macro/auth/retry
  policy consumer-owned and preserve prior evidence separately.

## [0.1.6] - 2026-10-16

- Move the compile-fail gate from the retired private prototype to production
  Wire/Method contracts, including synchronous and async consumer-shaped handler
  bindings. Keep Canic macro ownership and runtime policy with the consumer.

- Generate typed, bounded TypeScript codecs from the same explicit Rust Wire
  metadata and Method envelope. Add cross-language scalar/composition vectors
  and malformed-input checks. Keep generation optional and transport/auth policy
  consumer-owned. Qualify generated clients in Chromium against installed nested
  queries, protected updates, retained success/failure replies and peer calls.
  Retire the private browser guards and postcard-bindgen dependency; preserve
  historical evidence. Login/delegation and production rollout remain unqualified.
- Reject saturated schema resource maxima even under maximum caller allowances.
  Preserve existing wire bytes and keep workspace versions unchanged.

## [0.1.5] - 2026-10-16

- Add the unpublished Postcard-only Rust runtime: explicit field/variant derives,
  bounded collections/text/bytes, finite schema budgets, canonical typed method
  envelopes and immutable retry bytes. Keep Candid and CDK out of its dependency
  graph. Document the concrete Canic adapter contract and unsupported types.
- Qualify the runtime through installed nested queries, state-changing updates,
  two inter-canister deliveries, exact replay and same-contract upgrade recovery.
  Retire the private tuple replay fixture in current source; fresh fixture state
  uses a distinct contract identity. Preserve historical measurements separately.


## [0.1.4] - 2026-10-16

- Add a Postcard-only installed protected-command experiment with bounded stable
  receipts. Verify direct-caller authorization, exact replay, conflicting request
  rejection, atomic trap rollback, same-contract upgrade recovery and capacity
  refusal. Keep application failures distinct and retain their response bytes.
  This is evaluation evidence, not a production Canic replay/auth adapter.

## [0.1.3] - 2026-10-16

- Exercise the same Postcard request/reply contract through Chromium and two
  installed canisters. Validate peer replies, retain exact repeated bytes, verify
  the recipient caller and reject invalid dispatch/configuration attempts.
  This remains a stateless transport trial, not durable replay or a Canic adapter.

- Map Canic and sibling adoption boundaries against committed source snapshots.
  Separate a bounded endpoint trial from framework-wide RPC, auth, replay, hash
  and client changes; keep storage formats and sibling source edits out of scope.
  Add concrete consumer implementation/retirement steps, readiness checks and
  supporting-library boundaries, including cases requiring no codec dependency.

- Extend the installed Candid/Postcard comparison with bounded nested authority
  trees, optional expiries and maximum collection fixtures. Check list limits
  before element decoding, retain exact request/reply evidence and report new
  matched Wasm/instruction measurements without changing production APIs.

## [0.1.2] - 2026-10-16

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
