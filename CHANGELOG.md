# Changelog

## [0.4.1]

- Share TypeScript type rendering between finite schemas and consumer-supplied
  native registries. Host generators can emit typed method modules while keeping
  reflection, endpoint contracts and scalar checks at their consumer.

- Replace the custom Dragginz license with the standard MIT License for all
  workspace packages, including published runtime and derive crates.

- Keep owned codecs and endpoint payloads IC Wire-only. Retire Candid comparison
  code, the management probe and `evaluate-runtime`; preserve recorded measurements.
  Remove direct Candid dependencies, retaining browser and installed Wire tests.
  Suppress unused Candid service metadata without changing callable Wasm exports.
  SDK/CDK/PocketIC still contain Candid transitively in the private test harness.
- Adopt published IC Platform 0.1.9 for Principals, raw argument-size and
  stable-page checks, byte-exact traps, replies and rejects, removing the direct
  ic0 dependency. Preserve reply append/finalization order and UTF-8 reject bytes.
  Direct dependency checks now run in CI.
  CDK entrypoints/system helpers remain an evaluation-only exception under
  [#1 — Support the CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1).

- Adopt Shared Tooling 0.24.0 with dependency-role labels and matching grouped
  formatting and release metadata checks. Dependency selections are unchanged;
  role labels do not provide automated deployment exclusion enforcement.
- Isolate production codec diagnostics from the extracted endpoint-adapter fixture.
  Keep compiler diagnostics at IC Build and include adapter qualification in CI.
- Correct current ownership guidance and archive superseded migration handoffs;
  preserve historical reports without presenting retired replay or codec-selection
  experiments as active workflows. Runtime APIs and wire bytes are unchanged.

## [0.4.0] - 2026-10-17

- Adopt Shared Tooling 0.23.0, retaining the unmerged release-receipt helper under
  its own pinned snapshot. Select IC setup/checks explicitly and adopt ICP CLI
  1.7.0. Evaluation hosts use IC Host Artifacts/FS 0.27.1; runtime dependencies
  and wire contracts are unchanged.

- **Breaking:** remove the evaluation-only `adapter-fixture` Cargo feature and
  query/update attributes from `ic-wire-derive`. Workspace qualification now uses
  the unpublished `ic-wire-adapter-fixture` package; external callers must supply
  their own endpoint adapter. Canic remains the production macro owner.
- Stream deterministic conformance mutations from JSON Lines and share enum
  expansion across unit/newtype variants. Wire bytes, schemas and runtime APIs
  stay unchanged; no retained-data conversion or canister reset is required.

- Share the benchmark sampling/rejection path and native frame-budget calculation,
  preserving typed replies, admission order and canonical bytes.
- Consolidate fresh evaluation report identities under `provenance`, with bounded
  streaming file hashes and one compiler/server probe in the build driver. Browser
  reports become final only after retained-build verification. Historical evidence
  keeps its original format. Evaluation hosts accommodate Testkit 0.52 startup.

- Move browser fixture lifecycle to IC Testkit, compiler diagnostic mechanics to
  IC Build, and atomic release receipts to Shared Tooling. IC Wire keeps its
  test contracts, compiler recipes and release policy; upstream contributions
  are selected by immutable Git revisions.
- Bound browser fixture shutdown through forced termination and require every
  expected compiler diagnostic after a passing control. Release receipt checks
  reject symlinks and exact-byte tampering while preserving prior evidence on
  failed writes. Runtime APIs, wire bytes and package versions are unchanged.

## [0.3.5] - 2026-10-17

- Retire copied IcyDB SQL contract projections; actual endpoint and conversion
  qualification stays in [IcyDB #372](https://github.com/dragginzgame/icydb/issues/372).
  Retain generic codec and cross-language conformance coverage.
- Consolidate native scalar forwarding while preserving admission accounting,
  allocation-sensitive handlers and canonical wire bytes.
- Use IC Build's retained evidence export for evaluation artifacts, replacing
  local copy/hash/provenance plumbing. Select an immutable upstream revision
  pending [IC Build #58](https://github.com/dragginzgame/ic-build/issues/58)
  delivery; evaluations retain recipe, source custody and measurement ownership.

## [0.3.4] - 2026-10-17

- Add bounded native Serde payload/frame APIs with borrowed encoding, exact sizing
  and canonical decoding, sharing the existing Postcard envelope and output logic.
  Consumers retain explicit limits and schema/domain ownership.
  [IC Wire #5 — Native IcyDB admission](https://github.com/dragginzgame/ic-wire/issues/5).
- Include the shared native TypeScript codec from
  [PR #1](https://github.com/dragginzgame/ic-wire/pull/1), with recursive metadata,
  tuples, newtypes and sparse-array refusal. Support strict indexed-access checks
  without consumer patches ([#3](https://github.com/dragginzgame/ic-wire/issues/3)).
- Preserve TypeScript scalar checks through named aliases and bound text scratch
  buffers by the string's UTF-8 maximum instead of the remaining frame allowance.
  Concrete values no longer enumerate the type registry during alias resolution.

## [0.3.3] - 2026-10-17

- Simplify the browser evaluation to one canister and generated-codec transport
  checks. Remove fixture receipt caching, expiry/audience policy and peer retries;
  retain runtime benchmarks and the asynchronous macro proof. Adapt evaluation
  installation to the selected Testkit 0.51 API.

- Add `Bytes::into_vec` and `Text::into_string` so consumers can recover owned
  buffers without copying or allocating. Wire bytes and admission limits are unchanged.

## [0.3.2] - 2026-10-17

Pending PR integration; these changes are not yet on main.

- Share the TypeScript codec with native recursive contracts through concrete
  metadata, tuples, newtypes and explicit traversal limits. Reject sparse arrays
  before encoding and retain consumer-owned scalar validation.
  [PR #1 — Shared native TypeScript codec](https://github.com/dragginzgame/ic-wire/pull/1).
- Add an unpublished CDK endpoint companion that authorizes before argument
  sizing/copying and traps on response-encoding failure to roll back current-message
  writes. Qualify the installed endpoint through the shared build pipeline with
  retained build identities and stable-memory rollback evidence.
  [PR #2 — Bounded CDK endpoints](https://github.com/dragginzgame/ic-wire/pull/2).

## [0.3.1] - 2026-10-17

- Upgrade the derive parser to Syn 3 while retaining the Rust 1.91 floor and
  existing wire format. Preserve safe-handler admission in the macro fixture.
- Add `make publish` for explicit crates.io delivery of the runtime and derive
  packages, separate from Git release commands.
- Qualify IC Build 0.13 and Host Artifacts 0.25 in the evaluation hosts, including
  installed runtime, macro and browser trials. Keep these dependencies dev-only.

## [0.3.0] - 2026-10-17

- Prepare the first crates.io publication of `ic-wire` and `ic-wire-derive`, with
  self-contained runtime documentation, registry dependency requirements and
  package metadata. Keep evaluation tooling unpublished. Publication is separate
  from the maintainer-operated Git release.

- Retire historical canister benchmarks and use IC Build artifact records for
  maintained installed evaluations. Preserve saved measurement provenance.
- Relocate durable receipt-policy qualification to Relay; keep IC Wire’s browser
  fixture volatile and retire `evaluate-replay`. See [Relay #1 — Own durable
  receipt-policy qualification](https://github.com/dragginzgame/ic-relay/issues/1).

- Retire the four-codec screen and its private fixed/CBOR implementations; retain
  historical evidence and principal-byte nesting in production conformance.
  Remove screen-only runtime dev dependencies. Adopt Testkit 0.50 host startup
  and installation, and replace source-mutating diagnostic scripts with explicit
  pass/fail Rust fixtures and one locked offline runner.

- Retire the private typed-journey codec and Make target; qualify exact retry
  bytes and transport/application error separation against the production runtime.
  Use published Host artifact inspection in all three Wasm-reporting fixtures,
  with explicit artifact bounds and unchanged local export/measurement policy.

- Add an offline, explicitly invoked libFuzzer runtime harness and Make target,
  with bounded nested/float inputs, canonical round-trip assertions and sanitizer
  checks. Keep fuzz dependencies dev-only and instrumented builds isolated.

- Add exact request/reply frame-length helpers without output allocation,
  source-matched bounded IcyDB success fixtures and Rust/TypeScript differential
  mutation checks. Add offline inspection through generated contract modules;
  see [consumer qualification](docs/consumer-fixtures.md) for scope and limits.

- Extend installed codec measurements with tightly bounded scalar endpoints and
  heterogeneous SQL-shaped pages, including row/cell overflow rejection. Preserve
  prior reports and record the expanded matched Wasm and instruction matrix in
  [the runtime comparison](docs/runtime-comparison.md).

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
