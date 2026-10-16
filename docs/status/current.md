# Current handoff

Updated 2026-10-16. Workspace versions remain 0.1.6 after the maintainer release;
the pending batch is documented under 0.2.0 because the public Schema enum gains
float metadata and becomes non-exhaustive. Package versions are unchanged.

## Implemented runtime

The unpublished `ic-wire` library now provides a no_std + alloc Postcard runtime,
with the unpublished host-side `ic-wire-derive` crate. Its normal dependency graph
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

The [installed adoption proof](../runtime-adoption.md) uses these same production
functions in a nested query, a protected command and two real peer deliveries.
It checks retained success/failure responses, conflicting requests, trap rollback,
capacity refusal and same-contract upgrade replay. The private 0.1.4 tuple replay
probe is retired from current source; its historical evidence remains labeled.
This qualifies initial controlled Rust endpoint integration, not full Canic rollout.

## Scope and remaining work

Canic owns query/update macros, dispatch, auth, replay policy, transport callbacks,
metadata and exported-method validation. The [consumer assessment](../consumer-adoption.md)
traces those committed source owners. Siblings remain read-only and unmodified.
No publication, commit, push, release or version change is part of ordinary work.
All packages remain `publish = false`. No registry name is reserved.

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
Current-runtime performance comparisons and full IcyDB SQL integration remain
unmeasured/unqualified. Existing runtime/browser/macro evidence stays historical
at its recorded source hashes; this float addition is native/Node conformance.

Browser login/delegation, composite-query behavior and external-effect recovery
remain unqualified. SDK/CDK transports allocate raw replies before codec checks;
Canic must enforce any pre-copy allowance in its transport/callback layer.

Port the qualified local expansion pattern into one controlled Canic endpoint when sibling source
changes are authorized, using the concrete runtime guide and shared contracts.
Keep full auth/hash/client conversion coordinated; do not add compatibility lanes.
An installed IcyDB schema comparison remains optional and separate. Database rows,
ordered keys, journals and authenticated cursor formats stay outside scope.

## Evidence and tooling

Historical [codec comparison](../canister-comparison.md),
[nested authority](../nested-authority.md) and [replay](../command-replay.md) reports
retain their original source/build identities. They do not measure the new runtime
header or generic bounds overhead. No production Canic savings are established.
The runtime adoption report is correctness evidence, not a performance comparison.

Shared Tooling 0.15.4 remains pinned at
`879eb2b021522252a67e908dbc217a0f4efb2a6f`. See [development](../development.md)
for setup, focused checks and maintainer-operated Git releases, separate from
package publication. The disposable patch-release test now owns its synthetic
changelog, so the real pending 0.2.0 notes do not block its recovery scenarios. Hosted macOS qualification of the earlier temporary-path
fix remains unverified here. No broad local CI or sibling suite is required or run
for this batch; focused runtime/adapter results are recorded in the adoption proof.
