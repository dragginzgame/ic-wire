# Current Candid comparison

IC Wire has one production Serde/Postcard codec. `Wire` derives host-side layout
metadata, while Serde owns serialization. `Packed<T>` is an explicit bulk field
representation within that codec. Candid and PocketIC belong only to the
unpublished benchmark package.

## Which report to use

| Question | Maintained evidence |
| --- | --- |
| What happened to the native/TypeScript timing follow-ups? | [Qualification and rejected screens](packed-0612.md) |
| How does the public packed field compare? | [Promotion qualification](packed-promotion-0611.md): 68 fields/manifests and 1/10/100 records, both profiles |
| How does the broader ordinary codec compare? | [Full matrix](full-comparison-0611.md): thirteen cohorts and 597 overlapping case entries |
| Why choose runtime-limited packed fields? | [Design experiment](bulk-fields-0611.md): ordinary vectors, const bounds and the selected prototype |
| Why are ordinary large vectors expensive? | [Cost breakdown](cost-breakdown-0611.md): admission, preflight, canonical checking and raw Postcard controls |
| What did sharing bounded decoding save? | [Packed sharing](packed-sharing-0611.md): exact field limits retained across many DTOs |
| Which representation should a consumer choose? | [Packed-field guide](packed-fields.md) and the [runtime contract](runtime.md) |

## Current conclusions

All 716 packed-field/record encode/decode IC comparisons favor Wire over Candid.
Promotion preserves the prototype's IC counts, allocations, frames and total/code
Wasm sizes. This does not imply ordinary Vec fields acquired packed behavior.
In the broader standard corpus, opt-3 Wire wins 161/214 encodes and 155/214 decodes.

For the 100-record packed fixture, opt-3 uncompressed Wire Wasm is 655,798 bytes
(total) / 620,658 (code), versus Candid's 2,639,766 / 2,570,257. Whole-fixture
measurements do not predict an installed consumer's exact footprint.

Small packed integers can expand payloads; small u64 data can grow eightfold.
Native opt-z large-u16 decode regresses about 57% against the unpublished prototype
but still beats Candid. Tiny TypeScript arrays add overhead; large arrays improve.
Ordinary wide-i16 encode remains about 50x Candid's IC cost, and owned 1 MiB blob
decode remains about 2.37x in the broad comparison. Use the relevant report's
inputs and dependency pins rather than combining numbers across unrelated builds.

## Measurement contract

Compare equivalent logical inputs with recorded source, lockfile, compiler,
target, optimization profile, codec parameters and exact frame identities.
Keep native host time, IC instructions, allocations/peak heap, wire bytes, total
uncompressed Wasm and code-section bytes separate. Repeat timing samples and keep
small-input controls and regressions. Gzip is not an optimization objective.
Candid hash fixtures retain their declared nested-array schema; they are not
comparisons against a newly flattened Candid blob.

`make evaluate-codecs` owns the runner; see [evaluation](evaluation.md) for focused
checks and each linked report for its precise reproduction command. Optional IC
measurements require an explicitly prepared PocketIC binary. This does not qualify
application endpoints, authentication, storage migrations or installed clients.

## Historical evidence

The [historical comparison studies](history/candid-comparison.md) preserve previous
finite-codec experiments, rejected variants and their raw evidence. Their "pending"
and "current" labels describe the source revision of that study, not today's API.
The old serializer-generating finite Wire API is retired. Historical commands
must run against their recorded source, not be treated as current production modes.
