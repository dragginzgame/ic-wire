# Wire versus Candid: full 0.13.3 qualification matrix

Measured 1 November 2026 from Core **0.13.2** source plus the pending **0.13.3**
benchmark/type-coverage changes. Production codec/generator source and protocol
are unchanged by this study. The full **22 cohorts / 1,243 case entries / 4,972
primary instruction comparisons** pass the retained strict analyzer. These
overlapping controls are case entries, not 1,243 unique Rust types or a production
traffic distribution. No weighted overall latency/speedup is claimed.

- Wire wins **4,462** successful encode/decode instruction comparisons; Candid
  wins **510**, with no ties.
- Wire has smaller **uncompressed Wasm and code sections in all 44** cohort/profile
  pairs. All **88** compiled artifact hashes and declared optimization levels
  are verified.
- Frame sizes favor Wire in **775** entries, tie in **28** and favor Candid in
  **440**. Size counts use one profile/operation per logical case entry.
- All **26,440** IC operation rows have three fresh caller samples with identical
  instruction counts. There are no varying-counter rows and no anonymous-query
  cache controls in this run.

## Raw Wasm footprint

Total uncompressed bytes / code-section bytes. Both encoding and decoding are
reachable; these are complete fixtures, not projected linker savings in an
application retaining external Candid endpoints. Wire reuse/control operations
also remain reachable. Compression is used only for evidence storage.

| Fixture | Profile | Wire total / code | Candid total / code |
| --- | --- | ---: | ---: |
| Standard, 333 cases | opt-z | 429,354 / 387,841 | 1,079,024 / 1,010,886 |
| Standard, 333 cases | opt-3 | 662,430 / 606,280 | 1,610,988 / 1,527,857 |
| Toko-shaped, 10 cases | opt-z | 81,095 / 69,931 | 384,895 / 349,783 |
| Toko-shaped, 10 cases | opt-3 | 103,809 / 93,308 | 608,980 / 575,323 |
| 1 ordinary derived type | opt-z | 53,966 / 42,997 | 309,978 / 276,199 |
| 1 ordinary derived type | opt-3 | 74,871 / 48,129 | 449,732 / 400,926 |
| 10 ordinary derived types | opt-z | 99,438 / 87,634 | 445,995 / 409,777 |
| 10 ordinary derived types | opt-3 | 129,641 / 102,105 | 695,691 / 644,359 |
| 100 ordinary derived types | opt-z | 310,706 / 291,971 | 1,435,267 / 1,380,274 |
| 100 ordinary derived types | opt-3 | 398,675 / 364,245 | 2,106,847 / 2,036,736 |
| Packed, 109 cases | opt-z | 98,437 / 87,283 | 454,050 / 418,025 |
| Packed, 109 cases | opt-3 | 122,542 / 112,096 | 693,691 / 659,090 |

At 100 ordinary types, opt-3 Wire is **81.08% smaller** in total Wasm
and **82.12% smaller** in code section. All cohort/module identities are in
[artifacts.csv](artifacts.csv). Symbols and compressed sizes are not savings claims.

## IC instructions and wire bytes

Each primary comparison pairs the same logical input with the selected Wire lane
and Candid in its own compiled cohort. Bounded/ordinary/packed cohorts use the one
production Serde/Postcard codec with explicitly different field representations;
ordinary controls remain visible. Do not combine Candid counters from different
compiled contexts. IC counters include result destruction and loop overhead;
loop controls are recorded and are not subtracted.

| Standard corpus | Wire encode wins | Wire decode wins |
| --- | ---: | ---: |
| opt-z | 258 / 333 | 284 / 333 |
| opt-3 | 264 / 333 | 312 / 333 |

The 109-case generic Packed cohort wins **all 436** primary comparisons across
both profiles. Fixed-width packing can increase small-value payload sizes;
existing contracts require coordinated adoption. It is not a transparent Serde
rewrite. [Field selection](../../packed-fields.md) explains the tradeoff.

Selected opt-3 counters per operation; bytes include each codec's full frame:

| Shape | Wire encode | Candid encode | Wire decode | Candid decode | Wire / Candid bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 MiB byte-oriented blob | 1,049,704 | 2,115,234 | 1,049,508 | 1,068,979 | 1,048,590 / 1,048,588 |
| 16,384 wide i16, ordinary | 2,638,675.25 | 80,397.25 | 1,904,910.5 | 1,412,079.5 | 49,166 / 32,780 |
| 16,384 wide i16, packed | 33,489.25 | 80,320.25 | 33,476 | 1,411,787 | 32,782 / 32,780 |
| 128 binary-key map entries | 377,459.25 | 86,903.25 | 703,856.5 | 1,354,968 | 6,349 / 6,485 |
| 128 blob-valued results | 185,612 | 358,004 | 223,949 | 817,798 | 109,247 / 109,258 |

## Remaining losses and new cases

There is no universal performance win. The largest successful encode ratio is
ordinary wide u16 at opt-z: **10,487,432.5 / 86,393.5 instructions (121.39×)**.
The opt-3 maximum is the ordinary u16 bulk control: **4,113,267.25 / 80,505.25
(51.09×)**. The generic 1 MiB `Vec<u8>` encode remains **37.23×** Candid at opt-3.
The worst successful decode is the ordinary u64 bulk control at opt-z:
**14,670,163 / 1,193,038 (12.30×)**; the opt-3 maximum is ordinary i64:
**6,246,822 / 1,166,367 (5.36×)**. Byte-oriented and packed counterparts stay
separate and visible; their cost does not erase the ordinary-control losses.

All **35 appended standard cases** decode faster with Wire in both profiles.
Wire wins 29/35 new encodes at opt-z and 31/35 at opt-3. The new cases include
zero-sized unit/record collections, optional/result unit lists, empty nested lists,
binary-key maps, blob results, optional arrays, Box/Cow and enum shapes.

The largest new opt-3 encode loss is the 128-entry binary-key map, **4.34×**
Candid despite its smaller frame. Large optional-unit/result-unit lists and empty
inner lists also have opt-3 encode losses. At opt-z, the 4,096-unit list reaches
**9.47×**. These provide concrete targets for follow-up; a byte-oriented fixed-size
key representation is a candidate experiment, not a measured fix or proposed
public API. Native timings must be read separately: the unit-list opt-3 IC encode
win does not imply a native encode win.

Rejection cost is separate. At opt-3, rejecting a truncated 1 MiB ordinary byte
vector costs **42,472,926** instructions versus **4,289** (9,902.76×). At opt-z
it costs **268,965,615** versus **6,654** (40,421.64×). Ordinary sequence parsing
can traverse a long valid prefix before detecting EOF. Byte-oriented fields can
reject an unavailable range before copying it. Full paired rejection rows are in
[rejection-comparison.csv](rejection-comparison.csv). Successful roundtrips do not
establish identical resource policies, malformed acceptance or normalization.

## Native timing, allocation and heap

Two serial warmed native runs per profile use eleven calibrated batch samples
per operation, with batches of at least two milliseconds. Setup/equality checks
are excluded; result destruction and dynamic-call/loop overhead are included.
p95 describes batch means, not request tail latency. Allocation counts and peak
live heap use separate instrumented builds and one warm measured operation each;
the sample-count argument does not turn these into repeated allocator samples.

Opt-3 1 MiB byte blob, native medians in nanoseconds, shown for both runs:

| Operation | Wire run 1 / run 2 | Candid run 1 / run 2 |
| --- | ---: | ---: |
| Encode | 17,326.85 / 17,379.11 | 757,787.5 / 725,446.25 |
| Owned decode | 17,113.8 / 17,401.8 | 18,403.16 / 18,338.61 |

For that blob, Wire allocates once for encode/decode; Candid allocates 17/31
times, plus three encode reallocations. Peak live native bytes are Wire
**1,048,590 / 1,048,576** versus Candid **2,098,807 / 1,049,380** for encode/decode.
All per-case native, allocator and heap values remain in [comparison.csv](comparison.csv)
and [raw.json.gz](raw.json.gz). These are native allocator observations, not IC
heap-page instruction charges, endpoint cycle debits or cold-start latency.

## Inputs, scope and reproduction

The frozen graph selects Candid **0.10.39**, Postcard **1.1.3**, Serde **1.0.229**,
serde_bytes **0.11.19**, Core/derive **0.13.2**, Platform **0.15.3**, Types
**0.15.1**, Build/Host **0.73.3**, Testkit **0.75.2** and Metrics **0.24.3**.
[Supplier selection](suppliers.json), [frozen manifest](Cargo.toml.snapshot) and
[lock](Cargo.lock.snapshot) identify the actual inputs. During measurement the
working checkout moved Metrics to 0.24.4; [drift record](checkout-lock-drift.json)
keeps that separate. The frozen source/lock were unchanged and the newer checkout
edit was preserved. This is not qualification of that post-snapshot selection.

Both profiles use Rust **1.99.1 / LLVM 23.1.1**, wasm32-unknown-unknown, LTO, one
codegen unit, panic abort and stripped symbols. `evaluation` is opt-z;
`benchmark-speed` is opt-3. [Effective build records](effective-profiles.json)
verify all 88 optimization levels; no profile environment override was present.
Native host is Linux x86_64, AMD Ryzen Threadripper 7970X (32 cores/64 threads).
[Host/compiler context](host-context.json) and [end context](host-end.json) retain
load and allocator details. This shared host was not CPU-isolated; both runs and
all raw samples remain visible. Build wall times include cache effects and are
not controlled compiler-performance comparisons; [build times](build-times.csv)
are retained as context.

Testkit admits the explicit prepared **PocketIC server 16.0.0** with client
16.1.0. Its binary hash, compiler, limits and sampler identity are retained in
every cohort's raw identity. This codec study uses `performance_counter(0)` and
does not need the separate endpoint instruction-trace API. IC counters surround
only warm codec operations on an already-owned input, with four repetitions up
to 64 KiB and one above it. Three queries use unique eight-byte callers; the
guest does not read the caller. Initial heap preparation, input/reply transport,
authentication, handler work, cold start and physical cycle debits are excluded.
Borrowed/reuse lanes are labelled separately in [all-lanes.json.gz](all-lanes.json.gz).

Wire uses frame bytes 2 MiB, allocation allowance 256 MiB, nodes 8 Mi and depth
64. Candid uses reply bytes 2 MiB, decoding/skipping quotas 128 Mi each, 4,096
type entries and 64 KiB header bytes, plus Platform's bounded nesting policy.
Policies are not equivalent and schema-evolution guarantees differ. The depth-8
Wire-success/Candid-refusal control remains outside successful speed ratios.
Codec coverage does not imply TypeScript schema support for every Rust type.

[Source archive](source.tar.gz) freezes actual Rust inputs, including untracked
fixture additions, and excludes browser node_modules (not needed by this codec
study). [Head](source-head.txt), [dirty status](source-status.txt) and
[tracked patch](source-changes.patch) distinguish it from a clean release.
[Wasm archive](wasm.tar.gz) retains all 88 raw modules; their uncompressed sizes
and hashes are independently checked after extraction, including an independent
code-section parse; [archive verification](archive-verification.json) records it. Original full outputs
remain under target/candid-comparison-0133 and its isolated sibling build target.
Normal explicit Cargo cleanup may remove those target artifacts.

After preparing the recorded cache, compiler/target and matching server, run
from the extracted source with Git metadata directed at the recorded repository:

```sh
export GIT_DIR=/absolute/ic-wire/.git GIT_WORK_TREE=/absolute/ic-wire
export POCKET_IC_BIN=/absolute/prepared/pocket-ic
export CARGO_TARGET_DIR=/absolute/isolated-target
RUNS=2 SAMPLES=11 BATCH_MS=2 bash scripts/ci/evaluate-codec-matrix.sh /absolute/fresh-output
cd /absolute/fresh-output/source
CODEC_SOURCE_SNAPSHOT="$PWD" CORPUS_FEATURE=hotspots RUNS=2 SAMPLES=11 BATCH_MS=2 \
  bash scripts/ci/evaluate-codecs.sh /absolute/fresh-output/hotspots
node /absolute/ic-wire/docs/evidence/candid-comparison-0133/analyze.mjs \
  /absolute/fresh-output /absolute/fresh-analysis
```

The [retained analyzer](analyze.mjs) is byte-identical to the prior study's
analyzer. It validates paired inputs/frames, complete cohort/profile coverage,
eleven-sample native identities, three unique-caller IC samples and artifact
hashes. [Successful analysis](analysis.log) records 22/1,243/4,972. Preflight
[strict benchmark Clippy](clippy.log) also passed. No full CI, macOS execution,
new production optimization, release or publication is claimed.

All protocol bytes, budgets, field order, primitive/malformed acceptance and
retained retry/hash inputs are unchanged by this benchmark batch. The previous
[0.7.25 study](../candid-comparison-0725/README.md) used another corpus, Platform
graph and server; this is a fresh paired Wire/Candid baseline, not a matched
previous-release regression test.

[Checksums](checksums.sha256) cover every retained result/archive file.
