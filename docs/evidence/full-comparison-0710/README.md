# 0.7.10 full comparison evidence

This directory owns the source identities, analysis and results for the
[full comparison](../../full-comparison-0710.md). It does not add a production
codec or an automated release gate. Runtime and protocol behavior are unchanged.

The current source is `c2adbfb47da6c180bc470c0d5204a35d7b61d574`; the previous
release is `00b1686a80cd7a3068be0a24aaee0dc8239009c7`. The benchmark fixtures are
identical across those revisions. `baseline-normalization.patch` records the
root manifest/lock replacement used for the separate code-only comparison;
the original-release comparison retains 0.7.9's manifests and lock.

## Reproduce

Use isolated checkouts, the pinned Rust compiler/target and cached locked
Cargo dependencies. Prepare the explicit PocketIC server recorded in the report.
Run current and baseline work serially; do not compile alongside native timings.
No sibling checkout or installed application is needed.

```sh
export POCKET_IC_BIN=/path/to/prepared/pocket-ic
export CARGO_TARGET_DIR=/absolute/isolated/target
export RUNS=2 SAMPLES=11 BATCH_MS=2
benchmark_git_dir="$(git rev-parse --absolute-git-dir)"
benchmark_git_tree="$PWD"
bash scripts/ci/evaluate-codec-matrix.sh /absolute/run/current
(
  export GIT_DIR="$benchmark_git_dir" GIT_WORK_TREE="$benchmark_git_tree"
  cd /absolute/run/current/source
  CORPUS_FEATURE=hotspots bash scripts/ci/evaluate-codecs.sh /absolute/run/hotspots
)
```

The hotspot command deliberately uses the matrix's frozen source/lock, so a
concurrent checkout update cannot silently change the second run's dependencies.

Build each baseline's `codec-wasm` example with `--locked`, target
`wasm32-unknown-unknown`, profiles `evaluation` and `benchmark-speed`, and each
cohort's features from `scripts/ci/codec-cohorts.psv` plus `wire` or `candid`.
The retained `compare-baseline.sh RUN_ROOT normalized|release` script shows the
commands and directory layout. Set `CARGO_TARGET_DIR`, `POCKET_IC_BIN` and
`CODEC_REPORT` (the built reporter executable) explicitly. It compares
whole artifact bytes and total/code-section sizes. When any artifact differs,
run that baseline cohort's `codec-instructions` example with `wire,candid,ic`
and its cohort features, against the same explicit server. Identical artifact
bytes and input identities need no duplicate IC execution; this is equivalence
evidence, not a claim of separately measured baseline counters.

```sh
node docs/evidence/full-comparison-0710/analyze.mjs /absolute/run /absolute/analysis
```

The analyzer checks paired input/payload hashes, unique operation keys and sample
counts. `comparison.csv` contains the primary owned lanes and both native run
medians, allocation counts and heap measurements. `instruction-wins.csv` counts
wins/ties/losses without traffic weighting. `rejection-comparison.csv` pairs the
primary bad-header and truncated-input counters. `artifacts.csv` keeps uncompressed
Wasm and code-section sizes separate. `raw.json.gz` archives all IC/native
samples and identities; compression is storage only, not a codec size metric.
The all-lanes archive includes reuse, borrowing and rejection measurements.
`normalized-artifacts.csv` and `release-artifacts.csv` retain both baseline
comparisons. Differing artifacts require matched baseline IC samples; the analyzer
refuses to report an incomplete baseline as a completed regression check.

`source.tar.gz` freezes the measured source/manifests/runners. Extract it into
`RUN_ROOT/baseline` and `RUN_ROOT/release-baseline`, then apply
`previous-source.patch` in both to restore 0.7.9. For the normalized baseline
only, replace root Cargo.toml and Cargo.lock with the current snapshot versions.
This matches the manifest/lock change recorded in `baseline-normalization.patch`.

Source revisions, locked dependencies, compiler/settings and raw identities are
required to reproduce the run. Historical corpus changes or server changes are
not comparable regression baselines. Successful corpus operations qualify these
fixtures only; they do not replace the runtime tests, browser conformance, full
CI or consumer-owned deployment and recovery checks.
