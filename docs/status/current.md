# Current handoff

## Baseline and pending batch

0.6.12 is released at `b17c077`; current base `f470bcd9` has commit subject 0.6.13
but package versions/changelog remain pending 0.6.13. Incoming Shared Tooling
0.26.10 adoption is preserved. Versions, staging, commits, publication and delivery
remain maintainer-owned. No sibling sources were changed.

`ic_wire::ByteBuf` now re-exports serde_bytes::ByteBuf for owned binary fields.
serde_bytes is an alloc-only normal dependency; JSON metadata stays native-only.
The disposable consumer uses the public type with only ic-wire and Serde, tests
native builds with/without schema forwarding, and checks both Wasm graphs plus
Rust 1.91. ByteBuf generates Uint8Array metadata and retains existing bytes and
bulk admission. No new serializer or ordinary-Vec rewrite is added.
The [matched ByteBuf qualification](../evidence/bytebuf-0613/README.md) records
equal total/code Wasm sizes in both profiles and 1,100 unchanged IC counter pairs.
Opt-3 binaries are identical; opt-z differs in layout but not measured costs.

Bulk fields are now the default guidance for new binary and large homogeneous
integer/hash declarations. The shipped quickstart combines Bytes, Packed samples,
Packed hashes and ordinary labels. Its doctest passes with/without TypeScript;
three matching mixed-DTO vectors bring native projection conformance to 20 cases.
The guide fixes one representation per field and covers bounds, typed-array
clients and coordinated adoption. Existing ordinary-vector controls are retained.
This batch changes guidance/qualification only: runtime/generator implementations,
existing protocol bytes and admission are unchanged. Prior measured bulk evidence
supports the recommendation; no new performance or consumer-migration claim.

## Latest comparison

The [comparison-scan study](../evidence/hotspot-encoding-0613/comparison-scan/README.md)
locates large-blob overhead in exact byte scanning rather than serializer traversal.
Two exact alternatives regress IC costs for tiny Wasm savings; neither ships.
Five mixed-record cases bring the corpus to 87. All 7,616 matched pairs retain
input/frame identities and three equal samples; mixed records confirm the losses.
Corpus roundtrip/damaged-frame checks, strict Clippy and candidate comparison unit
tests pass. Runtime/protocol are unchanged. The earlier
[block-size screen](../evidence/hotspot-encoding-0613/decode-followup/README.md)
also rejected larger blocks because small blobs regressed.

The [hotspot follow-up](../hotspot-encoding-0613.md) retains a direct i16 varint
writer after ten 50-case IC screens and a matched 298-case standard rerun on the
current Tooling 0.67.0/Platform 0.6.14 lock. Wide-list encoding falls 53.68% at
opt-z and 34.28% at opt-3; small-list encoding falls 65.54%/57.33%. Standard
Wasm total/code grows 370/373 bytes and 673/673 bytes respectively. Ordinary
1/100-record and packed-record probes are unchanged. Native timings are separate;
opt-z wide-i16 decode regresses 17.91%, and standard small-u16 opt-z IC encode
regresses 18.17%. All measurements and small controls remain archived.

The ordinary wide-i16 opt-3 gap is still 32.82x Candid; generic 1 MiB Vec<u8>
encoding remains 37.23x. The existing byte-buffer hook instead costs 0.50x
Candid with identical Wire bytes, but bulk admission differs. No implicit field
rewrite is added. Exhaustive i16/Postcard byte and budget parity, runtime/borrowed
tests, TypeScript conformance, hotspot Clippy and Rust 1.91 Wasm checks pass.
Framing, schemas, payload bytes, acceptance, limits, hashing and retries are
unchanged. This is a small runtime improvement, not bulk parity.

The [expanded matrix](../full-comparison-0613.md) freezes the working tree based
on f470bcd9: 21 cohorts, 1,101 overlapping case entries, both profiles, 84 Wasms,
4,404 primary encode/decode pairs, 23,366 IC rows and 75,328 native/instrumented
rows. All artifact sizes/hashes and cross-mode/profile input/frame identities
validate. All three samples per IC row are identical. Raw source, samples,
parameters and analysis scripts are archived with the report.

Wire total/code Wasm is smaller in all 42 matched cohort/profile pairs.
Opt-3 100 ordinary records: 424,179 total / 389,650 code bytes versus Candid
2,081,252 / 2,012,071. Mixed-bulk 100 records: 655,798 / 620,658 versus
2,639,766 / 2,570,257. These are whole probes, not installed application savings.
All 840 generic packed-field/record IC pairs favor Wire. Standard opt-3 wins
232/298 encodes and 236/298 decodes. Ordinary wide-i16 encode remains 49.94x
Candid, generic 1 MiB Vec<u8> encode 37.23x, signed-byte lists 8.61x at 4,096
items, and owned byte-oriented 1 MiB blob decode 2.37x. Opt-z ordinary wide-u16
encode reaches 123.85x. Packed fields do not change normal vectors or support i8.

The outer script was edited during execution, causing a tail-parse error after
all 21 cohort commands completed. Frozen measurement inputs/helpers were unchanged;
all datasets validate. Corrected runner syntax and all 21 dispatches/final exit
pass with substituted measurements in a disposable Git fixture. No second full
measurement run occurred. Reproduce with make evaluate-codec-matrix and an
explicit prepared POCKET_IC_BIN; retain a fresh output directory.

Expanded standard/bulk roundtrip, selected-index and damaged-frame tests pass;
Clippy with IC features and Rust 1.91 Wasm checks pass. Native time and memory
remain separate from IC counters; no gzip objective, broad CI, installed-consumer
or real big-endian claim. That matrix batch changes fixtures/runners/docs only;
production runtime, protocol, acceptance, canonical hashing and retries are unchanged.

## Native schema feature and tooling

The pending [#17](https://github.com/dragginzgame/ic-wire/issues/17) fix gates
metadata dependencies/module/constants and derived impls to native targets.
Forwarding a schema feature on Wasm retains runtime Serde/codec support without
Wire's optional JSON dependency. The alloc-only byte dependency is now runtime
support. Consumer-owned metadata calls,
manual impls and direct JSON dependencies still need their own target guards.
The maintained disposable consumer check covers native metadata/exact frames,
normal Wasm graphs with/without forwarding, and Rust 1.91 forwarded Wasm.

Shared Tooling commit `4c0f198e` verifies 106 payloads and adds stderr-only Cargo
setup phases/timings while preserving stdout, failure status and tool identities.
The clean committed source has VERSION 0.26.10 and a 0.26.11 changelog heading;
the snapshot correctly records 0.26.10. The producer's uncommitted VERSION edit
was not exported. Common external IC guards and existing selections remain.

IC Build/Host/Testkit are selected at 0.67.0; the release-editor CLI is 0.67.0.
Only the five Tooling packages changed in Cargo.lock; Platform stays 0.6.14.
Wire does not use the archive/configuration constructors that gained required
arguments in 0.67. Diagnostic checks, qualification and driver/floor builds pass
without caller changes. A fresh one-record PocketIC smoke passes Wire/Candid
in both profiles with Testkit 0.67.0; tools, formatting, boundary checks and the
disposable Make/release-adapter fixtures pass. The setup telemetry/status fixture
passes. The new editor is prepared and admitted. Full comparison
figures above remain pinned to their archived Host/Testkit 0.66.3 source/lock;
do not combine them with adoption smoke checks. Production code/protocol are
unchanged, so no runtime optimization benchmark is claimed for this tooling batch.

## Production contract and previous qualification

Keep one production Serde/Postcard codec through ic_wire::native. Serde owns
serialization; Wire generates native host metadata without field IDs. Method IDs
and Limits are explicit; fields/variants follow declaration order. Coordinate
layout changes and retain exact same-release retry bytes. No legacy lane or
migration shim is maintained. Packed<T> is an explicit six-integer/hash field;
PackedI16<N> retains its tighter bound. See [runtime](../runtime.md) and the
[packed guide](../packed-fields.md). Storage semantics remain consumer-owned.

[Packed qualification](../packed-0612.md) records 4,144 boundary checks and the
rejected native alignment/TypeScript tiny-array screens. The runtime fuzzer
restores 112 deterministic seeds; 100,000 address-sanitized inputs passed with
leak checks disabled for ptrace. Rust endian helpers and 48 simulated big-endian
TypeScript cases pass; real big-endian execution remains unqualified.

## Consumer boundary

[Wire-side Blob qualification](../packed-consolidation-0611.md) covers hashes,
headers, owned roundtrips and limits. The maintainer selected Wire-side work only;
consumer adoption/installed measurement is still owned there. Canic owns endpoint
conversion, IDs, budgets, role selection and installed acceptance. Required
external principals remain on the Candid boundary. Runtime stays SDK-independent.
