# Consumer contract qualification and inspection

The 0.2.3 work adds source-matched IcyDB success/error projections, deterministic
Rust/TypeScript mutation checks, exact frame sizing and an offline packet viewer.
It does not change sibling repositories or replace their adapters.

The native API now also checks the source-matched Count/Grouped payloads against
the independent Postcard oracle. This remains a subset fixture, not a rerun of
IcyDB’s real endpoint/browser suite.

## IcyDB source boundary

Read-only inspection on 2026-10-17 observed IcyDB HEAD
`27035c56c02641a84626b8631e843a2a69e85624` with substantial working-tree changes.
The inspected files below are identified by SHA-256, not attributed to that commit:

| IcyDB path | Observed SHA-256 |
| --- | --- |
| `crates/icydb/src/db/sql/types.rs` | `d12fd6462def91951f593382221613042d3279da07df44406a132bdce150534c` |
| `crates/icydb/src/error.rs` | `379874b33c9cefaddd48b20dc5c87fa797a52eafdfd6ce145dd277076c5cb634` |
| `crates/icydb/src/wire/mod.rs` | `157bd1211587285b2b5cd7348b96250d1adb767dac3c5d0ba79177fb25ecf07c` |
| `crates/icydb-schema/src/wire/mod.rs` | `6c999f927ceea7ff3a31e9ce638c20516031adda7c8642836cb91b731aa5982f` |

The local [consumer fixture](../crates/ic-wire/examples/typescript_conformance/consumer/mod.rs)
reproduces the native SQL string, Count and Grouped success payload layouts. It
checks bytes against independent ordinary-Serde producers with the observed native
variant ordinals: Count=0, Projection=1, Grouped=2. The unused Projection oracle
slot preserves the ordinal; it does not implement that result shape. The grouped
fields are entity, columns, rows, row_count and next_cursor, in native order.
Display cells remain strings: SQL NULL and quoted marker-like text are distinct.

These fixtures use explicit IC Wire IDs and local static bounds. They prove
payload equality for the selected values and typed Rust/TypeScript agreement;
they are not the whole IcyDB endpoint protocol. The conformance method identity
is local. The bounded public-error projection preserves code, class, origin,
facts and query_field byte order against an independent tuple producer. Its raw
numeric samples test layout, not the validity of IcyDB diagnostic code meanings.
No actual IcyDB crate is linked, built or benchmarked by this fixture. The earlier
installed SQL-shaped matrix remains synthetic performance evidence.

Conversion demonstrates field/index-specific failures before oversized replacement
collections are allocated. A fifth grouped row produces a pagination-directed
error; excessive UTF-8 cell bytes identify the offending row and cell. Pagination,
row-count consistency and cursor authentication remain IcyDB decisions. Do not
truncate rows or reinterpret display strings to make an oversized value fit.

IcyDB's inspected native Payload adapter separately budgets Serde traversal and
supports recursive/native values. Its metadata is opaque. IC Wire's TypeScript
generator cannot project that adapter; bounded named DTOs or explicit native
metadata are still needed for generated browser clients. This batch does not
promote the native adapter into IC Wire or infer static bounds for arbitrary Serde.

The inspected `serve_raw` path rejects response-encoding failure after the handler
returns. A mutating consumer must review that failure contract: an explicit reject
does not roll back local effects. The [local macro proof](macro-adoption.md) traps
on that failure and tests rollback. Its installed query/sync/async update and 16
rejection cases were requalified with this batch. External-await recovery remains
a separate consumer obligation.

## Differential mutation checks

`make check-typescript` now checks 87 vectors in both directions and 17,320
mutations. Each vector gets low/high-bit mutations at every byte and fixed-seed
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

The existing macro fixture supplies the compiled local Canic expansion pattern;
production Canic changes remain with that repository. Full IcyDB projections,
recursive browser metadata, coverage-guided fuzzing and actual consumer performance
still need their own qualification. Keep storage, keys, journals and authenticated
cursor formats with their current owners. No compatibility lane is introduced.
