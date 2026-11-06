# Bounded host schema traversal and error diagnostics

Measured 2026-11-05 for [#46 — Support large composed registries within bounded host schema traversal](https://github.com/dragginzgame/ic-wire/issues/46)
and [#47 — Keep Error Display diagnostics independent of Debug formatting](https://github.com/dragginzgame/ic-wire/issues/47).
Baseline: `c8e632163f2c8d0f78499c1ac3a3e165edff396a`; candidate: its working tree
with the two fixes, before Testkit 0.87 adoption. Neither source is a new package
release; these timings do not qualify the subsequent dependency update.

## Matched native measurements

Rust 1.99.1 (`58cf8f9c3`), Cargo 1.99.1 (`5f94df478`), Linux
`x86_64-unknown-linux-gnu`, AMD Threadripper 7970X. Cargo `--release` defaults:
opt-3, no LTO, default codegen units, test unwinding; no Rust flags. Identical
manifests/dependencies and Cargo.lock SHA-256
`8e7ab6ae7acd382fb461561cf03676ff129d6b108c7fe5e9ff169c9331ac875a`.
Each source had its own fresh Cargo target, with no competing build/measurement.

Eleven samples per case; medians below. Generation includes rendering, complete
advice and disposal, excluding fixture construction and terminal output. One,
32 and 297 methods use 1,000, 20 and three repetitions/sample respectively.
Error formatting averages all five variants over 500,000 writes/sample into one
reused String with capacity 32; no terminal I/O or repeated buffer allocation.

| Operation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Generate one method | 11.317 µs | 11.468 µs | +1.33% |
| Generate 32 methods | 1.128235 ms | 1.095145 ms | −2.93% |
| Generate 297 methods | 6.713176 ms, **Complexity** | 11.249410 ms, **success** | +67.57%; different completed work |
| Format one error, averaged over five variants | 13.776694 ns | 6.420804 ns | −53.39% |

The large fixture has 594 roots, 112,751 expanded visits (largest root 7,790),
shared definitions and active-path recursion. The candidate retains all 15,925
warning paths and emits 214,150 bytes of TypeScript source. Controls emit the
same 3,530/25,473 source bytes on both sides. These are host source sizes, **not
Wasm sizes**. The failed large baseline produces no accepted source/report;
its shorter time is not an equivalent successful generation. Small-control
variation is not a material speed improvement claim. No codec time, IC
instructions, whole-module or code-section saving is claimed.

## Reproduction and source identity

Maintained probes are ignored tests in
[the composed fixture](../../crates/ic-wire/tests/native/bulk_lint/composition_tests.rs)
and [the native test owner](../../crates/ic-wire/tests/native/mod.rs).
Prepare disposable source copies under `target/evidence/`: baseline via
`git archive c8e632163f2c8d0f78499c1ac3a3e165edff396a`, candidate from the current
reviewed source. Overlay those two test files and `tests/native/bulk_lint/mod.rs`
from the candidate onto the baseline, preserving identical runners. Use distinct
absolute Cargo target directories and the same locked dependencies. In each copy:

```sh
WIRE_STUDY_SOURCE_HASH=REVIEWED_INPUT_HASH CARGO_TARGET_DIR=ABSOLUTE_LANE_TARGET \
  bash /home/adam/projects/ic-wire/scripts/ci/rust-policy.sh \
  cargo test --offline --locked --release --target x86_64-unknown-linux-gnu \
  -p ic-wire --features typescript --test native measure_ -- \
  --ignored --nocapture --test-threads=1
```

The input hash is SHA-256 of a compact, sorted JSON map from relative paths to
file SHA-256 values: root Cargo.toml/Cargo.lock and every file under
crates/ic-wire and crates/ic-wire-derive. The probe bakes this environment value
into its executable; compare printed values with the independently hashed inputs.
Recorded baseline/candidate hashes:

- `c399e2471b160587a0677f1d0237e29a29b864a064c56036485b4d20dd6bdb2d`
- `817f7529d4dad3fec92b1837ca3fbfae8d5c88a960dcf02c01543416feeade7e`

Raw manifests, logs, copied sources and samples are disposable under
`target/evidence/issues46-47/`. Only this compact result is maintained in Git.
The synthetic fixture is not the original Toko application registry.

## Diagnostic and capacity qualification

Normal formatting tests cover every Error variant. A focused Rust 1.99.1
experiment with `RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zfmt-debug=none`, separate targets
and the same regression test reproduced the baseline's bare `wire ` and passed
with explicit candidate literals. This one-off unstable-flag experiment neither
selects nightly nor changes compiler policy or recommended production flags.
Display changes `wire Malformed` to `wire malformed`; typed errors retain their
meanings. Current Wire/Canic/Toko Rust/TS source searches found no exact old-text
consumers; that is not a fleet-wide guarantee.

[The design table](../design.md#host-schema-traversal) reviews all six allowance
owners. Only rooted advice changes: 65,536 visits per root, 1,000,000 per report.
Tests preserve exact use-site acknowledgements and deterministic source, and
reject excessive per-root DAG expansion, aggregate work and reference depth.
Existing malformed schemas, recursive references and no-partial-report cases
pass. Runtime limits, protocol acceptance, bytes and method IDs are unchanged.

Earlier hash-advice work recorded bounded Display output (eight entries plus an
omission count), roughly 98.6% less formatting time for 1,024 diagnostics, and
13–16% more generation work when newly retaining hash warnings. Those historical
raw archives were retired; they are not current regression or reproduction proof.

## 6 November cost-advice extension

The audit adds warnings for ordinary bool, float, 128-bit integer and zero-sized
lists, without adopting new Packed types. Isolated sources differ only in the
production advice implementation; both use identical current test probes and
the lock SHA-256 `ba25b3060a68d00e1de64502fb1ac0b72fd462214d2e1bea6b475d320ad09552`.
That lock includes Platform 0.23.2 and cc 1.11.0; a concurrent later cc 1.11.1
update is not part of this paired measurement. Compiler/host/release settings
match the Rust 1.99.1 opt-3 setup above. Sources had separate fresh Cargo targets;
no concurrent compilation or measurement ran during sampling. Baked source hashes
were checked against all hashed files after both runs:

- Before: `d119f7cfbdf7e2f75c388540be8205275f7083e73141576b4bceca38e2a4ca89`
- After: `d79032edec7776d8e52d3226c21b5cd5e43d424a508ab1ba6554ec235fbe8b46`

Each row contains 11 samples. Existing composed controls use the repetitions
above; the new 1/64/512-type controls use 1,000/40/5 repetitions per sample.
Each new type is a tuple of bool, f64, u128 and unit lists. Generation includes
complete diagnostics and disposal, excluding fixture setup and terminal output.

| Host generation | Before median | After median | Change |
| --- | ---: | ---: | ---: |
| Existing one-method control | 10.963 µs | 11.612 µs | +5.9% |
| Existing 32-method control | 1.099587 ms | 1.102099 ms | +0.2% |
| Existing 297-method control | 10.982834 ms | 11.220575 ms | +2.2% |
| One newly warned type | 1.923 µs | 2.317 µs | +20.5% |
| 64 newly warned types | 105.687 µs | 117.967 µs | +11.6% |
| 512 newly warned types | 847.817 µs | 1,013.178 µs | +19.5% |

The last row now retains 2,048 diagnostics instead of zero; this added host
work is intentional. Source lengths match for every before/after control.
Runtime bytes, limits and schemas are unchanged. IC instructions, total
uncompressed Wasm and code-section deltas are not measured; no saving is claimed.

Reproduce with the unchanged `measure_composed_generation` and
`measure_new_cost_advice` ignored tests, filtering `bulk_lint::` with
`--ignored --nocapture --test-threads=1`. Use identical candidate test files in
both sources and the pre-extension `bulk_lint/mod.rs` for the baseline. Hash and
bake inputs as above, compile both before sampling, and verify fresh binaries.
Disposable sources, input maps and all 132 timing samples are under
`target/audit-hardening/`; they are not retained in Git.

The audit also reviewed standalone schema validation's discarded rendered
strings. It retains the existing shared traversal: a separate validator would
duplicate acceptance logic, while a rendering-sink refactor needs its own
measured study. No unmeasured refactor or complexity increase was retained.
