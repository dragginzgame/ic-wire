# Consumer contract qualification and inspection

IC Wire keeps generic bounded/native codec vectors, canonical mutation checks and
the offline packet viewer. IcyDB owns its actual SQL DTOs, domain conversions and
browser endpoint qualification in `testing/build/examples/wire_bindings/` and
`testing/wire-browser/`. Adoption evidence is tracked in
[IcyDB #372](https://github.com/dragginzgame/icydb/issues/372).

The former local Count/Grouped/public-error replicas and their bounded conversion
path are retired. IcyDB's own vectors cover display strings, cursor fields and
oversized-page refusal against actual contracts. Historical 0.2.3–0.3.4 results
remain evidence for their original source, not current endpoint qualification.
IC Wire still tests nested sequences, strings, results, enum identities and native
Postcard equivalence without depending on IcyDB's schema or numeric policy.

## Differential mutation checks

`make check-typescript` reports the current generic vector and mutation counts. Each vector gets low/high-bit mutations at every byte and fixed-seed
xorshift payloads of lengths 0–63 under its valid header. Rust acceptance is
compared with the generated TypeScript decoder. Every accepted mutation must
re-encode to exactly the same bytes in both implementations; rejected Rust input
must produce a typed WireError in TypeScript. Error categories need not have
identical names across the two APIs.

The existing truncation, identity, invalid length/tag/UTF-8, finite-float and
outgoing-value checks also run. This is reproducible bounded mutation testing,
not coverage-guided fuzzing, an exhaustive proof or a new performance result.
Generated cases and scripts stay under `target/typescript-conformance`; failures
identify their generated script location and retain the exact candidate bytes.

## Exact sizing and packet inspection

`encoded_request_len::<M>(&value)` and `encoded_response_len::<M>(&value)` return
complete envelope lengths, including the 11-byte header, without allocating an
output buffer. They enforce the same conservative schema admission and byte
ceiling as encoding. Custom serializers remain trusted and must be deterministic;
the helper cannot prevent allocations inside custom Serialize implementations.
These helpers enable response delivery checks before materializing bytes. Keep
an application size error distinct from a transport failure after a mutation.

Normal encoding still reserves the schema maximum in one serialization pass.
The new size-only pass is opt-in; using it before every encode adds traversal.
No new allocation/performance win is claimed for that choice. Borrowed arbitrary
Serde values and IcyDB's own native size adapter are not replaced by this Wire API.

After generating and compiling a trusted local TypeScript contract:

```sh
node scripts/dev/inspect-wire.mjs path/to/generated.js response HEX_BYTES
```

The viewer imports that module and uses its strict decoder, including method and
direction checks. It never guesses a schema or contacts a canister. Invalid hex,
wrong identity/direction and malformed payloads fail with a nonzero status.
Diagnostic JSON represents big integers as `{"$bigint":"..."}`, bytes as
`{"$bytes":"hex"}` and negative zero as `{"$float":"-0"}`. This is display output,
not another encoding or an input format for contract calls. Only import trusted
local generated modules: importing JavaScript executes that module.

## Remaining adoption work

Endpoint integration and full consumer projections require qualification at their
owner. Keep storage, keys, journals and authenticated cursor formats with their
current owners. No compatibility lane is introduced.
