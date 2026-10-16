# Current handoff

Updated 2026-10-16. The maintainer released/pushed 0.1.4, then requested a
Postcard-only path until Canic can adopt IC Wire. Workspace versions remain
0.1.4; the pending compatible addition is documented under 0.1.5.

## Implemented runtime

The unpublished `ic-wire` library now provides a no_std + alloc Postcard runtime,
with the unpublished host-side `ic-wire-derive` crate. Its normal dependency graph
excludes Candid, CDK and host tooling. Read [the runtime guide](../runtime.md)
for the maintained public API, wire rules, supported types and Canic adapter steps.

- `Wire` derives serialize records by explicit field IDs and enums by explicit
  variant IDs, independent of Rust declaration order. Generic/recursive derives
  and unbounded containers are excluded. IDs and method identity are consumer-owned.
- Bounded List, Bytes and UTF-8 Text compose with fixed-width integers, unit,
  bool, Option, Result, fixed arrays and derived records/enums. Principal bytes
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

General TypeScript generation for the runtime envelope, browser auth/delegation,
composite-query behavior and external-effect recovery remain unqualified. Existing
[browser](../browser-interop.md) and [peer](../peer-interop.md) trials use historical
private layouts and are not runtime clients. Do not present their success as a
browser rollout proof. The current CDK peer transport allocates its raw reply;
Canic must enforce pre-copy reply limits in its own transport/callback layer.

Next, integrate one controlled Rust-to-Rust Canic endpoint when sibling source
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
package publication. Hosted macOS qualification of the earlier temporary-path
fix remains unverified here. No broad local CI or sibling suite is required or run
for this batch; focused runtime/adapter results are recorded in the adoption proof.
