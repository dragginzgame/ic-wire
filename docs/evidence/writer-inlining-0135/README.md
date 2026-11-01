# Output-writer investigation after Core 0.13.4

Measured 2026-11-01 against released Core/derive **0.13.4**. Keep only the
Wasm `try_push` inlining hint. Production retains the existing fallible Vec
writer, its capacity checks and one Serde/Postcard codec. The new hint remains
uncommitted work for 0.13.5, outside the completed 0.13.4 deployment.

## Total uncompressed Wasm and code section

Both metrics come from the same final raw module. Code bytes are the section-10
payload, excluding section headers. These fixtures retain encoding and decoding;
their sizes do not establish application linker savings or code-limit headroom.

| Selected hint / profile | Total before → after | Delta | Code before → after | Delta |
| --- | ---: | ---: | ---: | ---: |
| 20 controls, opt-z | 94,193 → 94,193 | 0 | 81,742 → 81,742 | 0 |
| 20 controls, opt-3 | 121,096 → 121,085 | −11 | 109,653 → 109,642 | −11 |
| 100 types, opt-z | 310,637 → 310,637 | 0 | 291,902 → 291,902 | 0 |
| 100 types, opt-3 | 398,548 → 396,039 | −2,509 (−0.63%) | 364,120 → 361,615 | −2,505 (−0.69%) |

Both Candid smoke modules are bit-identical before/after: opt-z **402,762 total /
367,085 code**, opt-3 **624,858 total / 590,483 code**. The opt-z Wire modules are
also bit-identical. The 100-type follow-up measures Wire before/after, not another
complete Candid matrix. Archives only store evidence; compressed byte counts do
not influence selection or appear as Wasm footprint metrics.

## IC instructions, small controls and regressions

All **192 paired Wire operations** pass the unchanged data-only gate. Each row
has three distinct caller samples with identical IC counts; exact input/frame
hashes, bytes, dependencies and settings match. Candid counters are all identical.
All Wire opt-z operations and every recorded decode/refusal operation are
unchanged. Ten successful opt-3 encoding rows change; loop controls are retained
and not subtracted. Results include destruction and loop overhead, with setup
and reply transport excluded. No cycle saving is claimed.

| opt-3 operation | Before → after instructions | Change |
| --- | ---: | ---: |
| u64 0 / 127, fresh encode | 676.25 → 674.25 | −0.296% |
| u64 0 / 127, reused encode | 352.25 → 350.25 | −0.568% |
| u64 MAX, reused encode | 648.25 → 652.25 | +0.617% |
| Empty blob, fresh encode | 802.25 → 802.25 | 0% |
| 1 MiB blob, fresh encode | 1,049,699 → 1,049,699 | 0% |
| 16,384 wide i16, fresh encode | 2,638,670.25 → 2,638,670.25 | 0% |
| 128 byte-key-map entries, fresh encode | 62,376.25 → 62,376.25 | 0% |
| 4,096 optional units, fresh encode | 447,273.25 → 455,465.25 | +1.832% |
| 4,096 optional units, reused encode | 446,956.25 → 455,148.25 | +1.833% |

The optional-unit list is the largest measured regression: +8,192 instructions,
**1.83284%**. It remains within the `baseline × 1.05 + 64` screen. This is an
explicit tradeoff for the smaller opt-3 module, with the primary opt-z footprint
and counters unchanged. The opt-z 4,096-unit encode gap remains **181,764.5**
instructions versus Candid **19,167.75**; this hint does not fix that gap.

## Native observations and allocations

Two warmed native runs use eleven calibrated samples per operation, minimum
2 ms batches, including destruction. The hint is conditional on Wasm; no native
speed improvement is claimed. The shared host was not CPU-isolated, and observed
native changes include regressions and inconsistent directions. Preserve both
runs rather than selecting a preferred result. Opt-3 fresh encode medians, ns:

| Input | Run 1 before → after | Run 2 before → after |
| --- | ---: | ---: |
| u64 0 | 38.929 → 51.000 (+31.01%) | 38.649 → 51.299 (+32.73%) |
| Empty blob | 38.020 → 38.995 (+2.57%) | 38.140 → 38.847 (+1.85%) |
| 1 MiB blob | 18,384.047 → 18,843.461 (+2.50%) | 17,246.078 → 17,391.125 (+0.84%) |
| 16-entry byte-key map | 125.116 → 232.521 (+85.84%) | 124.068 → 120.532 (−2.85%) |
| 16,384 wide i16 | 45,281.078 → 45,827.484 (+1.21%) | 50,367.813 → 44,951.781 (−10.75%) |

The scalar increase is about 12 ns in both runs. All native allocation and
peak-live-heap observations are identical before/after, for both profiles and
both codecs. Those probes use separate instrumented builds and one warmed
operation, not eleven allocator samples. [All native pairs](native-push-paired.json.gz)
retain every operation and Candid control. Native timing is separate from the
deterministic IC counter results; no universal performance win is claimed.

## Rejected candidates

Independent patches start from the same baseline. Deltas below are total/code
bytes in the 100-type fixture; [screen.csv.gz](screen.csv.gz) retains every absolute
value, module hash and profile.

| Candidate | opt-z delta | opt-3 delta | Decision |
| --- | ---: | ---: | --- |
| Owned zero-filled slice writer | +811 / +644 | −477 / −466 | Reject primary size regression; no IC/native qualification |
| Inline all three output methods | −451 / −449 | −2,754 / −2,748 | Reject broad tradeoff: map encode 62,376.25 → 68,269.25 (+9.45%) |
| Inline only byte-slice extension | −451 / −449 | −257 / −255 | Size screen only; not adopted or IC-qualified |
| Inline only finalize | 0 / 0 | 0 / 0 | Reject: bit-identical modules |
| Inline only single-byte push | 0 / 0 | −2,509 / −2,505 | Keep after full smoke qualification |

The broad candidate also changes the opt-z Candid module from 402,762/367,085
to 402,634/366,958 (−128/−127), while all its measured Candid counters remain
identical. The strict fixed-artifact gate correctly refuses that pair. Its
[full analysis](broad/) is retained separately; it is not reported as a passing
regression gate. The selected single-byte hint keeps Candid artifacts/counters
identical and removes the broad candidate's measured map regression.

## Protocol, source and reproduction

The hint exposes a small existing output wrapper to Wasm inlining. It changes
no serializer representation, buffer/limit/error semantics, two-pass admission,
budgets, primitive acceptance or exact retry/hash bytes. Consumers require no
migration; u64 127 remains `7f`. No unsafe code, new mode or consumer profile
change is added. The applied source adds only an explanatory end-of-line comment
beyond the measured patch; runtime tokens and line numbers match.

Source release: `f83755b30f76704105164e163b9334310e106e44`. The
[baseline capsule](source.tar.gz) includes a common private `contract_sha256`
sampler identity addition on both sides. Parent Git context is delivery-record
commit `e6c0556`, not proof of the capsule's uncommitted experimental contents;
source hashes, independent patches and physical artifacts bind those contents.
Root own versions remain 0.13.4/0.3.0.

Rust **1.99.1**, LLVM **23.1.1**, host `x86_64-unknown-linux-gnu`, guest
`wasm32-unknown-unknown`. Primary `evaluation` recipe: opt-z, fat LTO, one codegen
unit, panic abort, symbol stripping. `benchmark-speed` changes only opt-level to
3 to check the speed-oriented footprint/instruction tradeoff. No post-link
optimizer or deployment transformation is applied. These are library recipes,
not a named application's complete deployment pipeline. Keep global Cargo config
fixed too; recorded identities do not constitute a hermetic build.

Pinned suppliers include Platform 0.16.0, Types 0.15.1, Testkit 0.76.0, Build/Host
0.73.5, Metrics 0.24.4, Candid 0.10.39, Postcard 1.1.3, Serde 1.0.229 and
serde_bytes 0.11.19. PocketIC server 16.0.0 SHA-256 is
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.

Extract the capsule into a new absolute work directory. Use one fixed source
path and exclusive Cargo target; prepare locked caches, toolchains and the
explicit server before offline measurement. Restore all baseline runtime/contract
files before each independent patch. The recorded local scripts
[screen-size.sh](screen-size.sh), [screen-narrow.sh](screen-narrow.sh),
[qualify-pair.sh](qualify-pair.sh) and [qualify-push.sh](qualify-push.sh) expect
`source/` beside their outputs and full relative runtime copies under
`variants/NAME/`; adapt their original cache/server paths on another host.
Copy restored source files immediately before building to refresh mtimes and
require library/guest `fresh=false`. Every recorded scale and formal Wire build
passed that check. Reconstruct variants from the capsule and independent patches.

The maintained codec runner uses `CORPUS_FEATURE=cost-smoke`, `RUNS=2`,
`SAMPLES=11`, `BATCH_MS=2`, both profiles and three fresh-caller IC samples. The
[existing strict analyzer](../candid-comparison-0133/analyze.mjs) produces
[selected/](selected/) from baseline/push-inline cohorts (40 entries, 160 primary
Wire/Candid comparisons). For a data-only replay, extract
[gate-inputs.tar.gz](gate-inputs.tar.gz) and [artifacts.tar.gz](artifacts.tar.gz)
into the same new directory, then run `make check-codec-costs` with baseline and
push-inline directories. This starts no build/server and rewrites no baseline.

[build-records.tar.gz](build-records.tar.gz), [fresh-builds.json](fresh-builds.json)
and [source context](source-context.json) retain compilation/source custody.
[Gate code](gate-source.tar.gz), full selected/broad raw data, exact patches and
[SHA256SUMS](SHA256SUMS) make the study replayable. Root strict library/sampler
Clippy and Rust 1.91 Wasm compilation pass. The preceding 0.13.4 full CI qualifies
that release; this follow-up uses focused checks and remains unreleased.

Raw `owned-slice.patch.gz` and `screen.csv.gz` preserve their original whitespace
and line endings; decompress them before replay or CSV analysis.
