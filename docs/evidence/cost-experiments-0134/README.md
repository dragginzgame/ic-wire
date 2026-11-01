# Focused cost investigation for Core 0.13.4

Measured 2026-11-01 against published Core 0.13.3 runtime source, with the same
new private 20-case harness on both sides. Keep the simpler `enter_many`
allowance/debit cleanup; reject the other four candidates. This is a small
footprint improvement with native timing tradeoffs, not a fix for every Candid
loss. Shared Tooling 0.27.24 was adopted canonically in the same working batch.

## Uncompressed Wasm

All values are bytes from the final raw module. Code bytes are the section-10
payload length, independently parsed from that same module; section headers
and other sections remain in total bytes. These complete fixtures retain both
encoding and decoding. They do not establish application linker savings or
remaining application code-limit headroom.

| Wire fixture / profile | Total before → after | Total delta | Code before → after | Code delta |
| --- | ---: | ---: | ---: | ---: |
| 20 controls, opt-z | 94,219 → 94,193 | −26 (−0.0276%) | 81,768 → 81,742 | −26 (−0.0318%) |
| 20 controls, opt-3 | 121,110 → 121,096 | −14 (−0.0116%) | 109,667 → 109,653 | −14 (−0.0128%) |
| 100 derived types, opt-z | 310,706 → 310,637 | −69 (−0.0222%) | 291,971 → 291,902 | −69 (−0.0236%) |
| 100 derived types, opt-3 | 398,675 → 398,548 | −127 (−0.0319%) | 364,245 → 364,120 | −125 (−0.0343%) |

The matched Candid smoke modules remain identical: opt-z **402,762 total /
367,085 code** and opt-3 **624,858 total / 590,483 code**. The 100-type follow-up
measures Wire before/after only; it is not another complete Candid matrix.
The [earlier full matrix](../candid-comparison-0133/README.md) retains its original
source/dependency graph and cannot substitute for this matched baseline.

## IC instructions and bytes

All 192 paired Wire operations pass the explicit `baseline × 1.05 + 64`
instruction screen. The largest increase is the 128-entry byte-key-map opt-3
encode: **62,120.25 → 62,376.25**, +256 instructions (**+0.4121%**).
Optional-unit-list opt-3 decode improves **256,857.5 → 240,473.5** (−6.3786%);
truncated-input refusal improves **256,820.75 → 240,436.75** (−6.3795%).
All paired frames, input hashes and Candid counters remain identical. All
three fresh-caller IC samples agree per row. Loop overhead and result destruction
are included; loop controls are retained rather than subtracted. Setup and reply
transport are excluded. No endpoint cycle measurement is claimed here.

Small controls and the remaining large-unit gap, instructions per operation:

| Input / profile / operation | Wire before → after | Change | Candid control |
| --- | ---: | ---: | ---: |
| u64 0, opt-z encode | 1,214.5 → 1,212.5 | −0.1647% | 10,916.75 |
| Empty blob, opt-z encode | 1,666.5 → 1,664.5 | −0.1200% | 22,273.5 |
| Empty unit list, opt-3 encode | 756.25 → 756.25 | 0% | 13,395.25 |
| Singleton unit list, opt-3 decode | 258.5 → 258.5 | 0% | 18,756 |
| 4,096 units, opt-z encode | 181,764.5 → 181,764.5 | 0% | 19,167.75 |
| 4,096 units, opt-z decode | 66,163.5 → 66,163.5 | 0% | 1,535,925.5 |
| 4,096 units, opt-3 encode | 778.25 → 778.25 | 0% | 13,129.5 |
| 4,096 units, opt-3 decode | 295.5 → 295.5 | 0% | 1,066,776 |

The opt-z unit encode remains about 9.48× Candid. Serde's default `collect_seq`
visits references; admission work survives this compiler setting even though
unit values emit no element bytes. Opt-3 can fold that loop. Hints on unit
serialization alone do not remove it. Automatically skipping Serde visits could
change admission or custom serialization semantics, so no unit-specific codec,
representation or unchecked path is added. This remains a separate research
target, with ordinary controls visible.

## Native timing and allocations

Native medians in nanoseconds, two serial runs of eleven calibrated batch
samples (minimum 2 ms per batch). The shared host is not CPU-isolated; small
changes and inconsistent directions are not established wins. Result destruction
is included. These are native timings, separate from IC instructions.

| opt-3 path | Run 1 before → after | Change | Run 2 before → after | Change |
| --- | ---: | ---: | ---: | ---: |
| Empty unit list encode | 39.504 → 34.685 | −12.20% | 39.368 → 38.857 | −1.30% |
| Singleton unit list encode | 39.628 → 36.941 | −6.78% | 38.997 → 34.977 | −10.31% |
| Singleton unit list decode | 10.875 → 9.836 | −9.56% | 11.045 → 10.053 | −8.98% |
| 4,096 units encode | 1,791.995 → 259.266 | −85.53% | 1,838.643 → 260.855 | −85.81% |
| 4,096 units decode | 664.783 → 869.400 | +30.78% | 674.460 → 868.843 | +28.82% |
| 4,096 empty records decode | 668.932 → 816.806 | +22.11% | 657.010 → 869.539 | +32.35% |

Keep the decode regression visible: approximately 0.2 µs per operation. The
largest native increase is 32.35%, on empty-record decoding. Opt-z unit-list
encode-reuse varies **4,325.869 → 5,219.422 ns (+20.66%)** in run 1 and
**5,188.691 → 4,327.082 ns (−16.61%)** in run 2; do not select a preferred run.
All allocation and peak-live-heap observations are identical before/after for
both profiles and every recorded operation. Those probes are separate warmed,
single-operation instrumented runs, not eleven allocator samples.

The selected cleanup is justified by simpler control flow and smaller raw/code
modules at both profiles, with modest IC movement. It is not a universal native
speed improvement. [All native pairs](native-paired.json.gz) retain regressions,
Candid controls and allocator/heap rows.

## Candidates and rejected work

Each patch is independent against the retained baseline. The corrected screen
measures three stable IC samples per operation and retains all physical modules.

| Candidate | opt-z total / code delta | opt-3 total / code delta | Decision |
| --- | ---: | ---: | --- |
| Outline `serialize_bytes` on Wasm | 0 / 0 | +15 / +13 | Reject: byte-key-map encode 62,120.25 → 124,522.25 at opt-3 |
| Inline unit/unit-struct serializers on Wasm | 0 / 0 | 0 / 0 | Reject: identical artifacts and counters |
| Skip zero-size heap charges | 0 / 0 | 0 / 0 | Reject: no size win, tiny instruction changes |
| Combine all budget checks, change check/commit ordering | −503 / −496 | +6 / +6 | Reject: opt-z unit decode 66,163.5 → 74,356.5 (+12.38%) |
| Direct checks retaining refusal/debit ordering | −26 / −26 | −14 / −14 | Keep, then repeat full native/IC smoke and 100-type size qualification |

The selected helper checks existing node/heap allowances before subtraction,
then commits the same heap and node debits. Sticky refusal, depth/overflow
rejection, allocation accounting, malformed acceptance, Postcard bytes and exact
retry/hash inputs are unchanged. Existing contracts require no migration. The
working source adds an explanatory comment to the exact measured patch; no
additional runtime tokens differ. No profiles, dependencies or own versions
change, and one production Serde/Postcard codec remains.

An earlier copied-workspace screen reused newer cached artifacts because copied
source retained older mtimes. It is **invalid** and contributes no result above.
[Quarantined identities](invalid-cache-control.json) record that failure. The
corrected runs use one fixed worktree, copy changed files immediately before
building, and require observed library/guest compilation. The final paired gate
also refuses a cached candidate. [Fresh-build evidence](fresh-builds.json) covers
both sides, both profiles and both fixtures.

## Reproduction and retained inputs

- Source base: `c2d3dfe5266e696a9a98983bdac53fb38024e522`, plus the identical
  new harness frozen in [source.tar.gz](source.tar.gz); that capsule contains
  baseline runtime bodies, not the selected patch. [Cargo.lock](Cargo.lock.snapshot)
  and manifest/toolchain snapshots bind inputs.
- Rust **1.99.1**, LLVM **23.1.1**, host `x86_64-unknown-linux-gnu`, guest
  `wasm32-unknown-unknown`. Candid **0.10.39**, Postcard **1.1.3**, Serde **1.0.229**,
  serde_bytes **0.11.19**, Platform **0.16.0**, Types **0.15.1**,
  Testkit **0.75.2**, Metrics **0.24.4**. The lock also retains native-only
  Platform 0.15.3; it is not the production endpoint supplier.
- Library recipe `evaluation`: opt-z, fat LTO, one codegen unit, panic abort,
  symbol stripping; `benchmark-speed` changes only opt-level to 3. The primary
  question is library code-section pressure under the size-oriented recipe;
  opt-3 checks the speed-oriented compiler tradeoff. Neither represents a
  particular consumer deployment. **No post-link optimizer or deployment
  transformation is applied.** Consumer profiles remain consumer-owned.
- PocketIC server **16.0.0**, SHA-256
  `69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
  IC settings: three callers/samples, four repetitions for small frames and
  one for large frames. Compiler, source/fixture/driver hashes, exact frames,
  environment overrides and artifact hashes are in the retained raw identities.
  Recorded overrides are empty; global Cargo config/build inputs must remain
  fixed too. This is reproducible qualification, not a hermetic build claim.

To repeat measurements, extract the capsule into a **new absolute work directory**
and prepare its locked caches/toolchains and explicit server first. Native
sampling records Git context; use a disposable checkout of the recorded base
with the capsule overlaid, or point `GIT_DIR`/`GIT_WORK_TREE` to the real read-only
Git context and extracted source. Run both sides from that same source path
with an exclusive `CARGO_TARGET_DIR`. Save the three baseline native source
files before applying any patch. Build the scale fixture before each smoke run:

```sh
# Run in the extracted source or disposable checkout. Use new output directories.
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export POCKET_IC_BIN=/absolute/prepared/pocket-ic
export CARGO_TARGET_DIR=/absolute/exclusive/build-cache
export RUNS=2 SAMPLES=11 BATCH_MS=2
touch crates/ic-wire/src/native/mod.rs crates/ic-wire-benchmarks/drivers/wasm.rs
for profile in evaluation benchmark-speed; do
  cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks \
    --example codec-wasm --target wasm32-unknown-unknown \
    --features wire,derive-scale --message-format=json
  # Retain build JSON, resolve its artifact with codec-report --artifact,
  # copy it before the next build, and measure with codec-report --code-size.
done
touch crates/ic-wire/src/native/mod.rs crates/ic-wire-benchmarks/drivers/wasm.rs
CORPUS_FEATURE=cost-smoke bash scripts/ci/evaluate-codecs.sh /absolute/new/baseline
# Apply admission-linear.patch with patch -p1, then repeat into new candidate outputs.
# Require library and guest fresh=false in both profiles' build JSON on each side.
```

The exact local command records [run-ic.sh](run-ic.sh) and
[qualify-final.sh](qualify-final.sh) retain original absolute paths; adapt those
paths for another machine. They are evidence, not a portable installation tool.
Repeat other independent patches from the saved baseline, never stacked.

For a data-only gate replay, extract [gate-inputs.tar.gz](gate-inputs.tar.gz)
and [final-wasm.tar.gz](final-wasm.tar.gz) into the same new directory. From the
current repository run `make check-codec-costs CODEC_BASELINE_DIR=/that/path/baseline
CODEC_CANDIDATE_DIR=/that/path/admission-linear`. This starts no server or build.
[Gate code and corruption fixtures](gate-and-corruption-fixtures.tar.gz) freeze
the exact checker. No compressed size is used as a Wasm metric; archives only
store evidence.

[final/raw.json.gz](final/raw.json.gz) retains every native/IC sample and identity;
[final/comparison.csv](final/comparison.csv) and
[final/rejection-comparison.csv](final/rejection-comparison.csv) retain paired
Candid results. The existing [strict analyzer](../candid-comparison-0133/analyze.mjs)
produced `final/` from the two recorded cohorts. [screen.csv](screen.csv),
[screen-raw.json.gz](screen-raw.json.gz) and [screen-wasm.tar.gz](screen-wasm.tar.gz)
retain all corrected candidates; [scale-100.csv](scale-100.csv) and
[scale-artifact-identities.json](scale-artifact-identities.json) bind the scale
modules. [SHA256SUMS](SHA256SUMS) covers the evidence directory.

Focused checks pass: 82 borrowed/native/runtime integration tests, strict runtime
and corpus Clippy, Rust 1.91 host/Wasm corpus compilation, installed endpoint
admission/rollback, 49 CLI checks, 18 gate fixtures and the 192-pair real gate.
The CLI fixture required permitted child-process execution after the restricted
sandbox returned `EPERM`; the rerun passed. Shared snapshot integrity, boundary,
pins and documentation checks are recorded separately. No new full CI or
full-corpus performance rerun is claimed; the preceding releases' full CI does
not qualify this subsequent uncommitted batch.
