# Proposed developer experience

The syntax below is historical design discussion, not the maintained API. Use
the [runtime guide](runtime.md) for compiling contracts and consumer adapters.

## Retired native experiment

The 0.1.2 typed journey and its private `wire_record!`, envelope and method binding
were removed in pending 0.2.3. Their original source remains in Git history.
Production runtime tests now cover retained retry bytes, transport/application
error separation, explicit field order, canonical encoding and malformed input.
`make test` and `make check-typed-diagnostics` exercise the maintained implementation.
Historical measurements retain their original source identities and bytes.

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
