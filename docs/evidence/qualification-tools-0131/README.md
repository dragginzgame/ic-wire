# Opt-in fuzzing and allocator qualification (#39)

2026-10-31. Core 0.13.1 pending patch; own versions remain Core 0.13.0 / Endpoint
0.2.0. The current Shared/Platform/Tooling adoption is preserved. These are local
uncommitted changes, with no publication, source commit or sibling checkout edit.

## Change and boundary

The unchanged 91-line coverage-guided target moves from the library example into
`ic-wire-qualification/examples/fuzz_runtime/mod.rs`. The existing private package
owns native-only optional libfuzzer-sys through feature `fuzz-runtime`. The public
library no longer declares that development dependency or example. `make fuzz-runtime`
keeps its existing corpus, sanitizer, nightly and command interface, selecting
the private feature internally. No new crate/workspace or compatibility wrapper
is introduced. [Preservation proof](preservation.json) records identical fuzz
source bytes and unchanged registry versions/checksums; only ownership/selection
edges change in Cargo.lock.

The private benchmark now selects native-only optional stats_alloc/dhat normal
dependencies through its existing allocations/heap features. Cargo dev-dependencies
remain nonoptional; neither profiler appears in ordinary builds. Existing per-target
mutual-exclusion guards are retained. Routine qualification explicitly selects
allocation-fixture rather than all-features. Benchmark CI checks both explicit
profiler modes on Clippy and Rust 1.91 across affected targets.

`scripts/ci/check-qualification-tools.sh`, called by check-qualification, enforces
ordinary codec/benchmark/qualification graph exclusions, explicit tool selection,
independent profiler graphs and native-only dependency edges, including Wasm with
the private features selected. It observes actual locked Cargo graphs without
building unsupported Wasm fuzz targets. [Graph results](graphs.json) distinguish
default exclusions from explicit inclusion. Optional tool packages remain in the
workspace lock; that is not evidence they are selected by ordinary builds.

## Executed checks

[Logs](checks/) retain:

- 49 codec tests (4 library, 6 borrowed, 34 native, 5 runtime) and 4 ordinary
  qualification tests; the new graph boundary gate passes.
- Three default Wire/Candid corpus roundtrip/refusal controls and Rust 1.91
  compilation of the ordinary native timing suite, with neither profiler selected.
- Prepared nightly-2026-07-29 fuzz smoke: 1,000 executions, deterministic restored
  packed seeds, no failure. Address checking remains enabled;
  ASAN_OPTIONS=detect_leaks=0 explicitly disables leak shutdown for this environment.
- One executed caller-storage test per profiler mode, each exercising 36 case/lane
  records with seven instrumentation samples; both allocations and heap pass.
  These runs verify capability, not measured optimization savings.
- Both profiler modes across benchmark targets pass strict Clippy and Rust 1.91.
  The relocated fuzz target separately passes Rust 1.91 and strict Clippy.
- Selecting allocations and heap together fails compilation with the maintained
  explicit diagnostic in both suite and caller-storage entrypoints. Those two
  expected compiler failures are successful negative checks, not passing builds.
- Manifest/Rust formatting, dependency pins/boundaries, shell/Perl syntax and
  documentation links pass. The changed benchmark manifest initially needed its
  target dependency tables sorted; only that manifest was formatted before the
  successful integrity rerun.

[Compiler](compiler.txt): Rust 1.99.1 / LLVM 23.1.1, Linux x86_64; minimum checks
use 1.91.0. Native/macOS target restrictions remain unchanged; no new macOS or full
hosted matrix was run. No full local CI, instruction comparison or Wasm/build-time
saving is claimed. This changes qualification dependency selection, not runtime
encoding/generator behavior. Protocol bytes, schema, budgets, primitive/malformed
acceptance and retry/hash inputs are unchanged; no new runtime benchmark applies.

## Reproduction

With the selected compilers, C++ toolchain and registry cache already prepared:

```sh
export CARGO_TARGET_DIR="$PWD/target/qualification-tools-repeat"
bash scripts/ci/check-qualification-tools.sh
cargo test --locked --offline -p ic-wire --lib --test runtime --test native --test borrowed
make check-qualification CARGO_TARGET_DIR="$CARGO_TARGET_DIR"
cargo +1.91.0 check --locked --offline -p ic-wire-qualification --example fuzz-runtime --features fuzz-runtime
cargo clippy --locked --offline -p ic-wire-qualification --example fuzz-runtime --features fuzz-runtime -- -D warnings
ASAN_OPTIONS=detect_leaks=0 FUZZ_RUNS=1000 make fuzz-runtime
mkdir -p "$CARGO_TARGET_DIR/profiler-smoke"
for mode in allocations heap; do
    IC_WIRE_STORAGE_EVIDENCE="$CARGO_TARGET_DIR/profiler-smoke" cargo test --locked --offline -p ic-wire-benchmarks --example caller-storage --features "$mode" native_measurement -- --ignored --test-threads=1
    cargo clippy --locked --offline -p ic-wire-benchmarks --all-targets --features "wire,candid,$mode" -- -D warnings
    cargo +1.91.0 check --locked --offline -p ic-wire-benchmarks --all-targets --features "wire,candid,$mode"
done
cargo test --locked --offline -p ic-wire-benchmarks --features wire,candid --test roundtrip
cargo +1.91.0 check --locked --offline -p ic-wire-benchmarks --bench suite --features wire,candid
make fmt-check check-pins check-boundary check-doc-links
```

The fuzz runner owns its existing separate target/fuzz-runtime output. Original
working logs and profiler smoke JSON remain under target/issues-39; essential
checks are retained here so explicit cleanup need not preserve target outputs.
