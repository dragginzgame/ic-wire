# Consumer adoption assessment

Evaluated 2026-10-16 against committed source exports. A single Canic endpoint is
an attainable integration trial once IC Wire has a usable runtime API. Replacing
Canic-owned RPC throughout the framework is a substantial coordinated change.
The difficult work is preserving contracts around the encoded bytes: authorization,
replay, hashing, client generation and deployment. Postcard itself is already
working in the local installed/browser experiments.

These are relative engineering estimates, not measured implementation times.
IC Wire's production library still exposes no codec API. Its explicit-layout
prototype supports scalar records; nested fixtures and browser guards are still
hand-maintained. Adoption cannot yet be reduced to adding a dependency and derives.

## Canic: existing integration points

The committed baseline is `644d4b13012b395d6211a6907e2e6aafdf391db9`. Canic had
419 dirty status entries when inspected; the analysis below uses the committed
export, not those in-flight changes. Recheck these owners before implementing
anything in Canic. No sibling source was changed or compiled.

| Owner | Existing behavior | Required adoption work |
| --- | --- | --- |
| [Endpoint expansion][endpoint] and [raw adapter][adapter] | Startup/access/dispatch stages already exist. Raw entrypoints bound arguments, but decode/reply paths and declaration attributes still use Candid. | Keep those stages; route owned method arguments and replies through the selected bounded wire contract. Preserve query/composite/update behavior and errors. Custom argument decoding alone is insufficient because reply encoding also changes. |
| [Ops calls][calls] and [infra calls][infra] | Raw request bytes already pass through. Infra exposes reply bytes; the higher ops result exposes Candid helpers. Empty calls default to Candid `DIDL` bytes. | Add a typed wire projection at the appropriate call owner, including reply limits and metrics. Explicitly encode empty wire requests; preserve cycle attachment, wait mode and transport failures. |
| [Typed RPC][rpc] | Response bounds require CandidType; capability envelopes and Result replies use Candid helpers. | Bind each request to the right response and contract/direction identity; preserve application errors separately from transport/codec failures. Use exact retained request bytes for retries. |
| [Auth extraction][auth] | Delegated token extraction independently decodes the first Candid ingress argument, deliberately allowing large later payloads. | Define bounded proof access for the wire envelope and preserve caller/audience/expiry checks. Do not decode an unbounded body just to obtain its proof, or leave the old first-argument parser on a wire endpoint. |
| [Replay][replay] | Cached response bytes include Candid and a compact special case for cycle transfers, with schema/receipt checks. | Deliberately replace owned response encoding while preserving same-release byte identity, reservation, commit and recovery semantics. A stateless echo test does not qualify replay. |
| [Hashing][hash] | Root-draining reservation hashes include a domain plus Candid-encoded content with the self-hash cleared. Other signed/hashed paths need their own inventory. | Specify exact producer/verifier bytes and domain rules together. Changing the serializer changes the hash input; canonical Postcard alone does not define semantic ordering or exclusions. |
| [Payload metadata][metadata] and [lifecycle expansion][lifecycle] | Compiled limits are attached to Candid declaration text; lifecycle adapters share argument machinery. | Replace owned interface/limit discovery deliberately and retain export validation. Trace installation producers and consumers, not just ordinary RPC methods. |
| [Host transport][host] | A local query transport already accepts/returns byte buffers. | Keep transport/identity ownership; replace typed payload projections and matching CLI/browser generators at the owned endpoint boundary. The existing browser trial qualifies only one update shape. |

The main benefit of Canic adoption would be centralizing this integration: callers
could use shared request/response types and normal endpoint declarations while
Canic handles dispatch and authentication. That is the intended outcome, not an
already implemented macro API.

The selected policy permits one coordinated pre-1.0 cut for owned endpoints.
Reinstall-only deployment removes the need for an old-format migration layer;
it does not remove same-release retry/recovery obligations. Do not introduce
parallel legacy/v2 lanes. External management, ledger and other Candid-defined
contracts retain Candid adapters. Transporting a new payload format does not
change IC ingress signatures, certificates or consensus envelopes.

## Relative effort across consumers

| Consumer and bounded scope | Relative difficulty | What remains consumer-owned |
| --- | --- | --- |
| One Canic read-only owned endpoint plus typed Rust caller | Moderate, after the runtime/contract API exists | Dispatch/access policy, query semantics, observability and external adapters |
| Canic's complete owned RPC/client surface | High; several coordinated implementation/qualification batches | Authorization, replay, hashes, installation and deployment policy |
| Relay typed application payloads | Low-to-moderate after the shared codec is usable | Durable delivery, recipient binding, retained bytes, ACK and retry policy |
| Replacing Relay envelopes as well | Separate moderate/high-risk protocol task | Frame identity, sequencing, recovery and persistence; payload adoption does not require this |
| IcyDB RPC/result DTOs | Moderate where both endpoints are controlled | Query semantics and database ownership |
| IcyDB schema authoring artifacts | Higher than ordinary DTOs; still needs direct comparison | Validation, canonical ordering, schema meaning/fingerprints and bounds |
| IcyDB rows, keys, journals and cursor formats | Outside this adoption scope | Selective reads, ordered keys, authentication and recovery formats |
| Blob Storage's owned service endpoints | Moderate-to-high coordinated host/client work | Upload lifecycle, receipts, funding semantics and external provider contracts |
| Auth application-token envelopes | High sensitivity; separate contract review | Signed payload/domain identity and verification; no rewriting of standard proof formats |
| Query/CLI clients | Moderate for a selected owned endpoint | Candid for external management/ledger contracts, certificate verification and transport |
| Jobs and similar endpoint-free libraries | Usually no direct adoption needed | Scheduling, state transitions and persistence integration; the application can encode a DTO |

Evidence for these narrower judgments:

- [Relay's library][relay-lib] is a scaffold; its [experimental wire codec][relay-wire]
  already bounds frames and uses binary bytes. There is no demonstrated Candid
  saving from replacing that envelope. Share payload encoding first if useful.
- [IcyDB schema encoding][icy-schema] is an internal authoring-artifact contract,
  not an RPC encoder. The [earlier assessment](shared-codec-evaluation.md) explains
  why database encodings cannot be replaced on round-trip correctness alone.
- [Blob endpoint declarations][blob] delegate export/auth attributes to their host,
  which is a useful integration point. Its [provider wire][provider] also includes
  a maintained Candid account-link shape that cannot be unilaterally changed.
- [Auth protocol types][auth-types] are passive application-token contracts;
  [signature storage][signatures] hashes host-selected domain and payload bytes.
  Replacing serialization must not silently change what is authenticated.
- [Query decoding][query] is explicitly Candid-bound and [ICRC clients][icrc] serve
  external ledger contracts. A wire client belongs beside those contract-specific
  projections, not as a global decoder replacement.
- [Jobs][jobs] explicitly owns no endpoints, database or lifecycle exports.
  It has no reason to depend on IC Wire merely because its caller adopts it.

## Consumer implementation sequence

This is a proposed adoption sequence, not supported API syntax or authorization
to change siblings. The existing Canic, IcyDB and Relay committed identities
were rechecked on 2026-10-16 and still match this assessment. Their working-tree
changes remain excluded. Difficulty estimates assume IC Wire first supplies a
usable runtime API; adding Postcard derives alone does not meet that prerequisite.

### IC Wire prerequisites

Before a consumer takes a dependency, qualify one shared declaration that owns
field order, variant identity, contract/direction identity and collection bounds.
Generate the Rust and TypeScript projections from it. Cover principals, strings,
bytes, nested sequences, options and application Result errors, with exact vectors
and rejection before excessive allocation. Define unit/empty requests explicitly.
The runtime must remain independent of CDK, Candid and host-side generators.

The scalar/browser and nested Rust fixtures prove parts of this separately.
They do not yet prove that a developer can declare a nested endpoint once and
get bounded, matching Rust and browser clients. The peer trial proves transport
and exact repetitions, but has no committed business effect to protect.

### Canic implementation and retirement

1. Choose one existing owned read endpoint and its contract crate. Replace its
   Candid payload projection at both the endpoint adapter and typed caller;
   retain the existing dispatch/access stages and raw transport. Bind the reply
   type and byte limit to the method. Update its tests and host/browser caller
   together, including empty arguments and application errors.
2. Qualify the protected-command boundary locally before adapting Canic auth or
   replay. Include wrong caller/audience, expired proof, duplicate identical
   request, reused request identity with different bytes, lost response after
   commit and recovery after interruption. A repeated committed request must
   return retained response bytes without applying the effect again. Any
   external effect needs its own reconciliation proof.
3. Inventory every owned hash/signature producer and verifier. Replace encoding
   together, retaining explicit domain separation, excluded fields and semantic
   ordering. Publish exact vectors. A successful decode is insufficient evidence
   for hash equivalence or correct authorization.
4. Update declaration/limit metadata, installation argument producers and actual
   Wasm export checks. Replace owned Candid-based client generation and fixtures.
   Keep external Candid adapters with their contract owners. Remove superseded
   owned Candid paths when all corresponding callers move in the coordinated cut.
5. Measure the actual Canic application with identical endpoints, business work,
   external dependencies and build settings. Qualify query/composite/update and
   lifecycle behavior where used. Coordinate canisters, browser assets and host
   tools under Canic's reinstall policy, explicitly resolving retained requests
   and recovery obligations before retirement. No dual-format lane is proposed.

The intended everyday application change is shared contract declarations and
regenerated clients; Canic should absorb the endpoint and call plumbing centrally.
That remains a design goal until the adapter compiles and passes these checks.

### IcyDB implementation and retirement

For application query/result messages, define a bounded DTO at the application
contract boundary, project database results into it, and update both endpoints
and clients. The application may use IC Wire without adding it to IcyDB core.
This is the smallest adoption path and preserves database query semantics.

For schema authoring artifacts, first run an installed local comparison against
the existing hand-written binary codec with equivalent validation, ordering and
limits. The native screen's Candid comparison does not establish a win over
IcyDB's current schema encoder. Count removed codec code and required adapters
as well as bytes, instructions and full Wasm. Only then consider replacing the
schema encoder/decoder and updating every artifact producer, loader, validator
and fingerprint consumer together. Retire the superseded schema format under an
explicit artifact replacement policy; do not silently reinterpret retained files.

Row slot directories, ordered keys, journals and authenticated cursor framing
remain database-owned. They do not need changes for the message adoption above.

### Supporting repositories

| Consumer | Concrete initial action | Acceptance boundary |
| --- | --- | --- |
| IC Relay | Encode an application DTO once and enqueue those bytes; decode at the controlled recipient. | Exact retained bytes across retries, recipient binding and unchanged ACK/recovery behavior. Replacing its already-binary envelope needs a separate benefit case. |
| IC Blob Storage | Change owned service request/reply contracts together with host endpoint adapters and upload/download clients. | Receipts, chunk limits and interruption recovery still pass; provider-defined Candid contracts remain intact. |
| IC Auth | Initially preserve existing proofs as bounded opaque bytes in the application contract. | Verify through the existing owner; changing signed token encoding requires producer/verifier and domain-vector review. |
| IC Query / frontend clients | Supply generated typed projections around raw query/update calls for owned methods. | Preserve identity, certificate verification, large-integer precision and reply bounds; external ICRC contracts remain Candid. |
| IC Testkit | Use raw PocketIC calls for the pilot; add a typed wire helper only when repeated consumers justify it. | Keep Candid helpers for Candid endpoints and distinguish transport rejection, codec failure and application error. |
| IC Host Tooling / IC Build | Reuse raw artifact/export inspection and build caching; update consumer recipes and metadata expectations where needed. | An owned method remains discoverable/validated without a Candid service declaration; schema/generator inputs participate in cache identity. No codec in generic build/filesystem primitives. |
| IC Memory / IC Storage / IC Metrics / IC Timers / IC Jobs | No direct codec adoption required by this proposal. Encode application-facing DTOs at their host boundary when needed. | Preserve allocation, persistence, arithmetic and scheduling contracts; avoid a new runtime dependency solely for fleet consistency. |
| IC Backup | No wholesale serialization replacement. Evaluate only a future application-owned endpoint adapter. | External IC management protocols, retained artifact identity and recovery journals keep their existing contracts. |

The added supporting-repository assessment is based on committed package/readme
ownership, plus Testkit's Candid call helper source; it is not a full API audit.
Inspected revisions: IC Testkit `bb00aa9a00dbdc07a9b58dd2a04553a2a9bfe385`,
IC Host Tooling `b99beeae7c3115ee5323ea3b48fccad2fe325596`,
IC Build `95567ebaed4b6e7b08276f57bb1a70f576c6eb3b`,
IC Memory `f4630f7dbe90c2c92f5029479f8420c1a1076a89`,
IC Storage `60d0f01470a2e1fe97fde8999c3864a8d543dfe9`,
IC Metrics `0efa8d8d7aadb2bcebbd929364a50d3c317924ca`,
IC Timers `32ea72e253fdf2adbeda76b320fdc70be885d1ed`, and
IC Backup `53d840bddb3de8680c4ab35a3c008c57ed61a244`.

## Concrete next qualification

Before requesting Canic source adoption, complete a local consumer-shaped slice:

1. Freeze a bounded nested request/Result response with explicit layout and variant
   authority, including principals and errors. Generate Rust and browser bindings
   from that authority, with malformed input rejected before oversized allocation.
2. Exercise a real canister-to-canister request and a browser call against the same
   installed endpoint. Keep IC transport errors distinct from application failures.
3. Add a protected command with caller/proof binding and a retained-response replay
   fixture. Prove that retries use identical request and response bytes and cannot
   repeat an already committed effect. Keep auth/replay policy outside the codec.
4. Produce a concrete Canic adapter diff and matched Canic artifact measurements
   only after separate sibling-edit authorization. Include all owned client and
   declaration producers. Do not project the small fixture's savings onto Canic.

Installed IcyDB-shaped schema measurement remains useful, but does not block the
Canic endpoint trial or make IcyDB adoption mandatory. The current nested fixture
already shows a favorable encoding result; the biggest remaining adoption risk
is now the correctness and ergonomics of the shared contract/adapters.

## Source identity

All links below bind to the inspected commits. Relay, IcyDB, Auth, Jobs and Query
also had local changes, which are excluded. These observations are a source audit,
not consumer tests or an estimate of production Wasm savings.

[endpoint]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-macros/src/endpoint/expand/mod.rs

[adapter]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-macros/src/adapter/mod.rs

[calls]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/ops/ic/call.rs

[infra]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/infra/ic/call.rs

[rpc]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/ops/rpc/mod.rs

[auth]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/access/auth/token.rs

[replay]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/ops/replay/mod.rs

[hash]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/ops/root_draining_reservation/mod.rs

[metadata]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-core/src/ingress/payload_contract/mod.rs

[lifecycle]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-macros/src/lifecycle/mod.rs

[host]: https://github.com/dragginzgame/canic/blob/644d4b13012b395d6211a6907e2e6aafdf391db9/crates/canic-host/src/replica_query/transport/mod.rs

[relay-lib]: https://github.com/dragginzgame/ic-relay/blob/049b5ecd3ea926e83a637be28e8997d1ed630cb2/crates/ic-relay/src/lib.rs

[relay-wire]: https://github.com/dragginzgame/ic-relay/blob/049b5ecd3ea926e83a637be28e8997d1ed630cb2/crates/ic-relay/fixtures/wire.rs

[icy-schema]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-schema/src/codec.rs

[blob]: https://github.com/dragginzgame/ic-blob-storage/blob/f0555981ed6dda5d19f4fbf2b679964276f2f9a6/crates/ic-blob-storage-contracts/src/endpoints/mod.rs

[provider]: https://github.com/dragginzgame/ic-blob-storage/blob/f0555981ed6dda5d19f4fbf2b679964276f2f9a6/crates/ic-blob-storage-contracts/src/provider/wire/mod.rs

[auth-types]: https://github.com/dragginzgame/ic-auth/blob/04b684a463a9f5e8070d1ee4f344be92831303fd/crates/ic-auth-protocol-types/src/lib.rs

[signatures]: https://github.com/dragginzgame/ic-auth/blob/04b684a463a9f5e8070d1ee4f344be92831303fd/crates/ic-auth/src/signature_store.rs

[query]: https://github.com/dragginzgame/ic-query/blob/21e44f80dee656c9d52777385c1b81aff58a413d/crates/ic-query/src/candid_decode.rs

[icrc]: https://github.com/dragginzgame/ic-query/blob/21e44f80dee656c9d52777385c1b81aff58a413d/crates/ic-query/src/icrc/ledger.rs

[jobs]: https://github.com/dragginzgame/ic-jobs/blob/c497c0f7d2ac5c812c5592bb54a04b4d7350f15a/crates/ic-jobs/src/lib.rs
