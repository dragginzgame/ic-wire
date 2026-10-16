# Rust runtime and Canic integration

IC Wire now provides a Postcard-only `no_std` + `alloc` runtime and a host-side
`Wire` derive. The runtime has no Candid, CDK or host-tooling dependency. Packages
remain unpublished. Canic can integrate a controlled Rust endpoint against this
API; its query/update macros, auth policy and rollout remain Canic-owned.

## Shared contract

This example compiles as a library documentation test:

```rust
use ic_wire::{Bytes, Limits, List, Method, Text, Wire};

#[derive(Debug, PartialEq, Wire)]
pub struct BalanceRequest {
    #[wire(id = 10)]
    pub account: u64,
    #[wire(id = 20)]
    pub labels: List<Text<32>, 4>,
}

#[derive(Debug, PartialEq, Wire)]
pub enum BalanceError {
    #[wire(id = 7)]
    Missing,
}

pub struct Balance;
impl Method for Balance {
    type Request = BalanceRequest;
    type Response = Result<u64, BalanceError>;
    const ID: u64 = 0x42414c414e4345;
    const NAME: &'static str = "balance";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 256, allocation: 1024, nodes: 32, depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Limits {
        bytes: 64, allocation: 128, nodes: 8, depth: 4,
    };
}

let request = BalanceRequest { account: 7, labels: List::new(vec![])? };
let retained = ic_wire::encode_request::<Balance>(&request)?;
// The transport sends retained.as_bytes() to Balance::NAME unchanged.
let decoded = ic_wire::decode_request::<Balance>(retained.as_bytes())?;
assert_eq!(decoded, request);
let reply = ic_wire::encode_response::<Balance>(&Ok(123))?;
assert_eq!(ic_wire::decode_response::<Balance>(reply.as_bytes())?, Ok(123));
# Ok::<(), ic_wire::Error>(())
```

Fields serialize in ascending explicit `wire(id)` order; source declaration order
is irrelevant. IDs are schema authority, not tags transmitted per field. Renaming
or reordering source fields while preserving IDs/types preserves bytes. Adding,
removing or changing a field is a coordinated contract change, even if its type
is Option. Choose a new method contract ID for incompatible bytes; update all
producers/consumers and resolve retained requests before retiring the old layout.
There is one current contract, without fallback readers or automatic migration.

Enums use explicit u32 `wire(id)` tags and support unit variants or one payload
field (use a named record for several values). Rust variant order is irrelevant.
Records support up to 16 named fields, including an empty record. Generic derives,
recursive graphs, maps, floats and architecture-dependent integers are excluded.
Supported leaves are unit, bool and fixed-width signed/unsigned integers through
128 bits. Option, Result, bounded List/Bytes/Text and Serde-supported fixed arrays
compose. Result uses Postcard's fixed tags 0=Ok, 1=Err; Option uses 0=None, 1=Some.

Principal values cross as `Bytes<29>`; adapters convert/validate with their existing
principal type. IC Wire adds no principal identity type or Candid dependency.
A field of `Bytes<29>` alone establishes a byte bound, not caller authentication.
Keep the dependency name `ic-wire` so generated paths resolve. Consumers do not
need to depend directly on Serde or derive its traits for these records.

## Resource and failure contract

Each type has a conservative static Shape. Before encoding or decoding, the
method admits its maximum envelope bytes, value heap plus codec scratch, visited
nodes and nesting depth against Limits. This deliberately admits the whole schema,
not merely the current small value. Tighten field bounds if a schema is rejected.
The allocation figure counts requested storage, not allocator bookkeeping or the
caller's input/output custody. Static arrays and record stack storage are not heap;
node/depth allowances bound traversal but are not instruction-counter estimates.
Manual Wire implementations are trusted contract code and must satisfy these
bounds. Prefer the derive and supplied bounded types.

Length hints are checked before list allocation. When Postcard hides a hint,
decode visits at most N elements and rejects an extra element without decoding
it. This covers zero-byte unit elements. Bytes/text check their lengths before
copying, and text rejects invalid UTF-8. Container allocation failures and malformed
payloads surface as Error::Malformed through Postcard; codec scratch allocation
failure is Error::Allocation. All failures remain typed at the runtime boundary.

The complete envelope is ASCII `IW`, then the method's u64 ID in little-endian,
then direction byte 0=request or 1=response, then canonical Postcard payload.
The 11-byte header is checked before payload decode. Exact consumption rejects
trailing bytes. Re-encoding rejects nonminimal representations. Empty arguments
and unit responses still carry this header. RequestBytes and ResponseBytes are
immutable and method-bound; retain their exact bytes for retry or hashing.
Consumer hash domains, semantic ordering and excluded fields remain consumer-owned.
Encoding is deterministic for the supplied types, not a general signature policy.

## Canic adapter work

The installed fixture's shared contracts and replay module demonstrate these exact
calls; this section does not claim Canic macros have been changed.

1. Bind each owned query/update to a shared Method. Canic's macro should validate
   that the function's argument and return types match that binding at compile
   time. Keep one request object; an empty request is unit or an empty record.
2. Check raw IC argument length against REQUEST_LIMITS.bytes before copying it.
   Call decode_request, then Canic's existing startup/auth/semantic stages in
   their required order. Caller-only checks can run before decoding; proof-dependent
   checks require the admitted envelope. No handler runs after admission failure.
3. Dispatch the existing synchronous/async handler; encode_response bounds the
   reply and supplies exact bytes to the raw reply primitive. Keep application
   Result errors inside the response, separate from codec and transport failure.
4. Outbound adapters encode_request once, send those bytes to Method::NAME and
   decode_response under the same Method. Canic still owns query/update selection,
   cycles, deadlines, call errors, access policy and safe retry decisions.
5. Keep wire method/limit metadata and actual Wasm export validation together.
   Remove Candid generation from those owned adapters. No Candid selector or
   codec framework is added here. This does not replace the CDK executor or
   platform APIs or alter third-party protocols.

The installed adoption trial runs a nested ordinary query and a protected update
using these production functions. It checks exact replay, wrong identity, malformed
input, application failure, trap rollback and identical-contract upgrade recovery.
Two real inter-canister deliveries reuse the typed request and apply one effect.
The evaluation uses the CDK call transport, which allocates reply bytes before
IC Wire sees them. Canic must enforce any pre-copy reply allowance in its own
callback/transport layer; runtime decode limits cannot retroactively bound that
allocation. Its direct-caller auth and local atomic effect do not qualify delegated proofs,
composite queries, async external-effect recovery or Canic's whole RPC surface.

Existing browser experiments use historical private tuples; they are not clients
for this runtime envelope. New TypeScript generation and login/delegation rollout
must be qualified before moving browser-facing endpoints. Start Canic adoption
with a controlled Rust-to-Rust endpoint; do not claim fleet-wide rollout readiness.

## Compile-time rejection examples

Missing field identities are rejected:

```compile_fail
#[derive(ic_wire::Wire)]
struct Missing { value: u64 }
```

Duplicate identities are rejected:

```compile_fail
#[derive(ic_wire::Wire)]
struct Duplicate { #[wire(id = 1)] a: u64, #[wire(id = 1)] b: bool }
```

Unbounded containers cannot enter a derived contract:

```compile_fail
#[derive(ic_wire::Wire)]
struct Unbounded { #[wire(id = 1)] items: Vec<u64> }
```

A method cannot encode the wrong request type:

```compile_fail
use ic_wire::{Limits, Method};
struct Read;
impl Method for Read {
    type Request = u64;
    type Response = bool;
    const ID: u64 = 1;
    const NAME: &'static str = "read";
    const REQUEST_LIMITS: Limits = Limits { bytes: 64, allocation: 128, nodes: 8, depth: 4 };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
let _ = ic_wire::encode_request::<Read>(&true);
```
