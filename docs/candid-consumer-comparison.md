# Consumer-shaped and derive-scaling comparisons

This study compares the pending 0.5.22 Wire runtime with the benchmark's selected
Candid 0.10.39 through IC Platform 0.6.7. It does not build or modify Toko, import
its database, or measure endpoint transport, database work, TypeScript or business
logic. No published dependency or protocol changes are introduced by these fixtures.

The [complete pending 0.5.22 rerun](candid-comparison.md#complete-pending-0522-rerun)
is the latest comparison after shared output/header setup, symbol stripping and
corrected Wasm assertions.
It covers both compiler profiles, all five cohorts, native time, IC instructions,
allocation/heap measurements and uncompressed/code-section sizes. The tables below
preserve earlier experiment evidence and must not be combined with the new runs.

## Toko shapes and input identity

Source is Toko `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2`:
`fleets/toko/project/instance/src/dto/{billing,token}.rs` and
`backend/src/toko/src/api.rs`. The fixture copies field names/types for
`ProjectFundingEstimate`, `ProjectBillingSummary`, `CreateTokenVariantInput`
and `TokenFilterValues`, adding the Serde traits needed for the Wire comparison.
These are synthetic values, not sampled production traffic or a drop-in migration.
Toko's current DTOs do not all implement the required Wire/Serde contracts.

IcyDB **v0.258.0**, selected by Toko's manifest, supplies the atom representations:
ULIDs serialize as 16-byte blobs and deserialize through a fixed-width sequence
visitor; typed IDs forward to their key. The fixture preserves that binary
representation and checks exactly 16 elements without allocating an ID buffer.
Its JSON input identity uses byte arrays rather than canonical ULID strings;
JSON is outside codec measurements. Decimals in that IcyDB version serialize
as text in both Candid and binary Serde.

The 0/1/50-item `TokenCard` page is deliberately a **projection**, not a copy of
the full `TokenSummary` graph. Each item has a ULID, name, optional 112-byte
description, two tags, two asset IDs, minted count, optional decimal text and a
stale flag. Its decimal is already formatted as `1234.5678`; decimal formatting,
parsing and validation are not included. The enclosing page has the same
`items`, `next_offset`, `has_more` and `total` fields as Toko's `Paginated<T>`.
Do not apply this page's ratios to full token summaries without measuring them.

Funding/billing inputs use 30 requested days, 28 observed days, realistic-order
trillion-cycle balances/burns and a timestamp of 1,792,627,200 seconds. The variant
request has a key, label, description, optional asset ID and display order.
Filter cases contain 0/50/500/1,000 token IDs, eight tags, plus missing-attribute
lists of `n/10` and `n/20` IDs. Other missing-field lists are empty. Inputs are
fully specified in [the maintained fixture](../crates/ic-wire-benchmarks/fixtures/toko/mod.rs)
and identified by hashes in the evidence.

## Derive cohorts

The [derive fixture](../crates/ic-wire-benchmarks/fixtures/scale/mod.rs) selects
1, 10 or 100 **distinct Rust record types**, not repeated instances of one type.
Every type is runtime-selectable and exercised in the native/IC corpus. Wire-only
artifacts instantiate `#[derive(ic_wire::Wire)]`; Candid-only artifacts instantiate
`CandidType`, Serde `Serialize` and `Deserialize`. Wire's derive supplies its Serde
implementations. There is no native-Serde Wire lane in these artifacts.

Each type has six fields: a uniquely named u64 amount, bool, signed delta,
optional u64, fixed byte array and label. Array widths cycle through 1–16, so
this cohort has 100 nominal records with 16 structural width patterns, not 100
unrelated application schemas. Wire bounds the label to 64 bytes and declares
field IDs 1–6; Candid carries the field labels. Inputs are equal across codecs.
The Wire header remains included. Compiler sharing of equivalent generated code
is allowed and is part of the size result. This synthetic cohort establishes no
universal per-type size or build-time cost for arbitrary application schemas.

## Measurement contract

Both optimization profiles use Rust 1.99.1 / LLVM 23.1.1, LTO, one codegen unit,
panic abort and stripped debug information. `evaluation` is opt-z and
`benchmark-speed` is opt-3. Native timing uses two runs per profile, 11 batch
samples each and 2 ms calibration on an unpinned Threadripper 7970X. Allocation
counts and DHAT peak-live heap are separate instrumented runs, excluding input
construction and already-held inputs. Native latency is not IC instruction cost.

PocketIC 16.1.0 measures three samples per operation using the same limits and
counter protocol as the [main suite](candid-comparison.md). Exact returned frames
must match native fixtures. Output sizes include the Wire frame or Candid header.
Linked Wasm/gzip measurements cover complete fixture canisters, including dispatch,
input generation and reply code, not marginal codec symbols or Toko deployment
savings. Gzip uses level 9 with no timestamp.

`build-times.csv` records one build-command duration per profile/codec/cohort,
rounded to whole seconds. Dependencies are warm; fixture feature selections force
separate builds. These observations include code generation/linking and do not
isolate macro expansion or establish statistically repeatable build-time ratios.

## Results

All **2,440 IC rows** across the four cohorts have three agreeing samples; frames
match their native counterparts. The standard corpus is unchanged. Runtime
source is release `91244e9` plus the pending Wasm inlining change from the
[preceding experiment](candid-comparison.md#wasm-serialization-inlining--pending-0522).
All cohort snapshots have the same runtime source identity.

### Toko-shaped messages

Opt-3, owned native-Serde Wire versus Candid. Bytes include each codec's framing;
instructions are per operation, with setup outside the counter.

| Workload | Wire / Candid bytes | Encode instructions, Wire / Candid | Decode instructions, Wire / Candid |
| --- | ---: | ---: | ---: |
| Funding estimate | 45 / 83 | 2,478.25 / 42,205.25 | 4,173.50 / 46,260.50 |
| Billing summary | 77 / 171 | 5,408.25 / 71,447.25 | 9,353.50 / 70,250.50 |
| Create variant | 96 / 130 | 2,860.25 / 60,094.25 | 6,256.50 / 65,216.50 |
| Filter, 50 tokens | 1,035 / 1,081 | 14,400.25 / 84,320.25 | 116,836.50 / 236,996.50 |
| Filter, 1,000 tokens | 19,617 / 19,663 | 170,638.25 / 172,445.25 | 2,039,054.50 / 2,925,759.50 |
| Token-card projection, 50 items | 10,798 / 11,004 | 219,873.25 / 213,439.75 | 743,981.50 / 1,845,958.50 |

Candid wins the 50-item page's IC encoding by 2.93%; Wire uses 59.70% fewer
instructions decoding it. The 1,000-token filter's encoding is nearly tied
(Wire −1.05%); Wire decoding uses 30.31% fewer instructions. Small-record
wins do not imply the same ratios for large lists or full application DTOs.

Native times below show the range of the **two run medians**, in microseconds;
these are not confidence intervals or request tail latencies.

| Opt-3 native workload | Wire µs | Candid µs |
| --- | ---: | ---: |
| Billing encode | 0.125–0.149 | 2.236–2.377 |
| Billing decode | 0.183–0.199 | 1.668–1.736 |
| 1,000-token filter decode | 36.792–44.872 | 67.937–70.101 |
| 50-item projection encode | 3.676–3.902 | 4.846–5.236 |
| 50-item projection decode | 20.404–21.219 | 47.241–52.869 |

The projection's native encode favors Wire while its IC encode favors Candid.
Do not substitute one metric for the other. Ownership and memory also matter:

| Opt-3 owned decode | Wire allocations + reallocations | Candid allocations + reallocations | Peak live bytes, Wire / Candid |
| --- | ---: | ---: | ---: |
| Billing | 0 + 0 | 67 + 4 | 0 / 3,570 |
| 1,000-token filter | 12 + 18 | 88 + 2 | 19,688 / 21,028 |
| 50-item projection | 351 + 4 | 864 + 3 | 24,866 / 22,613 |

Wire makes fewer allocation requests, but its projection decode has about 10%
higher peak live heap. The filter also reallocates more often with Wire. Raw
allocation totals, retained bytes and all other operations remain in the evidence;
allocation counts alone do not establish lower memory or fewer copied bytes.

### Linked size with derives

Each cell is **Wire / Candid**, in bytes. The synthetic cohort uses finite Wire
derives, unlike the native-Serde Toko cohort.

| Cohort | opt-z Wasm | opt-z gzip | opt-3 Wasm | opt-3 gzip |
| --- | ---: | ---: | ---: | ---: |
| Toko-shaped | 145,116 / 515,108 | 40,065 / 151,439 | 162,764 / 711,355 | 45,120 / 193,124 |
| 1 derived type | 75,506 / 407,619 | 28,717 / 130,135 | 96,056 / 516,682 | 40,040 / 161,156 |
| 10 derived types | 138,142 / 577,920 | 35,408 / 152,380 | 183,154 / 812,402 | 48,104 / 196,052 |
| 100 derived types | 464,914 / 1,820,491 | 48,709 / 213,315 | 600,570 / 2,482,831 | 63,650 / 268,417 |

From 1 to 100 types, opt-3 Wasm grows **504,514 Wire bytes versus 1,966,149
Candid bytes**; gzip grows 23,610 versus 107,261 bytes. At opt-z the corresponding
Wasm growth is 389,408 versus 1,412,872 bytes. These deltas include fixture
construction/dispatch and codec specialization; they are not isolated derive size.
Repeated structural patterns compress well, so gzip results for unrelated,
heterogeneous production schemas may differ considerably.

Within the 100-type cohort, opt-3 Wire encode uses 25.6–35.4× fewer instructions
and decode uses 15.5–20.1× fewer across these small records. For the unchanged
`record_000` control, Wire/Candid costs are 1,625.25/57,479.50 encode and
2,701.50/54,304.50 decode, with 43/99 bytes. The one-type control costs
1,528.25/57,406.50 encode and 2,325.50/53,869.50 decode: adding types is not
instruction-neutral even for an unchanged value. These results do not contradict
the larger-list Candid wins in the standard and Toko-shaped cohorts.

Single build observations for 1/10/100 types were **1/1/5 seconds for Wire versus
3/5/20 seconds for Candid at opt-3**, and 1/1/4 versus 2/4/14 seconds at opt-z.
Use these as recorded dependency-warm build durations, not a macro-expansion
speed claim. No clean dependency build or repeated compiler-timing study ran.

### Retained evidence

[Per-case summary](evidence/consumer-cohorts-0522/summary.csv) includes both native
run medians, IC instructions, payload size, allocation counters, peak heap and
input/frame hashes. [Artifact sizes and build observations](evidence/consumer-cohorts-0522/artifacts.csv)
cover both profiles and codecs. [Source/compiler identities](evidence/consumer-cohorts-0522/identity.json)
and [all samples and full identities](evidence/consumer-cohorts-0522/raw.json.gz)
retain the detailed data without thousands of inline report rows. The archive is
a gzip-compressed JSON object keyed by cohort, then measurement directory, with
`identity` and `samples` entries. All source snapshots, binaries and Cargo build
messages remain locally under `target/{toko,derive-1,derive-10,derive-100}-0522/`.

## Reproduction

Use the checked-in source and lockfile with the documented compiler and explicitly
prepared PocketIC server. Select one cohort per fresh output directory:

```sh
CORPUS_FEATURE=toko POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/toko-comparison
CORPUS_FEATURE=derive-scale-1 POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/derive-1-comparison
CORPUS_FEATURE=derive-scale-10 POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/derive-10-comparison
CORPUS_FEATURE=derive-scale POCKET_IC_BIN=/path/to/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/derive-100-comparison
```

The runner explicitly selects `wire-native` or `wire-finite` as the report's
required primary lane. Missing partners and duplicate rows still refuse. The
first one-type report exposed the former hardcoded native-lane requirement;
its unchanged measurements were retained and reported after the correction.
`make check-consumer-benchmarks` qualifies all four corpus selections; the main
`check-benchmarks` also includes it. Full CI and macOS were not run for this study.

## Code-section footprint follow-up

The IC's [resource-limit reference](https://docs.internetcomputer.org/references/resource-limits/)
limits the Wasm **code section** to 12 MiB. Whole-module and gzip sizes do not
measure this ceiling. The runner now appends `code_section_bytes` to `wasm.csv`;
`codec-report --code-size FILE` reads the uncompressed core module's section
framing and returns section 10's payload length. It is not a Wasm validator.

The original fixture's roundtrip `assert_eq!` linked per-type debug formatting
into both codec artifacts. Wasm now uses an equality assertion without formatting
the values; native checks retain their detailed diagnostics. The equality check
is preserved. This removes benchmark overhead, not production codec code.
Earlier tables and instruction samples above remain historical measurements of
the original fixture; they must not be paired with these new artifacts.

| 100 types, corrected fixture | opt-z Wire / Candid | opt-3 Wire / Candid |
| --- | ---: | ---: |
| Code-section bytes | 286,970 / 1,362,784 | 373,173 / 2,010,766 |
| Whole Wasm bytes | 364,046 / 1,722,441 | 473,134 / 2,336,172 |
| gzip-9 bytes | 41,598 / 208,294 | 56,719 / 261,301 |

These still include construction, JSON identity, dispatch and correctness checks;
they are neither isolated derive sizes nor predicted application savings. The
sixteen repeated structural patterns also limit extrapolation to real schemas.
No installed consumer was measured. Production code and protocol are unchanged.

Three runtime hint experiments were rejected. Forcing derived Serde methods
inline reproduces the baseline artifacts byte-for-byte. Preventing their
inlining adds 2,742/3,059 code-section bytes at opt-3/opt-z. Preventing finite
encode/decode inlining on the corrected fixture reduces total opt-3 Wasm by
94 bytes but **adds 15,461 code-section bytes**, worsening the relevant ceiling.
No experimental hints remain. Shared framing/validation is a candidate for a
separate experiment, not an established size saving.

[Sizes and artifact hashes](evidence/derive-footprint-0522/sizes.csv) and
[compiler/source identity](evidence/derive-footprint-0522/identity.json) retain
both codec builds and the rejected variants. They use the same Rust compiler,
lockfile, target and LTO/one-codegen-unit profiles as the earlier cohort, with
GNU gzip `-n -9`. Local artifacts/logs are in `/tmp/wire-footprint/`; the original
artifacts remain in `target/derive-100-0522/`. Reporter framing tests, the 100-type
roundtrip/malformed-frame check and focused Clippy pass. IC instructions were not
rerun for this size-only correction; no instruction improvement is claimed.

## One default: shared finite output setup

The next footprint pass retains one codec strategy with no public size/speed
selector. Fresh finite encoding now shares allocation and frame-header setup
across contract types through a non-inlined Wasm helper. Per-byte serialization
and canonical comparison retain their specialization. This reduces code emitted
at derived codec call sites; the derive syntax and generated field layout do not
change. Protocol bytes, canonical admission, typed failures, allocation policy,
scratch reuse and exact retries are unchanged. No migration is required.

| Fixture/profile | Code section before → after | Whole Wasm before → after | gzip-9 before → after |
| --- | ---: | ---: | ---: |
| 100 types, opt-3 | 373,173 → 363,873 | 473,134 → 446,945 | 56,719 → 59,741 |
| 100 types, opt-z | 286,970 → 286,447 | 364,046 → 364,026 | 41,598 → 41,849 |
| Standard corpus, opt-3 | 266,604 → 265,210 | 387,133 → 382,733 | 116,334 → 116,126 |
| Standard corpus, opt-z | 186,743 → 186,796 | 331,820 → 332,197 | 95,798 → 95,952 |

The 100-type opt-3 code section shrinks 2.49%, while gzip grows 5.33%.
The standard opt-z code section grows 53 bytes. This is a measured default
tradeoff for code-section pressure, not a universal size win or an application
savings estimate. Both fixtures include their existing setup/dispatch overhead.

For the 100-type cohort, opt-3 encode costs 0.49–0.68% more IC instructions and
decode costs 4.23–5.66% more. Opt-z encode costs 2.73–3.62% more and decode
0.065–0.099% more. Unchanged decode source does not guarantee unchanged generated
code. In the standard corpus the largest percentage increase is a unit variant's
opt-3 encoding (+32 instructions, 6.83%); the largest absolute increase is native
small-u64-list encoding at opt-z (+245,781 instructions, 2.62%). Native here names
the Serde API measured on IC, not host CPU time.

Rejected experiments explain why sharing stops at setup. Shared tuple helpers
produced no code-section saving. Moving byte writes and canonical comparison out
of line saved 5.75% in the 100-type opt-3 code section but increased 1 MiB generic
byte-vector decode by 82.44%. Sharing output writes alone saved 1.46%, while
finite 1,024-hash manifest encoding regressed 23.38%/28.29% at opt-3/opt-z.
Neither byte-level experiment remains in production.

[Instruction pairs](evidence/derive-setup-0522/instructions.csv),
[artifact sizes](evidence/derive-setup-0522/sizes.csv),
[identity and parameters](evidence/derive-setup-0522/identity.json),
[selected source patch](evidence/derive-setup-0522/selected.patch) and
[complete raw samples](evidence/derive-setup-0522/raw.json.gz) retain the evidence.
The baseline is the corrected fixture from the preceding section, with incoming
pending runtime work preserved. Every one of the 19,352 rows across two corpora
and four variants has three identical IC samples; all candidate input/frame
hashes match baseline. Compiler, lockfile, profiles and codec parameters match.
PocketIC is the explicitly prepared 16.1.0 server. Local artifacts are under
`target/derive-compact-*`; narrower rejected builds are in `/tmp/wire-shared-tuples/`.
Runtime tests, both corpus roundtrip/malformed-frame checks, lint and native/Wasm
MSRV pass. Host timing/allocations were not remeasured; no native performance
claim is made. Full CI and macOS were not run.

## Remaining footprint probes

A follow-up against the complete pending 0.5.22 rerun tested two additional
Wasm-only out-of-line hints, using the same 100-type fixture and opt-3 profile:

| Candidate | Total Wasm bytes | Code-section bytes |
| --- | ---: | ---: |
| Baseline | 446,945 | 363,873 |
| Shared frame-header validation | 446,812 | 363,691 |
| Explicitly shared bulk canonical comparison | 446,945 | 363,873 |

Neither is retained. Header sharing saves only 182 code bytes (0.05%); it was
not advanced to instruction qualification. Bulk comparison yields no size saving.
The runtime remains exactly at the complete rerun's source, with one strategy.
[Sizes/hashes](evidence/remaining-footprint-0522/sizes.csv) and
[compiler/experiment identity](evidence/remaining-footprint-0522/identity.json)
retain these rejected probes; local artifacts are in `/tmp/wire-payload-size/`.

The baseline also contains a 48,306-byte `name` custom section. Offline removal
of that section reduces total Wasm to 398,639 bytes (10.81%) but leaves the
363,873-byte code section unchanged. This is a potential consumer build/debugging
tradeoff, not a runtime optimization or extra headroom under the code-section
limit. It removes function-name metadata useful for diagnostics. This artifact
was not installed or instruction-tested, and no build profile was changed.
Native timing, allocation and IC instructions were not rerun for rejected probes.

## Small savings retained after instruction qualification

Following the maintainer's preference to retain small savings when implementation
complexity is low, header validation now stays out of line on Wasm. The runtime
change is one target-specific attribute and a comment; native inlining is untouched.
The workspace's `evaluation` profile now uses `strip = "symbols"`, inherited by
`benchmark-speed`. This removes diagnostic names from both codecs' artifacts.
Consumer workspaces must select their own build profile; dependency profiles are
not imposed by IC Wire. There is still one codec strategy and no per-type setting.

Compared with the complete rerun above, measured Wire sizes are:

| Fixture/profile | Total Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| Standard, opt-3 | 382,733 → 314,412 | 265,210 → 264,599 |
| 100 types, opt-3 | 446,945 → 398,228 | 363,873 → 363,691 |
| Standard, opt-z | 332,197 → 221,815 | 186,796 → 186,739 |
| 100 types, opt-z | 364,026 → 305,391 | 286,447 → 286,569 |

Most file savings are removed metadata, which does not improve the code-section
ceiling. Header sharing yields a small opt-3 code saving; the 100-type opt-z code
section grows 122 bytes. The build-profile change is applied equally to Candid:
its opt-3 total Wasm is now 860,566 bytes for the standard fixture and 2,079,944
for 100 types, with code sections of 788,971 and 2,010,763 bytes. Link/layout
changes can alter code size slightly; metadata removal is not a code-saving claim.

The tradeoff is measured. Standard opt-3 decode adds 22–48 instructions, at worst
48/180.5 = 26.59% for scalar floating-point cases. Standard encode is unchanged
in both profiles. The 100-type opt-3 cohort adds 27–29 decode instructions
(0.77–1.02%), with encoding unchanged. At opt-z it adds 25 encode/decode
instructions (0.66–0.87%/0.32–0.49%). The largest absolute standard regression is
opt-z recursive depth-8 decode: +34,208 instructions (3.52%). Rejected/truncated
frames are measured separately in the retained evidence. Protocol bytes,
canonicality, typed errors, limits and public APIs remain unchanged.

Both stripped codec modules installed and ran in PocketIC. All 4,838 IC rows
have three agreeing samples and the same input/frame hashes as the complete
baseline. Runtime tests, both corpus roundtrip/malformed-frame checks, Clippy,
native/Wasm MSRV, formatting and documentation checks pass. Host timing and heap
metrics were not rerun; earlier native results describe the prior build profile.
Full CI and macOS were not run. The loss of function names reduces diagnostic
backtrace detail; this is documented as a consumer build choice.

[Sizes/hashes](evidence/small-savings-0522/sizes.csv),
[instruction pairs](evidence/small-savings-0522/instructions.csv),
[raw samples](evidence/small-savings-0522/raw.json.gz),
[compiler/source identity](evidence/small-savings-0522/identity.json) and
[selected patch](evidence/small-savings-0522/selected.patch) retain both profiles.
Source snapshots, artifacts and logs are in `target/small-savings-0522-*` and
`/tmp/wire-small-savings/`. Baseline evidence remains unchanged. Reproduce using
the standard runner with the standard and `derive-scale` corpora; compare against
the complete rerun's source snapshots, not a different compiler or dependency set.
