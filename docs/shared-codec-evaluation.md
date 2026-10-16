# Shared codec evaluation: IC Wire and IcyDB

Evaluated 2026-10-16. This is a source-based architecture assessment of an
optional direction, not a codec selection or measured performance comparison.

## Recommendation

Keep a shared message/value codec as a candidate in IC Wire's existing
evaluation. Start with typed request/response payloads and an IcyDB-shaped schema
artifact. Do not make IcyDB adoption a prerequisite for selecting IC Wire's
encoding, or commit to replacing IcyDB's persisted formats.

The source supports sharing checked byte operations and potentially a complete
message codec. It does not establish that one format for messages, rows, keys,
journals and cursor tokens would be simpler or smaller. Several differences are
required by the consumers' contracts. The strongest initial benefit hypothesis
is shared bounds enforcement, test vectors and developer tooling. Wasm and
instruction savings remain unmeasured.

This narrows the earlier suggestion that one shared codec is inherently the
stronger destination. Shared ownership is useful only when it removes more
implementation and maintenance work than its adapters and contract coupling add.

## Evidence identity and limits

The reproducible source baseline is IcyDB commit
`e104fe1c9f5921336f0e5b292acbda99f8311476`. Relevant files were exported with
`git archive` into an isolated inspection directory; the links below resolve to
that commit. The sibling checkout has substantial incoming dirty work, including
an untracked bounded writer and changes consolidating schema, journal and cursor
byte helpers. That work was inspected as context only. It is not attributed to
the committed baseline, qualified here, or copied into IC Wire.

This assessment inspected implementations, call boundaries and existing test
source. It did not execute IcyDB, compile an extraction, benchmark codecs, or
change sibling files. IC Wire still has no codec or public API. No claim about
performance, native macOS behavior or no_std compatibility follows from this
source review.

## What can actually be shared

| Boundary | Observed implementation | Assessment |
| --- | --- | --- |
| Schema authoring artifacts | Hand-written tagged binary encoding over bounded big-endian primitives; input limits, explicit counts and recursive depth checks; decoded values are validated and re-encoded to reject noncanonical input. [Sources][schema-codec] [and primitive helpers][schema-wire]. | Best first IcyDB-shaped consumer for the evaluation. Shared encoding can own byte mechanics; schema validation and canonical ordering still belong to IcyDB. |
| Database byte primitives | Borrowed checked reads, explicit endianness, bounded strings and exact-consumption checks in [ByteReader][byte-reader]. The [schema writer][schema-db-wire] bounds output and rejects a sticky overflow at finish. | Real overlap, but these are helpers rather than a general serialization API. Consolidation alone does not establish a substantial product benefit. |
| Persisted rows | An [envelope][row-envelope] carries physical layout identity. A [slot directory][row-contract] supports [lazy, selective slot decoding][row-reader] under an accepted schema contract. | Keep database-owned. A candidate used inside field payloads must preserve borrowing and selective reads without routing every row through an owned generic value tree. |
| Primary/index keys | [Primary-key encoding][keys] transforms signed integers to preserve byte ordering. [Index keys][index-keys] have their own identity, framing and comparison contract. | A generic message encoding cannot be substituted on round-trip correctness alone. Keep outside the initial codec scope. |
| Cursor tokens | [Token framing][tokens] bounds the envelope, checks exact length and authenticates its bytes before decoding the payload. | Shared payload helpers may fit; authentication order, route identity, MAC coverage and token limits remain local. |
| Journals and hashes | [Journal decoding][journal] uses dedicated tags, framing and bounds; [hash streams][hashes] encode domain-owned inputs directly. | Byte reuse is possible only with exact format/hash-vector preservation or a separately authorized contract change. A shared codec does not own recovery or retry policy. |

Two concrete distinctions prevent treating these as interchangeable serializers:

- A schema `i64` is ordinary big-endian two's-complement bytes. Primary keys
  encode the same integer after flipping its sign bit. Thus `-1` has payload
  `ff ff ff ff ff ff ff ff` in the schema primitive, but
  `7f ff ff ff ff ff ff ff` in the ordered key. That difference is intentional;
  both round-trip, but only the latter matches signed numeric order when bytes
  are compared lexicographically. See [schema primitives][schema-wire] and
  `encode_ordered_i64` in [key taxonomy][keys].
- The current row framing contains an 11-byte envelope, a 2-byte slot count and
  8 bytes of directory per slot, before slot payloads: `13 + 8 × field_count`.
  This is a source-derived layout formula from [row emission][row-contract],
  not a measured overhead comparison or savings estimate. Its directory serves
  selective reads; imposing it on every RPC would require a demonstrated need.

## What extraction would still require

The checked primitives are plausible seeds, but the existing packages are not
drop-in IC Wire dependencies. [icydb-schema][schema-manifest] directly depends
on Candid and domain scalar types; [icydb-core][core-manifest] also depends on the
CDK and database/runtime packages. Their codecs refer to schema or internal
database errors and use `std` paths and owning collections.

A viable extraction would need:

1. A narrow `core`/`alloc` implementation with codec-owned typed failures and no
   database, Candid or CDK dependency. Source inspection suggests this is feasible
   for the primitive helpers; compilation and target tests are still required.
2. One explicit wire specification for field/variant identities, numbers,
   containers and canonical bytes. Supporting both existing big- and little-
   endian database primitives does not justify configurable IC Wire formats.
3. Decode budgets for total bytes, elements, nesting and allocation. Current
   primitive checks and schema-specific limits are useful examples, but they
   are not a general compositional resource budget. In particular,
   `WireReader::read_count` assumes every collection member consumes at least one
   byte; a generic unit-value sequence must have an explicit bounded contract.
4. A chosen failure contract for writes. The schema artifact writer returns
   errors immediately, while the database schema writer remembers overflow and
   rejects at `finish`. Combining those call sites requires preserving failure
   handling, not simply renaming both writers to one type.
5. Working derives and typed method bindings. IcyDB's hand-written schema codecs
   do not supply the Candid-like developer journey proposed for IC Wire.
6. Consumer-owned normalization and hash domains. Canonical scalar encoding does
   not determine the semantic ordering of schema entities or what belongs in a
   fingerprint. Existing decode/re-encode checks also carry work and allocation
   costs that an alternative must account for rather than silently remove.

Importing the database `Value` model into IC Wire would couple the codec to
IcyDB's semantic types. Creating an additional common-codec crate now would add
another owner before the benefit is established. If the candidate wins, IC Wire
itself is the proposed owner, and future consumers depend on it.

## Comparison and decision gates

| Option | Benefit hypothesis | Cost/risk | Current disposition |
| --- | --- | --- | --- |
| IC Wire selects an existing compact codec; IcyDB keeps its formats | Fastest route to evaluating developer ergonomics and RPC savings | Some byte-level logic remains independently owned | Keep as the control option; shared ownership is not an acceptance requirement |
| IC Wire owns a codec derived from IcyDB's bounded binary approach | Shared bounds, canonical vectors and encoding implementation for suitable payloads | New general type support, derives, errors and budgets; adapters must actually retire duplicated work | Worth a bounded candidate comparison, without presuming it wins |
| Extract only primitive readers/writers | One place to harden byte arithmetic | Small shared surface; consumer serialization remains; new dependency coupling | Do only when a selected codec and actual callers justify it |
| Force all IcyDB persistence and RPC through one format | Superficially uniform API and implementation | Ordered keys, lazy row access, schema authority and recovery contracts expand the design substantially | Do not pursue within this investigation's initial scope |

The [existing evaluation plan](evaluation.md) remains the implementation gate.
Add one small IcyDB-shaped schema artifact to its representative message
fixtures; keep it local here. For the shared-codec hypothesis, compare the
candidate against both the current hand-written shape and the existing-codec
control using identical values and behavior. Count the adapters, annotations
and validation code required by each, as well as encoded bytes.

Require all of the following before recommending cross-repository adoption:

- Compiling typed message and schema-artifact examples using the same selected
  implementation, with exact error/budget behavior and stable wire identities.
- Malformed-length, truncation, invalid-tag/UTF-8, trailing-byte, depth and element
  budget tests; deterministic vectors and exact-byte retry preservation.
- Matched linked Wasm and IC instruction measurements, including artifacts where
  external Candid remains linked. A common crate does not guarantee one machine-
  code instance: generic instantiations and retained dependencies still matter.
- A concrete deletion/adaptation plan showing which consumer implementations
  disappear and which remain. Treat fewer source copies as maintenance evidence,
  not as a compiled-size or instruction result.
- A net benefit without extra format modes, compatibility lanes, storage
  semantics or a larger developer API solely to accommodate IcyDB.

If the existing compact codec wins, use it for IC Wire and leave IcyDB adoption
optional. If sharing only saves a few primitive wrappers, retain independent
consumer formats. Storage adoption would need its own evidence for selective
reads, key ordering, retained bytes and recovery, and separate authorization.
No numeric acceptance threshold or performance claim is introduced here.

[schema-codec]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-schema/src/codec.rs
[schema-wire]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-schema/src/codec/wire.rs
[byte-reader]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/codec/reader.rs
[schema-db-wire]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/schema/wire.rs
[row-envelope]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/codec/mod.rs
[row-contract]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/data/persisted_row/contract.rs
[row-reader]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/data/persisted_row/reader/structural_slot_reader.rs
[keys]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/key_taxonomy.rs
[index-keys]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/index/key/codec/mod.rs
[tokens]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/cursor/token/codec.rs
[journal]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/journal/codec.rs
[hashes]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/src/db/codec/hash_stream.rs
[schema-manifest]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-schema/Cargo.toml
[core-manifest]: https://github.com/dragginzgame/icydb/blob/e104fe1c9f5921336f0e5b292acbda99f8311476/crates/icydb-core/Cargo.toml
