# Current handoff

Updated 2026-10-17. Main is released at 0.3.2 (5b1ddc6); first crates.io
publication was 0.3.0 for ic-wire-derive and ic-wire. Evaluation remains unpublished.
The 0.3.2 notes describe open TypeScript and CDK companion PRs, explicitly pending
integration. Their code is not yet on main; both updated PR heads passed Linux,
macOS ARM and macOS Intel CI. See [the notes](../../CHANGELOG.md#032) for PR links.

Pending 0.3.3 adds ownership recovery for Bytes and Text without copying or
allocating, matching List's existing ownership API. Wire bytes, admission limits
and package versions are unchanged.
Focused runtime tests verify buffer identity on recovery; library Clippy and
the Rust 1.91 Wasm check pass. The log is `target/033-ownership.log`.

## Evaluation cleanup

The current browser fixture uses one canister for signed query/update round trips,
typed application errors and malformed replies. Its cache, expiry/audience checks
and peer retry path are removed. Disposable fixture contracts change together;
frozen browser evidence retains its original identities. Runtime/Candid benchmarks
remain. Canic PR #591 now covers synchronous raw entrypoints, but explicitly
retains this repository’s async macro proof, so that fixture stays.
All three evaluation installers use the selected Testkit 0.51 InstallSpec fields.
The installed browser trial passes five Rust vectors, 68 truncated replies,
signed updates, application failures and certificate rejection. Retained runtime
and macro trials pass too, including the 16 macro rejection/rollback cases.
Evaluation-example Clippy and Rust 1.91 host/browser-Wasm checks pass. Fresh
logs are retained at `target/browser-trim-installed.log` and
`target/browser-trim-clippy.log`; frozen evidence remains unchanged.

## Publication and parser follow-up

The explicitly requested `make publish` completed successfully, including registry
availability confirmation for both 0.3.0 packages. The new Make target delegates
to Cargo’s ordered workspace publication; Git release delivery stays separate.
The publication log is retained at `target/publish-0.3.0.log`. No Git staging,
commit, tag, release or visibility change was performed by the publication task.

The working derive crate now selects Syn 3.0.9. Its evaluation-only adapter uses
Syn’s Safety enum and rejects unsupported function modifiers; an unsafe-handler
negative fixture checks admission. Syn 2 remains transitively through other macro
dependencies. This changes host parsing, not runtime wire bytes or Wasm behavior.
The runtime and derive package source still targets Rust 1.91. This follow-up was
released in Git at 0.3.1; 0.3.0 contains the previously qualified Syn 2 code.

Syn 3 qualification passes: 13 runtime tests, Rustdoc, three macro pipeline
tests, all 19 negative diagnostic cases on Rust 1.99 and 1.91, Clippy, Wasm MSRV,
88 Rust/TypeScript vectors and 17,664 differential mutations. The installed macro
trial passes query/sync/async exports, typed failures and 16 rejection/rollback
cases. Formatting, pins, documentation links and the disposable Make fixture pass.
No full CI or native macOS run was performed for this follow-up.

The selected IC Build 0.13.0 and Host Artifacts 0.25.0 updates also pass evaluation
example Clippy and Rust 1.91 checks without API adaptations. All three installed
trials pass with the new host graph: runtime comparisons, macro rejection/rollback
and signed browser calls. These dependencies remain dev-only. Logs are retained
at `target/031-host-qualification.log` and `target/031-installed.log`; fresh reports
and build identities remain under their existing target/browser artifact owners.
`make check-package` verifies both current source archives with Syn 3, including
the staged derive dependency (`target/031-package.log`). This check uploads nothing;
that check preceded the maintainer's Git release of 0.3.1.

## Latest cleanup

The historical installed tuple/authority/peer codec experiment and
`evaluate-canisters` are retired; frozen evidence remains reproducible from its
recorded checkout. Current comparisons retain the required strict Candid helpers.
IC Build 0.12 now builds all three installed trials and retains artifact records
through consumption; reports retain digests and Wasm identities. Source must stay
unchanged during each run; before/after checks do not detect reverted edits.

Durable receipt-policy scenarios are relocated into an isolated Relay contribution
([Relay #1 — Own durable receipt-policy qualification](https://github.com/dragginzgame/ic-relay/issues/1)).
The sibling working checkout remains untouched. IC Wire's `evaluate-replay` and
stable receipt layout are removed; its browser fixture uses bounded volatile state.
Historical adoption/browser reports keep their original identities. This is a
compatible contributor-tooling cleanup under pending 0.3.0; no public codec,
wire bytes, MSRV or package version changes. Relay integration remains owner-reviewed in [Relay PR #2](https://github.com/dragginzgame/ic-relay/pull/2).

All three installed trials pass through the shared build pipeline on Linux.
Focused canister tests/Clippy and Rust 1.91 host/Wasm checks pass; 12 codec
runtime tests, formatting, pins, documentation links and the disposable Make
fixture pass. Relay’s isolated probe passes nine native tests, focused Clippy,
Rust 1.91 host/Wasm checks and the installed receipt scenarios. No full CI or
native macOS execution was run. IC Wire staging, commits and versions are untouched.

## Implemented runtime

The `ic-wire` library now provides a no_std + alloc Postcard runtime,
with the host-side `ic-wire-derive` crate. Its normal dependency graph
excludes Candid, CDK and host tooling. Read [the runtime guide](../runtime.md)
for the maintained public API, wire rules, supported types and Canic adapter steps.

- `Wire` derives serialize records by explicit field IDs and enums by explicit
  variant IDs, independent of Rust declaration order. Generic/recursive derives
  and unbounded containers are excluded. IDs and method identity are consumer-owned.
- Bounded List, Bytes and UTF-8 Text compose with fixed-width integers, unit,
  bool, finite f32/f64 wrappers, Option, Result, fixed arrays and derived records/enums. Principal bytes
  use Bytes<29>; existing consumer principal validation stays at the adapter.
- Method binds request/response types, exported name, identity and per-direction
  limits. Schema maxima bound bytes, value heap plus scratch, nodes and depth
  before decoding. Input admission is conservative over the entire schema.
- Canonical envelopes include marker, method identity and direction. Strict decode
  rejects trailing/noncanonical bytes; immutable typed buffers retain exact bytes.
  There is no Candid mode or runtime-owned retry/auth/storage policy.

The historical [installed adoption proof](../runtime-adoption.md) used these same production
functions in a nested query, a protected command and two real peer deliveries.
It checks retained success/failure responses, conflicting requests, trap rollback,
capacity refusal and same-contract upgrade replay. The private 0.1.4 tuple replay
probe is retired from current source; its historical evidence remains labeled.
This qualifies initial controlled Rust endpoint integration, not full Canic rollout.

## Scope and remaining work

Canic owns query/update macros, dispatch, auth, replay policy, transport callbacks,
metadata and exported-method validation. The [consumer assessment](../consumer-adoption.md)
traces those committed source owners. The ordinary sibling working checkouts remain unmodified.
No publication, commit, push, release or version change is part of ordinary work.
The runtime and derive packages are published; evaluation remains `publish = false`.
Further publication requires explicit maintainer authorization.

The optional host TypeScript generator now projects the runtime envelope from
Wire metadata. `make check-typescript` passes 83 Rust/TypeScript vectors in both
directions plus malformed-input/value checks, using Node 24.21.0 and TypeScript
7.0.2. Rust runtime tests (11), doctests (6), clippy and host/Wasm MSRV checks
qualify this addition. Saturated resource maxima now always fail admission.
This does not measure performance or JavaScript heap usage.

The [generated browser fixture](../browser-interop.md) now passes installed
Chromium queries, protected signed updates, exact success/failure replay and two
peer deliveries with one effect. Rust vectors, 68 truncated replies, auth/input
rejections and wrong-root certificate rejection are recorded in
[the saved report](../evidence/runtime-browser.json). Hand-written browser guards
and postcard-bindgen are retired from current source. Historical JSON evidence
remains unchanged. The public deterministic seed is only a disposable test identity. Focused
fixture clippy, six native evaluation tests, the Rust 1.91 host fixture build,
TypeScript checks, pins, documentation links and local tooling checks pass.

The production diagnostic gate covers 11 invalid derive/adapter shapes. The new
[local macro proof](../macro-adoption.md) additionally tests actual evaluation-only
query/update attributes here while Canic changes independently. Three native
pipeline tests, seven attribute diagnostic cases and installed Wasm checks cover
query/sync/async exports, early rejection, typed errors, unit replies and response
encoding failure rollback. Repeated unreceipted commands still produce new effects.
The attribute diagnostics pass on Rust 1.99 and 1.91; the feature also compiles
for Wasm at the Rust 1.91 floor. Clippy, ShellCheck and focused tooling checks pass.
This is not Canic macro integration, delegated auth or external-await recovery.

FiniteF32/FiniteF64 now preserve finite IEEE bits, including signed zero and
subnormals, and reject NaN/infinity. TypeScript rejects silent f32 rounding.
The conformance corpus includes bounded heterogeneous SQL-shaped pages, not actual
IcyDB contracts. String/Vec still convert through existing bounded wrappers.
Recursive values remain unsupported; rarity does not remove their admission needs.
The [current-runtime comparison](../runtime-comparison.md) now includes matched
before/after scratch optimization evidence. Encoding initializes emitted bytes
only; canonical decode compares against input without a second output allocation.
The same mixed schema now uses 1.75–21.17× fewer codec instructions than 0.2.1
and 3.04–52.61× fewer than strict Candid. Internal Wasm grows by 4,722 bytes;
management-enabled Wasm grows by 3,635 bytes. All request/reply hashes match.
Admission remains conservative; bytes, typed failures and public APIs are unchanged.
The expanded matrix now also covers standalone u64 endpoints and heterogeneous
SQL-shaped pages: 43.79–61.64× and 6.64–33.45× fewer codec instructions than strict
Candid respectively, in internal-only builds. The eight-row page is 914 versus
987 bytes. These are synthetic local contracts, not actual IcyDB SQL adoption.
Historical runtime/browser/macro evidence remains at its original source hashes.

Browser login/delegation, composite-query behavior and external-effect recovery
remain unqualified. SDK/CDK transports allocate raw replies before codec checks;
Canic must enforce any pre-copy allowance in its transport/callback layer.

Port the qualified local expansion pattern into one controlled Canic endpoint when sibling source
changes are authorized, using the concrete runtime guide and shared contracts.
Keep full auth/hash/client conversion coordinated; do not add compatibility lanes.
An installed IcyDB schema comparison remains optional and separate. Database rows,
ordered keys, journals and authenticated cursor formats stay outside scope.

## Consumer qualification

[Source-matched IcyDB fixtures](../consumer-fixtures.md) now cover native SQL text,
Count/Grouped successes and public-error layouts, bounded conversions and display-string
fidelity. These are a source-hashed subset of an actively changing sibling, not
full endpoint adoption. Its dynamic native Payload adapter remains opaque to
TypeScript generation; recursive/native browser projections remain unqualified.
Exact request/response length helpers include the header without allocating output
storage. The offline viewer decodes using trusted generated contract modules.
The current conformance gate passes 87 vectors and 17,320 Rust/TypeScript mutation
comparisons, with canonical re-encoding of accepted inputs. This is deterministic
mutation testing. A separate `make fuzz-runtime` coverage-guided harness now passes
100,000 inputs with address/leak sanitizers on Linux. It covers malformed frames,
bounded payloads, canonical re-encoding and exact length helpers. See
[development](../development.md#coverage-guided-runtime-fuzzing) for prepared-tool
requirements, corpus/replay handling and qualification limits. Sustained campaigns
and native macOS fuzz execution remain unqualified. Runtime tests, clippy, Wasm MSRV,
the disposable formatting fixture and installed macro rollback checks pass.
The IcyDB response-encoding reject path needs consumer review for mutating calls;
the local macro proof retains trap-based rollback. Sibling files remain untouched.

## Evidence and tooling

The four-codec screening executable and `evaluate-codecs` are retired; its saved
evidence remains historical. Production conformance retains nested principal-byte
length cases and now passes 88 vectors and 17,664 mutations. The runtime crate no
longer has Candid, Ciborium, ic_principal or SHA-256 dev dependencies. Candid remains
in the separate comparison canister. Five evaluation hosts use published Testkit
0.50 startup ownership and installation, preserving fresh topology and funding.
Explicit UI modules replace source-rewriting diagnostic scripts; all 18 negative
cases and valid controls pass on Rust 1.99 and 1.91. Host dependencies stay dev-only.
Before this cleanup, all five installed experiments passed with the managed lifecycle: current/historical
codec comparisons, macro rollback, replay/peer delivery and signed browser calls.
Fresh reports stay under target/ and browser artifacts; frozen evidence is unchanged.
Runtime tests, clippy, host/Wasm MSRV and tooling checks pass on Linux. Native macOS
qualification remains with the configured hosted matrix.

The private typed-journey implementation and `evaluate-journey` target are retired.
Its retry assertion now uses production RequestBytes and typed application errors;
existing runtime tests own identity, field-order and malformed-input coverage.
Three evaluation hosts now use published `ic-host-artifacts` 0.24.0 for bounded
Wasm facts; only the host dev-dependency graph changes. Historical evidence stays
at its original source identities. Canic macro-proof relocation is coordinated on
[Canic #590](https://github.com/dragginzgame/canic/issues/590#issuecomment-6231405790).
Its [current PR](https://github.com/dragginzgame/canic/pull/591) implements synchronous
entrypoints and installed rollback checks, but explicitly leaves the asynchronous
macro proof here. The local fixture remains until that coverage has an owner.
The cleanup passes 13 runtime tests with TypeScript, evaluation-example clippy
and Rust 1.91 checks, local tooling checks, and installed runtime/macro proofs.
The shared inspector remains dev-only; no codec or canister dependency changes.

The expanded current-runtime comparison passes 144 measured calls, 248 rejection
checks, matched export checks and two real management calls. New fixtures pass
clippy with both codecs, Rust 1.91 host/Wasm checks and the disposable Make fixture. The optimized runtime
passes 12 runtime tests with TypeScript enabled, six doctests, all 83 TypeScript
vectors, focused clippy and the Rust 1.91 Wasm check.
The installed replay/rollback trial also passes with the optimized codec; its
fresh report is retained in `target/runtime-adoption-evidence/report.json`.
The disposable Make/formatting fixture includes the benchmark modules and shapes;
`make check-tooling` qualifies the complete fixture after the missing-file fix.

Historical [codec comparison](../canister-comparison.md),
[nested authority](../nested-authority.md) and [replay](../command-replay.md) reports
retain their original source/build identities. They do not measure the new runtime
header or generic bounds overhead. No production Canic savings are established.
The runtime adoption report is correctness evidence, not a performance comparison.

Shared Tooling 0.15.4 remains pinned at
`879eb2b021522252a67e908dbc217a0f4efb2a6f`. See [development](../development.md)
for setup, focused checks and maintainer-operated Git releases, separate from
package publication. The disposable patch-release test now owns its synthetic
changelog, so minor-release notes do not block its recovery scenarios. Hosted macOS qualification of the earlier temporary-path
fix remains unverified here. No broad local CI or sibling suite is required or run
for this batch; focused runtime/adapter results are recorded in the adoption proof.
