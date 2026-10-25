# Lossless arbitrary unsigned evidence

For owned unsigned observations that may exceed u128, use the existing
`ic_wire::Bytes<N>` as a **minimal unsigned big-endian magnitude**. Zero is an
empty byte string; every nonzero value starts with a nonzero byte. There is no
sign byte, fixed-width padding, decimal text or LEB128 integer inside the buffer.
The ordinary Postcard byte-length prefix remains. This is a documented consumer
convention over an existing primitive, not a new Wire integer type or codec.

`N` is a consumer-selected magnitude-byte limit. The examples and qualification
use 64 bytes (up to 512 bits), not a Wire-wide maximum or a prescribed Canic bound.
Choose the field and operation limits to admit the retained evidence your
application needs; do not replace overflow evidence with u128 or saturate it.

## Declaration and exact bytes

```rust
#[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
struct RefillEvidence {
    cycles_sent: Option<ic_wire::Bytes<64>>,
}
```

The derive is host-only metadata. With the `typescript` feature it projects to
`{k:"record",fields:[{name:"cycles_sent",value:{k:"option",item:{k:"bytes",max:64}}}]}`.
The generated field is `null | { some: Uint8Array }`; bytes never pass through a
JavaScript number. Wire's explicit option representation distinguishes missing
observation from recorded zero.

Payload examples for this single-field record (excluding the unchanged frame):

| Observation | Payload hex |
|---|---|
| None | `00` |
| Some(0) | `01 00` |
| Some(1) | `01 01 01` |
| Some(u128::MAX) | `01 10` followed by sixteen `ff` bytes |
| Some(2^128) | `01 11 01` followed by sixteen `00` bytes |
| Some(2^256) | `01 21 01` followed by thirty-two `00` bytes |

For an inspection DTO, apply the same convention independently to `cycles`,
`reserved_cycles` and `memory_size`. Keep the consumer's status/error union,
including `InspectionReserveRequired`, authorization, retained snapshots and
recovery logic unchanged. This guide does not migrate or qualify an installed
Canic endpoint. External management/CMC Candid and stable storage stay with their
existing owners and encodings. Introducing this field layout into a consumer is
a coordinated contract change; update its identity and clients as appropriate.
No existing Wire payload or acceptance rule changes merely by publishing this guide.

## External Nat boundary

The consumer owns the arithmetic dependency. With its existing external `Nat`
and `num_bigint::BigUint`, use the following conversions. They are compiled and
tested in the unpublished [boundary qualification](../crates/ic-wire-benchmarks/tests/roundtrip/unsigned_magnitude/mod.rs).
No Candid or BigUint dependency is added to either published Wire crate.
Project the external Nat explicitly; its own Serde representation is not this
convention and should not be inferred from the external type.

```rust,ignore
use ic_platform::candid::contract::Nat;
use num_bigint::BigUint;
use ic_wire::{Bytes, Error};
type Magnitude = Bytes<64>;

fn from_nat(value: &Nat) -> Result<Magnitude, Error> {
    if value.0.bits() > 512 { return Err(Error::Limit); }
    let bytes = if value.0.bits() == 0 { Vec::new() }
                else { value.0.to_bytes_be() };
    Magnitude::new(bytes)
}
fn to_nat(value: &Magnitude) -> Result<Nat, Error> {
    if value.as_slice().first() == Some(&0) { return Err(Error::Malformed); }
    Ok(Nat(BigUint::from_bytes_be(value.as_slice())))
}
```

The outbound check precedes magnitude-buffer allocation; it never truncates or
saturates. The inbound check precedes construction of the arbitrary integer.
Convert optional evidence with `value.as_ref().map(from_nat).transpose()` and
`value.as_ref().map(to_nat).transpose()`. Perform the existing u128 conversion
and missing/recorded/overflow classification **after** lossless transport.
Do not make successful conversion to u128 a prerequisite for retaining evidence.

## Browser conversion

Keep generated DTO fields as Uint8Array. These bounded application helpers can
convert to/from bigint where arithmetic or overflow classification is needed:

```typescript
function magnitudeToBigInt(bytes: Uint8Array): bigint {
  if (!(bytes instanceof Uint8Array) || bytes.length > 64 || bytes[0] === 0)
    throw new Error('unsigned magnitude');
  let value = 0n;
  for (const byte of bytes) value = (value << 8n) | BigInt(byte);
  return value;
}
function bigIntToMagnitude(value: bigint): Uint8Array {
  if (typeof value !== 'bigint' || value < 0n || (value >> 512n) !== 0n)
    throw new Error('unsigned magnitude');
  const bytes: number[] = [];
  while (value !== 0n) { bytes.push(Number(value & 255n)); value >>= 8n; }
  return Uint8Array.from(bytes.reverse());
}
```

Only individual masked bytes become numbers. The bound check occurs before
building the output array; parsing is bounded by the admitted byte length.
The Rust/browser conformance suite checks exact request and response frames for
None, zero, u128 maximum, 2^128, 2^256 and the maximum admitted 512-bit value,
plus bigint conversions, negative/oversized values and redundant leading zeroes.

## Admission and canonicality responsibilities

Wire admits **opaque bytes**, not an arithmetic type. `Bytes<N>` deliberately
accepts `[0]` and `[0,1]`; the numeric consumer must reject them using the boundary
check above. Do not silently strip leading zeroes from received evidence.
The canonical unsigned forms are `[]` and `[1]` respectively. If an application
needs this semantic check inside Deserialize, it can own a checked newtype and
matching browser check; the generic byte primitive must retain its opaque meaning.

Native frame byte/node/depth/allocation budgets apply before the owned byte-buffer
visitor copies a value; `Bytes<N>` checks its field bound before allocating the
owned buffer. Truncated/nonminimal length prefixes and oversized fields decode
as `Error::Malformed`; exhausted operation budgets report `Error::Limit`.
`Bytes::new` reports `Limit` for an oversized outbound value. The consumer
conversion above reports `Malformed` for a nonminimal numeric magnitude.
Browser frame/node/depth and field-byte bounds apply before materializing an
oversized field; browser limits do not claim to meter JavaScript engine heap.

Native endpoints need only the existing no_std + alloc byte codec. The metadata
and generated-browser machinery remain host-only; using a Nat at an external
boundary does not introduce Candid parsing into the native Wire payload.
Consumers retain the original frame for retry/signature purposes. This convention
does not reinstate whole-frame canonicalization or change normalization semantics.
