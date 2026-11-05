# Standard byte-array keys: matched qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Measured 1 November 2026 on Core/derive 0.13.2 plus pending 0.13.3 changes.
`ic_wire::ByteArray<N>` re-exports the standard `serde_bytes` type and adds host
metadata, using the existing production Serde/Postcard codec. This is an explicit
field representation, not a codec mode or an automatic rewrite of ordinary arrays.

**Keep the option for hot binary keys.** For a 128-entry map at opt-3 it reduces
encode instructions **78.50%** and decode instructions **68.32%** versus ordinary
array keys. Both operations beat Candid. It increases the fixture's uncompressed
Wasm **1,526 bytes (2.11%)** and frame **128 bytes (2.02%)**. At opt-z the module
increases **1,990 bytes (3.34%)**. This does not reduce Wasm size; consumers near a
code limit should weigh that measured cost against the instruction improvement.
Application linker impact and multiple different array widths are not measured.

## Inputs and contract

The focused cohorts encode actual `BTreeMap<Key, Option<Blob>>` values at lengths
0, 1, 16 and 128. Each key is `[0x95;32]` with its first byte replaced by the
entry index. Even entries contain a deterministic 32-byte blob; odd entries have
no blob. Input setup and semantic equality checks are outside measurement.

- Control: Wire key `[u8;32]`.
- Candidate: Wire key `serde_bytes::ByteArray<32>`.
- Candid: the identical original fixed-array-key map in **both** cohorts.

Ordered-pair JSON identifies logical inputs only; the measured values remain maps.
The analyzer checks identical logical-input hashes, Candid frame hashes and the
expected one-byte-per-key increase for Wire. Empty Wire frames are identical.
Both Candid cohorts have identical counters and sizes here, although artifact
hashes differ. The original 333-case corpus and its array-key controls remain
unchanged. Do not subtract this focused control from counters in the larger
[full matrix](../candid-comparison.md): linked contexts differ.

Ordinary `[0_u8,255]` retains payload `00 ff`; explicitly selecting `ByteArray<2>`
produces `02 00 ff`. Its schema is `{"k":"bytes","min":2,"max":2}` rather than
an ordinary two-element array. The standard Serde adapter on `[u8;N]` already
uses length-prefixed bytes; Wire can now describe it, including optional fields.
New metadata does not change existing Serde serializers or decode rules.

Rust and TypeScript enforce exactly N bytes. ByteArray uses bulk byte admission,
so adopting it changes budgets as well as bytes: coordinate contracts, regenerate
clients and qualify limits. Retained retry/hash inputs must not be reconstructed
under the new layout. No legacy lane, transport, endpoint or normalization rule
is added. Browser map semantics remain consumer-owned; the conformance fixture
qualifies the explicit ordered-entry-pair schema against matching map bytes.

## IC instructions and wire size

Absolute counts per operation; changes are relative to the ordinary-array control.
Each row retains three identical samples from fresh callers. Result destruction
and loop overhead are included; loop controls are retained and not subtracted.

| Profile | Entries | Encode before → after | Change | Decode before → after | Change |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z | 0 | 1,283.5 → 1,293.5 | +0.78% | 491.5 → 490.5 | −0.20% |
| opt-z | 1 | 4,902.5 → 3,052.5 | −37.74% | 6,686.5 → 1,815.5 | −72.85% |
| opt-z | 16 | 53,455.5 → 23,185.5 | −56.63% | 100,989.5 → 22,627.5 | −77.59% |
| opt-z | 128 | 418,204.5 → 175,974.5 | −57.92% | 892,485.5 → 266,594.5 | −70.13% |
| opt-3 | 0 | 428.25 → 430.25 | +0.47% | 375 → 393 | +4.80% |
| opt-3 | 1 | 3,037.25 → 1,270.25 | −58.18% | 5,082 → 1,356 | −73.32% |
| opt-3 | 16 | 36,672.25 → 8,317.25 | −77.32% | 77,852 → 17,942 | −76.95% |
| opt-3 | 128 | 288,996.25 → 62,121.25 | −78.50% | 699,614 → 221,623 | −68.32% |

Small empty-map regressions remain visible: the opt-3 decode adds 18 instructions.
At 128 entries, opt-3 Candid uses **87,093.25 encode / 1,352,002.5 decode**;
opt-z Candid uses **221,355.5 / 1,667,902**. Candidate Wire beats Candid on all
16 successful comparisons across these four cases and two profiles. Control Wire
wins 14/16. This is not a universal performance guarantee for other maps or widths.

| Entries | Ordinary Wire bytes | ByteArray Wire bytes | Candid bytes |
| ---: | ---: | ---: | ---: |
| 0 | 12 | 12 | 20 |
| 1 | 78 | 79 | 87 |
| 16 | 804 | 820 | 828 |
| 128 | 6,349 | 6,477 | 6,485 |

Bad-header rejection is unchanged: 238.5 instructions at opt-z, 236.25 at opt-3.
At 128 entries truncated-input rejection improves from 890,593.5 to 264,878.5
at opt-z (−70.26%) and 698,007.25 to 220,145.25 at opt-3 (−68.46%). All sizes,
rejections and Candid controls are retained in Retired raw output (see retention notice),
Retired raw output (see retention notice) and Retired raw output (see retention notice).

## Uncompressed Wasm

Complete matched fixtures with encoding and decoding reachable, not application
linker savings. Candid's unchanged lane is retained in each module selection.

| Profile | Ordinary Wire total / code | ByteArray Wire total / code | Delta total / code | Candid total / code |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 59,546 / 49,385 | 61,536 / 51,372 | +1,990 / +1,987 | 318,006 / 284,812 |
| opt-3 | 72,482 / 62,764 | 74,008 / 64,292 | +1,526 / +1,528 | 502,926 / 470,846 |

All eight modules, hashes, byte counts and independently parsed code sections
are retained in Retired raw output (see retention notice) and Retired raw output (see retention notice).
Compression is evidence storage only. No gzip savings or symbol-based savings
claim is made. Merely adding schema support is host-only; these module increases
measure adopting the byte-array representation in the codec workload.

## Native timing and allocation

Two serial warmed runs of eleven calibrated batch samples per operation, minimum
two milliseconds. Native timing includes result destruction; it is separate from
IC instructions and not request tail latency. The host is shared, not CPU-isolated.

Opt-3, 128 entries, native medians in nanoseconds:

| Operation | Ordinary run 1 / run 2 | ByteArray run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: | ---: |
| Encode | 3,535.73 / 3,594.11 | 844.60 / 781.63 | 1,551.73 / 1,762.01 |
| Decode | 12,987.32 / 13,898.57 | 7,073.59 / 6,057.28 | 30,146.39 / 33,373.67 |

Encode falls 76.11% / 78.25%; decode falls 45.53% / 56.42%. Empty-map native
changes are small and mixed: opt-z encode +0.99% / +0.84%, decode −3.86% / +3.68%.
Do not replace the IC empty-map regression with a native-time conclusion.

Separate instrumented builds measure one warm allocation/heap operation, not
eleven independent allocator samples. At 128 entries Wire allocation counts stay
**1 encode / 85 decode**. Encode peak live bytes rise **6,349 → 6,477** with the
frame; decode peak live bytes remain **15,608**. All raw timing, allocation/heap,
identity and instruction samples are in Retired raw output (see retention notice); the paired
CSV includes both runs and all allocator observations.

## Qualification and reproduction

Frozen source/lock: Core/derive **0.13.2**, Platform **0.15.3**, canonical Types
**0.15.1**, Build/Host **0.73.3**, Testkit **0.75.2**, Metrics **0.24.4**, Candid
**0.10.39**, Serde **1.0.229**, serde_bytes **0.11.19**, Postcard **1.1.3**.
This follow-up freezes the later checkout lock, independently of the full matrix's
Metrics 0.24.3 selection. Both cohorts use exactly the same source and lock.

After preparing the recorded toolchain, target, Cargo cache and matching server:
