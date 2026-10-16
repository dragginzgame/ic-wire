# Initial payload screen

Measured locally on 2026-10-16. This evaluates the sharing hypothesis; it does not
select a codec or implement a production protocol. The runtime library still has
no dependencies or public API. All candidate dependencies belong to a host-only
example in the existing unpublished package.

## Result

Payload bytes for identical synthetic values:

| Fixture | Candid | Postcard | CBOR | Fixed comparator |
| --- | ---: | ---: | ---: | ---: |
| Small request | 30 | 2 | 27 | 9 |
| Small response | 37 | 4 | 25 | 16 |
| Nested authority | 129 | 78 | 138 | 108 |
| Blob (4096 bytes) | 4127 | 4100 | 4114 | 4108 |
| Relay batch | 1251 | 1203 | 1282 | 1228 |
| Relay ACK | 63 | 7 | 33 | 37 |
| Schema artifact | 97 | 39 | 133 | 60 |

Postcard is smallest for all seven fixtures. Its advantage over Candid ranges
from 27 bytes (0.65%) on the blob to 56 bytes (88.89%) on the ACK; the request
saves 28 bytes (93.33%). These are payload differences, not execution or
code-size savings. CBOR exceeds Candid on authority, batch and schema fixtures.
The simple fixed comparator never beats Postcard here.

The schema artifact supports further investigation of a shared message/schema
encoding: it needs no database machinery to serialize. It does not establish
that switching IcyDB would reduce maintenance, execution cost or deployed Wasm.
The [source assessment](shared-codec-evaluation.md) still governs the distinction
between shareable serialization and database-owned persistence/key contracts.

Do not extract IcyDB's codec or adopt one universal format on this evidence.
Continue with a bounded Postcard-based experiment only if explicit wire field
identity and the complete developer experience can be demonstrated. An executable
test shows that reordering same-typed fields silently swaps their meaning under
ordinary Postcard derives. Successful decoding and canonical re-encoding cannot
detect this. The fixed comparator has an explicit handwritten field list, but no
validated schema identity mechanism. Neither is a selected IC Wire contract.

## Reproduction and parameters

```sh
make evaluate-codecs > /tmp/codec-screen.json
cargo test --locked -p ic-wire --example codec-evaluation
```

The [saved JSON](evidence/codec-screen.json) records 28 lengths and payload SHA-256
values, semantic JSON hashes, source hashes, lockfile hash and observed compiler.
[Fixture source](../crates/ic-wire/examples/codec_evaluation/fixtures/mod.rs)
freezes all values, principal bytes, ordering and the deterministic blob pattern.
The test suite checks the saved payload vectors and semantic hashes. These JSON
hashes identify evaluation inputs; they are not a production hashing contract.

This run used Rust 1.99.0 (b940084d7), x86_64-unknown-linux-gnu, default dev profile,
no custom compiler flags, native execution and the committed-to-be lockfile.
The executable queries `rustc -vV` at execution; it is observed toolchain metadata,
not embedded build provenance. Reproduce with the supplied Make target in this
workspace using its pinned toolchain; a separately copied executable cannot
certify its build compiler. No timing was sampled. Cargo.lock fixes transitive
versions; root/child manifests fix features. Candidate settings:

- Candid 0.10.38: default features, one typed argument, decoder quota 100,000,
  skipping quota 1,000, maximum type length 64 and header 1,024 bytes.
- Postcard 1.1.1: default features disabled, `alloc` enabled, ordinary Serde
  representation, no COBS framing, compression or application envelope.
- Ciborium 0.2.2: default features, named struct maps, byte strings for blobs,
  recursion limit 16. This is its serializer output, not a claim of compliance
  with a standardized canonical-CBOR profile.
- Fixed comparator: independently authored finite layout, big-endian fixed-width
  integers, bool 0/1, u32 lengths for text, blobs, principals and lists. Record
  field order is explicit. No row header, MAC, storage key or IcyDB framing.

All candidates receive the same bounded shapes: wire limit 65,536 bytes, text
64 UTF-8 bytes, blobs 4,096 bytes, and schema-specific list bounds (4, 8 or 32).
Principal values use ic_principal 0.1.5. Candid and Ciborium declare Apache-2.0;
Postcard declares MIT OR Apache-2.0 in the reviewed crate metadata. See their
primary documentation: [Candid](https://docs.rs/candid/0.10.38/candid/),
[Postcard](https://docs.rs/postcard/1.1.1/postcard/) and
[Ciborium](https://docs.rs/ciborium/0.2.2/ciborium/).

## What is and is not proved

Every fixture undergoes repeated byte-identical encoding, typed decoding and
strict re-encoding equality. The harness rejects remaining bytes. Tests cover
every truncated prefix of the small request, appended bytes, oversized outer
input, invalid fixed bool, nonminimal Postcard integer, hostile sequence lengths,
excess CBOR indefinite-sequence elements, and oversized fixed blob lengths.
String/blob wrappers check their bounds before copying; list visitors reject
oversized size hints before reserving their finite schema bounds.

These checks do not prove arbitrary hostile-input safety. Dependency-internal
allocations, principal decoding, aggregate work/memory budgets, nested attack
coverage and fuzzing still need qualification. Errors have typed harness
categories, but dependency-specific malformed reasons are collapsed to Decode;
this is not an accepted public error taxonomy. Encoding is limited to the trusted
finite corpus: the output-length check occurs after encoding and does not supply
an allocation budget for arbitrary user-written serializers.

The re-encoding check establishes equality to this implementation's output for
these values. It does not supply stable schema/method identity, map-order rules,
float normalization, a signed/hashable production representation, or retry
storage. No transport, exact-byte retry lifecycle, endpoint/client integration,
custom derive, matched Wasm build, IC instruction count, native timing,
allocation measurement or compressed deployment measurement is included.

Focused Linux validation passed: six example tests, Clippy with warnings denied,
formatting, the Make/hook fixture, dependency-free normal graph, and library
Wasm/MSRV checks. Those library checks qualify only the scaffold. The evaluation
has not been executed on macOS; hosted CI remains responsible for that coverage.
