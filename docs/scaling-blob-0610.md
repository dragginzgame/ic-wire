# Generator names, packed-field scaling and owned blobs (0.6.10)

This batch retains the generator cleanup and packed scaling cohorts. The owned
blob experiment is rejected: its modest instruction saving adds Wasm and slows
small inputs. Production Serde/Postcard encoding, payload acceptance, canonical
hashing and same-release retry bytes are unchanged. No extra production codec,
compatibility reader or automatic consumer-layout change is introduced.

## Generator cleanup

Generated modules now import `createCodec` as `$createCodec` and initialize
`$types` from a registry literal. The existing consumer identifier grammar excludes
`$`, so these private helpers cannot capture consumer exports. `types`,
`createCodec` and `JSON` are accepted declaration names. Own `__proto__` keys use
computed-property syntax, preserving the former JSON.parse behavior without a
global JSON dependency. Quoted keys inside JSON strings remain escaped.
Built-in field/frame types remain qualified through globalThis.

Strict TypeScript compilation covers consumer aliases and method exports with
these names, built-in names and globalThis. Execution checks exercise the newly
allowed method exports and the existing __proto__ registry fixture.

Matched host-generation timings compare released c0841e7 with this renderer,
using one or one hundred named records, each containing bytes, packed i16 and a
u32 list. These are metadata-generation times, not codec time or IC instructions.
Two runs each collect eleven samples (1,000 renders/sample for one type,
20 for one hundred), after twenty warmups, with result destruction included.

| Registry | Before run medians | After run medians | Generated TypeScript bytes before → after |
| --- | ---: | ---: | ---: |
| 1 type | 3.091–3.163 µs | 2.591–2.643 µs | 554 → 507 |
| 100 types | 306.885–308.093 µs | 247.868–253.911 µs | 41,918 → 36,921 |

Paired run medians improve 16.2–16.4% and 17.6–19.2%, respectively. Avoiding the
second JSON string encoding also shrinks this generated source by 8.5%/11.9%.
These observations do not establish TypeScript startup or endpoint latency.

## Packed-field scaling

The existing six-field record roster now also supports packed cohorts, without
copying the hundred record declarations. Each record retains a uniquely named
u64 field, bool, i64, optional u64 and text. Its byte-array field is replaced by
eight i16::MAX values: Wire uses PackedI16 and Candid uses Vec<i16>.
The shared cohort uses bound 16,384 everywhere. The varied cohort cycles through
bounds 1,024 to 16,384 in 1,024-element increments: 1/10/16 distinct bounds at
1/10/100 nominal DTOs. Payload values and frame hashes match across bound choices.
These are deployed Serde/Candid derives; host-only Wire metadata is not linked.

| Types / profile | Shared-bound Wire total / code | Varied-bound Wire total / code | Candid total / code |
| --- | ---: | ---: | ---: |
| 1 / opt-z | 63,089 / 51,196 | 63,087 / 51,194 | 304,802 / 271,198 |
| 10 / opt-z | 85,626 / 73,001 | 101,050 / 88,371 | 398,484 / 362,975 |
| 100 / opt-z | 263,834 / 244,453 | 292,476 / 272,957 | 1,283,379 / 1,229,497 |
| 1 / opt-3 | 93,851 / 66,256 | 93,850 / 66,255 | 449,222 / 400,570 |
| 10 / opt-3 | 121,953 / 93,676 | 145,467 / 117,134 | 656,974 / 606,354 |
| 100 / opt-3 | 393,641 / 358,597 | 437,288 / 402,094 | 2,098,605 / 2,029,665 |

Sizes are complete uncompressed Wasm bytes and code-section payload bytes.
Candid columns use the shared-cohort build; the varied one-type Candid builds
are ten total bytes smaller with identical code-section sizes. Full hashes and
all sizes are retained. These are fixture totals, not symbol-attributed savings.

At 100 DTOs, varied bounds add 28,642 total / 28,504 code bytes at opt-z
(+10.9%/+11.7%), or 43,647 / 43,497 at opt-3 (+11.1%/+12.1%). Reusing a bound
reduces code in this fixture, but limits must still describe the actual contract.
Sharing more of the packed decoder internally is a potential follow-up, not an
implemented saving or a reason to enlarge consumer limits indiscriminately.

For record_000 in the 100-type shared cohort, opt-z encode/decode costs
4,967.5 / 7,057.5 instructions versus Candid's 84,240.5 / 81,976.5. Opt-3 is
2,120.25 / 3,782.5 versus 58,825.25 / 55,752.5. Varied bounds add nineteen
instructions per operation at opt-z and none at opt-3 for this record. Frames
are 59 bytes for Wire and 114 for Candid. Every record is runtime-selectable and
all IC cases retain three repeated samples. Do not extrapolate these results
to arbitrarily many types or installed consumers.

## Owned-blob experiment: rejected

An isolated ten-case cohort compares 0/1/32/63/64/65/128/4,096/65,536/1,048,576
random bytes, with ordinary owned, bounded owned, borrowed and Candid controls.
The candidate expands exact XOR/OR bulk comparison from 64-byte to 256-byte
blocks. It never skips canonical validation, changes admission or allocates a
comparison buffer. Candidate source is archived, not included in production.

| Owned decode / profile | Baseline IC | Candidate IC | Change | Candid IC |
| --- | ---: | ---: | ---: | ---: |
| 1 MiB / opt-z | 2,573,930 | 2,414,186 | -6.21% | 1,073,103 |
| 1 MiB / opt-3 | 2,524,404 | 2,401,533 | -4.87% | 1,065,188 |
| 128 bytes / opt-z | 1,706.5 | 1,936.5 | +13.48% | 23,754 |
| 128 bytes / opt-3 | 1,358 | 1,402 | +3.24% | 15,880.5 |
| Empty / opt-z | 794.5 | 794.5 | unchanged | 22,656.5 |
| Empty / opt-3 | 508 | 508 | unchanged | 14,867 |

64/65-byte controls also regress; 4 KiB improves 5.46%/4.32%. Candidate Wire
Wasm grows from 46,466 to 46,948 bytes at opt-z (+482, +1.04%) and 52,178 to
52,757 at opt-3 (+579, +1.11%). Code sections grow by the same amounts, from
37,379 to 37,861 and 43,138 to 43,717. Candid artifacts remain byte-identical.
The candidate still costs roughly 2.25x Candid instructions for the large owned
blob. This does not justify a larger runtime and small-input regressions.

Native release comparison is unchanged by the candidate. Opt-3 1 MiB owned
decode run medians are 33.77–33.91 µs before and 33.32–34.72 µs after; this
variation is not a claimed speedup. Candid controls are 17.91–18.85 µs across
runs. Wire requests one 1,048,576-byte allocation in both variants; Candid
requests 25 allocations totaling 1,050,661 bytes, with peak live 1,049,380 bytes.
Borrowed Wire uses no allocation and takes 826 opt-3 IC instructions; that
control does not provide an owned result. Copying and verifying owned data are
separate from the borrowed path's same-region canonical check.

## Identity, reproduction and validation

Baseline is released 0.6.9, c0841e7. Matched runs use the same Cargo.lock,
Postcard 1.1.3, IC Platform 0.6.10, Host/Testkit 0.66.0 and PocketIC 16.1.0.
Rust 1.99.1 / LLVM 23.1.1 targets x86_64 Linux (Threadripper 7970X) and
wasm32-unknown-unknown. Profiles use LTO, one codegen unit, panic abort and
stripped symbols; evaluation is opt-z, benchmark-speed is opt-3.

Blob native time uses two runs/profile, eleven calibrated samples per operation,
at least 2 ms/batch or 1,048,576 repetitions. Allocation and peak-heap runs are
separate. Timing is unpinned, warm-cache and includes result destruction.
IC runs take three samples of four operations (one for frames above 64 KiB),
using performance counter 0 without overhead subtraction. Setup and returned
frame verification sit outside the counter. All samples agree within each IC
row; input/frame hashes match across variants and measurement modes.

[Scaling](evidence/scaling-blob-0610/scale.csv),
[blob measurements](evidence/scaling-blob-0610/blob.csv),
[generator measurements](evidence/scaling-blob-0610/generator.csv),
[artifacts](evidence/scaling-blob-0610/artifacts.csv) and
[identity](evidence/scaling-blob-0610/identity.json) retain the summarized data.
The [source/sample archive](evidence/scaling-blob-0610/raw.json.gz) contains exact
source snapshots, both rejected/retained variants, scripts and all samples.
Extract the appropriate snapshot and replay scale.sh, blob.sh or generator.sh,
adapting absolute workspace/cache/output paths and using new output directories.
The blob snapshot deliberately narrows the benchmark corpus; it is not a new
production mode. Archive compression is storage only, not a Wasm objective.

Maintained cohort selections are packed-scale and packed-scale-shared, optionally
combined with derive-scale-1 or derive-scale-10. The evaluate-codecs runner accepts
these combinations for future full native/IC runs. This scaling study measures
Wasm and IC cost, not native DTO throughput. Focused roundtrips, strict Clippy,
Rust 1.91 checks, generated TypeScript conformance and documentation checks pass.
No full CI, installed-consumer run or big-endian qualification is claimed.
