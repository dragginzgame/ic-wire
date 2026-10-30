# Current Candid comparison

IC Wire has one production Serde/Postcard codec. Wire derives host layout
metadata; Serde owns serialization. `Packed<T>` is an explicit field representation.
Candid and PocketIC remain in the unpublished benchmark package.

The [30 October full comparison](evidence/candid-comparison-0725/README.md)
measures current 0.7.24 source for pending 0.7.25 qualification. It covers all 21
maintained cohorts plus the focused hotspot corpus: 1,208 case entries across
both optimization profiles, with IC instructions, frame bytes, repeated native
timing, allocation/heap measurements and uncompressed Wasm/code-section sizes.
Production code and protocol are unchanged by this benchmark refresh.

## Main results

Opt-3 whole benchmark modules, in bytes:

| Fixture | Wire total / code | Candid total / code |
| --- | ---: | ---: |
| Standard, 298 cases | 573,508 / 518,796 | 1,409,623 / 1,329,553 |
| Toko-shaped, 10 cases | 103,841 / 93,308 | 609,004 / 575,323 |
| 100 ordinary derived types | 398,739 / 364,245 | 2,106,847 / 2,036,736 |

These are complete benchmark modules, not projected net savings in an application
retaining external Candid endpoints. The report includes every cohort and profile.

The standard corpus favors Wire on **233 of 298 encodes and 277 of 298 decodes**
at opt-3. A byte-oriented 1 MiB blob encodes in **1,049,704 instructions** versus
Candid's **2,115,234**, and its owned decode uses **1,049,508** versus **1,068,979**.
The Wire frame is two bytes larger: 1,048,590 versus 1,048,588 bytes.
Borrowed decoder counters start with an already-owned input slice; they do not
include bringing ingress into Wasm memory.

Large ordinary vectors remain explicit weak cases: generic 1 MiB `Vec<u8>` encode
costs **37.23× Candid**, and 16,384 wide i16 values cost **32.82×**, at opt-3.
At opt-z, ordinary wide u16 encode reaches **119.63×**. These are current
representation comparisons, not evidence that this benchmark refresh introduced
a regression.

Byte-oriented fields avoid generic byte-vector work. For the matched wide-i16
case, bounded packed Wire uses **33,489.25 encode / 33,476 decode instructions**
versus Candid's **80,320.25 / 1,411,787**, with two more frame bytes. The 109-case
generic Packed cohort wins all **436** primary encode/decode instruction
comparisons across both profiles. Packed fields can expand small values relative
to ordinary Wire varints. Select representations for the contract and workload;
coordinate existing schemas and preserve retry bytes. See the
[field selection guide](packed-fields.md).

Malformed-input cost is separate: at opt-3, rejecting a truncated 1 MiB ordinary
`Vec<u8>` costs **42,472,926 instructions** versus Candid's **4,289**. The
byte-oriented blob counterpart costs **636** versus **4,290**. Ordinary sequences
can traverse a long valid prefix before discovering truncation; byte-oriented
fields can reject an unavailable byte range before copying it. The full report
keeps these rejection losses visible alongside successful-codec results.

## Measurement boundaries

The current run locks Candid 0.10.39, Postcard 1.1.3 and Platform 0.8.3, using
Rust 1.99.1 and an explicit PocketIC 16.1.0 server admitted by Testkit 0.70.4.
Each IC sample uses a unique caller to avoid query-response caching. The original
cached-query controls and fresh-query audit are retained with the same artifacts.
The previous 0.7.10 study used different dependencies and server versions; this
refresh is not a matched previous-release regression test.

`make check-benchmarks` checks correctness, lint and compiler-floor support.
It does **not** execute a performance regression gate. Use an explicitly prepared
`POCKET_IC_BIN` with `make evaluate-codec-matrix`, plus the separate hotspot run,
for the full comparison. See [development](development.md) and the report's
[retained evidence](evidence/candid-comparison-0725/README.md) for reproduction.

Native time, codec-body IC instructions, allocations/heap, frame bytes, total raw
Wasm and code-section bytes are separate measurements. Gzip is not an optimization
goal. Hash-array Candid fixtures retain their nested schema; flat byte-oriented
manifest controls are also measured. Resource policies and schema-evolution
guarantees differ. Borrowing/reuse lanes are labelled separately, and container
roundtrips do not qualify every Rust collection for generated TypeScript metadata.
This is codec qualification, not endpoint authentication, latency or deployment proof.

The separate [Wire/Candid/Tripwire ingress study](evidence/tripwire-comparison-0725/README.md)
measures complete physical updates, including input copies, selected business
work, replies and cycle debits. It uses a common pinned nightly for those three
libraries and a focused blob shape. It is not the broad matrix above or a complete
SDK qualification; do not subtract absolute values across the two studies.

## Historical evidence

- [0.7.10 full comparison](full-comparison-0710.md): the prior full matrix and
  matched 0.7.9 regression qualification, including normalized dependency controls.
- [One-pass decoding cut](evidence/one-pass-070/README.md): the released change
  transferring collection/custom-Serde normalization to consumers.
- [Expanded 0.6.13 matrix](full-comparison-0613.md) and
  [hotspot follow-up](hotspot-encoding-0613.md): earlier decoder and encoder results.
- [Packed promotion](packed-promotion-0611.md), [design study](bulk-fields-0611.md)
  and [sharing study](packed-sharing-0611.md): field representation decisions.
- [Earlier comparisons](history/candid-comparison.md): retired finite-codec and
  rejected experiments, reproducible only at their recorded source revisions.
