# IC Wire scope and decisions

## Goal

Rust developers should define ordinary shared types and typed endpoints without
writing serialization plumbing. Internal Canic traffic should avoid Candid type
tables and generic decoding while retaining bounded input handling, exact retries
and deterministic bytes wherever values are hashed or signed.

The maintainer approved setting up this investigation on 2026-10-16. The maintainer selected 0.3.0 for the first registry publication on 2026-10-17.
Preparation does not execute publication. Success requires developer
ease, documentation and a meaningful measured advantage, not merely another codec.

## Ownership

| Owner | Responsibility |
| --- | --- |
| IC Wire | One binary encoding, bounded encode/decode, typed errors, canonical rules and shared test vectors |
| Consumer contract crate | Requests, responses, field/variant identities and semantic validation |
| Canic | Authentication, endpoint macros, runtime dispatch and fleet policy |
| IC Relay | Delivery envelopes, queues, addressing, ACKs and exact-byte retry |
| IcyDB | Database semantics, persistence and ordered-key formats |
| Optional future host tooling | Schema inspection and client generation, only after demonstrated need |

IC Host Tooling's current documented boundary is execution outside canisters.
The runtime codec therefore belongs here, not in ic-host-tools or ic-host-idl.
No host companion, generic plugin architecture or schema language is required
for the initial evaluation. No new principal identity type should be invented;
evaluate the existing standalone principal representation without importing Candid.

## Initial protocol requirements and current resolution

- Define integer widths, signed encoding, lengths, endianness, booleans, strings,
  principal bytes, options, arrays, sequences, records and enums exactly.
- Select explicit stable identities or a documented positional schema discipline;
  do not silently use Rust enum order as durable wire authority.
- Bound input bytes, output bytes, nesting, elements and allocation. A short input
  with malicious lengths must not trigger unbounded work or allocation.
- Reject truncation, invalid tags, invalid UTF-8 and trailing data with typed errors.
- Define canonical output and whether noncanonical input is rejected. Hash/signature
  paths must never depend on map iteration order or ambiguous encodings.
- Define the method binding and request/response identity needed to prevent decoding
  a response under an unrelated contract. Keep application rejection distinct from
  transport and codec errors.
- Specify empty arguments and empty results explicitly. Raw calls must not inherit
  the existing Canic transport's Candid empty-argument default.
- Decide unknown-field and incompatible-schema behavior explicitly. Initial scope
  is coordinated Rust peers, not Candid's cross-language subtyping guarantees.

The [runtime contract](runtime.md) now resolves the Rust subset with explicit
field/variant IDs, finite schema budgets, method/direction identity and canonical
Postcard envelopes. It has no Candid mode. General browser generation and
consumer-specific hash/auth policies remain outside that qualified subset.

The maintainer selected Postcard for controlled inter-canister calls and owned
user endpoints after the payload and installed-canister experiments. This selects
the encoding engine; the broader consumer integration still requires
qualification. The [browser trial](browser-interop.md) covers one scalar/blob
update endpoint, with generated bindings and a bounded client wrapper.

The [IcyDB sharing assessment](shared-codec-evaluation.md) supports evaluating a
shared message/value codec, while keeping adoption optional. Similar byte helpers
do not establish that RPCs, persisted rows and ordered keys should share one
format. If a shared implementation wins, IC Wire is its owner; IcyDB's accepted
schema authority, storage layouts, authentication and recovery remain local.

## Canic adoption boundary

A future Canic batch must trace endpoint macros, RPC requests/replies, auth argument
extraction, inspection hooks, installation arguments, host clients, fixtures and
interface validation tooling. Moving endpoint bytes alone is insufficient.
Candid also supplies replay response bytes and attestation, provisioning, runtime
and draining-authority hash inputs. Those require deliberate canonical replacement.

Candid stays in adapters for external management, ledger, registry and other
Candid-defined contracts. Upstream Auth/Blob interfaces retain their own owners.
The CDK and Auth protocol types currently retain Candid dependencies; removing
imports does not prove the linked codec is gone. Public application endpoints
can use the selected wire format, but generic Candid clients will not understand
those endpoints. Endpoint export validation must still verify actual callable
methods even when an internal .did contract is removed.

This investigation does not authorize editing Canic, IcyDB, Relay or upstream
contracts. The eventual coordinated Canic cut follows its reinstall-only policy;
no old-format migration or dual protocol lane is proposed. Within a selected
release, replay, interruption recovery and byte-for-byte retries remain required.
