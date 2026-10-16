# Proposed developer experience

All syntax below is illustrative. Wire, endpoint integration and typed method
bindings are not implemented. Do not present these snippets as working quickstarts.

## Executable native experiment (0.1.1)

Run `make evaluate-journey` for the
[typed journey example](../crates/ic-wire/examples/typed_journey/mod.rs).
This is a compiling native experiment using Serde derives and Postcard, not an
implementation of the proposed Wire or Canic syntax below. `make test` includes
four tests for this experiment alongside the six payload-screen tests.

The single balance binding owns its method name, contract ID, request and reply
types. The caller prepares a request once, passes immutable bytes to a transport
closure, and decodes an application outcome separately from transport/codec
errors. The endpoint adapter decodes before invoking the supplied handler.
The closure in this example is a native loopback, not an IC transport or an
authentication implementation. Retry tests establish byte reuse, not delivery
deduplication or exactly-once effects.

The finite layout uses explicit tuple conversions: request `(account: u64,
at_revision: u64)`, reply `(tag: u8, amount: u64)` with tag 0 for balance and
tag 1 for not-found (amount must be zero). An outer `(contract: u32,
direction: u8, body)` tuple binds contract 0x4241 and direction 0/request or
1/reply. Postcard integer encoding applies. The request for account 7 at revision
42 is `c1 84 01 00 07 2a`; the extra four identity bytes are absent from the
earlier unframed payload screen. There is only one experimental contract, no
legacy reader or compatibility lane.

Explicit conversions make Rust declaration reordering byte-invariant, including
enum variant order. A future incompatible layout must receive a distinct contract
identity; the handwritten ID is not a derived schema fingerprint. Tests do not
establish safe independent schema evolution. Canonical re-encoding rejects
nonminimal integers and trailing bytes. A 64-byte stack buffer bounds encoded
output before allocation; the supported request/reply types contain only scalars
and allocate no body collections. This does not qualify arbitrary Serde types.

The existing lockfile resolves Postcard 1.1.3 for this run, on Rust 1.99.0,
x86_64-unknown-linux-gnu, default dev/test profiles. No dependency update was
made for this experiment. The earlier saved screen reports its historical run;
its six vector tests still pass under the current lockfile.

The result demonstrates a small concrete binding, but requires handwritten tuple
conversions and handler/client glue. It does **not** yet meet the Candid-like
ergonomics criterion. A custom derive, compile-fail diagnostics, installed IC
endpoint behavior and matched Wasm/instruction measurements remain unproved.
Do not select or adopt Postcard based on this example alone.

## Shared types

```rust,ignore
use ic_wire::Wire;

#[derive(Wire)]
pub struct GetBalance {
    pub account: AccountId,
}

#[derive(Wire)]
pub struct Balance {
    pub amount: u128,
}

#[derive(Wire)]
pub enum BalanceError {
    Unauthorized,
    AccountNotFound,
}
```

AccountId is an application-owned type. Exact field/tag annotations, supported
derive shapes and method declaration syntax remain decisions for the prototype.
Any required annotations must be included when evaluating ergonomics.

## Actor endpoints

```rust,ignore
use accounts_contracts::{Balance, BalanceError, GetBalance};

#[canic::query]
fn balance(request: GetBalance) -> Result<Balance, BalanceError> {
    let caller = canic::caller();
    accounts::balance(caller, request.account)
}
```

This assumes future Canic integration; current Canic endpoint macros use Candid.
The adapter reads bounded raw arguments, decodes once, invokes the authenticated
handler and encodes the reply. No CandidType, .did declaration or manual byte
handling should be needed in the actor's lib.rs for these endpoints. Management
calls may still link Candid through an external adapter.

## Typed caller

```rust,ignore
let result = client
    .call(accounts_contracts::BALANCE, &GetBalance { account })
    .await?;
```

The shared BALANCE declaration must bind method name, request and response types;
callers should not independently guess the response decoder. The outer result
represents transport/codec failure; the decoded endpoint result represents
application success or BalanceError. Authentication remains the endpoint owner's
responsibility. Raw bytes do not weaken caller, subject or audience bindings.

## Documentation acceptance

Before calling this usable, provide a compiling quickstart for both endpoints
and clients, supported-type reference, limits and error examples, exact wire
specification, schema-change rules and Canic/Relay integration examples. Exercise
ordinary structs/enums, nested values, a byte payload, malformed input and a
real typed round trip. Compile-fail cases should demonstrate useful diagnostics
for unsupported types or incorrect method/request bindings.

A standalone consumer should be able to use the codec without depending on Canic
or Relay. Relay keeps payloads opaque and does not wrap the wire payload in a
Candid blob. Relay's own envelope and application payload need not duplicate
identity fields or decode the same message multiple times.
