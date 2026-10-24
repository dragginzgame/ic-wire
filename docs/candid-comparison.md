# Current Candid comparison

IC Wire has one production Serde/Postcard codec. Wire derives host layout
metadata; Serde owns serialization. Packed<T> is an explicit field representation.
Candid and PocketIC remain in the unpublished benchmark package.

## Pending 0.7.0 decoding cut

The [one-pass qualification](evidence/one-pass-070/README.md) records the approved
normalization contract change, matched 87-case IC/native results and 100-record
Wasm sizes. Earlier full matrices below describe the old decoding contract; their
encoding results remain historical controls, not a rerun of the new decoder.

## Which report to use

| Question | Evidence |
| --- | --- |
| What changed in the largest instruction losses? | [Ordinary-list hotspot follow-up](hotspot-encoding-0613.md): matched i16 improvement, residual gaps, byte-buffer selection and measured regressions |
| What is the latest full comparison? | [Expanded 0.6.13 matrix](full-comparison-0613.md): 21 cohorts, 1,101 overlapping case entries, both profiles and all cost metrics |
| How was public Packed qualified? | [Promotion qualification](packed-promotion-0611.md): prototype/public equivalence |
| What was the previous broad baseline? | [0.6.11 matrix](full-comparison-0611.md): thirteen cohorts and 597 overlapping case entries |
| Why choose runtime-limited packed fields? | [Design study](bulk-fields-0611.md): ordinary vectors, const bounds and selected prototype |
| Why are ordinary vectors expensive? | [Cost breakdown](cost-breakdown-0611.md): admission, preflight, canonical checking and raw Postcard controls |
| What did bounded decoder sharing save? | [Packed sharing](packed-sharing-0611.md): exact limits across many DTOs |
| What happened to native/TypeScript timing follow-ups? | [Qualification and rejected screens](packed-0612.md): layout sensitivity and tiny-array tradeoffs |
| Which field should consumers select? | [Packed guide](packed-fields.md) and [runtime contract](runtime.md) |

## Full-matrix baseline

The subsequent [hotspot follow-up](hotspot-encoding-0613.md) retains a
byte-identical i16 writer. Its matched standard rerun reduces wide-i16 opt-3
encoding from 49.94x to 32.82x Candid, with 673 additional total/code Wasm bytes.
It records regressions and leaves the generic-byte-vector gap unchanged. The
figures below describe the full matrix before that optimization.

Wire total/code Wasm is smaller in all 42 matched cohort/profile pairs. At opt-3,
100 ordinary records use 424,179 total / 389,650 code bytes versus Candid's
2,081,252 / 2,012,071. The 100 mixed-bulk records use 655,798 / 620,658 versus
2,639,766 / 2,570,257. These are whole fixtures, not installed application savings.
Applications retaining external Candid endpoints still retain their required code.

All 840 generic packed-field/record IC encode/decode comparisons favor Wire.
The 298-case standard corpus wins 232/298 encodes and 236/298 decodes at opt-3.
Ordinary wide-i16 encode still costs 49.94x Candid, generic 1 MiB Vec<u8> encode
37.23x, signed-byte list encode 8.61x at 4,096 items, and owned byte-oriented
1 MiB blob decode 2.37x. Ordinary wide-u16 encode reaches 123.85x at opt-z.
Packed does not implicitly alter normal vectors and does not support i8 today.

Small packed values can expand payloads relative to ordinary Wire varints.
Select byte-oriented fields for binary data, and packed words where their fixed
encoding suits the contract. Coordinate schemas/clients and preserve retry bytes.
Runtime container benchmarks do not establish automatic TypeScript metadata
support for every Rust collection. Borrowed/reuse lanes remain separately labelled.

## Measurement and reproduction

The archived run uses Platform 0.6.12; a concurrent 0.6.14 lock update was preserved
but was not rebuilt for these figures. Use the report's exact source/lock pins.

Use `POCKET_IC_BIN=/prepared/server make evaluate-codec-matrix` for the full run;
see [development](development.md) and the report's frozen source/runner evidence.
Compare identical inputs, compiler, lock, flags, target, profiles and limits.
Keep native time, IC instructions, allocations/heap, frame bytes, total raw Wasm
and code-section bytes distinct. Gzip is not an optimization objective.
Hash-array Candid fixtures retain their nested schema; flat byte-oriented manifest
controls are also measured. Resource policies and evolution guarantees differ.

The full report records an outer-runner tail error after all cohorts completed,
its cause and corrected-runner checks. All measurement data independently validate.
No broad CI, installed endpoint, application-latency or new protocol claim follows.

## Historical evidence

[Earlier comparisons](history/candid-comparison.md) retain finite-codec and rejected
experiments. Their pending/current labels belong to their recorded source, not
today's API. The serializer-generating finite Wire API is retired; old commands
must run against their pinned revision.
