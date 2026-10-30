# SDK integration

Wire owns one Serde/Postcard codec, resource admission and typed codec errors.
The SDK owns endpoint selection, authentication, copying IC arguments, replies,
transport, retries, persistence and rollback. No IC SDK is needed by Wire's core codec.
These interfaces address [#29](https://github.com/dragginzgame/ic-wire/issues/29).
For typed IC exports using Platform's maintained generator, the separate opt-in
[endpoint integration](endpoints.md) supplies framed I/O and pre-copy admission.
SDK authentication, delivery and application policy remain local.

## Payloads and explicit frames

Use Rust `native::encode` / `native::decode` when the enclosing protocol selects
the contract, for example an IC method name with a parameter tuple and a direct
return value. Their TypeScript counterpart is `createPayloadCodec`; the host
`generate_payload_registry` emits typed `encode` / `decode` wrappers. A payload
entry contains `name`, `schema`, `bytes`, `nodes` and `depth`. Obtain the last
three fields from `typescript::payload_limits`; obtain the schema from
`native_schema` or a `SchemaRegistry` reference. Pass the same registry to the
payload generator. Recursive definitions use the existing named registry.

Payload unit is empty; u64 7 and the one-element tuple `(7u64,)` both encode as
`07`. Payload limits count no header and allow a zero-byte unit payload. Node
and depth limits remain positive. Scalar checks, primitive validation,
full-consumption checks, bulk advice and schema complexity limits still apply.
Payload diagnostic paths begin `/payloads/NAME`; exact-path acknowledgements
work as with the framed generator.

For standalone constructor arguments or payload schemas, use
`typescript::validate_payload_schema(&types, &schema)` without generating a
client module. It reuses the generator's schema checks, including references,
identifiers, alias cycles, unused definitions and shared complexity allowance.
It allows recursion through records and collections. This validates metadata;
runtime limits and value/scalar checks still apply when encoding or decoding.
Use `lint_bulk_roots` separately when reviewing bulk-field performance.

`generate_registry` and `createCodec` retain their explicit 11-byte
`IW`/method-ID/direction envelope and request/response methods. Payload codecs
never strip or construct a temporary envelope. Selecting a different public
endpoint layout requires coordinated consumers; adding these APIs changes no
existing framed bytes, identities or acceptance rules. Neither API binds a schema
to a deployment automatically. Retain original bytes for exact retries/signatures.

## Caller-owned output

`native::encode_to_slice`, `encode_request_to_slice` and
`encode_response_to_slice` return a mutable slice of only the completed encoding.
Their lifetime is tied to the caller's storage. Use scoped or stack storage when
appropriate, and publish the returned bytes synchronously only after success.
The SDK owns any buffer loan across an await; these native codec APIs supply no
async buffer custody.

```rust
fn main() -> Result<(), ic_wire::Error> {
    let limits = ic_wire::Limits { bytes: 64, allocation: 1024, nodes: 64, depth: 16 };
    let mut storage = [0_u8; 64];
    let bytes = ic_wire::native::encode_to_slice(&7_u64, limits, &mut storage)?;
    assert_eq!(bytes, [7]);
    Ok(())
}
```

A short buffer returns `Error::Limit` before writing. If a custom serializer
fails during the output pass, no successful slice is returned and storage contents
are unspecified. Discard that attempt. This API retains the owned encoder's
admission/sizing pass and bounded output pass, using Postcard's slice flavor
through the same Serde admission adapter. It introduces no inline/spill writer,
callback escape mechanism or automatic fallback. Caller storage does not lower
`Limits::allocation`; its conservative ledger is unchanged. Custom Serde remains
trusted and may allocate independently of codec output.

`native::with_encoded(value, limits, scratch, consume)` chooses scratch when the
admitted payload fits and owned output otherwise, before the output pass. It
retains two-pass admission and the same allocation ledger. It drops an owned
input before calling the synchronous consumer, and never calls it after encoding
failure. Pass a reference when ownership should stay with the caller; root value
size still participates in admission. Callback bytes cannot escape as a borrow
or inside a future. Copy them when retaining retries or crossing an await.

```rust
let limits = ic_wire::Limits { bytes: 4096, allocation: 65536, nodes: 4096, depth: 32 };
let mut scratch = [0; 128];
ic_wire::native::with_encoded(7_u64, limits, &mut scratch, |bytes| {
    assert_eq!(bytes, [7]);
    // An SDK may synchronously copy the completed payload into its reply here.
})?;
# Ok::<(), ic_wire::Error>(())
```

This helper encodes payloads only. Existing framed slice/owned APIs retain their
identities and directions. The SDK can choose `encode` / `encode_into` when it
needs owned bytes or a reusable Vec directly. Buffer sizing, synchronous reply
and rollback policy stay in the SDK. See the
[caller-storage study](evidence/caller-storage-0719/README.md) for matched
instruction, allocation and uncompressed Wasm measurements and their limits.

## Custom scalar values and public client types

Implement the scalar's Serde representation and verified `Wire` metadata in the
consumer. A metadata `check` names a validator; it does not change the serialized
representation, convert a value or change its TypeScript type. An adapter module
can forward both `createCodec` and `createPayloadCodec` to Wire's shared runtime,
supplying the consumer's check map. Select that module through `runtime_module`.
Existing Principal-like, arbitrary unsigned and signed representations remain
consumer-owned; Wire does not impose a replacement scalar convention.

Generated types describe serialized values. Expose application classes or other
ergonomic types through an ordinary typed facade that converts to/from those
values. For example, if generated `count` encodes bigint, the consumer can write:

```typescript
import { count } from './payloads.js';
export class Counter {
  constructor(readonly value: bigint) {
    if (value < 0n) throw new RangeError('negative counter');
  }
}
export const counter = {
  encode: (value: Counter): Uint8Array => count.encode(value.value),
  decode: (bytes: Uint8Array): Counter => new Counter(count.decode(bytes))
};
```

The facade owns the public type and both conversions. For nested records/enums,
convert their scalar fields explicitly and preserve the generated wire types at
the codec boundary. Wire adds no inference of SDK types or custom serializer
layouts. These conversions must preserve the actual consumer Serde contract;
changing its byte representation remains a coordinated contract change.
The conformance fixture compiles and executes the facade against the generated
payload codec, alongside separate scalar-check tests.

## Shipping JavaScript

A host generator can write `typescript::RUNTIME` to `wire-runtime.ts` next to
its generated modules. Compile them with the consumer's pinned TypeScript
compiler, for example:

```sh
tsc --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes \
  --target ES2022 --module NodeNext --moduleResolution NodeNext \
  --outDir build/generated generated/*.ts
```

Keep the consumer's Node/module package configuration consistent with its output.
The runtime imports no transport or SDK. Ship the compiled JavaScript with its
generated clients; there is no separately maintained JavaScript implementation.
Host generation stays outside the deployed Wasm dependency graph. Qualify the
installed SDK path separately, including response failures and rollback; these
codec tests do not establish endpoint lifecycle or production throughput.
