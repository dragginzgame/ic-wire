# Rust runtime and Canic integration

IC Wire now provides a Postcard-only `no_std` + `alloc` runtime and a host-side
`Wire` derive. The runtime has no Candid, CDK or host-tooling dependency. Packages
are published on crates.io starting at 0.3.0. Canic can integrate a controlled Rust endpoint against this
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
recursive graphs, maps, raw f32/f64 and architecture-dependent integers are
excluded. Supported leaves are unit, bool, fixed-width signed/unsigned integers
through 128 bits, and FiniteF32/FiniteF64. Option, Result, bounded List/Bytes/Text and Serde-supported fixed arrays
compose. Result uses Postcard's fixed tags 0=Ok, 1=Err; Option uses 0=None, 1=Some.

Principal values cross as `Bytes<29>`; adapters convert/validate with their existing
principal type. IC Wire adds no principal identity type or Candid dependency.
Owned values can leave their bounded wrappers through `Bytes::into_vec`,
`Text::into_string` and `List::into_vec` without copying or allocating. Consumers
then own the recovered buffers; constructing another wrapper checks its bound again.
A field of `Bytes<29>` alone establishes a byte bound, not caller authentication.
Keep the dependency name `ic-wire` so generated paths resolve. Consumers do not
need to depend directly on Serde or derive its traits for these records.

## Finite floats and SQL-shaped messages

`FiniteF32::new(value)` and `FiniteF64::new(value)` admit finite primitives and
return `Error::Malformed` for NaN or infinity. `.get()` recovers the primitive.
Decoding rejects non-finite IEEE-754 bit patterns, including signaling NaNs.
Postcard encodes these transparent wrappers as native little-endian 4/8-byte
floats. They add no heap allocation or dynamic traversal to Rust's scalar shape.
Signed zero and subnormals are preserved, without consumer-specific normalization.

```rust
use ic_wire::{FiniteF32, FiniteF64};
assert_eq!(FiniteF32::new(-0.0)?.get().to_bits(), (-0.0_f32).to_bits());
assert_eq!(FiniteF64::new(2.5)?.get(), 2.5);
assert!(FiniteF64::new(f64::INFINITY).is_err());
# Ok::<(), ic_wire::Error>(())
```

Generated TypeScript uses `number`. A FiniteF32 input must already be exactly
representable as f32; use `Math.fround(value)` when intentionally converting a
JavaScript number. For example, `0.1` alone rejects while `Math.fround(0.1)` is
accepted. FiniteF64 accepts finite JavaScript numbers. Both reject NaN/infinity;
f32 overflow, underflow or silent rounding also rejects on encoding.

The conformance fixture includes an empty and populated SQL-shaped page with
bounded columns/rows, explicit cell tags, signed/unsigned integers, both finite
floats, text, bytes, booleans, null and an optional continuation value. This is a
local consumer-shaped DTO, not IcyDB's actual SQL API. Convert owned String/Vec
values through Text/List/Bytes at the message boundary; their constructors check
bounds without granting unrestricted Wire implementations to those containers.

IcyDB's scalar-boundary documentation at commit
`996498ad4edb5569964b68e0c0e3f5bb9df7d1c6` already specifies finite-only floats
and normalizes negative zero (read-only inspection of `icydb-core/src/types/mod.rs`). That policy belongs in IcyDB's conversion; IC Wire does
not silently apply it. Recursive values still need separate bounded admission and
schema-generation work. Ordinary finite schemas retain the existing static resource
model. Cross-language correctness does not establish new performance savings or
qualify every SQL value kind, nested recursive value, or deployed SQL endpoint.

The new Float metadata variant makes Schema non-exhaustive for future additions.
External metadata matches must include a fallback. This is a source API change
for the next minor release, not a change to previously supported wire bytes.

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
Encoding reserves the admitted maximum but initializes only emitted bytes.
Canonical validation compares emitted chunks directly with the input without a
second output allocation. Schema admission still reserves the same conservative
heap-plus-scratch allowance for both directions.

The complete envelope is ASCII `IW`, then the method's u64 ID in little-endian,
then direction byte 0=request or 1=response, then canonical Postcard payload.
The 11-byte header is checked before payload decode. Exact consumption rejects
trailing bytes. Re-encoding rejects nonminimal representations. Empty arguments
and unit responses still carry this header. RequestBytes and ResponseBytes are
immutable and method-bound; retain their exact bytes for retry or hashing.
Consumer hash domains, semantic ordering and excluded fields remain consumer-owned.
Encoding is deterministic for the supplied types, not a general signature policy.

## Canic adapter work

Keep application services callable with consumer-owned values; endpoint adapters
own wire decoding, conversion and reply encoding. A future Candid endpoint could
use separate DTOs and conversions to call the same services. `Wire` derives emit
Serde implementations for IC Wire's explicit layout, and bounded wrappers express
its contract; they are not a promise of another protocol's representation.
Method IDs and retained canonical bytes also remain specific to IC Wire. A future
adapter must preserve domain checks and commit semantics and qualify its own
types, clients and hash/retry behavior. No Candid mode or codec-selection API is
needed in this runtime to keep that extension possible.

The installed fixture's shared contracts and browser module demonstrate these exact
calls. A separate [local macro proof](https://github.com/dragginzgame/ic-wire/blob/main/docs/macro-adoption.md) tests actual evaluation-only
query/update attributes; neither changes Canic macros.

1. Bind each owned query/update to a shared Method. Canic's macro should validate
   that the function's argument and return types match that binding at compile
   time. Keep one request object; an empty request is unit or an empty record.
2. Check raw IC argument length against REQUEST_LIMITS.bytes before copying it.
   Call decode_request, then Canic's existing startup/auth/semantic stages in
   their required order. Caller-only checks can run before decoding; proof-dependent
   checks require the admitted envelope. No handler runs after admission failure.
3. Dispatch the existing synchronous/async handler; encode_response bounds the
   reply and supplies exact bytes to the raw reply primitive. If response encoding
   fails after a handler changes local state, trap to roll back the transaction;
   a normal rejection can commit that mutation. This does not undo effects already
   committed before an external await. Keep application
   Result errors inside the response, separate from codec and transport failure.
4. Outbound adapters encode_request once, send those bytes to Method::NAME and
   decode_response under the same Method. Canic still owns query/update selection,
   cycles, deadlines, call errors, access policy and safe retry decisions.
5. Keep wire method/limit metadata and actual Wasm export validation together.
   Remove Candid generation from those owned adapters. No Candid selector or
   codec framework is added here. This does not replace the CDK executor or
   platform APIs or alter third-party protocols.

`make check-typed-diagnostics` compiles the actual runtime test contracts in an
isolated workspace with locked, previously fetched dependencies offline. It first
checks valid synchronous and async consumer-shaped adapters, each binding its
handler argument and response to the same Method associated types. It then checks
11 expected compilation failures: missing field IDs, duplicate field/variant IDs,
unbounded String, generic and tuple contracts, wrong request/reply types, wrong
handler argument/return types and an async handler with the wrong future output.
Unexpected diagnostics print their logs; failed fixtures are retained for inspection.
These are private adapter-shape checks, not implemented Canic macros, executor or
auth policy. Canic should preserve these type relationships in its own expansions
and run macro-owned compile-fail tests when integration begins.

The installed adoption trial runs a nested ordinary query and a protected update
using these production functions. It checks exact replay, wrong identity, malformed
input, application failure, trap rollback and identical-contract upgrade recovery.
Two real inter-canister deliveries reuse the typed request and apply one effect.
The evaluation uses the CDK call transport, which allocates reply bytes before
IC Wire sees them. Canic must enforce any pre-copy reply allowance in its own
callback/transport layer; runtime decode limits cannot retroactively bound that
allocation. Its direct-caller auth and local atomic effect do not qualify delegated proofs,
composite queries, async external-effect recovery or Canic's whole RPC surface.

The [installed browser fixture](https://github.com/dragginzgame/ic-wire/blob/main/docs/browser-interop.md) now uses this runtime envelope
and generated clients for queries, protected updates and typed replies. Controlled
direct-caller auth is qualified; login/delegation and coordinated frontend rollout
still need consumer qualification. Do not claim fleet-wide rollout readiness.

## TypeScript generation

Enable `ic-wire`'s `typescript` feature in a host generator, then write
`ic_wire::typescript::generate::<YourMethod>()?` to a method-specific `.ts` file
and `ic_wire::typescript::RUNTIME` to `wire-runtime.ts` beside it. Generation uses
`Wire::SCHEMA`, emitted by the same derive that owns Rust field order, variant
IDs and bounds. It rejects inadmissible methods, opaque manual implementations
and excessively large/deep metadata. Hand-written metadata must faithfully match
its serializer and shape; it is trusted code, not a schema supplied by a caller.
The optional feature adds serde_json only to generation; leave it disabled in
canister dependencies. No SDK or transport is imported by generated modules.

Each module exports `Request`, `Response`, `methodName`, `encodeRequest`,
`decodeRequest`, `encodeResponse` and `decodeResponse`. With the SDK transport
already configured, an update adapter can use the following integration pattern
(the installed fixture qualifies this pattern with its controlled test identity):

```typescript
import * as apply from './generated/apply.js';
const bytes = apply.encodeRequest(request);
const reply = await agent.update(canisterId, {
  methodName: apply.methodName, arg: bytes,
});
const response = apply.decodeResponse(reply.reply);
```

Retain the encoded bytes for retries rather than regenerating them from a mutable
object. Canic still owns safe retry, authentication, query/update selection and
certificate verification. Browser transport reply allocation happens before decode.

| Rust value | TypeScript projection |
| --- | --- |
| `u8` through `u32`, `i8` through `i32` | Checked integer `number` |
| `u64`, `u128`, `i64`, `i128` | `bigint` |
| `()`, `bool` | `null`, `boolean` |
| `FiniteF32`, `FiniteF64` | Finite `number`; f32 must be exactly representable |
| `Bytes<N>`, `Text<N>` | `Uint8Array`, UTF-8 byte-bounded `string` |
| `List<T,N>`, `[T;N]` | `ReadonlyArray<T>`; bounded/exact count checked at runtime |
| `Option<T>` | `null` or `{ some: T }`, preserving nested options and unit |
| `Result<T,E>` | `{ ok: T }` or `{ err: E }` |
| Derived record | Named object; fields encoded in explicit ID order |
| Derived enum | `{ tag: "Variant", value: payload }`; unit payload is `null` |

Outgoing values are checked as well as incoming bytes. `WireError.code` is one
of `limit`, `malformed`, `identity` or `value`. Decoding rejects nonminimal varints,
unknown tags, invalid UTF-8, trailing bytes and wrong method/direction; encoding
rejects unsafe integer representations, lone UTF-16 surrogates and extra fields.
A leading Unicode BOM is retained as text. Decoded byte buffers own their data.
Logical node/depth, frame, collection and UTF-8 byte bounds are enforced before
unbounded work/allocation. JavaScript engine heap usage is not the Rust allocation
budget: object overhead, temporary buffers and engine limits differ.

`make check-typescript` uses prepared browser-harness Node and TypeScript tools.
It generates 83 Rust vectors covering every supported primitive and composition,
type-checks the emitted modules, and checks both directions in Node, including
truncation, identity errors and hostile input/value cases. This is codec
conformance; the separate installed browser fixture qualifies transport and
controlled direct-caller auth. Neither is performance evidence.
The maintained fixture is `crates/ic-wire/examples/typescript_conformance/`.

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

## Delivery sizing and inspection

Use `encoded_request_len::<M>(&request)` or
`encoded_response_len::<M>(&response)` to measure the complete frame without an
output buffer. These helpers retain whole-schema admission and require
deterministic serializers. A separate size check adds a serialization pass;
ordinary encoding stays single-pass. See [consumer qualification and the offline
packet viewer](https://github.com/dragginzgame/ic-wire/blob/main/docs/consumer-fixtures.md) for qualification ownership and diagnostics.

## Package and license

Use the registry dependency `ic-wire = "0.3"`.
Enable `features = ["typescript"]` only when generating client codecs on the host.
The runtime supports Rust 1.91 and `wasm32-unknown-unknown`.

Licensed under the Dragginz Source Available License — Verifiable ICP Burn,
Version 1.0; see the packaged LICENSE. This is not an OSI-approved license.

## Native Serde payloads

`ic_wire::native` handles consumer-owned Serde types, including String, Vec,
tuples, maps and recursive enums, using the same Postcard encoding and envelope.
It does not infer `Wire`, static maxima or TypeScript metadata for those types.
Consumers own declaration order, compatible method IDs, deterministic serializers
and domain validation (including finite floats where required). Unordered maps
are unsuitable unless their serializer establishes deterministic ordering.

```rust
use ic_wire::{Limits, native};
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Request { text: String, values: Vec<u64> }
let limits = Limits { bytes: 4096, allocation: 65536, nodes: 4096, depth: 64 };
let request = Request { text: "hello".into(), values: vec![1, 2] };
let bytes = native::encode_request(7, &request, limits)?;
assert_eq!(native::encoded_frame_len(&request, limits)?, bytes.len());
assert_eq!(native::decode_request::<Request>(7, &bytes, limits)?, request);
# Ok::<(), ic_wire::Error>(())
```

`encode`/`decode` and `encoded_len` operate on payloads without headers.
`encode_request`/`encode_response` accept borrowed Serialize values without an
owned Deserialize requirement. Their decode counterparts require owned values.
`encoded_frame_len` includes the 11-byte header for either direction. Retain the
returned bytes unchanged for any consumer-authorized retry; IDs alone do not
establish idempotency. Sizing does not retain a serialization for later delivery.

Limits are explicit per operation. Native `bytes` bounds the payload or full
frame, respectively. `allocation` reserves that entire byte allowance for output
scratch before admitting value storage. The remaining allowance charges eight
times visited value sizes and text/byte lengths; node and depth accounting follows
Serde traversal, including seeds and enum wrappers. It is deliberately conservative,
not an allocator/peak-memory measurement. Unlike finite Wire schema admission,
it follows actual values. Different borrowed/owned representations may require
different traversal allowances. Native decoding suppresses collection size hints
and admits elements before their visitors can allocate. Canonical checking streams
serialization against the input without allocating a second output buffer.

Custom Serde code is trusted: work and allocations outside these visited boundaries
must be bounded by its owner. Standard/custom Deserialize allocations may abort on
allocator exhaustion; the adapter does not make arbitrary Serde allocation fallible.
The codec's own output reserve is fallible. Resource refusals are `Error::Limit`;
invalid payloads, trailing bytes, identity and noncanonical encodings retain typed
errors. Caller input custody is excluded. Native depth allowances must fit the
consumer's stack budget. No schema-independent performance claim is made.

The traversal adapters are adapted from IcyDB's native admission implementation;
its MIT notice is retained in `src/native/LICENSE-MIT`. IcyDB's numeric policies,
SQL semantics, endpoint roster and storage encodings are not imported.

## Concrete native TypeScript metadata

`typescript::RUNTIME` accepts trusted concrete metadata through
`createCodec(method, types, checks)`, including named recursive `ref` nodes,
`tuple` items and transparent `newtype` wrappers. Consumers supply byte, node and
depth allowances for both directions. Dynamic lengths are checked before copying
or descending. Missing/cyclic aliases and sparse arrays reject. Concrete nodes
and named references can name trusted scalar checks that run before encoding and
after decoding; every check along an alias chain applies.

Metadata extraction, domain validation and endpoint identities remain consumer-owned.
The finite Rust generator still rejects `Schema::Opaque`; it does not infer native
metadata or JavaScript heap bounds. The shared runtime compiles with strict
TypeScript, `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.
