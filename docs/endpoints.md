# Typed Wire endpoints

Enable `ic-wire`'s optional `endpoint` feature in a standalone canister. It
connects ordinary Serde handlers to published IC Platform 0.11.0's maintained
query/update generator. The default Wire codec stays `no_std + alloc` and has no
Platform dependency. The integration uses Platform's `std` runtime without Candid,
Canic, an extra executor or a second encoding.

```toml
[dependencies]
ic-wire = { version = "0.10.0", features = ["endpoint"] }
```

This guide describes the pending 0.10.0 integration; the version above becomes
available only after publication. The
[standalone fixture](../examples/endpoint/Cargo.toml) uses the local workspace
package for qualification. Git releases and registry publication remain separate.

Platform embeds its IC0 wrappers and executor. Endpoint users must adopt
Platform 0.11 throughout their application's runtime dependencies, including timer
integrations; do not combine it with an older external executor. Cargo's singleton
guard rejects conflicting runtime owners. Core-only Wire consumers need no runtime
migration. Handler declarations, frame bytes, method identities and Wire admission
rules are unchanged; retained request/retry bytes remain valid. See
[Platform's runtime migration](https://github.com/dragginzgame/ic-platform/blob/421edbc/docs/migration.md#embedded-ic0-and-executor).

Platform scopes that Cargo guard to executor/runtime features. Native-only
Platform dependencies can coexist without claiming another runtime; endpoint
builds still select the guard and cannot mix executor owners.

```rust,ignore
use ic_wire::endpoint::query;

#[query]
fn echo(value: u64) -> u64 { value }
```

`id`, `request` and `response` are optional constant-expression overrides.
Both directions default to 65,536 full-frame bytes, 8,388,608 bytes of conservative
traversal allowance, 65,536 charged value visits and nesting depth 32. These are
`endpoint::DEFAULT_REQUEST_LIMITS` and `DEFAULT_RESPONSE_LIMITS`; allocation is
not a measured heap cap and excludes the argument copy. Core codec functions still
require explicit limits. Override either direction independently, for example
`#[query(response = MY_REPLY_LIMITS)]`.

The automatic ID is `native::method_id(exported_name)`, evaluated at compile time:
FNV-1a-64 over UTF-8 `ic-wire:method:` followed by the exact name, using offset
14695981039346656037, prime 1099511628211 and wrapping u64 arithmetic. Rust module
paths, type spelling and execution mode do not participate. Clients can use the
generated contract's `id` or the runtime-independent `native::method_id` function.
This is a routing identity, not authentication or a schema fingerprint; consumers
retain duplicate-ID admission because 64-bit hashes can collide.

`name = "public-name"` selects the exported name and therefore its automatic ID.
Pinning that name preserves the ID across a Rust function rename. The handler takes
one owned typed value or no arguments for unit, and returns one Serde value,
including tuple or unit. Use a typed tuple to group multiple inputs. Generic
functions, receivers, unsafe/extern/const functions and variadics are rejected. Custom Serde implementations remain trusted exactly as in the
native codec; limits do not constrain work outside their Serde visits.

`update` selects an update; `query(composite, ...)` selects a composite query.
Either supports synchronous or asynchronous functions according to Platform's
execution contracts. These attributes belong on module-level functions. Do not
stack another entrypoint macro. A renamed dependency or framework re-export can
supply `crate = "framework::internal::wire"`.

## Synchronous borrowing

Add `borrow` when a synchronous handler reads borrowed fields:

```rust,ignore
#[derive(serde::Serialize, serde::Deserialize)]
struct RequestRef<'a> {
    #[serde(borrow)]
    data: ic_wire::BytesRef<'a, 32>,
}

#[ic_wire::endpoint::query(borrow, id = 7, request = REQUEST, response = RESPONSE)]
fn echo(value: RequestRef<'_>) -> RequestRef<'_> { value }
```

Platform still copies ingress into Wasm memory once. The wrapper retains that
argument vector through decoding, the handler and reply encoding. Borrowed fields
point into this vector, avoiding another allocation/copy for those fields; this
is not a borrow from IC ingress memory. A borrowed response can refer to the same
buffer because it is encoded before the vector is dropped. Rust prevents these
references from escaping into static storage. Direct references such as `&str`
also work; custom DTOs use `'_` in the handler signature.

`borrow` requires one request and a synchronous function. Async handlers retain
the owned path. The ordinary owned path drops the raw vector after decoding;
using `borrow` retains it longer, so select it for DTOs that actually borrow.
This changes neither framing nor validation: equivalent `Bytes<N>` and
`BytesRef<'_, N>` fields use identical bytes and bounds. Borrowing is not lazy
validation, and request rejection and reply traps retain the same semantics.
The [generated-endpoint study](evidence/borrowed-endpoints-0724/README.md)
measures both ownership choices through Platform's actual exports.

## Bounds, bytes and transactions

The Wire admission guard checks `msg_arg_data_size()` before Platform allocates
and copies its argument vector. The typed decoder then delegates to
`native::decode_request`, including full-input, identity, direction, primitive,
allocation, node and depth checks. The response uses `native::encode_response`.
Both byte allowances include the eleven-byte frame header. There is one copy,
one decode, one handler call and one bounded response encoding operation.
Encoding still has the native codec's two Serde passes.

Explicit declarations keep their exact protocol. For method ID 7 and request `42u8`,
both the old Canic adapter and an explicitly configured integration use
`49 57 07 00 00 00 00 00 00 00 00 2a`; the response changes only the direction
byte at offset 10 from `00` to `01`. Unit still requires an eleven-byte identity
frame. There are no fallback encodings, legacy lanes or automatic migrations.
Keep exact original retry bytes and encoder hash inputs as before.

Omitting settings previously failed compilation; it now selects defaults.
Removing `id = 7` from `echo` selects 3346031142226386015 instead: its request
becomes `49 57 5f 5c cc 87 98 7d 6f 2e 00 2a`. Removing custom bounds can likewise
change admission. Existing consumers must retain overrides or coordinate their
IDs/budgets and regenerate metadata together. There is no old-ID fallback.

Malformed requests trap before dispatch. Reply encoding failures trap instead of
rejecting, rolling back synchronous update mutations. An async future that
finishes without suspension shares that boundary. After an inter-canister await,
prior transactions have committed: a later reply-encoding trap rolls back only
the current callback's changes. Query state never persists. Consumers retain
application transaction design, retries and error policy.

Generated guards now check the existing eleven-byte header before copying the
complete argument. Previously they checked only the frame-size allowance before
the copy; identity and direction were checked during decoding. Accepted frames,
bytes, limits and errors are unchanged, and decoding still performs full validation.
The [admission study](evidence/tripwire-adoption-0725/README.md) records the small
valid-call overhead and reductions in large invalid-request costs.

## Schema projection and framework composition

`contract = ECHO` is optional. It emits a constant with the handler's exact
`Contract<Request, Response>` types and its `name`, `mode`, `id`, `request` and
`response` settings. With Wire's native `typescript` feature, call
`ECHO.request_schema()` and `ECHO.response_schema()`; these delegate directly to
`typescript::native_schema` and require the existing `Wire` metadata trait.
Use the existing schema registry/generator for recursive or aggregate schemas.
There is no new registry, custom section or final artifact checker. Native
projection alone does not prove Wasm type/layout parity or final export admission.

Frameworks can use the same lower-level helpers with Platform's generator around
their existing guarded function. This is useful for Canic's transport-rejection
branch, whose wrapper `Result` is not the declared Wire response:

When a typed contract constant is available, its `admit_request()` method performs
both size and header admission. The standalone `admit_request(Limits)` used below
checks size only; `decode_request` subsequently validates the complete frame.

```rust,ignore
fn admit() -> Result<(), String> {
    ic_wire::endpoint::admit_request(REQUEST);
    Ok(())
}
fn decode(bytes: Vec<u8>) -> Request {
    ic_wire::endpoint::decode_request(&bytes, METHOD_ID, REQUEST)
}
#[ic_platform::cdk::update(
    guard = "admit", decode_with = "decode", manual_reply
)]
async fn entry(request: Request) {
    match existing_guarded_handler(request).await {
        Ok(response) => ic_platform::cdk::api::msg_reply(
            ic_wire::endpoint::encode_response(&response, METHOD_ID, RESPONSE)
        ),
        Err(access_error) => ic_platform::cdk::api::msg_reject(access_error.to_string()),
    }
}
```

The framework must keep the pre-copy admission guard and exactly one reply path.
It should not export the guarded function separately. The
[standalone fixture](../examples/endpoint/src/lib.rs) includes an installed
composition example proving that dispatch executes once and access refusals keep
their transport meaning. It contains no Canic policy implementation.

## Ownership audit and later Canic adoption

The read-only Canic audit used HEAD
`b9ed7f298bdaf8c9cceac142edf7e56b15d33208`. Its checkout had unrelated dirty work;
the five requested source files were unmodified relative to that HEAD.

| Current Canic machinery | Subsequent adoption |
| --- | --- |
| `crates/canic-macros/src/adapter/wire/mod.rs`: `decode` and `reply` | Replace generated Wire I/O with these helpers or the typed attribute. The `declaration` custom section remains Canic-owned. |
| `crates/canic-macros/src/adapter/mod.rs`: Wire branch of `RawAdapter::expand`, `decode`, `reply`; manual `entrypoint`/`export_entrypoint` for that branch | Delegate Wire entrypoint generation to Platform, supplying Wire hooks. Other Canic endpoint/lifecycle paths are outside this extraction. |
| `crates/canic-macros/src/endpoint/expand/mod.rs`: Wire adapter selection around the guarded wrapper | Compose the same wrapper once; preserve startup, authentication, Fleet admission, dispatch and access-denial policy. |
| `crates/canic-macros/src/endpoint/expand/wire/mod.rs`: typed registration projection | May consume the typed contract's actual schemas/settings instead of reconstructing them. Registration itself remains Canic-owned. |
| `crates/canic/src/endpoint/registration/mod.rs` | Keep Canic's native provider registry and `canic:wire` artifact declaration/admission. Do not replace them with a Wire registry. |

Canic also retains role bundles, instrumentation and final artifact admission.
No Canic or Platform file was edited. Upstream implementation and standalone
qualification are complete here; Canic adoption, publication and coordinated
consumer dependency updates are separate work.

## Focused qualification

`make evaluate-endpoints EVIDENCE_DIR=/absolute/new/path` measures every exported
endpoint in the standalone fixture using a prepared instrumented `POCKET_IC_BIN`.
It checks the case inventory against compiled exports and covers owned/borrowed,
sync/async, composite, unit, automatic/default and guarded forms, plus rejection
and reply-failure paths. It records eight samples per case/profile after warmup,
complete physical execution instructions, observed cycle debits, frame bytes and
total/code-section Wasm. See the [matrix report](evidence/endpoint-matrix-0724/README.md).
This qualifies Wire's fixture endpoints; consumer application endpoints still
need their own workload and authentication measurements.

Run `make check-endpoint` for native metadata, compile-fail diagnostics, Clippy,
Wasm compilation, the Rust 1.91 floor and runtime dependency boundaries. It is
included in the ordinary core CI roster. With an explicitly prepared server,
run `POCKET_IC_BIN=/path/to/pocket-ic make check-endpoint-ic` for successful,
malformed/wrong-identity, request/reply boundary, synchronous rollback, async and
framework-composition cases. Testkit owns server admission and lifecycle.

See [cost evidence](evidence/endpoints/README.md) for the separate matched
inline-hook/extracted-hook study and its limits. No broad suite or actual Canic
adoption is implied by these focused results.
