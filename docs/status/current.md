# Current handoff

## Released 0.7.18; pending 0.7.19 issue fixes

HEAD remains `c748379325266d7fa614fefee9d745eb4a88acca`. The compatible patch
adds APIs and metadata support; it changes no existing frames, identities,
primitive acceptance, native admission or retained retry/hash inputs. Own package
versions remain 0.7.18. Staging, commits, publication and release remain maintainer-owned.

Both [#29](https://github.com/dragginzgame/ic-wire/issues/29) and
[#30](https://github.com/dragginzgame/ic-wire/issues/30) are implemented locally.
Keep them open until delivery and required producer qualification; SDK adoption
and installed endpoint behavior remain consumer-owned. No consumer checkout was
changed and no remote code delivery is claimed.

## Delivered local behavior

Serde's container `crate = "path"` attribute now works with Wire. Direct and
re-exported paths retain identical metadata tokens, positional fields, variant
indices and frames. Unsupported layout attributes and invalid placement/duplicates
still reject. Rust and executable TypeScript fixtures cover records and enums.

TypeScript adds `createPayloadCodec`; host `generate_payload_registry` and
`payload_limits` share the existing registry validation, traversal, scalar checks
and mandatory bulk advice. Payload unit is empty; u64 7 and `(7u64,)` are `07`.
Framed APIs retain the IW/ID/direction envelope. Payload choice is explicit;
changing an endpoint layout still requires coordinated consumers. Named recursive
schemas, tuple arguments, records/enums, malformed inputs and budgets are qualified.

Rust adds `encode_to_slice`, `encode_request_to_slice` and
`encode_response_to_slice`, using Postcard's slice output through the same bounded
Serde adapter and two-pass sizing/admission. Short buffers reject before writing;
other output-pass failures return no successful slice and leave unspecified
storage contents. Consumers publish only Ok and own scratch size/fallback and async
lifetimes. The result cannot outlive storage. No inline/spill writer was added.

The [SDK recipe](../sdk-integration.md) explains consumer-owned scalar serialization,
checks versus value conversion, typed facades and compiling the shared runtime
into JavaScript. A facade is compiled and executed against generated payload code.

The earlier allocation report remains a documentation/test clarification, not a
unit-conversion fix. A 22-byte response with `bytes: 64` needs allocation 832 to
encode and 704 to decode; 512 rejects and 1,024 admits both. Division by eight
preserves the deliberately weighted traversal ledger. Public regressions and the
independent overflow/remainder reference remain in place.

## Evidence and tradeoffs

The [caller-storage study](../evidence/caller-storage-0719/README.md) retains matched
0.7.18 controls, source/compiler/dependency identities, repeated raw IC and native
samples, allocator/live-heap/capacity data, host generation, JS timing and
uncompressed total/code Wasm sizes. Full local artifacts are under
`target/caller-storage-0719-qualified/`; the earlier failed fixture attempt is
preserved separately and is not qualification evidence.

Existing Vec Wasm is byte-identical before/after in both profiles, with identical
IC counts. Slice output lowers fixture Wasm by 1,151/190 bytes at opt-z/opt-3;
ordinary Vec-4096 encoding uses 6.61%/7.13% fewer IC instructions. Opt-3 unit costs
20 more instructions. Native opt-3 large-blob timing regresses 9.46%; JS framed
unit decode regresses about 21 ns (+19.40%) in the aggregate. No universal speed-up
or installed-canister saving is claimed. Caller storage remains a real memory cost.

## Focused checks and remaining boundaries

Passing checks: 46 native and 16 runtime tests with TypeScript, two derive tests,
nine compile-fail diagnostics plus passing control, nine successful and five
compile-fail doctests, strict library/derive Clippy, and the Rust 1.91 forwarded
Wasm library build. Final generated strict TS/Node conformance passes 165 framed
vectors, 35,228 differential mutations, 29 native projection vectors, new payload
and facade cases, cross-runtime admission and 54 simulated-endian packed cases.
220 existing generated TS modules remain byte-identical; shared runtime changes
and new payload modules are explicit. Measurements run separately from these gates.

Incoming dependency edits remain intact: lockfile Build/Testkit/host 0.68.12 and
Platform 0.6.21; no manifest minimums or managed tooling were adopted in this batch.
Shared snapshot remains 0.26.21 at `66c91c4412856baa147482e648fc80428db8b179`.
An incoming dirty edit in `crates/ic-wire-benchmarks/fixtures/mod.rs` changes
`derive(CandidType)` to `derive(ype)`. It is untouched and would break the Candid
feature's gate; correct it before release validation. Full CI/Candid comparison,
macOS and installed SDK qualification were not run here.

Historical studies remain at [runtime 0.7.16](../evidence/runtime-0716/README.md)
and [encoder follow-up](../evidence/encoder-followup-0716/README.md). Their frozen
dependency graphs do not qualify the incoming graph. Consumers retain ownership
of normalization, original retry bytes, authentication, transport and rollback.
